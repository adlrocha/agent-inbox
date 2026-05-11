#!/usr/bin/env python3
"""
Nibble LLM Privacy Proxy
========================
Intercepts API calls from sandboxed agents to Anthropic/OpenAI, scans
prompts for PII/secrets using OpenAI's privacy-filter model, and either
redacts in-place or blocks the request.

Endpoints:
  POST /v1/messages          -> Anthropic Messages API
  POST /v1/chat/completions  -> OpenAI Chat Completions API
  GET  /health               -> Proxy health + model status

Env:
  PRIVACY_FILTER_PORT       Port to listen on (default: 8474)
  PRIVACY_FILTER_MODE       "redact" | "block" | "flag" (default: redact)
  PRIVACY_FILTER_DEVICE     "cpu" | "cuda" (default: cpu)
  ANTHROPIC_UPSTREAM        Anthropic API base URL
  OPENAI_UPSTREAM           OpenAI API base URL
"""

from __future__ import annotations

import json
import os
import re
import sys
import time
import asyncio
from contextlib import asynccontextmanager
from typing import Any, Literal
from urllib.parse import urljoin

import httpx
from fastapi import FastAPI, Request, Response
from fastapi.responses import StreamingResponse

# ── Configuration ────────────────────────────────────────────────────────────
PORT = int(os.getenv("PRIVACY_FILTER_PORT", "8474"))
MODE = os.getenv("PRIVACY_FILTER_MODE", "redact")  # redact | block | flag
DEVICE = os.getenv("PRIVACY_FILTER_DEVICE", "cpu")
ANTHROPIC_UPSTREAM = os.getenv("ANTHROPIC_UPSTREAM", "https://api.anthropic.com")
OPENAI_UPSTREAM = os.getenv("OPENAI_UPSTREAM", "https://api.openai.com")

# Fast regex pre-filter — if none match, skip the ML model entirely.
SECRET_PATTERNS = [
    re.compile(r"sk-[a-zA-Z0-9]{20,}", re.IGNORECASE),               # OpenAI keys
    re.compile(r"AKIA[0-9A-Z]{16}"),                                  # AWS access keys
    re.compile(r"gh[pousr]_[A-Za-z0-9_]{36,}", re.IGNORECASE),       # GitHub tokens
    re.compile(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}"),  # Emails
    re.compile(r"\b\d{3}[-.]?\d{2}[-.]?\d{4}\b"),                  # SSN-ish
    re.compile(r"\b(?:password|passwd|pwd)\s*[:=]\s*\S+", re.IGNORECASE),
    re.compile(r"-----BEGIN (RSA |EC |DSA |OPENSSH )?PRIVATE KEY-----"),
    re.compile(r"\b[0-9a-f]{64}\b"),                                 # Hex secrets
    re.compile(r"\b\d{4}[ -]?\d{4}[ -]?\d{4}[ -]?\d{4}\b"),        # Credit-card-ish
]


def _needs_ml_scan(text: str) -> bool:
    """Fast heuristic: only run the model if regex finds something suspicious."""
    return any(p.search(text) for p in SECRET_PATTERNS)


# ── Model loading (deferred until first request) ─────────────────────────────
_classifier = None
_model_loaded_at: float | None = None
_model_load_time_ms: float = 0.0


def _load_model() -> Any:
    """Lazy-load the privacy-filter model."""
    global _classifier, _model_loaded_at, _model_load_time_ms
    if _classifier is not None:
        return _classifier

    t0 = time.perf_counter()
    try:
        from transformers import pipeline
        _classifier = pipeline(
            "token-classification",
            model="openai/privacy-filter",
            device=DEVICE if DEVICE != "cpu" else -1,
            aggregation_strategy="simple",
        )
    except Exception as e:
        print(f"[privacy-proxy] Failed to load model: {e}", file=sys.stderr)
        raise

    _model_load_time_ms = (time.perf_counter() - t0) * 1000
    _model_loaded_at = time.time()
    print(
        f"[privacy-proxy] Model loaded in {_model_load_time_ms:.0f}ms "
        f"(device={DEVICE})",
        file=sys.stderr,
    )
    return _classifier


def _redact_text(text: str) -> tuple[str, list[dict]]:
    """Run privacy-filter on text and return (redacted, detections)."""
    if not text or not isinstance(text, str):
        return text, []

    if not _needs_ml_scan(text):
        return text, []

    clf = _load_model()
    spans = clf(text)
    if not spans:
        return text, []

    # Sort spans by start position, descending, so we can replace in-place
    spans_sorted = sorted(spans, key=lambda s: s["start"], reverse=True)
    redacted = text
    detections: list[dict] = []

    for span in spans_sorted:
        label = span.get("entity_group", span.get("entity", "unknown"))
        start = int(span["start"])
        end = int(span["end"])
        score = float(span.get("score", 1.0))
        detections.append({
            "label": label,
            "text": redacted[start:end],
            "start": start,
            "end": end,
            "score": round(score, 4),
        })
        redacted = redacted[:start] + f"[REDACTED: {label}]" + redacted[end:]

    return redacted, detections


