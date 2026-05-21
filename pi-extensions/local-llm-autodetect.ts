import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";

export default async function (pi: ExtensionAPI) {
  const baseUrl = "http://localhost:6969/v1";

  try {
    const res = await fetch(`${baseUrl}/models`, { signal: AbortSignal.timeout(5000) });
    if (!res.ok) {
      console.warn(`[local-llm-autodetect] /v1/models returned ${res.status} — falling back to static models.json`);
      return;
    }

    const payload = (await res.json()) as {
      data?: Array<{
        id: string;
        object?: string;
        owned_by?: string;
      }>;
    };

    const models = (payload.data ?? []).filter(
      (m) => m.object === "model" || m.object === undefined
    );

    if (models.length === 0) {
      console.warn("[local-llm-autodetect] No models found at /v1/models");
      return;
    }

    pi.registerProvider("local-llama", {
      baseUrl,
      api: "openai-completions",
      apiKey: "dummy",
      compat: {
        supportsDeveloperRole: false,
        supportsReasoningEffort: false,
        thinkingFormat: "qwen-chat-template",
      },
      models: models.map((m) => ({
        id: m.id,
        name: `${m.id} (Local)`,
        reasoning: true,
        input: ["text"] as ("text" | "image")[],
        contextWindow: 65536,
        maxTokens: 4096,
        cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 },
      })),
    });

    console.log(`[local-llm-autodetect] Registered ${models.length} model(s) from ${baseUrl}`);
  } catch (err) {
    console.warn("[local-llm-autodetect] Could not reach local llama-server:", err instanceof Error ? err.message : String(err));
  }
}
