#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0332
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

Task: t0332 (milestone MS-NewTargets, CSS style correctness). Expand the \`border-color\` and \`border-style\` shorthand properties into their four per-edge longhands, mirroring the EXISTING \`border-width\` shorthand handling. Today these two shorthands have NO dedicated arm, so they fall through to the generic \`name => { properties.insert(name, value) }\` arm and are stored under the raw key \`border-color\` / \`border-style\`. The paint path resolves a single border color via \`get_border_color\` -> \`find_color\`, which returns only the FIRST color token. So \`border-color: red green blue yellow;\` (top/right/bottom/left) renders ALL FOUR edges red instead of per-edge colors. This is wrong on real Wiki/News tables and boxes.

CONFIRMED GAP (verified by orchestrator): in \`src/style/mod.rs\`, the declaration-application match block (around lines 196-298) has dedicated arms for \`\"margin\"\`, \`\"padding\"\`, and \`\"border-width\"\` (line ~212). The \`\"border-width\" =>\` arm is the EXACT template: it calls \`expand_1_to_4(&value)\` and inserts \`border-top/right/bottom/left-width\`. There is NO \`\"border-color\"\` or \`\"border-style\"\` arm, so both fall through to the generic arm (line ~296). Paint's \`get_edge_color\` (\`src/paint/mod.rs\` ~241) already reads \`border-top-color\` etc. per edge BEFORE the shorthand fallback, so populating these per-edge longhands is sufficient for color — NO paint change is needed. (\`border-style\` longhands are not yet consumed by paint; populating them is still the correct style behavior and matches the \`border\` shorthand already doing so — leave a \`// TODO(spec):\` noting paint does not yet honor per-edge border-style.)

WHAT TO DO (ONE MODULE — src/style/ ONLY):
1. Add a new \`\"border-color\" =>\` match arm IMMEDIATELY mirroring the existing \`\"border-width\" =>\` arm: call \`expand_1_to_4(&value)\` and insert \`border-top-color\`, \`border-right-color\`, \`border-bottom-color\`, \`border-left-color\`.
2. Add a new \`\"border-style\" =>\` match arm the same way: \`expand_1_to_4(&value)\` -> \`border-top-style\`, \`border-right-style\`, \`border-bottom-style\`, \`border-left-style\`.
   Reuse \`expand_1_to_4\` EXACTLY as \`border-width\` does — do NOT write a new expansion helper. Place both new arms next to \`\"border-width\"\` for readability. Add a \`// TODO(spec):\` on the \`border-style\` arm noting that paint does not yet honor per-edge \`border-style\` (dashed/dotted/none suppression is a separate paint task — OUT OF SCOPE).

SCOPE — STRICT, ONE MODULE. \`git diff --name-only origin/main...HEAD\` MUST list ONLY files under \`src/style/\` (plus any var/ PNG+html, which are fine). Do NOT touch paint/, css/, layout/, engine/, or any other module. Do NOT modify, delete, weaken, rename, or \`#[ignore]\` ANY existing test anywhere. Do NOT add dependencies. If you think you must do any of these, STOP, leave a \`// TODO(spec):\` note, and report the blocker — do not expand scope.

REQUIRED NEW TEST (in src/style/mod.rs \`#[cfg(test)] mod tests\`, mirror the existing border/border-width tests): name it \`test_border_color_and_style_shorthands_expand_per_edge\`. Parse \`div { border-color: red green blue yellow; border-style: solid dashed dotted double; }\` via \`parse_stylesheet\`, compute styles for a \`<div>\`, and assert per-edge: \`border-top-color\` == what \`div { border-top-color: red; }\` produces, \`border-right-color\` == green, \`border-bottom-color\` == blue, \`border-left-color\` == yellow; and \`border-top-style\` == solid, \`border-right-style\` == dashed, \`border-bottom-style\` == dotted, \`border-left-style\` == double. Add a second case \`div { border-color: red green; }\` asserting top==bottom==red and right==left==green (the 2-value 1-to-4 rule). This test MUST fail before your change and pass after.

GATES (all must pass before commit):
- \`cargo test\` (entire suite) green — including your new test and ALL existing tests.
- \`cargo clippy --all-targets -- -D warnings\` clean (NO \`unwrap\`/\`expect\` in non-test production code — I-6).
- \`cargo fmt\` then \`cargo fmt --check\` clean.

VERIFIED-IN-WINDOW (REQUIRED — this affects rendering): after gates pass, create \`var/t0332-border.html\` containing exactly \`<html><body><div style=\"border-width: 10px; border-style: solid; border-color: #ff0000 #00ff00 #0000ff #ffff00; width: 80px; height: 40px;\">x</div></body></html>\` (\`mkdir -p var\` first), then render the shipping-path PNG and SAVE it:
\`cargo run --example render_local_png -- /workspaces/wt/t0332/var/t0332-border.html --width 160 --height 100 --out /workspaces/wt/t0332/var/t0332-border.png\`
The PNG MUST show FOUR DIFFERENT edge colors: RED top, GREEN right, BLUE bottom, YELLOW left (NOT all red). Report the saved PNG path in your summary. (The orchestrator will independently re-verify this PNG before merge.)

COMMIT: after ALL gates pass AND the PNG is saved, \`git add -A\` then \`git commit -m \"feat(style): expand border-color and border-style shorthands to per-edge longhands (t0332)\"\`. Then run \`git diff --name-only origin/main...HEAD\` and confirm ONLY files under \`src/style/\` (plus the var/ PNG+html) are listed. Do NOT push and do NOT open a PR — the orchestrator reviews, gates, and merges. Report a concise summary: the one-module change you made, the reused helper + new test, gate results, and the saved PNG path." \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  < /dev/null
