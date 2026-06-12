#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0385
LOG=/workspaces/toy-browser/var/log/t0385.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

Task t0385 — implement the `cover`, `contain`, and `auto` keyword values for CSS `background-size` when blitting a background image. Touch ONLY files under src/paint/. Do NOT edit layout/, dom/, style/, engine/, script/, html/, css/, main.rs, or any other module. If something genuinely requires another module, leave a `// TODO(spec): ...` comment in the closest src/paint/ file and stop.

Background (read before coding):
- Read src/paint/mod.rs around lines 1050-1100. There is existing background-size handling: `if let Some(size_val) = style.get("background-size")` with "Simple background-size support (trivial explicit px or percentages)" and a `// TODO(spec): background-size other values (cover/contain/auto)` comment (~line 1087).
- Find where the background image is blitted: the destination box dimensions (the element's painted background area, e.g. border-box/padding-box width & height in px) and the image's intrinsic pixel dimensions (decoded image width/height) are both available nearby. Read how the existing explicit-px/percentage branch computes the scaled draw size, and reuse the same blit/scale call path.

Semantics to implement (CSS backgrounds spec, single-value keyword forms):
- `auto`: use the image's intrinsic size (no scaling). This is the default — ensure it is handled (likely already the fallback).
- `contain`: scale the image (preserving aspect ratio) to the LARGEST size such that BOTH dimensions fit inside the background area. scale = min(area_w / img_w, area_h / img_h); draw_w = img_w * scale; draw_h = img_h * scale.
- `cover`: scale the image (preserving aspect ratio) to the SMALLEST size such that BOTH dimensions COVER the background area. scale = max(area_w / img_w, area_h / img_h); draw_w = img_w * scale; draw_h = img_h * scale.
- Matching is ASCII-case-insensitive; trim the value. Guard against zero/negative intrinsic dimensions (skip scaling, avoid division by zero) — panic-free.
- This task is ONLY about the computed draw width/height. Positioning/clipping/repeat are handled elsewhere and out of scope — feed the computed draw_w/draw_h into the existing blit path exactly as the px/percentage branch does.

Implement (minimal, idiomatic, matching surrounding code) in src/paint/mod.rs:
1. Extend the `background-size` parsing branch to recognize the keywords `cover`, `contain`, `auto` (in addition to the existing px/percentage handling).
2. Compute draw_w/draw_h per the semantics above and route through the existing scaled-blit code path. Replace/trim the `// TODO(spec): ... cover/contain/auto` comment now that they are handled (leave a narrower TODO if some sub-case like two-value `auto auto` or intrinsic-ratio edge cases remains).
3. Panic-free (AGENTS.md I-6): no unwrap()/expect()/panicking indexing or division-by-zero in non-test code.

Add unit tests in the existing `#[cfg(test)] mod tests` block in src/paint/mod.rs (find a helper that computes the background draw size, or factor the size computation into a small `fn` you can unit-test directly — prefer extracting a pure helper like `fn resolve_bg_size(area: (f32,f32), intrinsic: (f32,f32), value: &str) -> (f32,f32)` and testing IT):
- `test_bg_size_contain`: area 200x100, image 50x50 -> contain scales to 100x100 (min(200/50,100/50)=2).
- `test_bg_size_cover`: area 200x100, image 50x50 -> cover scales to 200x200 (max(200/50,100/50)=4).
- `test_bg_size_auto`: area 200x100, image 50x50 -> 50x50 (intrinsic, unchanged).
- `test_bg_size_zero_intrinsic_safe`: image 0x0 -> no panic, returns a safe value (e.g. 0x0).

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green, commit:
  git add -A && git commit -m "feat(paint): support cover/contain/auto for background-size (t0385)"
Then print "T0385 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
