#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0299
eval "$(grep -m1 '^export GEMINI_API_KEY=' ~/.bashrc)"
mkdir -p /workspaces/underrated-meta/var/worker-logs
exec gemini -p "You are a Gemini worker on \`underrated\` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow all of it (esp. I-1..I-7).

Task: t0299 — implement the box-shadow **spread radius** in the paint module (outer shadow only).
Read: docs/SPEC.md and docs/ARCHITECTURE.md (under /workspaces/underrated-meta/).
Target module: src/paint/ (touch ONLY src/paint/mod.rs; do NOT touch other modules, lib.rs, or other worktrees).

WHY: We already paint a single OUTER box-shadow as a SolidRect offset by offset-x/offset-y (see src/paint/mod.rs,
search for the comment \`// Paint box-shadow if present\`). The CSS box-shadow syntax is
\`offset-x offset-y [blur-radius] [spread-radius] [color]\`. Currently blur AND spread are ignored (there are
\`// TODO(spec): box-shadow v1 ... (blur/spread ignored ...)\` markers). This task adds spread-radius support ONLY.

CURRENT STATE (verify before coding, do NOT re-implement):
  - The box-shadow branch flattens the value, bails on none/inset/comma, then collects \`length_values\` (in order)
    and an optional color. It uses \`length_values[0]\` = offset-x and \`length_values[1]\` = offset-y, builds
    \`shadow_rect = layout_box.rect\` translated by the offsets, and pushes a SolidRect.
  - With 4 length values the order is: [0]=offset-x, [1]=offset-y, [2]=blur-radius, [3]=spread-radius.

SCOPE (in src/paint/mod.rs ONLY):
  1. When \`length_values.len() >= 4\`, treat \`length_values[3]\` as the spread radius \`spread\`. (blur = [2] stays
     ignored — keep a TODO(spec) noting blur is still unimplemented.)
  2. Apply spread by INFLATING the shadow rect on all four sides by \`spread\` BEFORE/AFTER the offset translation:
       origin.x -= spread; origin.y -= spread; size.width += 2*spread; size.height += 2*spread.
     A negative spread shrinks the rect (this is valid CSS). After inflation, if the resulting width or height is
     <= 0, do NOT push the shadow SolidRect (a fully-shrunk shadow is invisible). Keep the existing guard that the
     ORIGINAL border box has positive dimensions.
  3. When \`length_values.len() < 4\` (no spread given), behaviour is unchanged (spread = 0).
  4. Update/keep the TODO(spec) comments accurately: spread is now implemented; blur, inset, and multiple comma-
     separated shadows remain out of scope.

TESTS (in src/paint/mod.rs, #[cfg(test)], mirror the existing box-shadow tests near \`.shadowed { ... box-shadow: 5px 5px #ff0000; }\`):
  - Positive spread: e.g. \`box-shadow: 0px 0px 0px 10px #ff0000\` on a 100x50 green box. Assert the emitted shadow
    SolidRect is larger than the border box by 2*spread in each dimension (compare the shadow rect width/height to
    the box, or count red pixels and assert they exceed the no-spread case), and is centered on the box.
  - Negative spread that fully collapses the rect (e.g. spread = -100 on a 100x50 box -> width 100-200 < 0): assert
    NO red shadow rect is emitted.
  - Keep the existing offset-only and box-shadow:none tests passing.

CONSTRAINTS (AGENTS.md): no unwrap/expect in non-test code (I-6); no test skip/ignore (I-4); 1 task = 1 module (I-5);
do not edit the main tree or other worktrees (I-3). Use the existing CssValue/flatten_value/scale_color_alpha helpers
already imported in this file — do not add new crates (I-1).

WORKFLOW:
  1. Implement in src/paint/mod.rs only.
  2. Run: cargo fmt; cargo clippy --all-targets -- -D warnings; cargo test -p underrated paint (or the crate's full test).
  3. \`git add -A && git commit\` with message: \`feat(paint): support box-shadow spread radius for outer shadow (t0299)\`
     ending with the Co-Authored-By trailer required by AGENTS.md.
  4. Confirm \`git -C /workspaces/wt/t0299 status\` is clean and \`git diff --name-only origin/main..HEAD\` lists ONLY
     src/paint/mod.rs. COMMIT before you finish — do not leave work uncommitted.
" -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta 2>&1
