#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0382
LOG=/workspaces/toy-browser/var/log/t0382.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

Task t0382 — parse `background-repeat` and `background-position` (+ optionally `background-size`) and honor them when blitting a background image.

Targets: src/css/values.rs (parse helpers if needed), src/style/mod.rs (store the longhands), and src/paint/mod.rs (consume at blit). Touch ONLY those three files. Do NOT edit raster/, layout/, engine/, dom/, html/, main.rs, or any other module. If something genuinely requires another module, leave a `// TODO(spec): ...` comment in the closest of the three target files and stop.

Context:
- Background IMAGE blitting already happens in src/paint/mod.rs (search for `dom.get_image(` and the surrounding background paint code, around line ~1422). Today the decoded image is blitted at the box origin with no offset/repeat control.
- Most CSS longhands are stored GENERICALLY: in src/style/mod.rs the property loop falls through to `parse_value(...)` and `properties.insert(name, value)` for any property it does not special-case. So `background-repeat: no-repeat` and `background-position: 10px 20px` will ALREADY be stored as raw values via the fallthrough — verify this by reading src/style/mod.rs. You likely DO NOT need to add a match arm unless you need to normalize them; if the raw stored value is usable in paint, leave style/mod.rs UNCHANGED (preferred) and just consume in paint.
- `background-position` values: keywords (left/center/right/top/bottom) and/or lengths/percentages; for v0 support: two-value `<len|%> <len|%>` and the keywords left/top/center/right/bottom. `background-repeat`: `repeat` (default) | `no-repeat` | `repeat-x` | `repeat-y`.

Implement:
1. In src/paint/mod.rs, at the background-image blit site, read the node's computed style for `background-repeat` and `background-position` via the same `style.get("...")` pattern used elsewhere in this file.
2. Apply `background-position` as an (x,y) pixel offset of the image within the box (resolve percentages against (box_size - image_size) per CSS; resolve keywords: left/top=0, center=50%, right/bottom=100%; lengths=px). Clip to the box bounds.
3. Apply `background-repeat`: default `repeat` tiles the image across the box in both axes; `no-repeat` draws once; `repeat-x`/`repeat-y` tile on one axis only. Keep it pixel-correct but simple (integer tiling loop within the box clip). Default (property absent) MUST remain today's behavior or `repeat` — pick whichever matches current default and note it; do NOT regress existing background-image tests.
4. `background-size`: OPTIONAL. If trivial (`cover`/`contain`/explicit px), add it; otherwise leave a `// TODO(spec): background-size` and skip — do not block the task on it.
5. Panic-free (AGENTS.md I-6): no unwrap()/expect()/panicking indexing in non-test code.

Add unit tests in the existing `#[cfg(test)] mod tests` in src/paint/mod.rs (reuse the existing display-list/paint test helpers and the existing pattern for injecting a decoded image via the DOM/`get_image` stub — copy an existing background-image paint test and adapt):
- `test_background_no_repeat_single_blit`: a box larger than the image with `background-repeat: no-repeat` produces exactly ONE image blit item.
- `test_background_repeat_tiles`: same box with `background-repeat: repeat` (or default) produces MULTIPLE image blit items covering the box (assert count > 1).
- `test_background_position_offset`: `background-position: 10px 20px` with `no-repeat` places the single blit at x-offset 10, y-offset 20 relative to the box origin (assert the item's coordinates).
Match assertions to the ACTUAL display-list item type/shape this codebase emits for image blits — read the existing tests to find the right item variant and fields.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green, commit:
  git add -A && git commit -m "feat(paint): honor background-repeat and background-position when blitting images (t0382)"
Then print "T0382 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
