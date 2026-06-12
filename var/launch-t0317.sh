#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0317
# Auth: prefer canonical var/.env, fall back to bashrc export.
if grep -q '^export GEMINI_API_KEY=' /workspaces/underrated-meta/var/.env 2>/dev/null; then
  eval "$(grep -m1 '^export GEMINI_API_KEY=' /workspaces/underrated-meta/var/.env)"
elif grep -q '^GEMINI_API_KEY=' /workspaces/underrated-meta/var/.env 2>/dev/null; then
  export "$(grep -m1 '^GEMINI_API_KEY=' /workspaces/underrated-meta/var/.env)"
else
  eval "$(grep -m1 '^export GEMINI_API_KEY=' ~/.bashrc)"
fi
mkdir -p /workspaces/underrated-meta/var/worker-logs
exec gemini -p "You are a Gemini worker on \`underrated\` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow all of it (esp. I-1..I-7). One task = one module.

Task: t0317 — fix CSS unitless-zero lengths (\`width:0\`, \`height:0\`, etc.) being treated as \`auto\` in the LAYOUT module ONLY.
Target module: src/layout/mod.rs (touch ONLY this file; do NOT touch src/css, lib.rs, other modules' mod.rs, src/engine, src/paint, or other worktrees).

ROOT CAUSE (already diagnosed — do not re-investigate from scratch, just verify and fix):
Per the CSS spec, \`0\` is the ONLY length value allowed to omit its unit. The CSS resolver at src/css/resolve.rs:372 stores a unitless number token (e.g. \`0\`) as \`CssValue::Number(0.0)\`, NOT \`CssValue::Length(0.0, LengthUnit::Px)\`. The layout module only recognizes \`CssValue::Length(_, _)\` as a definite length:
  - \`has_definite_width\` at src/layout/mod.rs:823 matches only \`Some(CssValue::Length(_, _))\`, so for \`width:0\` it is FALSE and the block stretches to the full containing width (a stray full-width fragment).
  - \`get_px\` at src/layout/mod.rs:653 matches only \`Some(CssValue::Length(v, LengthUnit::Px))\` and otherwise returns the caller's default; so \`height:0\` falls back to the auto/content height.
You MUST NOT modify src/css/resolve.rs (that is a different module and changing how Number is stored there would break unitless properties like opacity/line-height/z-index). Fix it on the LAYOUT consumer side.

REPRO (confirm the bug first via a quick test, then fix):
HTML: \`<div style='width:0;height:20px;background-color:rgb(255,0,0)'></div>\` inside a body. Before the fix, layout gives this div content_width == full containing width (e.g. 400) instead of 0. \`width:0px\` (with unit) already correctly yields 0. The goal is to make unitless \`0\` behave identically to \`0px\`.

SCOPE (in src/layout/mod.rs ONLY):
  1. Teach \`get_px\` to treat an explicit unitless zero as a zero length: when the property value is \`CssValue::Number(n)\` with \`n == 0.0\`, return \`0.0\` (NOT the default). Keep the existing \`Length(v, Px)\` arm. Leave all other \`CssValue::Number(_)\` (non-zero, which are invalid as lengths) falling through to \`default\` as today.
  2. Teach \`has_definite_width\` (line ~823) to also treat \`Some(CssValue::Number(n))\` with \`n == 0.0\` as a definite width (so the \`else\` definite-width branch runs and content_width becomes 0). A non-zero unitless number must remain NON-definite (stays auto).
  3. Do a grep within src/layout/mod.rs for any other place that decides 'definite vs auto' for height/min-width/max-width by matching \`CssValue::Length\` directly (e.g. \`height_is_auto_or_absent\`, clamp_width/clamp_height callers). Apply the SAME unitless-zero handling ONLY where it is needed so that \`height:0\`, \`min-width:0\`, \`max-width:0\` behave like their \`0px\` forms. Do not change behavior for non-zero unitless numbers anywhere.
  4. Use a small float tolerance is unnecessary — these are literal \`0.0\` from the parser; compare with \`== 0.0\` (clippy may warn on float-cmp; if so, use \`n.abs() < f32::EPSILON\` or \`n == 0.0\` with an \`#[allow]\` ONLY if clippy forces it — prefer \`n == 0.0\` and only add allow if -D warnings fails).

TDD (write tests FIRST, then implement until green) — add \`#[test]\`s in the existing tests module of src/layout/mod.rs (follow the existing layout-test style that builds a DOM + styles and calls layout_node / the existing test helpers, then asserts on \`LayoutBox.rect.size\`):
  - A block with \`width:0\` has \`rect.size.width == 0.0\` (and assert that \`width:0px\` gives the same, as a control).
  - A block with \`height:0\` has \`rect.size.height == 0.0\`.
  - A control: a block with \`width:200px\` still has width 200, and a block with NO width is still auto (== containing width). A block with an invalid \`width:5\` (unitless non-zero) stays auto (NOT 5) — assert it equals the containing width, documenting that unitless non-zero is rejected.

VERIFIED-IN-WINDOW (REQUIRED — this changes rendering): after green, render the shipping path to a PNG and save it under var/:
  printf '%s' \"<!DOCTYPE html><html><head><style>body{margin:0;padding:0}.z{width:0;height:20px;background-color:rgb(255,0,0)}.s{width:120px;height:20px;background-color:rgb(0,128,255)}</style></head><body><div class=\\\"z\\\"></div><div class=\\\"s\\\"></div></body></html>\" > var/t0317-unitless-zero.html
  cargo run -q --example render_local_png -- var/t0317-unitless-zero.html --width 400 --height 120 --out var/t0317-unitless-zero.png
  The red \`width:0\` box must produce NO red pixels (zero-area, suppressed); only the blue 120px box renders. Confirm the PNG exists and mention its path in your final message.

When done: run \`cargo test\`, \`cargo clippy --all-targets -- -D warnings\`, \`cargo fmt --check\`, \`cargo doc --no-deps\` — ALL must be green. Then \`git add -A && git commit\` with message exactly:
  fix(layout): treat unitless zero (width:0/height:0) as a definite zero length (t0317)
Then print the final \`git log -1 --oneline\`, run \`git status --porcelain\` and confirm the working tree is clean (commit any leftover var/*.png/.html you created). Do NOT push or open a PR (the orchestrator handles that)." \
  -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta \
  > /workspaces/underrated-meta/var/worker-logs/t0317.log 2>&1
