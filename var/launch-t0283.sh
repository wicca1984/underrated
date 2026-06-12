#!/usr/bin/env bash
# Launcher for Gemini worker t0283 — percentage max-width/min-width clamping in block layout (src/layout/mod.rs).
# Dispatched via setsid so it survives the orchestrator tick (memory: worker-dispatch-must-setsid).
set -euo pipefail

WT=/workspaces/wt/t0283
LOG=/workspaces/toy-browser/var/worker-logs/t0283.log

read -r -d '' PROMPT <<'EOF' || true
You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow ALL of it (especially I-1..I-7).

Task: t0283 — Resolve percentage `max-width` and `min-width` against the containing block in block layout.
Read: docs/ARCHITECTURE.md and docs/SPEC.md (layout / box model) under /workspaces/underrated-meta/.
Target module: src/layout/mod.rs ONLY. Do NOT touch any other module or any file outside src/layout/mod.rs (no lib.rs additions), and do NOT touch any other worktree. Do NOT modify the css parser.

Background (verify before changing):
- `fn clamp_width(style, mut width) -> f32` (around src/layout/mod.rs:672) currently clamps ONLY when `max-width`/`min-width` is `CssValue::Length(_, LengthUnit::Px)`. Percentage values (`CssValue::Length(_, LengthUnit::Percent)` — confirm the exact percent variant name used in this codebase, e.g. `LengthUnit::Percent`/`Pct`; grep for how percentage widths are resolved for the `width` property in this same file) are silently ignored. This breaks responsive centered columns (e.g. a container with `max-width: 90%`), contributing to a Google-top-page layout regression where the central search column fails to be width-constrained and spans the full viewport.
- `clamp_width` is called at two sites (search `clamp_width(`), both inside `resolve_margins_and_width`-adjacent code where the containing block width is in scope as `containing_width`.

Goal:
- Make `clamp_width` also honor percentage `max-width` and `min-width`, resolving the percentage against the containing block width (CSS2 §10.4: percentages resolve against the width of the containing block).
- To do this, thread the containing block width into `clamp_width`: change its signature to `fn clamp_width(style: &ComputedStyle, mut width: f32, containing_width: f32) -> f32` and pass `containing_width` at BOTH call sites (it is already in scope there).
- For `max-width`: if the value is a percentage `p`, the resolved max is `containing_width * p / 100.0`; clamp `width` down to it. Keep the existing px behavior unchanged.
- For `min-width`: if the value is a percentage `p`, the resolved min is `containing_width * p / 100.0`; clamp `width` up to it. Keep the existing px behavior unchanged.
- CSS precedence (CSS2 §10.4): min-width wins over max-width when they conflict — preserve the current ordering (max clamp first, then min clamp) which already yields min-beats-max. Do not change height clamping (`clamp_height`) in this task.
- Mirror exactly how the existing code reads a percentage length for `width` (use the same `CssValue`/`LengthUnit` variants and the same helper, e.g. a `get_percent`-style accessor if one exists, otherwise match on `CssValue::Length(v, LengthUnit::Percent)`). If no percent variant or accessor exists, do NOT invent parser changes — leave a `// TODO(spec):` and report it instead.
- No `unwrap()`/`expect()` in non-test code (I-6). Mirror neighboring error/Option handling.

Approach: test-first (TDD).
Acceptance (must be green) — add unit tests in the existing `#[cfg(test)]` module of src/layout/mod.rs, copying the style of the existing `max-width-test` (search for `max-width-test` / the test that exercises px max-width clamping):
- A block with `width: 1000px; max-width: 50%` inside an 800px containing block resolves to content width 400px (50% of 800).
- A block with `width: 100px; min-width: 50%` inside an 800px containing block resolves to content width 400px.
- Keep an existing or add a px max-width assertion to prove the px path is unchanged.
- Keep ALL existing layout tests green (do not weaken or delete any existing assertion or test — deleting/altering foreign tests to force green is a hard violation).

Done when: `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check` ALL pass.
Commit (you MUST git add + git commit before finishing — uncommitted work is lost):
  `feat(layout): resolve percentage max-width/min-width against containing block (t0283)`
Comments and identifiers in English.
If the spec is ambiguous or conflicts with real browser behavior, do NOT guess — leave a `// TODO(spec):` and report it.
End with a short summary of exactly what changed and the test names you added.
EOF

cd "$WT"
exec gemini -p "$PROMPT" \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  < /dev/null > "$LOG" 2>&1
