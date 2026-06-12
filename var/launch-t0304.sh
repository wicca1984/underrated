#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0304
eval "$(grep -m1 '^export GEMINI_API_KEY=' ~/.bashrc)"
mkdir -p /workspaces/underrated-meta/var/worker-logs
exec gemini -p "You are a Gemini worker on \`underrated\` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow all of it (esp. I-1..I-7).

Task: t0304 — implement CSS **text-shadow** painting in the paint module.
Read: docs/SPEC.md and docs/ARCHITECTURE.md (under /workspaces/underrated-meta/).
Target module: src/paint/ (touch ONLY src/paint/mod.rs; do NOT touch other modules, lib.rs, src/css, src/layout, src/font, or other worktrees).

WHY: \`text-shadow: <offset-x> <offset-y> <blur>? <color>?\` is widely used for headings/labels. We render text but
IGNORE text-shadow entirely (\`grep -rn 'text-shadow\|text_shadow' src\` → no matches). box-shadow is already implemented
in this same file, so the parsing/color/offset machinery to REUSE is right here. This is part of paint breadth (MS-NewTargets).

CURRENT STATE (verify before coding, REUSE — do NOT add new modules/crates):
  - src/paint/mod.rs around line 680: \`// Paint box-shadow if present\` / \`if let Some(box_shadow_val) = style.get(\"box-shadow\")\`.
    Study how it: reads the value, \`flatten_value(...)\` into leaves, parses offset-x / offset-y / blur / color, and pushes
    shadow display items BEFORE the box. MIRROR this parsing approach (offsets as lengths, an optional color, color default
    = current text color).
  - Find where TEXT is emitted: \`DisplayItem::Text\` is pushed around line 1038-1042. Study exactly what fields a Text
    display item carries (position x/y, the string, font size, color, etc.) and how the text color is obtained.

SCOPE (in src/paint/mod.rs ONLY):
  1. At the text-emission site (~line 1038), BEFORE pushing the main \`DisplayItem::Text\`, check \`style.get(\"text-shadow\")\`.
     If present, parse it like box-shadow: one shadow = \`<offset-x> <offset-y> <blur>? <color>?\`.
       - offset-x, offset-y: lengths in px (REUSE the same length parsing box-shadow uses).
       - blur: OPTIONAL; our raster has no real blur, so PARSE it but you MAY ignore the blur radius for rendering
         (leave a \`// TODO(spec): text-shadow blur not rasterized\` ). Do NOT invent a blur implementation.
       - color: OPTIONAL; if omitted, default to the element's current text color.
  2. Emit a SHADOW copy of the text as an extra \`DisplayItem::Text\` (same string/font/size) positioned at
     (text_x + offset_x, text_y + offset_y) with the shadow color, pushed BEFORE the real text item so the real text
     paints on top. Support a comma-separated list of shadows (multiple text-shadows): emit one shadow text item per entry,
     in order, all before the real text. If you must choose, paint later list entries first so earlier ones sit on top
     (CSS paints the first shadow on top); if ambiguous leave a \`// TODO(spec):\` and pick a single consistent order.
  3. Keep the change surgical and localized to the text-emission site. Reuse existing color/length parsing helpers in this
     file (e.g. whatever box-shadow uses); do NOT add new dependencies or new display-item variants.

TESTS (in src/paint/mod.rs, #[cfg(test)], mirror existing paint unit tests / the box-shadow test if present):
  - A text node with \`text-shadow: 2px 2px red\` produces an EXTRA Text display item at offset (+2,+2) with the shadow
    color, in addition to (and before) the original text item. Assert both items exist and the shadow's position/color.
  - A text node WITHOUT text-shadow produces exactly ONE Text item (regression guard — no extra item).
  - A comma list \`text-shadow: 1px 1px blue, 3px 3px green\` produces TWO shadow items plus the real text.
  Find how existing paint tests build a styled text node and inspect the produced display list; mirror that exactly.

CONSTRAINTS (AGENTS.md): no unwrap/expect in non-test code (I-6) — follow existing \`.to_std_string().unwrap_or_default()\`
style; no test skip/ignore (I-4); 1 task = 1 module (I-5); do not edit the main tree, src/css, src/layout, src/font,
lib.rs, or other worktrees (I-3). Do not add new crates (I-1).

WORKFLOW:
  1. Implement in src/paint/mod.rs only.
  2. Run: cargo fmt; cargo clippy --all-targets -- -D warnings; cargo test (or at least the paint tests).
  3. \`git add -A && git commit\` with message:
     \`feat(paint): implement CSS text-shadow painting (t0304)\`
     ending with the Co-Authored-By trailer required by AGENTS.md.
  4. Confirm \`git -C /workspaces/wt/t0304 status\` is clean and \`git diff --name-only origin/main..HEAD\` lists ONLY
     src/paint/mod.rs. COMMIT before you finish — do not leave work uncommitted." \
  -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta
