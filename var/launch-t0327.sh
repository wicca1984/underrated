#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0327
# Auth: prefer canonical var/.env, fall back to bashrc export.
if grep -q '^export GEMINI_API_KEY=' /workspaces/underrated-meta/var/.env 2>/dev/null; then
  eval "$(grep -m1 '^export GEMINI_API_KEY=' /workspaces/underrated-meta/var/.env)"
elif grep -q '^GEMINI_API_KEY=' /workspaces/underrated-meta/var/.env 2>/dev/null; then
  export "$(grep -m1 '^GEMINI_API_KEY=' /workspaces/underrated-meta/var/.env)"
else
  eval "$(grep -m1 '^export GEMINI_API_KEY=' ~/.bashrc)"
fi
mkdir -p /workspaces/underrated-meta/var/worker-logs
LOG=/workspaces/underrated-meta/var/worker-logs/t0327-$(date -u +%Y%m%dT%H%M%SZ).log
exec gemini -p "You are a Gemini worker on \`underrated\` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow all of it (esp. I-1..I-7). One task = one module/concern.

Task: t0327 (milestone MS-NewTargets, parallel lane — inline layout correctness). REGRESSION FOLLOW-UP to t0326. t0326 added \`<br>\` forced line breaks, and its UNIT test passes (consecutive \`<br><br>\` produce 3 line boxes incl. an empty middle box). BUT the SHIPPING render path is still visually wrong: the empty line produced by consecutive \`<br><br>\` COLLAPSES — it is not visible.

CONFIRMED REPRODUCTION (already captured by the orchestrator):
- Fixture: var/t0326-br.html = \`<html><body>line one<br>line two<br><br>line four</body></html>\`.
- Rendering it through the shipping path produces a PNG where 'line one', 'line two', 'line four' appear on THREE consecutive lines with NO blank line between 'line two' and 'line four'. Correct behavior: \`line two<br><br>line four\` must leave ONE visible blank line, so 'line four' should sit on the 4th line with a visible gap.
- So although \`layout_inline_run\` (src/layout/inline.rs) returns an empty line box WITH height for the second \`<br>\`, that vertical advance is LOST somewhere between inline layout and paint (likely block-flow height aggregation / how line boxes feed block layout, or paint culling empty boxes). ROOT-CAUSE it — do not just trust the unit test.

INVESTIGATION (read before writing):
- src/layout/inline.rs: the \`<br>\` element handler and the final-line flush guarded by \`if !current_line_children.is_empty()\` near the end of \`layout_inline_run\`, and the returned \`(Vec<LayoutBox>, f32)\` (the f32 = content height = cursor_y).
- src/layout/block.rs (and src/layout/mod.rs): how the block container CONSUMES the line boxes returned by layout_inline_run — does it use the returned cursor_y height, or does it recompute height from non-empty boxes? Does it drop/skip empty line boxes (children empty) when stacking? This is the most likely defect site.
- The paint path only if the above are correct.

SCOPE — STRICT. This is the LAYOUT module only. You may modify files UNDER src/layout/ ONLY (most likely src/layout/block.rs and/or src/layout/inline.rs and/or src/layout/mod.rs). Do NOT modify anything outside src/layout/ (do NOT touch html/, css/, paint/, engine/, style/, font/, script/). Do NOT modify, delete, weaken, rename, or \`#[ignore]\` ANY existing test anywhere (including the t0326 multi-br test). Do NOT touch lib.rs or module registration. \`git diff --name-only origin/main...HEAD\` MUST list ONLY files under src/layout/ (plus, optionally, NEW fixture/PNG files under var/). If the fix genuinely requires touching a module OUTSIDE src/layout/, STOP, leave a \`// TODO(spec):\` note, and report the blocker — do NOT expand scope.

REQUIREMENTS:
1. Fix the root cause so that consecutive \`<br><br>\` leaves a visible blank line in the block-flow vertical positions (the empty line box's height must be carried into block layout, so the following content is pushed down by one full line height per empty line).
2. Add a Rust unit/integration test (in the most appropriate src/layout/ file) that exercises the BLOCK-LEVEL outcome: lay out \`line one<br>line two<br><br>line four\` through the block-flow entry point that the shipping path uses (NOT only layout_inline_run in isolation) and assert that the 4th text fragment's absolute y-offset is greater than the 3rd by at least ~2 line-heights relative to 'line two' (i.e. a blank line exists). The existing t0326 inline-only test must remain and keep passing.
3. Do NOT regress single \`<br>\` (one break = one new line, no extra blank), normal wrapping, or whitespace collapsing.

GATES (all must pass before commit):
- \`cargo test\` (entire suite) green.
- \`cargo clippy --all-targets -- -D warnings\` clean (NO \`unwrap\`/\`expect\` in non-test production code — I-6).
- \`cargo fmt\` then \`cargo fmt --check\` clean.
- \`cargo doc --no-deps\` clean.

VERIFIED-IN-WINDOW (REQUIRED — UI affecting): render var/t0326-br.html through the SHIPPING path to var/t0327-br-empty.png. Find the exact example by reading examples/ (it is \`render_local_png\`): e.g. \`cargo run --example render_local_png -- var/t0326-br.html var/t0327-br-empty.png\`. Open/verify the PNG: it MUST now show 'line one', 'line two', a BLANK line, then 'line four' (four line-slots, gap visible). Keep var/t0327-br-empty.png. If the example emits PPM, convert to PNG so the orchestrator can view it.

COMMIT: after ALL gates pass and the corrected PNG exists, \`git add -A\` then \`git commit -m \"fix(layout): preserve blank line from consecutive <br> in block flow (t0327)\"\`. Then run \`git diff --name-only origin/main...HEAD\` and confirm ONLY allowed paths are listed. Do NOT push and do NOT open a PR — the orchestrator reviews, gates, and merges. Report a concise summary: root cause, the file(s)/lines changed, gate results, and the PNG path." \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  < /dev/null >>"$LOG" 2>&1
