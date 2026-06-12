#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0329
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

Task: t0329 (milestone MS-NewTargets, layout correctness). Fix a CONFIRMED visual regression in the SHIPPING render path: an empty line produced by a leading or directly-consecutive \`<br>\` COLLAPSES to zero height instead of reserving one line of vertical space.

CONFIRMED REPRO (already verified by the orchestrator via the shipping path \`cargo run --example render_local_png\`):
- BROKEN: \`<html><body>A<br><br>B</body></html>\` renders as just two stacked lines \"A\" then \"B\" — the empty line between them is GONE. Expected: three lines — \"A\", a BLANK line, then \"B\" (i.e. B is advanced by 2x line-height below A).
- ALSO BROKEN: \`A<br><br>Line C<br>Line D\` (br-br immediately after the first text run) drops the blank line.
- ALREADY WORKING (do NOT regress): \`line one<br>line two<br><br>line four\` correctly preserves the blank line. There is an existing test \`test_consecutive_br_block_advance\` in src/layout/mod.rs that covers this WORKING case and MUST keep passing.

ROOT-CAUSE DIRECTION (verify before coding): the existing fix (commit t0327) only handled the case where the empty \`<br>\` line is sandwiched between two \`<br>\`-terminated content lines. An empty line box that has NO inline content (the leading/standalone consecutive-\`<br>\` case) still gets zero height and collapses. The fix is to make an empty line box created by a \`<br>\` reserve exactly one line-height of vertical advance, in the same layout path \`layout_document\` uses.

SCOPE — STRICT, ONE MODULE. You may ONLY edit files under \`src/layout/\` (expected: \`src/layout/mod.rs\` and/or \`src/layout/inline.rs\`). \`git diff --name-only origin/main...HEAD\` MUST list ONLY files under \`src/layout/\`. Do NOT touch net/, engine/, loader/, script/, text/, font/, or any other module. Do NOT modify, delete, weaken, rename, or \`#[ignore]\` ANY existing test anywhere — especially \`test_consecutive_br_block_advance\`. Do NOT add dependencies. If you think you must do any of these, STOP, leave a \`// TODO(spec):\` note, and report the blocker — do not expand scope.

REQUIRED NEW TEST (in src/layout/mod.rs \`#[cfg(test)] mod tests\`, mirror the style of \`test_consecutive_br_block_advance\`): name it \`test_leading_consecutive_br_empty_line\`. Build the DOM for \`<html><body>A<br><br>B</body></html>\` via \`parse_document\` + \`compute_styles\` + \`layout_document(&dom, &styles, 800.0)\`. Assert that the empty line between \"A\" and \"B\" reserves vertical space: the box for \"B\" must have \`origin.y >= y_of_A + 2.0 * line_height - EPSILON\` where \`line_height = 8.0\` (the builtin bitmap font line height). Use the same EPSILON / structure-navigation idiom the existing test uses. This test MUST fail before your fix and pass after.

GATES (all must pass before commit):
- \`cargo test\` (entire suite) green — including the existing \`test_consecutive_br_block_advance\` and your new \`test_leading_consecutive_br_empty_line\`.
- \`cargo clippy --all-targets -- -D warnings\` clean (NO \`unwrap\`/\`expect\` in non-test production code — I-6).
- \`cargo fmt\` then \`cargo fmt --check\` clean.

VERIFIED-IN-WINDOW (REQUIRED — this is a rendering task): after gates pass, render the shipping-path PNG and SAVE it:
\`cargo run --example render_local_png -- /workspaces/wt/t0329/var/t0329-br.html --width 200 --height 80 --out /workspaces/wt/t0329/var/t0329-br.png\`
where the html file contains exactly \`<html><body>A<br><br>B</body></html>\`. Create the html file first (\`mkdir -p var\`). The PNG MUST show three lines: \"A\", a blank gap, then \"B\". Report the saved PNG path in your summary. (The orchestrator will independently re-verify this PNG before merge.)

COMMIT: after ALL gates pass AND the PNG is saved, \`git add -A\` then \`git commit -m \"fix(layout): reserve blank line for leading/standalone consecutive <br> in shipping path (t0329)\"\`. Then run \`git diff --name-only origin/main...HEAD\` and confirm ONLY files under \`src/layout/\` (plus the var/ PNG+html which are fine) are listed. Do NOT push and do NOT open a PR — the orchestrator reviews, gates, and merges. Report a concise summary: the root cause you found, the one-module change you made, the new test, gate results, and the saved PNG path." \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  < /dev/null
