#!/usr/bin/env bash
set -euo pipefail
# Robust auth: var/.env is the source of truth (bashrc can be wiped on rebuild).
set -a
[ -f /workspaces/underrated-meta/var/.env ] && . /workspaces/underrated-meta/var/.env
set +a
cd /workspaces/wt/t0263
exec gemini \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  -p 'You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories (/workspaces/underrated-meta/AGENTS.md) and follow ALL of it, especially invariants I-1..I-7.

Task: t0263 — Apply the CSS `opacity` property when painting, as a per-element alpha multiplier threaded through the recursive painter. Cards, buttons, overlays and faded text on real pages (Google/Wikipedia/News) rely on `opacity`; today the property is parsed into computed style but NO paint code consumes it, so semi-transparent elements render fully opaque. This advances MS-NewTargets / paint completeness.

Target module: src/paint/mod.rs (touch ONLY this file — both the painter code and the inline `#[cfg(test)] mod tests`). Do NOT modify any other file. `git diff --name-only` must show ONLY: src/paint/mod.rs.

Reuse / facts (verified — do NOT reinvent):
- The display list is `DisplayList(pub Vec<DisplayItem>)` and `DisplayItem` variants carry `color: Color` where `Color` is `crate::css::values::Color` (already imported at line 1). Inspect the `DisplayItem` enum (~line 12) to see every variant that carries a paintable color (rect/background, text, border edges, etc.).
- The painter is the recursive function that walks the layout tree and pushes `DisplayItem`s. Find it and the point(s) where each `DisplayItem` is constructed with a resolved `Color`.
- `Color` is an enum (see `crate::css::values::Color`, e.g. `Color::Rgba(u8,u8,u8,u8)` plus other variants). To scale alpha you must resolve a `Color` to its rgba components, scale the alpha (4th channel, 0..=255), and rebuild a `Color::Rgba(...)`. CHECK css/values.rs for an existing `to_rgba`/`resolve`/`as_rgba` helper on `Color` and REUSE it; if none exists, write a small private helper INSIDE src/paint/mod.rs (do NOT add a method to css/values.rs — that is a different module and forbidden here). Named/other non-rgba variants must be resolved to rgba before scaling; if a variant cannot be resolved, leave the color unchanged (do not panic).
- Raster already performs correct src-over alpha blending honoring the alpha channel (src/raster/mod.rs `blend`), so scaling a DisplayItem color''s alpha is sufficient to make it render translucent. You do NOT touch raster.
- `style.get("...")` returns `Option<&CssValue>` (see existing `get_border_color`/`resolve_text_color` helpers for the pattern of reading a property off a `ComputedStyle`).

Semantics — implement EXACTLY this:
- Read each element''s computed `opacity` via `style.get("opacity")`. Accept a number (`CssValue::Number`) or a percentage (`CssValue::Percentage`); map percentage p to p/100. Absent / unparseable → treat as `1.0`. Clamp the result to the range `[0.0, 1.0]`.
- Thread an accumulated opacity factor through the recursive painter as a parameter (e.g. `inherited_opacity: f32`, default `1.0` at the root). At each element, `effective = inherited_opacity * clamp(own_opacity)`, pass `effective` to children, and multiply `effective` into the alpha channel of EVERY `Color` this element emits into the display list (backgrounds, borders, text). New alpha = `round(orig_alpha as f32 * effective)` clamped to `0..=255`.
- Add a `// spec: https://www.w3.org/TR/css-color-3/#transparency` comment, and a `// TODO(spec):` comment stating that true group/stacking-context opacity (compositing the element subtree as a single group, so overlapping descendants do not double-blend) is NOT implemented — this uses a multiplicative per-element alpha approximation. Do NOT attempt real group compositing (no offscreen buffer) — keep it small and safe.

Acceptance (must all be green) — add inline unit tests mirroring the EXISTING paint tests in the same file (find the `#[cfg(test)] mod tests` block and reuse its helpers for building a styled layout tree + producing a `DisplayList`):
  1. An element with `opacity: 0.5` and an opaque (`alpha == 255`) background emits a background `DisplayItem` whose color alpha is ~127 (assert `(alpha as i32 - 127).abs() <= 1`).
  2. Nested opacity multiplies: parent `opacity: 0.5` containing a child `opacity: 0.5` with an opaque background → child background alpha ~64 (assert within +/-1).
  3. `opacity: 1` (or absent) leaves an opaque color''s alpha at exactly 255 (no regression).
  4. `opacity: 0` yields alpha 0.
  Assert against the alpha channel of the emitted DisplayItem colors (resolve to rgba in the test the same way the impl does).

Done when ALL of these pass:
  - cargo test
  - cargo clippy --all-targets -- -D warnings
  - cargo fmt --check
No unwrap/expect/panic/unsafe in non-test code (I-6). No `unsafe` anywhere (forbidden). No test skip/ignore (I-4). Keep the diff limited to src/paint/mod.rs — `git diff --name-only` must show ONLY that file. Commit on this branch with: `feat(paint): apply opacity as per-element alpha multiplier (t0263)`. Comments and identifiers in English. IMPORTANT: commit your work before finishing (do not leave changes uncommitted). End with a one-paragraph summary and the names of the tests you added. If the spec is ambiguous or conflicts with real browser behavior, do NOT decide on your own — leave a `// TODO(spec):` and report it.'
