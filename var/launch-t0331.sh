#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0331
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
First, read the entire AGENTS.md passed via --include-directories and follow all of it (esp. I-1..I-7). One task = one module/concern.

Task: t0331 (milestone MS-NewTargets, CSS style correctness). Expand the \`border\` shorthand property so that its COLOR and STYLE longhands are populated, not just the width. Right now \`border: 1px solid #ccc;\` only sets the border-*-width longhands; the color (#ccc) and line-style (solid) are DROPPED. The paint path draws a border whenever a border width is > 0, but it reads the color from \`border-*-color\` (falling back to text color/black when unset). So today \`border: 1px solid #ccc\` renders as a BLACK border instead of light-gray. This is wrong on most real Wiki/News pages.

CONFIRMED GAP (verified by orchestrator): in \`src/style/mod.rs\`, the declaration-application match block has a \`\"border\" =>\` arm (around line 220) that calls \`find_border_width\` and inserts only \`border-top/right/bottom/left-width\`. It ends with \`// TODO(spec): border-style, border-color, etc.\`. The neighboring \`\"outline\" =>\` arm (just below, ~line 238) is the exact template: it uses \`find_outline_style\` and \`find_outline_color\` and inserts \`outline-style\`/\`outline-color\`. The paint path (\`src/paint/mod.rs\`) already consumes \`border-top-color\` etc. via \`get_edge_color\`, so populating these longhands is sufficient — NO paint change is needed.

WHAT TO DO (ONE MODULE — src/style/ ONLY):
1. In the existing \`\"border\" =>\` arm, after inserting the border widths, also extract the color and the line-style from the shorthand value and insert the four per-edge longhands for each:
   - color -> \`border-top-color\`, \`border-right-color\`, \`border-bottom-color\`, \`border-left-color\`
   - style -> \`border-top-style\`, \`border-right-style\`, \`border-bottom-style\`, \`border-left-style\`
   Reuse the EXISTING helpers \`find_outline_color\` and \`find_outline_style\` (border and outline share the same <color> and <line-style> grammar — do NOT duplicate a named-color table or a style-keyword list). If no color token is present, do not insert any \`border-*-color\` (leave the existing fallback behavior). If no style token is present, do not insert any \`border-*-style\`.
   - Keep the existing width logic exactly as-is (including the medium fallback). Only ADD the color/style insertions.
2. Replace the \`// TODO(spec): border-style, border-color, etc.\` line: keep a \`// TODO(spec):\` only for what remains out of scope (e.g. per-edge differing values, \`border-image\`). Paint honoring \`border-style: none\` to suppress drawing is OUT OF SCOPE (paint module) — leave a \`// TODO(spec):\` note for it, do NOT touch paint.

SCOPE — STRICT, ONE MODULE. \`git diff --name-only origin/main...HEAD\` MUST list ONLY files under \`src/style/\` (plus any var/ PNG+html, which are fine). Do NOT touch paint/, css/, layout/, engine/, or any other module. Do NOT modify, delete, weaken, rename, or \`#[ignore]\` ANY existing test anywhere. Do NOT add dependencies. If you think you must do any of these, STOP, leave a \`// TODO(spec):\` note, and report the blocker — do not expand scope.

REQUIRED NEW TEST (in src/style/mod.rs \`#[cfg(test)] mod tests\`, mirror the existing border/background-color tests): name it \`test_border_shorthand_expands_color_and_style\`. Parse \`div { border: 1px solid red; }\` via \`parse_stylesheet\`, compute styles for a \`<div>\`, and assert: \`border-top-width\` is the 1px length (as the existing width test asserts), \`border-top-color\` equals what \`div { border-top-color: red; }\` produces, and \`border-top-style\` equals what \`div { border-top-style: solid; }\` produces (check all four edges for at least color). Add a second case \`div { border: 2px dashed #ccc; }\` asserting \`border-left-color\` is the #ccc color and \`border-bottom-style\` is dashed. This test MUST fail before your change and pass after.

GATES (all must pass before commit):
- \`cargo test\` (entire suite) green — including your new test and ALL existing tests.
- \`cargo clippy --all-targets -- -D warnings\` clean (NO \`unwrap\`/\`expect\` in non-test production code — I-6).
- \`cargo fmt\` then \`cargo fmt --check\` clean.

VERIFIED-IN-WINDOW (REQUIRED — this affects rendering): after gates pass, create \`var/t0331-border.html\` containing exactly \`<html><body><div style=\"border: 10px solid #ff0000; width: 80px; height: 40px;\">x</div></body></html>\` (\`mkdir -p var\` first), then render the shipping-path PNG and SAVE it:
\`cargo run --example render_local_png -- /workspaces/wt/t0331/var/t0331-border.html --width 160 --height 100 --out /workspaces/wt/t0331/var/t0331-border.png\`
The PNG MUST show a RED border (not black) around the box. Report the saved PNG path in your summary. (The orchestrator will independently re-verify this PNG before merge.)

COMMIT: after ALL gates pass AND the PNG is saved, \`git add -A\` then \`git commit -m \"feat(style): expand border shorthand to color and style longhands (t0331)\"\`. Then run \`git diff --name-only origin/main...HEAD\` and confirm ONLY files under \`src/style/\` (plus the var/ PNG+html) are listed. Do NOT push and do NOT open a PR — the orchestrator reviews, gates, and merges. Report a concise summary: the one-module change you made, the reused helpers + new test, gate results, and the saved PNG path." \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  < /dev/null
