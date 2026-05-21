# Skills — AI Factory Pipeline

This directory contains the skill files for the **AI Factory pipeline** — a structured,
chip-design-inspired development workflow built into every nibble sandbox.

## 3-Tier Pipeline

The pipeline adapts its depth to the size and risk of each task:

```
Quick:     SPEC ──▶ IMPLEMENT ──▶ VERIFY ──────────────────────── done*
Standard:  SPEC ──▶ IMPLEMENT ──▶ VERIFY ──▶ AUDIT ───────────── done*
Full:      SPEC ──▶ IMPLEMENT ──▶ VERIFY ──▶ AUDIT ──▶ QA GATE ─ done

* QA Gate fires for ANY tier if Audit finds unfixed Critical/High findings.
```

| Tier | When | Stages |
|------|------|--------|
| **Quick** | ≤3 functions, no security/API change | Spec → Implement → Verify |
| **Standard** | 4–15 functions | Spec → Implement → Verify → Audit |
| **Full** | 16+ functions, security-sensitive, API changes | Full pipeline with QA Gate |

## Skills

| Skill | Purpose |
|-------|---------|
| `factory-pipeline` | Tier classification + orchestration. Load first. |
| `factory-spec` | Blueprint design with tiered templates (quick/standard/full). |
| `factory-verify` | Testing + adversarial analysis + risk scoring. Merged from previous TDD, adversarial, and risk-score skills. |
| `factory-qa-gate` | Human approval gate. Fires for Critical/High findings (any tier) or always (Full tier). |
| `factory-lessons` | Continuous improvement log. Loaded once at pipeline start, appended at pipeline end. |

## Directory layout

```
skills/
  factory-pipeline/SKILL.md    # Pipeline manifest — load first
  factory-spec/SKILL.md        # Spec stage (tiered templates)
  factory-verify/SKILL.md      # Verify + Audit stages
  factory-qa-gate/SKILL.md     # QA Gate stage
  factory-lessons/SKILL.md     # Continuous improvement log
```

Each file is plain markdown with YAML frontmatter (`name`, `description`) for skill discovery.

## Artifact locations (inside sandbox repos)

```
.nibble/factory/
  blueprints/                   # Spec blueprints (committed)
  reports/audit/                # Adversarial + risk findings (gitignored — stale after fixes)
  reports/qa/                   # QA gate decisions (committed — audit trail)
```

## Installation

`install.sh` copies all `skills/factory-*/SKILL.md` directories to `~/.claude/skills/`,
which Claude Code scans automatically.

## When to skip the pipeline

The pipeline is mandatory for non-trivial changes. It may be skipped (with human
confirmation) for typo fixes, single-value config tweaks, formatting/linting changes,
or documentation-only changes.
