#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0303
eval "$(grep -m1 '^export GEMINI_API_KEY=' ~/.bashrc)"
mkdir -p /workspaces/underrated-meta/var/worker-logs
exec gemini -p "You are a Gemini worker on \`underrated\` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow all of it (esp. I-1..I-7).

Task: t0303 — implement CSS **aspect-ratio** sizing in the layout module.
Read: docs/SPEC.md and docs/ARCHITECTURE.md (under /workspaces/underrated-meta/).
Target module: src/layout/ (touch ONLY src/layout/mod.rs; do NOT touch other modules, lib.rs, src/css, src/paint, src/dom, or other worktrees).

WHY: \`aspect-ratio: W / H\` (and the single-number form \`aspect-ratio: 1.5\`) lets a box derive one dimension from
the other. Real pages (images, media embeds, cards) rely on it. We currently IGNORE the property entirely
(\`grep -rn 'aspect-ratio\|aspect_ratio' src\` → no matches). This is part of layout breadth (MS-NewTargets).

CURRENT STATE (verify before coding, REUSE existing helpers — do NOT add new modules/crates):
  - In src/layout/mod.rs around line 389: \`let border_box_height = clamp_height(style, get_px(style, \"height\", content_height))\`.
    This is where the box's height is finalized. The box width is already known at this point.
  - Study the existing helpers in this file: \`get_px(style, key, default)\`, \`clamp_height(...)\`, \`clamp_width(...)\`,
    and how \`style.get(\"...\")\` returns a parsed CSS value. Look at how an existing length/number property is read
    (e.g. how \`get_px\` or a numeric property like \`opacity\`/\`flex-grow\` parses its value) and MIRROR that parsing style.

SCOPE (in src/layout/mod.rs ONLY):
  1. Add a small helper \`fn get_aspect_ratio(style: &ComputedStyle) -> Option<f32>\` that reads \`style.get(\"aspect-ratio\")\`
     and returns width/height as a single f32:
       - \`aspect-ratio: 1.5\`  → Some(1.5)
       - \`aspect-ratio: 16 / 9\` (two numbers separated by '/') → Some(16.0/9.0)
       - \`auto\` or unset or non-positive → None.
     Parse from the value's textual/number form the SAME way other numeric props are parsed in this file
     (use \`.to_std_string().unwrap_or_default()\` style / existing number extraction; NO unwrap/expect in non-test code).
  2. Apply it ONLY when height is NOT explicitly set (i.e. height is auto). Concretely: when \`style.get(\"height\")\`
     is absent/auto AND aspect-ratio is Some(r), compute the height from the already-known box WIDTH:
     \`height = border_box_width / r\` (use the content/border width consistent with how content_height is used at line 389),
     and feed THAT through \`clamp_height\` instead of \`content_height\`. If height IS explicitly set, aspect-ratio is ignored
     (explicit height wins — do NOT override it).
  3. Keep the change surgical and localized to the height-resolution site (~line 389). Do not restructure layout.
     If width-from-height (the inverse case) is ambiguous in our model, leave a \`// TODO(spec):\` and implement ONLY the
     common width→height direction.

TESTS (in src/layout/mod.rs, #[cfg(test)], mirror existing layout unit tests in this file):
  - A block with a fixed width (e.g. 200px), height auto, \`aspect-ratio: 2 / 1\` → resulting border-box height ≈ 100px.
  - Single-number form \`aspect-ratio: 4\` with width 200px, height auto → height ≈ 50px.
  - A block with BOTH \`height: 30px\` and \`aspect-ratio: 2 / 1\` → height stays 30px (explicit height wins).
  - A block with no aspect-ratio behaves exactly as before (regression guard).
  Find how existing tests build a styled node + run layout and assert on the resulting box height; mirror that exactly.

CONSTRAINTS (AGENTS.md): no unwrap/expect in non-test code (I-6); no test skip/ignore (I-4); 1 task = 1 module (I-5);
do not edit the main tree, src/css, src/paint, src/dom, lib.rs, or other worktrees (I-3). Do not add new crates (I-1).

WORKFLOW:
  1. Implement in src/layout/mod.rs only.
  2. Run: cargo fmt; cargo clippy --all-targets -- -D warnings; cargo test (or at least the layout tests).
  3. \`git add -A && git commit\` with message:
     \`feat(layout): implement CSS aspect-ratio sizing (t0303)\`
     ending with the Co-Authored-By trailer required by AGENTS.md.
  4. Confirm \`git -C /workspaces/wt/t0303 status\` is clean and \`git diff --name-only origin/main..HEAD\` lists ONLY
     src/layout/mod.rs. COMMIT before you finish — do not leave work uncommitted." \
  -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta
