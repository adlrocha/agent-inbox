#!/usr/bin/env bash
# browser.sh — CDP browser control for nibble agents.
#
# Uses Chromium launched with --remote-debugging-port=9222 and a persistent
# profile at ~/.nibble/chromium-profile (shared across all sandboxes).
#
# Usage:
#   browser start                      — launch Chromium in debug mode (if not running)
#   browser tabs                       — list open tabs (id, title, url)
#   browser page <id>                  — get text content of a tab
#   browser eval <id> <js>             — run JavaScript in a tab, return result
#   browser click <id> <selector>      — click a CSS selector in a tab
#   browser navigate <id> <url>        — navigate a tab to a URL
#   browser stop                       — kill the debug Chromium instance

set -euo pipefail

CDP_HOST="localhost"
CDP_PORT="9222"
CDP_BASE="http://${CDP_HOST}:${CDP_PORT}"
PROFILE_DIR="$HOME/.nibble/chromium-profile"
CHROMIUM_BIN=""

# ── Helpers ───────────────────────────────────────────────────────────────────

die() { echo "error: $*" >&2; exit 1; }

find_chromium() {
    for bin in chromium chromium-browser google-chrome-stable google-chrome; do
        if command -v "$bin" >/dev/null 2>&1; then
            echo "$bin"
            return
        fi
    done
    die "chromium not found. Install with: sudo apt-get install chromium  (or chromium-browser)"
}

cdp_running() {
    curl -sf "${CDP_BASE}/json/version" >/dev/null 2>&1
}

require_websocat() {
    if ! command -v websocat >/dev/null 2>&1; then
        echo "Installing websocat..." >&2
        local arch
        arch="$(uname -m)"
        local url
        case "$arch" in
            x86_64)  url="https://github.com/vi/websocat/releases/latest/download/websocat.x86_64-unknown-linux-musl" ;;
            aarch64) url="https://github.com/vi/websocat/releases/latest/download/websocat.aarch64-unknown-linux-musl" ;;
            *) die "No prebuilt websocat for $arch — install manually from https://github.com/vi/websocat/releases" ;;
        esac
        curl -sL "$url" -o "$HOME/.local/bin/websocat"
        chmod +x "$HOME/.local/bin/websocat"
        echo "websocat installed to ~/.local/bin/websocat" >&2
    fi
}

# Send a CDP command over WebSocket and return the result.
# Usage: cdp_ws <wsUrl> <json-method> [<json-params>]
cdp_ws() {
    local ws_url="$1"
    local method="$2"
    local params="${3:-{}}"
    require_websocat
    local payload
    payload=$(printf '{"id":1,"method":"%s","params":%s}' "$method" "$params")
    echo "$payload" | websocat --no-line -n1 "$ws_url" 2>/dev/null
}

# Resolve a tab id (numeric index from `browser tabs` or raw CDP targetId).
# Returns the webSocketDebuggerUrl for that tab.
resolve_tab_ws() {
    local id="$1"
    local tabs
    tabs=$(curl -sf "${CDP_BASE}/json" | jq -c '[.[] | select(.type == "page")]')
    local count
    count=$(echo "$tabs" | jq 'length')
    local ws_url

    # Numeric index (1-based from `browser tabs` output)
    if [[ "$id" =~ ^[0-9]+$ ]] && [ "$id" -ge 1 ] && [ "$id" -le "$count" ]; then
        ws_url=$(echo "$tabs" | jq -r ".[$(( id - 1 ))].webSocketDebuggerUrl")
    else
        # Try as raw targetId
        ws_url=$(echo "$tabs" | jq -r --arg id "$id" '.[] | select(.id == $id) | .webSocketDebuggerUrl')
    fi

    [ -n "$ws_url" ] && [ "$ws_url" != "null" ] || die "Tab '$id' not found. Run 'browser tabs' to list available tabs."
    echo "$ws_url"
}

# ── Commands ──────────────────────────────────────────────────────────────────

cmd_start() {
    if cdp_running; then
        echo "Chromium debug session already running on port ${CDP_PORT}."
        return
    fi
    CHROMIUM_BIN="$(find_chromium)"
    mkdir -p "$PROFILE_DIR"
    echo "Starting Chromium with remote debugging on port ${CDP_PORT}..."
    nohup "$CHROMIUM_BIN" \
        --remote-debugging-port="${CDP_PORT}" \
        --user-data-dir="${PROFILE_DIR}" \
        --no-first-run \
        --no-default-browser-check \
        >/dev/null 2>&1 &
    # Wait up to 5s for CDP to become reachable
    local i=0
    while ! cdp_running && [ $i -lt 10 ]; do
        sleep 0.5
        (( i++ )) || true
    done
    cdp_running || die "Chromium started but CDP port not reachable after 5s"
    echo "Chromium ready (pid $!)."
}