# ── Request body scanning ────────────────────────────────────────────────────
def _scan_content(content: Any) -> tuple[Any, list[dict]]:
    """Recursively scan content blocks and redact PII.

    Handles:
      - str (OpenAI, simple Anthropic)
      - list of {type: "text", text: "..."} (Anthropic content blocks)
      - list of mixed blocks (only scans text blocks)
    """
    all_detections: list[dict] = []

    if isinstance(content, str):
        redacted, dets = _redact_text(content)
        return redacted, dets

    if isinstance(content, list):
        new_blocks = []
        for block in content:
            if isinstance(block, dict) and block.get("type") == "text":
                text = block.get("text", "")
                redacted, dets = _redact_text(text)
                new_block = dict(block)
                new_block["text"] = redacted
                new_blocks.append(new_block)
                all_detections.extend(dets)
            elif isinstance(block, dict) and block.get("type") == "tool_result":
                # Tool results can also contain text content
                new_block = dict(block)
                tool_content = block.get("content")
                if isinstance(tool_content, str):
                    redacted, dets = _redact_text(tool_content)
                    new_block["content"] = redacted
                    all_detections.extend(dets)
                elif isinstance(tool_content, list):
                    new_block["content"], dets = _scan_content(tool_content)
                    all_detections.extend(dets)
                new_blocks.append(new_block)
            else:
                new_blocks.append(block)
        return new_blocks, all_detections

    # Unknown content shape — pass through
    return content, []


def _scan_request_body(body: bytes) -> tuple[bytes, list[dict], bool]:
    """Scan an API request body for PII.

    Returns:
      (modified_body, detections, pii_found)
    """
    if not body:
        return body, [], False

    try:
        data = json.loads(body)
    except json.JSONDecodeError:
        return body, [], False

    if not isinstance(data, dict):
        return body, [], False

    messages = data.get("messages")
    if not isinstance(messages, list):
        return body, [], False

    all_detections: list[dict] = []
    modified = False

    for msg in messages:
        if not isinstance(msg, dict):
            continue
        content = msg.get("content")
        new_content, dets = _scan_content(content)
        if dets:
            msg["content"] = new_content
            all_detections.extend(dets)
            modified = True

    if not modified:
        return body, [], False

    return json.dumps(data, ensure_ascii=False).encode("utf-8"), all_detections, True


# ── FastAPI app ──────────────────────────────────────────────────────────────

@asynccontextmanager
async def lifespan(app: FastAPI):
    """Warm up the model in the background so first request is faster."""
    asyncio.get_running_loop().run_in_executor(None, _load_model)
    yield


app = FastAPI(title="Nibble LLM Privacy Proxy", lifespan=lifespan)
client = httpx.AsyncClient(timeout=httpx.Timeout(300.0))


@app.get("/health")
async def health() -> dict:
    healthy = _classifier is not None
    return {
        "status": "healthy" if healthy else "loading",
        "mode": MODE,
        "model_loaded_at": _model_loaded_at,
        "model_load_time_ms": round(_model_load_time_ms, 1),
        "device": DEVICE,
    }


async def _proxy_request(request: Request, upstream_base: str) -> Response:
    """Generic proxy: scan body, then forward to upstream."""
    path = request.url.path
    if request.url.query:
        path += "?" + request.url.query

    target = urljoin(upstream_base, path)

    # Copy headers, stripping hop-by-hop ones
    headers: dict[str, str] = {}
    for k, v in request.headers.items():
        kl = k.lower()
        if kl in ("host", "content-length", "transfer-encoding"):
            continue
        headers[k] = v

    # Read and optionally scan body
    body = await request.body()
    detections: list[dict] = []

    if request.method == "POST" and body:
        scanned_body, detections, pii_found = _scan_request_body(body)

        if pii_found and MODE == "block":
            labels = list({d["label"] for d in detections})
            error_body = {
                "error": {
                    "type": "privacy_filter_blocked",
                    "message": (
                        f"Request blocked: detected {len(detections)} PII/secrets span(s) "
                        f"({', '.join(labels)}). Remove sensitive data from your prompt or "
                        f"switch proxy mode to 'redact' in ~/.nibble/config.toml."
                    ),
                    "detections": detections,
                }
            }
            return Response(
                content=json.dumps(error_body),
                status_code=400,
                media_type="application/json",
                headers={"X-Privacy-Filter-Blocked": "true"},
            )

        if pii_found and MODE == "flag":
            # Forward redacted body, but add header so client can log it
            headers["X-Privacy-Filter-Flagged"] = json.dumps(
                [{"label": d["label"], "score": d["score"]} for d in detections]
            )

        body = scanned_body

    # Forward to upstream
    req = client.build_request(
        method=request.method,
        url=target,
        headers=headers,
        content=body,
    )
    resp = await client.send(req, stream=True)

    # Stream response back
    return StreamingResponse(
        content=resp.aiter_raw(),
        status_code=resp.status_code,
        headers=dict(resp.headers),
    )


@app.api_route("/v1/messages", methods=["POST", "GET", "OPTIONS"])
async def anthropic_proxy(request: Request):
    return await _proxy_request(request, ANTHROPIC_UPSTREAM)


@app.api_route("/v1/chat/completions", methods=["POST", "GET", "OPTIONS"])
async def openai_proxy(request: Request):
    return await _proxy_request(request, OPENAI_UPSTREAM)


# Catch-all for other API endpoints (models, embeddings, etc.)
@app.api_route("/{path:path}", methods=["GET", "POST", "PUT", "DELETE", "OPTIONS", "PATCH"])
async def catchall_proxy(request: Request, path: str):
    # Try Anthropic first for /v1/* paths, otherwise OpenAI
    if path.startswith("v1/"):
        upstream = ANTHROPIC_UPSTREAM if "anthropic" in path else OPENAI_UPSTREAM
    else:
        upstream = OPENAI_UPSTREAM
    return await _proxy_request(request, upstream)


if __name__ == "__main__":
    import uvicorn

    print(f"[privacy-proxy] Starting on port {PORT} (mode={MODE}, device={DEVICE})")
    uvicorn.run(app, host="127.0.0.1", port=PORT, log_level="warning")
