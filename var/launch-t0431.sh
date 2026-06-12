#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0431
LOG=/workspaces/toy-browser/var/log/t0431.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything you need is in the local source files and this prompt. Reading local files is fine; network/web search is forbidden and wastes time.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-5 one-module; I-6 NO unwrap/expect in library/src code; test code MAY use unwrap/panic as the existing tests do).

Task t0431 — Implement comma-separated MULTIPLE box-shadows in the paint layer. Touch ONLY `src/paint/mod.rs` (plus tests in that same file). Do NOT edit any other file under src/.

WHY this is new (do not duplicate): a SINGLE outer box-shadow (offset-x, offset-y, optional spread, color) is ALREADY implemented and painted as a SolidRect. But comma-separated MULTIPLE shadows are explicitly bailed out. Read the existing code carefully first:

In `src/paint/mod.rs`, the block starting at `// Paint box-shadow if present` (search that exact string, ~line 1019). It does:
  1. `flatten_value(box_shadow_val, &mut leaves)` to get a flat Vec<CssValue> of the leaves.
  2. Scans leaves for keywords: `inset`, `none`, and comma `","` (a `CssValue::Keyword(",")`).
  3. At the guard `if leaves.is_empty() || has_none || has_inset || has_comma { /* TODO ... */ }` it does NOTHING when there is a comma (multiple shadows) — this is the gap you must fill.
  4. The `else` branch parses a single shadow: collects `length_values` (from `CssValue::Length`/`CssValue::Number`) and a `color_value` (`CssValue::Color` or via `find_color`), then if `length_values.len() >= 2` builds a `shadow_rect` from `offset_x`, `offset_y`, optional `spread` (index 3), translates/inflates the border box `layout_box.rect`, and pushes a `DisplayItem::SolidRect`.

YOUR CHANGE — support multiple comma-separated shadows while preserving ALL existing single-shadow behavior:
- Keep bailing out (paint nothing) for `has_none` or `has_inset` or empty (inset and `none` remain out of scope — do NOT implement inset).
- When `has_comma` is true: SPLIT the `leaves` slice into segments delimited by the `CssValue::Keyword(",")` leaves. For EACH segment, run the SAME single-shadow parsing logic (offset_x, offset_y, optional spread at index 3, color via Length/Number/Color/find_color) and push one `DisplayItem::SolidRect` per valid segment (segment needs `length_values.len() >= 2` and positive resulting rect, exactly mirroring the single case). Paint order: per CSS spec the FIRST listed shadow is on TOP, so later shadows should be painted FIRST (pushed before) so earlier ones overlay them — to keep it simple and deterministic, push segments in REVERSE order (last segment first). Add a `//` comment noting this stacking order.
- IMPORTANT: refactor the single-shadow parse+push into a small local helper closure or `fn` taking the segment leaves (the `&[CssValue]`), the `layout_box`, the fallback text color, and `effective_opacity`, returning an `Option<DisplayItem>` (or pushing into `items`). Then call it once for the single case and once per segment for the comma case. This avoids code duplication. Match the EXISTING logic byte-for-byte (spread default 0.0, color fallback to `style.get("color")` then black, positive-dimension guards, `scale_color_alpha`).
- Keep I-6: NO `unwrap`/`expect` in non-test code. No new panics.
- Remove or update the now-stale `// TODO(spec): ... multiple shadows out of scope` comments: multiple (non-inset) shadows are now IN scope; keep a TODO only for blur and inset.

Tests — add to the existing `#[cfg(test)]` area near `test_box_shadow_emits_offset_rect` (search it). Add ONE `#[test] fn test_box_shadow_multiple_comma_shadows()` mirroring the existing test harness: build a small HTML/CSS fixture with an element using e.g. `box-shadow: 5px 5px #ff0000, 10px 10px #0000ff;`, run the same paint pipeline the neighboring test uses, and assert that BOTH shadow SolidRects are emitted (two distinct shadow rects at the two offsets, with the two distinct colors). Put a short `//` comment above it naming what it guards. Keep the existing single-shadow tests passing unchanged.

Run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(paint): paint comma-separated multiple box-shadows (t0431)"
Then print "T0431 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
