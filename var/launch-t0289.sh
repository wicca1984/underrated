#!/usr/bin/env bash
# Launcher for Gemini worker t0289 — text-decoration-color paint support.
# Target: src/paint/mod.rs ONLY. Dispatched via setsid (memory: worker-dispatch-must-setsid).
set -euo pipefail

WT=/workspaces/wt/t0289
LOG=/workspaces/toy-browser/var/worker-logs/t0289.log

read -r -d '' PROMPT <<'EOF' || true
You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow ALL of it (especially I-1..I-7).

Task: t0289 — implement the CSS `text-decoration-color` property in the paint stage. Currently underline /
overline / line-through lines are always drawn in the element's text color; `text-decoration-color` lets the
decoration lines use a different color while leaving the glyph color unchanged.

Target module: src/paint/mod.rs ONLY. Do NOT touch any other file (no other src/ module, no fixtures, no
other worktree, no Cargo.toml). 1 task = 1 module (I-5). No `unwrap`/`expect` in non-test production paths (I-6).

Read first (verify every claim against the ACTUAL code before editing — do not trust this description blindly):
- `struct TextDecorations` (~line 122): currently `{ underline, overline, line_through }` (all bool). You will
  add a color field, e.g. `color: Option<Color>` (use whatever Color type this module already uses for
  `DisplayItem::SolidRect.color` — find it, do NOT invent a new type).
- `fn get_text_decorations(style: &ComputedStyle) -> TextDecorations` (~line 130): parses the
  `text-decoration` / `text-decoration-line` keywords. Add parsing of the `text-decoration-color` property
  here into the new `color` field. Use the SAME color-parsing path the rest of paint/style already uses for
  CSS color values (search the codebase for how `color` / `background-color` CssValue is turned into a Color —
  reuse that helper; do NOT hand-roll hex parsing). If `text-decoration-color` is absent or unparseable,
  leave `color = None`.
- `fn resolve_text_decorations(...)` (~line 225): this walks ancestors to resolve which lines apply. Propagate
  the `color` through so the final resolved decoration carries the nearest explicitly-set
  `text-decoration-color` (mirror how the bool flags are resolved; a child's own
  `text-decoration-color` should win over an ancestor's). Keep it simple and consistent with the existing
  resolution logic — do NOT redesign it.
- The draw site (~lines 615-655): three `if decorations.underline / overline / line_through` blocks each push
  a `DisplayItem::SolidRect { rect, color: scale_color_alpha(&color, effective_opacity) }` where `color` is the
  TEXT color. Change ONLY the decoration rects to use the decoration color when present:
  `let deco_color = decorations.color.unwrap_or(color);` then
  `color: scale_color_alpha(&deco_color, effective_opacity)`. The glyph `DisplayItem::Text` MUST keep using the
  original text `color` (do NOT change glyph color).

Scope / spec notes:
  // TODO(spec): text-decoration-color v1 — single computed color only; `currentColor` keyword falls back to
  // the text color (i.e. treat unset/`currentColor` as None). text-decoration shorthand color parsing and
  // per-line distinct colors are out of scope.
  Leave that TODO(spec) marker where you parse the property so scope is explicit.

Add ONE Rust integration test next to the existing text-decoration paint tests (search for
`text-decoration: underline` in the `mod tests` block, ~line 1386/1455). Name it
`fn test_paint_text_decoration_color()` that:
   - renders a fixture like `p { text-decoration: underline; text-decoration-color: #00ff00; color: #ff0000; }`
     over some text,
   - asserts the emitted display list contains a `SolidRect` (the underline) whose color is the GREEN
     decoration color (#00ff00), AND at least one `Text` item whose color is the RED text color (#ff0000),
   - i.e. proves the underline color and glyph color diverge.
  Follow the EXACT assertion style of the neighbouring text-decoration tests (how they pull items out of the
  display list and inspect `SolidRect` / `Text` colors). Test-side `unwrap`/`assert!` is fine (I-6 forbids
  unwrap only in non-test production code).

Do NOT delete, weaken, `#[ignore]`, or alter any existing test or assertion to force green (hard violation).
Do NOT change glyph/text color behavior or any unrelated decoration logic.

Done when (run from the worktree root /workspaces/wt/t0289):
  `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check` ALL pass.

Commit (you MUST `git add -A && git commit` before finishing — uncommitted work is lost; the worktree may be
force-removed after you exit):
  `feat(paint): support text-decoration-color for underline/overline/line-through (t0289)`

End with a short summary: the Color type/field you added to TextDecorations, the color-parsing helper you
reused, and confirmation that the new test asserts the underline SolidRect is green while the glyph Text is red.
EOF

cd "$WT"
exec gemini -p "$PROMPT" \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  < /dev/null > "$LOG" 2>&1
