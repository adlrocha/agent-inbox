/**
 * Token Speed Extension for pi
 *
 * Shows real-time output tokens-per-second in the footer.
 * Tracks token counts from message_update events during streaming
 * and displays them alongside model info and git branch.
 *
 * Performance: negligible — just in-memory counters and a footer render callback.
 *
 * Command: /tokspeed  (toggle on/off, enabled by default)
 */

import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { truncateToWidth, visibleWidth } from "@earendil-works/pi-tui";

export default function (pi: ExtensionAPI) {
	let enabled = true;

	pi.on("session_start", async (_event, ctx) => {
		if (!ctx.hasUI) return;
		if (!enabled) return;

		// ── Per-turn tracking ──────────────────────────────────────────
		let turnStartTime = 0;  // ms, 0 = not tracking
		let turnEndTime = 0;

		// ── Cumulative history ─────────────────────────────────────────
		let cumulativeTokens = 0;
		const turnSpeeds: Array<{ tokens: number; spd: number }> = [];

		// ── Helpers ────────────────────────────────────────────────────
		function formatSpeed(tokPerSec: number): string {
			if (tokPerSec < 1) return `${tokPerSec.toFixed(1)} tok/s`;
			if (tokPerSec < 100) return `${tokPerSec.toFixed(0)} tok/s`;
			return `${(tokPerSec / 1000).toFixed(1)}k tok/s`;
		}

		// ── Track streaming timing via text deltas ───────────────────
		pi.on("message_update", async (event) => {
			if (event.message.role !== "assistant") return;
			if (event.assistantMessageEvent.type !== "text_delta") return;

			if (turnStartTime === 0) {
				turnStartTime = Date.now();
			}
			turnEndTime = Date.now();
		});

		// ── Record turn speed using actual token count from usage ──────
		pi.on("message_end", async (event) => {
			if (event.message.role !== "assistant") return;
			if (turnStartTime === 0) return;

			const outputToks = (event.message as any).usage?.output ?? 0;
			const elapsedSec = (turnEndTime - turnStartTime) / 1000;
			const spd = elapsedSec > 0 && outputToks > 0 ? outputToks / elapsedSec : 0;

			if (spd > 0) {
				turnSpeeds.push({ tokens: outputToks, spd });
				cumulativeTokens += outputToks;
			}

			turnStartTime = 0;
			turnEndTime = 0;
		});

		// ── Footer ─────────────────────────────────────────────────────
		ctx.ui.setFooter((tui, theme, footerData) => {
			const unsub = footerData.onBranchChange(() => tui.requestRender());

			return {
				dispose: unsub,
				invalidate() {},
				render(width: number): string[] {
					// Weighted average tok/s across completed turns
					let avgSpeed = 0;
					if (turnSpeeds.length > 0) {
						const totalWeighted = turnSpeeds.reduce(
							(sum, t) => sum + t.tokens * t.spd,
							0,
						);
						avgSpeed = totalWeighted / cumulativeTokens;
					}

					// Live speed for current turn (show accent colour while streaming)
					let currentSpeed = 0;
					if (turnStartTime > 0) {
						// Use last known average as a stand-in while streaming
						currentSpeed = avgSpeed;
					}

					// Build speed string
					let speedStr: string;
					if (currentSpeed > 0) {
						speedStr = theme.fg("accent", formatSpeed(currentSpeed));
					} else if (avgSpeed > 0) {
						speedStr = theme.fg("dim", formatSpeed(avgSpeed));
					} else {
						speedStr = theme.fg("dim", "— tok/s");
					}

					// Token counts, cost, and compaction count from session
					let inputTokens = 0;
					let outputTokens = 0;
					let sessionCost = 0;
					let compactionCount = 0;
					for (const e of ctx.sessionManager.getBranch()) {
						if (e.type === "compaction") {
							compactionCount++;
						} else if (e.type === "message" && e.message.role === "assistant") {
							const u = (e.message as any).usage;
							if (u) {
								inputTokens += u.input ?? 0;
								outputTokens += u.output ?? 0;
								sessionCost += u.cost?.total ?? 0;
							}
						}
					}

					const fmt = (n: number) =>
						n < 1000 ? `${n}` : `${(n / 1000).toFixed(1)}k`;

					// Session cost estimate (pre-calculated in usage.cost.total)
					let costStr = "";
					if (sessionCost > 0) {
						const costColor =
							sessionCost >= 5 ? "error" : sessionCost >= 1 ? "warning" : "dim";
						const costLabel =
							sessionCost < 0.01 ? "<$0.01" : `$${sessionCost.toFixed(2)}`;
						costStr = theme.fg(costColor, costLabel);
					}

					// Compaction count
					const compactStr =
						compactionCount > 0
							? theme.fg("warning", `↺${compactionCount}`)
							: "";

					// Context window usage
					const ctxUsage = ctx.getContextUsage();
					let ctxStr = "";
					if (ctxUsage) {
						const fmtCtxWindow = (n: number) =>
							n >= 1_000_000
								? `${(n / 1_000_000).toFixed(n % 1_000_000 === 0 ? 0 : 1)}M`
								: `${(n / 1000).toFixed(0)}k`;
						const windowStr = fmtCtxWindow(ctxUsage.contextWindow);
						if (ctxUsage.percent !== null) {
							const pct = ctxUsage.percent;
							const color =
								pct >= 90 ? "error" : pct >= 70 ? "warning" : "dim";
							ctxStr =
								theme.fg(color, `${Math.round(pct)}%`) +
								theme.fg("dim", `/${windowStr}`);
						} else {
							ctxStr = theme.fg("dim", `?/${windowStr}`);
						}
					}

					const sep = theme.fg("dim", " · ");
					const left = [
						ctxStr,
						theme.fg("dim", `↑${fmt(inputTokens)} ↓${fmt(outputTokens)}`) + " " + speedStr,
						costStr,
						compactStr,
					].filter(Boolean).join(sep);

					const branch = footerData.getGitBranch();
					const branchStr = branch ? ` (${branch})` : "";
					const right = theme.fg(
						"dim",
						`${ctx.model?.id || "no-model"}${branchStr}`,
					);

					const pad = " ".repeat(
						Math.max(1, width - visibleWidth(left) - visibleWidth(right)),
					);
					return [truncateToWidth(left + pad + right, width)];
				},
			};
		});
	});

	pi.registerCommand("tokspeed", {
		description: "Toggle token speed display in footer",
		handler: async (_args, ctx) => {
			enabled = !enabled;
			if (enabled) {
				ctx.ui.notify("Token speed display enabled", "info");
			} else {
				ctx.ui.setFooter(undefined);
				ctx.ui.notify("Token speed disabled (default footer restored)", "info");
			}
		},
	});
}
