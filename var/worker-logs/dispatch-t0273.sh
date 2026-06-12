#!/usr/bin/env bash
set -euo pipefail
# Robust auth: var/.env is the source of truth (bashrc can be wiped on rebuild).
set -a
[ -f /workspaces/underrated-meta/var/.env ] && . /workspaces/underrated-meta/var/.env
set +a
cd /workspaces/wt/t0273
exec gemini \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  -p 'You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories (/workspaces/underrated-meta/AGENTS.md) and follow ALL of it, especially invariants I-1..I-7.

Task: t0273 — Implement the CSS `outline` property: parse the `outline` shorthand into longhands and paint an outline as a frame drawn OUTSIDE the border box. Unlike `border`, an outline does NOT take up space and does NOT affect layout/geometry at all — it is paint-only on top of the existing border box. This is a deliberately small, two-module feature (style parse + paint render). Implement `outline-offset: 0` (the default, no offset) and leave non-zero `outline-offset` as a documented TODO(spec).

Scope: touch ONLY these two files — `src/style/mod.rs` and `src/paint/mod.rs` (each including its inline `#[cfg(test)] mod tests`). `git diff --name-only` MUST show ONLY those two files. Do NOT modify layout, css/values, or any other module.

Reuse / facts (verified — read these before writing):
- STYLE (src/style/mod.rs): the `border` shorthand is expanded around line 220 (`"border" => { ... }`). Mirror that idiom for an `"outline" => { ... }` arm that expands the `outline` shorthand `<width> || <style> || <color>` (order-independent, any subset) into the longhands `outline-width`, `outline-style`, `outline-color`. Reuse the SAME value-classification helpers/patterns the `border` arm uses to decide which token is a width (CssValue::Length / line-width keyword thin|medium|thick), which is a color (CssValue::Color), and which is a style keyword (none|solid|dashed|dotted|double|…). If the existing `border` arm only partially classifies, keep `outline` at parity with it — do NOT invent richer parsing than `border` has. Longhands set directly (`outline-width:`, `outline-style:`, `outline-color:`) must also work and must win over the shorthand per normal cascade (same as border longhands).
- PAINT (src/paint/mod.rs): the border edges are emitted in the block starting ~line 401 (`let border_top = get_border_width(style, "border-top-width");` … four SolidRect strips). The border box rect is `layout_box.rect` → `x=rect.origin.x, y=rect.origin.y, w=rect.size.width.max(0.0), h=rect.size.height.max(0.0)` (see the existing `let rect = layout_box.rect; let x = …` just below). Reuse `get_border_width`-style reading via `style.get("outline-width")` and the color-reading idiom used by `get_border_color`/`get_edge_color` (match `Some(CssValue::Color(c))`). Apply `scale_color_alpha(&color, effective_opacity)` exactly like the border strips do.

Implement:
1. STYLE: add the `"outline" => { … }` shorthand-expansion arm at parity with `border`. Default `outline-color` when omitted should follow the `border` arm''s convention (if `border` defaults color, match it; otherwise leave it unset and let paint fall back to the text `color`/black). Do NOT use unwrap/expect/panic/unsafe.
2. PAINT: AFTER the existing border-strip block (so the outline paints outside/over the border), add an outline block:
   - Read `outline_style = style.get("outline-style")` keyword; if absent or `none`, paint NOTHING (early skip).
   - Read `outline_width` via the same px-resolution as border widths; support the `thin|medium|thick` keywords mapping to e.g. 1.0/3.0/5.0 px if the border code maps them, otherwise treat keywords as medium=3.0. If width <= 0.0, paint nothing.
   - Resolve `outline_color`: `outline-color` if a Color, else fall back to the element''s computed text `color`, else black (same fallback chain as `get_border_color`). Per spec the `invert` keyword is allowed — implement `invert` as a documented `// TODO(spec):` and fall back to the text color for now (do NOT implement true XOR inversion).
   - Emit 4 SolidRect strips forming a frame whose INNER edge is flush with the border box and which extends OUTWARD by `outline_width` on every side (top/bottom strips span the full outer width `w + 2*ow`; left/right strips fill the remaining height). Treat all dashed/dotted/double/groove/etc. styles as a solid frame for now (one `// TODO(spec):` covering non-solid outline styles + outline-offset + invert is enough). Guard every rect against empty/negative dimensions exactly like the border strips do (no panic on weird sizes, I-6).
   - Geometry (outline-offset = 0): with `ow = outline_width`:
       top:    Rect::new(x - ow, y - ow, w + 2*ow, ow)
       bottom: Rect::new(x - ow, y + h, w + 2*ow, ow)
       left:   Rect::new(x - ow, y, ow, h)
       right:  Rect::new(x + w, y, ow, h)
     Only push each strip when its width>0 and height>0.
3. Leave a single consolidated `// TODO(spec): outline non-solid styles, outline-offset != 0, and color:invert are not implemented (solid frame, zero offset, color falls back to text color).` near the paint block.

Acceptance — add inline unit tests in BOTH files:
- src/style/mod.rs: a test that parses `outline: 2px solid red;` and asserts the computed style exposes `outline-width` (2px length), `outline-style` (solid keyword), and `outline-color` (red Color). Mirror the construction idioms of the existing border-shorthand test(s) in this file. Also assert a longhand-only form (`outline-style: solid; outline-width: 3px;`) resolves correctly.
- src/paint/mod.rs: a test that lays out/paints a simple block with `outline: 2px solid red;` (reuse the existing paint-test harness in this file — find a border or background paint test and mirror its DOM/style/layout/paint construction) and asserts the display list contains SolidRect items whose union extends OUTSIDE `layout_box.rect` by the outline width on at least one side (e.g. a rect with `origin.x < box.x` or `origin.y < box.y`), with the red color. Also assert `outline-style: none` (or absent) produces NO such outside rect. Do NOT weaken or remove any existing test.

Done when ALL of these pass in this worktree:
  - cargo test
  - cargo clippy --all-targets -- -D warnings
  - cargo fmt --check
No `unwrap`/`expect`/`panic!` in non-test code (I-6). No new dependencies (I-1). Comments and identifiers in English.
Commit (you MUST commit before finishing): `feat(style,paint): parse outline shorthand and paint outline frame outside the border box (t0273)`.
End with a short English summary of exactly what changed in each file and the `// TODO(spec):` you left.'
