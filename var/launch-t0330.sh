#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0330
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

Task: t0330 (milestone MS-NewTargets, CSS style correctness). Implement the \`background\` shorthand property so that at minimum its color longhand is expanded to \`background-color\`. Right now \`background: blue;\` is dropped (only the explicit longhand \`background-color: blue;\` works), so real-world pages that use the shorthand render with no background fill.

CONFIRMED GAP (verified by orchestrator): in \`src/style/mod.rs\`, the declaration-application match block (around line 196, alongside the existing \`\"margin\"\`, \`\"padding\"\`, \`\"border-width\"\`, \`\"border\"\`, \`\"outline\"\` arms) has NO arm for \`\"background\"\`. There is an explicit \`// TODO(spec): other shorthand properties like background, font, transition, etc.\` at ~line 251. The paint path (\`src/paint/mod.rs\`) only reads \`background-color\`, so the shorthand never produces a fill.

WHAT TO DO (ONE MODULE — src/style/ ONLY):
1. Add a \`\"background\" =>\` arm in that same match block. Extract the color token from the shorthand value list and insert it as the \`background-color\` longhand (mirror the existing \`\"outline\"\` arm + its \`find_outline_color\` helper at ~line 1228). Add a sibling helper \`find_background_color(value: &CssValue) -> Option<CssValue>\` that returns the first \`CssValue::Color(..)\` (and recognizes named-color keywords the same way the outline/border color finders do — reuse whatever color-detection helper they use; do NOT duplicate a named-color table). If no color is present in the shorthand, do nothing for color (leave other background longhands untouched — they are out of scope).
2. Do NOT attempt to expand the other background longhands (image/position/repeat/size/etc.) — leave a \`// TODO(spec):\` note for those. Keep this minimal and correct.

SCOPE — STRICT, ONE MODULE. \`git diff --name-only origin/main...HEAD\` MUST list ONLY files under \`src/style/\` (plus any var/ PNG+html, which are fine). Do NOT touch paint/, css/, layout/, engine/, or any other module. Do NOT modify, delete, weaken, rename, or \`#[ignore]\` ANY existing test anywhere. Do NOT add dependencies. If you think you must do any of these, STOP, leave a \`// TODO(spec):\` note, and report the blocker — do not expand scope.

REQUIRED NEW TEST (in src/style/mod.rs \`#[cfg(test)] mod tests\`, mirror the style of the existing background-color tests around line 2228/2366): name it \`test_background_shorthand_sets_background_color\`. Parse a stylesheet like \`div { background: blue; }\` via \`parse_stylesheet\`, compute styles for a \`<div>\`, and assert \`style.get(\"background-color\")\` equals the same value that \`div { background-color: blue; }\` produces. Add a second assertion that a shorthand with extra tokens (e.g. \`background: blue none;\`) still yields the color. This test MUST fail before your change and pass after.

GATES (all must pass before commit):
- \`cargo test\` (entire suite) green — including your new \`test_background_shorthand_sets_background_color\` and ALL existing tests.
- \`cargo clippy --all-targets -- -D warnings\` clean (NO \`unwrap\`/\`expect\` in non-test production code — I-6).
- \`cargo fmt\` then \`cargo fmt --check\` clean.

VERIFIED-IN-WINDOW (REQUIRED — this affects rendering): after gates pass, render the shipping-path PNG and SAVE it:
\`cargo run --example render_local_png -- /workspaces/wt/t0330/var/t0330-bg.html --width 200 --height 80 --out /workspaces/wt/t0330/var/t0330-bg.png\`
where the html file contains exactly \`<html><body><div style=\"background: blue; width: 100px; height: 40px;\">x</div></body></html>\`. Create the html file first (\`mkdir -p var\`). The PNG MUST show a solid blue rectangle. Report the saved PNG path in your summary. (The orchestrator will independently re-verify this PNG before merge.)

COMMIT: after ALL gates pass AND the PNG is saved, \`git add -A\` then \`git commit -m \"feat(style): expand background shorthand color longhand to background-color (t0330)\"\`. Then run \`git diff --name-only origin/main...HEAD\` and confirm ONLY files under \`src/style/\` (plus the var/ PNG+html) are listed. Do NOT push and do NOT open a PR — the orchestrator reviews, gates, and merges. Report a concise summary: the one-module change you made, the new helper + test, gate results, and the saved PNG path." \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  < /dev/null
