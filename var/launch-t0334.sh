#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0334
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

Task: t0334 (milestone MS-NewTargets, list-marker rendering correctness). This DIRECTLY EXTENDS the just-merged t0333 disc-marker fix. Today, the layout stage emits a Unicode bullet glyph per \`list-style-type\`: \`disc\` -> U+2022 (\`\u{2022}\`), \`circle\` -> U+25E6 (\`\u{25E6}\`), \`square\` -> U+25AA (\`\u{25AA}\`) (see \`src/layout/mod.rs\` around line 460). The built-in font is ASCII-only, so ALL THREE render as empty 'tofu' boxes. t0333 fixed ONLY \`disc\` (U+2022) in the paint stage by drawing a filled \`SolidRect\` instead of the glyph. \`circle\` (U+25E6) and \`square\` (U+25AA) STILL render as tofu — confirmed by the t0333 TODO comment which explicitly says 'circle/square ... remain out of scope'. Nested lists on real Wiki pages use \`circle\` and \`square\` at deeper levels, so this matters for MS-NewTargets real-page completeness.

CONFIRMED GAP (verified by orchestrator): in \`src/paint/mod.rs\` at line ~1060, the marker interception is:
  \`let is_marker = layout_box.text.as_deref() == Some(\\\"\u{2022}\\\");\`
It matches ONLY U+2022. The \`if is_marker { ... }\` block (lines ~1062-1072) computes a centered square \`s\` (clamped 2.0..10.0) and pushes ONE filled \`DisplayItem::SolidRect\`. U+25E6 and U+25AA fall through to the \`else\` text-painting branch and become tofu.

WHAT TO DO (ONE MODULE — src/paint/ ONLY):
1. Generalize the marker detection. Replace the single-codepoint check so it recognizes ALL THREE list-marker codepoints: U+2022 (\`\u{2022}\`, disc), U+25E6 (\`\u{25E6}\`, circle), U+25AA (\`\u{25AA}\`, square). Capture WHICH marker it is (e.g. match \`layout_box.text.as_deref()\` into a small local enum or just keep two booleans like \`is_filled_marker\` for disc/square vs \`is_hollow_marker\` for circle). Keep the existing centered-square geometry (\`s\`, \`x\`, \`y\`) EXACTLY as-is — reuse it for all three.
2. RENDERING per marker type, using ONLY the existing \`DisplayItem::SolidRect\` primitive (do NOT add a new DisplayItem variant — paint has no ellipse/stroke primitive):
   - \`disc\` (U+2022) and \`square\` (U+25AA): push ONE filled \`SolidRect\` of size \`s\` at (x, y) — IDENTICAL to today's disc path. (A filled square is already the pragmatic ASCII-safe stand-in for disc; it is the exact correct shape for square.)
   - \`circle\` (U+25E6): draw a HOLLOW square outline (a ring stand-in, visually distinct from the filled disc/square) by pushing FOUR thin \`SolidRect\` edges around the \`s\`-by-\`s\` box: top edge, bottom edge, left edge, right edge, each with thickness \`t = (s * 0.25).clamp(1.0, 2.0)\`. All four use \`scale_color_alpha(&color, effective_opacity)\` exactly like the current filled marker. Ensure no negative/zero-size rects (guard if \`s\` is tiny — if \`s <= 2.0 * t\` just fall back to the filled square so the marker never disappears).
3. Update the existing \`// TODO(spec):\` comment on that block: now disc=filled-square stand-in, square=filled-square (correct), circle=hollow-square stand-in; a true round disc/circle still needs a filled/stroked-ellipse primitive (OUT OF SCOPE), and image markers + \`list-style-position: inside\` remain out of scope.

SCOPE — STRICT, ONE MODULE. \`git diff --name-only origin/main...HEAD\` MUST list ONLY files under \`src/paint/\` (plus any var/ PNG+html, which are fine). Do NOT touch layout/, style/, css/, engine/, or any other module — in particular do NOT change the codepoints layout emits. Do NOT modify, delete, weaken, rename, or \`#[ignore]\` ANY existing test anywhere (the existing \`test_ul_disc_marker_paints_solid_rect_not_tofu_glyph\` MUST still pass unchanged). Do NOT add dependencies. If you think you must do any of these, STOP, leave a \`// TODO(spec):\` note, and report the blocker — do not expand scope.

REQUIRED NEW TEST (in src/paint/mod.rs \`#[cfg(test)] mod tests\`, mirror the existing \`test_ul_disc_marker_paints_solid_rect_not_tofu_glyph\`): name it \`test_ul_circle_and_square_markers_paint_solid_rect_not_tofu\`. Build a small DOM with a \`<ul style=\\\"list-style-type: circle\\\">\` containing an \`<li>\` with text, and another with \`list-style-type: square\`, run layout + build the display list, and assert: (1) NO \`DisplayItem::Text\` item has \`text\` equal to \`\u{25E6}\` or \`\u{25AA}\` (no tofu glyphs leak through), and (2) at least one \`DisplayItem::SolidRect\` is emitted for each as the marker (positioned to the LEFT of the list-item content, same assertion style as the disc test). This test MUST fail before your change and pass after.

GATES (all must pass before commit):
- \`cargo test\` (entire suite) green — including your new test and ALL existing tests (esp. the disc test).
- \`cargo clippy --all-targets -- -D warnings\` clean (NO \`unwrap\`/\`expect\` in non-test production code — I-6).
- \`cargo fmt\` then \`cargo fmt --check\` clean.

VERIFIED-IN-WINDOW (REQUIRED — this affects rendering): after gates pass, create \`var/t0334-markers.html\` (\`mkdir -p var\` first) containing exactly:
\`<html><body><ul style=\\\"list-style-type: disc\\\"><li>disc</li></ul><ul style=\\\"list-style-type: circle\\\"><li>circle</li></ul><ul style=\\\"list-style-type: square\\\"><li>square</li></ul></body></html>\`
then render the shipping-path PNG and SAVE it:
\`cargo run --example render_local_png -- /workspaces/wt/t0334/var/t0334-markers.html --width 240 --height 160 --out /workspaces/wt/t0334/var/t0334-markers.png\`
The PNG MUST show THREE visible markers (NONE blank/tofu): a filled square for disc, a hollow square outline for circle, and a filled square for square. Report the saved PNG path in your summary. (The orchestrator will independently re-verify this PNG before merge.)

COMMIT: after ALL gates pass AND the PNG is saved, \`git add -A\` then \`git commit -m \\\"fix(paint): draw filled/hollow markers for circle and square list bullets instead of tofu (t0334)\\\"\`. Then run \`git diff --name-only origin/main...HEAD\` and confirm ONLY files under \`src/paint/\` (plus the var/ PNG+html) are listed. Do NOT push and do NOT open a PR — the orchestrator reviews, gates, and merges. Report a concise summary: the one-module change, the new test, gate results, and the saved PNG path." \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  < /dev/null