cmd_stop() {
    pkill -f "remote-debugging-port=${CDP_PORT}" 2>/dev/null && echo "Chromium debug session stopped." || echo "No debug session found."
}

cmd_tabs() {
    cdp_running || die "Chromium not running in debug mode. Run: browser start"
    local tabs
    tabs=$(curl -sf "${CDP_BASE}/json" | jq -c '[.[] | select(.type == "page")]')
    local count
    count=$(echo "$tabs" | jq 'length')
    if [ "$count" -eq 0 ]; then
        echo "No open tabs."
        return
    fi
    echo "$tabs" | jq -r 'to_entries[] | "\(.key + 1)\t\(.value.id)\t\(.value.title)\t\(.value.url)"' \
        | column -t -s $'\t' -N "NUM,ID,TITLE,URL"
}

cmd_page() {
    local id="${1:-}"
    [ -n "$id" ] || die "Usage: browser page <tab-id>"
    cdp_running || die "Chromium not running in debug mode. Run: browser start"
    local ws_url
    ws_url=$(resolve_tab_ws "$id")
    local result
    result=$(cdp_ws "$ws_url" "Runtime.evaluate" \
        '{"expression":"document.body.innerText","returnByValue":true}')
    echo "$result" | jq -r '.result.value // .error.message // "empty"'
}

cmd_eval() {
    local id="${1:-}"
    local js="${2:-}"
    [ -n "$id" ] && [ -n "$js" ] || die "Usage: browser eval <tab-id> <javascript>"
    cdp_running || die "Chromium not running in debug mode. Run: browser start"
    local ws_url
    ws_url=$(resolve_tab_ws "$id")
    local result
    result=$(cdp_ws "$ws_url" "Runtime.evaluate" \
        "$(jq -n --arg expr "$js" '{"expression":$expr,"returnByValue":true}')")
    echo "$result" | jq -r '.result.value // .result // .error.message // "null"'
}

cmd_click() {
    local id="${1:-}"
    local selector="${2:-}"
    [ -n "$id" ] && [ -n "$selector" ] || die "Usage: browser click <tab-id> <css-selector>"
    cdp_running || die "Chromium not running in debug mode. Run: browser start"
    local ws_url
    ws_url=$(resolve_tab_ws "$id")
    local js
    js="document.querySelector($(jq -n --arg s "$selector" '$s')).click()"
    local result
    result=$(cdp_ws "$ws_url" "Runtime.evaluate" \
        "$(jq -n --arg expr "$js" '{"expression":$expr,"returnByValue":true}')")
    echo "$result" | jq -r 'if .error then "error: \(.error.message)" else "clicked" end'
}

cmd_navigate() {
    local id="${1:-}"
    local url="${2:-}"
    [ -n "$id" ] && [ -n "$url" ] || die "Usage: browser navigate <tab-id> <url>"
    cdp_running || die "Chromium not running in debug mode. Run: browser start"
    local ws_url
    ws_url=$(resolve_tab_ws "$id")
    cdp_ws "$ws_url" "Page.navigate" \
        "$(jq -n --arg url "$url" '{"url":$url}')" \
        | jq -r '"navigated to \(.result.frameId // "unknown frame")"'
}

# ── Dispatch ──────────────────────────────────────────────────────────────────

CMD="${1:-}"
shift || true

case "$CMD" in
    start)    cmd_start ;;
    stop)     cmd_stop ;;
    tabs)     cmd_tabs ;;
    page)     cmd_page "$@" ;;
    eval)     cmd_eval "$@" ;;
    click)    cmd_click "$@" ;;
    navigate) cmd_navigate "$@" ;;
    *)
        echo "Usage: browser <command> [args]"
        echo ""
        echo "Commands:"
        echo "  start                  Launch Chromium in debug mode (if not running)"
        echo "  stop                   Kill the debug Chromium instance"
        echo "  tabs                   List open tabs with numeric IDs"
        echo "  page <id>              Get text content of a tab"
        echo "  eval <id> <js>         Run JavaScript in a tab"
        echo "  click <id> <selector>  Click a CSS selector in a tab"
        echo "  navigate <id> <url>    Navigate a tab to a URL"
        echo ""
        echo "Tab <id> is the NUM column from 'browser tabs' (1-based)."
        exit 1
        ;;
esac
