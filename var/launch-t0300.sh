#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0300
eval "$(grep -m1 '^export GEMINI_API_KEY=' ~/.bashrc)"
mkdir -p /workspaces/underrated-meta/var/worker-logs
exec gemini -p "You are a Gemini worker on \`underrated\` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow all of it (esp. I-1..I-7).

Task: t0300 — implement the non-solid **outline styles** (double, dotted, dashed) in the paint module.
Read: docs/SPEC.md and docs/ARCHITECTURE.md (under /workspaces/underrated-meta/).
Target module: src/paint/ (touch ONLY src/paint/mod.rs; do NOT touch other modules, lib.rs, or other worktrees).

WHY: We already paint outlines as four SOLID strips (search src/paint/mod.rs for the comment
\`// Paint outline if style is present and not none\`). The \`outline-style\` keyword is parsed by the style module
into \`outline-style\` (values: none/solid/double/dotted/dashed/...). Right now paint treats every non-none style as
SOLID — see the marker \`// TODO(spec): outline non-solid styles and color:invert are not implemented (solid frame, color falls back to text color).\`
This task adds DOUBLE, DOTTED and DASHED rendering for the outline. (color:invert stays out of scope — keep a TODO.)

CURRENT STATE (verify before coding, do NOT re-implement):
  - The outline branch resolves \`ow\` (outline-width), \`offset\` (outline-offset), the outline color, then pushes four
    SolidRect strips (top/right/bottom/left) forming a solid frame at distance \`d = offset + ow\` outside the border box.
  - There is ALREADY a helper \`fn paint_decoration_line(...)\` (search for it) used by text-decoration that renders
    solid/double/dotted/dashed lines as repeated SolidRects. STUDY it and REUSE the SAME dash/dot/double heuristics for
    visual consistency (do NOT invent new metrics; do NOT duplicate it wholesale — factor a small shared helper IF clean,
    otherwise mirror its dash/dot length and gap constants).

SCOPE (in src/paint/mod.rs ONLY):
  1. Read \`outline-style\` keyword. For \`solid\` keep the current four-strip behaviour UNCHANGED (regression-safe).
  2. \`double\`: render the frame as TWO concentric thin solid frames with a gap, total thickness == \`ow\`
     (mirror the double-line split used by paint_decoration_line; if ow is too small for 3 visible bands, fall back to solid).
  3. \`dotted\` / \`dashed\`: render each of the four edges as a row/column of repeated small SolidRects (dots = squares of
     side ~ow; dashes = rects ~3*ow long) with gaps, using the SAME length/gap constants as paint_decoration_line.
  4. Keep \`outline-offset\` (positive AND negative) and the existing color resolution (outline-color -> color -> black)
     working for every style. Keep the existing guard that strips with width<=0 or height<=0 are not pushed.
  5. Update the TODO(spec) comment: double/dotted/dashed now implemented; color:invert and \`groove/ridge/inset/outset\`
     remain out of scope (fall back to solid for those).

TESTS (in src/paint/mod.rs, #[cfg(test)], mirror the existing \`test_paint_outline\` and the
\`test_paint_text_decoration_style_dotted/dashed/double\` tests):
  - dotted/dashed: assert the emitted outline produces MANY small SolidRects of the outline color (more than the 4
    solid strips), i.e. each edge is segmented.
  - double: assert the outline color appears as two separated bands (more than 4 rects, arranged as two concentric frames).
  - solid regression: \`outline: 2px solid red\` still emits exactly 4 strips (keep the existing assertion green).
  - \`outline-style: none\` still emits zero outline rects.

CONSTRAINTS (AGENTS.md): no unwrap/expect in non-test code (I-6); no test skip/ignore (I-4); 1 task = 1 module (I-5);
do not edit the main tree or other worktrees (I-3). Reuse existing CssValue/Color/Rect/scale_color_alpha helpers
already in this file — do not add new crates (I-1).

WORKFLOW:
  1. Implement in src/paint/mod.rs only.
  2. Run: cargo fmt; cargo clippy --all-targets -- -D warnings; cargo test -p underrated paint (or the crate's full test).
  3. \`git add -A && git commit\` with message: \`feat(paint): implement outline double/dotted/dashed styles (t0300)\`
     ending with the Co-Authored-By trailer required by AGENTS.md.
  4. Confirm \`git -C /workspaces/wt/t0300 status\` is clean and \`git diff --name-only origin/main..HEAD\` lists ONLY
     src/paint/mod.rs. COMMIT before you finish — do not leave work uncommitted.
" -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta 2>&1
