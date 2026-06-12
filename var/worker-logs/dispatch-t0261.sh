#!/usr/bin/env bash
set -euo pipefail
# Robust auth: var/.env is the source of truth (bashrc can be wiped on rebuild).
set -a
[ -f /workspaces/underrated-meta/var/.env ] && . /workspaces/underrated-meta/var/.env
set +a
cd /workspaces/wt/t0261
exec gemini \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  -p 'You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories (/workspaces/underrated-meta/AGENTS.md) and follow ALL of it, especially invariants I-1..I-7.

Task: t0261 — Add GIF (first-frame) image decoding to the image module and wire it into the format sniffer. Real news/article pages embed GIF assets; without this they fail to blit. This advances render fidelity for MS-NewTargets (News sites: more image formats).

Target module: src/image/mod.rs (touch ONLY src/image/mod.rs and its inline tests). You MAY add exactly ONE new dependency line to Cargo.toml for the `gif` decoder crate. Do NOT modify src/loader, src/paint, src/raster or any other module. Read other modules read-only as needed. `git diff --name-only` must show ONLY: src/image/mod.rs, Cargo.toml, and Cargo.lock (the lockfile updates automatically — do not hand-edit it).

Reuse / facts (verified — do NOT reinvent, mirror the EXISTING decoders):
- The public output type is `DecodedImage { pub width: u32, pub height: u32, pub rgba: Vec<u8> }` (RGBA8, 4 bytes/pixel) at the top of src/image/mod.rs.
- Existing decoders `decode_png(bytes: &[u8]) -> Option<DecodedImage>` (src/image/mod.rs:49) and `decode_jpeg(bytes: &[u8]) -> Option<DecodedImage>` (src/image/mod.rs:116) both: take `&[u8]`, return `None` on ANY failure (use `.ok()?` / `?` — never unwrap/expect/panic, I-6), build `rgba` as RGBA8, and return `Some(DecodedImage { width, height, rgba })`. Follow this exact shape.
- The sniffer `decode_image(bytes: &[u8]) -> Option<DecodedImage>` (src/image/mod.rs:179) dispatches by magic-byte prefix: PNG `[137,80,78,71,13,10,26,10]`, JPEG `[0xFF,0xD8,0xFF]`, else `None`.
- `use std::io::Cursor;` is already imported at the top of the file.

Dependency:
- Add `gif = "0.13"` to the `[dependencies]` section of Cargo.toml (alongside `png` and `jpeg-decoder`). Use a plain version string; do not enable extra features. Run a build so Cargo.lock updates.

Semantics — implement EXACTLY:
- New public function `pub fn decode_gif(bytes: &[u8]) -> Option<DecodedImage>` that decodes ONLY the FIRST frame of the GIF and returns it as RGBA8. Use the gif crate with RGBA color output so the frame buffer is already RGBA8:
    let mut options = gif::DecodeOptions::new();
    options.set_color_output(gif::ColorOutput::RGBA);
    let mut decoder = options.read_info(Cursor::new(bytes)).ok()?;
    let frame = decoder.read_next_frame().ok()??; // returns Option<&Frame>; `??` flattens Result<Option<_>>
    let width = frame.width as u32;
    let height = frame.height as u32;
    let rgba = frame.buffer.to_vec();
  Then sanity-check `rgba.len() == (width as usize) * (height as usize) * 4`; if not, return `None`. Return `Some(DecodedImage { width, height, rgba })`. Never panic on malformed input — every fallible step uses `.ok()?` / `?` and returns `None`.
  (Note: the gif crate field is `frame.buffer` of type `Cow<[u8]>`; `.to_vec()` materializes it. `frame.width`/`frame.height` are `u16`. Confirm exact names against the gif 0.13 API and adjust only if the compiler disagrees — do NOT change the RGBA8 output contract.)
- TODO(spec): add a `// TODO(spec):` comment noting that only the first frame is decoded (animation/disposal/sub-frame offsets are not yet composited) and that the logical screen size may exceed the first frame`s size.
- Wire into the sniffer: in `decode_image`, add a branch that detects the GIF magic prefix `bytes.starts_with(b"GIF8")` (covers both `GIF87a` and `GIF89a`) and calls `decode_gif(bytes)`, placed before the final `else { None }`.

Approach:
1. Add `gif = "0.13"` to Cargo.toml `[dependencies]`.
2. Add `pub fn decode_gif` as specified.
3. Add the `GIF8` branch to `decode_image`.
4. No unwrap/expect/panic/unsafe in non-test code (I-6).

Acceptance (must all be green) — add inline unit tests in src/image/mod.rs mirroring the existing image tests (`mod tests` already exists; tests may embed a tiny GIF as a base64 string and decode it via `crate::loader::decode_base64(...)`, exactly as the existing JPEG tests do with `JPEG_BASE64_2`):
  - A minimal valid 1x1 (or small) GIF89a decodes to `Some(DecodedImage)` with the expected width/height and `rgba.len() == width*height*4`. (Build the base64 from any minimal valid GIF; verify the bytes start with `GIF89a`.)
  - `decode_gif(b"not a gif")` and `decode_gif(&[])` both return `None` (no panic).
  - `decode_image(<the gif bytes>)` routes to the GIF decoder and returns `Some(...)` (proves the sniffer branch works); `decode_image(b"unknown")` still returns `None`.
  - cargo test
  - cargo clippy --all-targets -- -D warnings
  - cargo fmt --check
Done when all three pass. No unwrap/expect in non-test code (I-6). No unsafe (forbidden). No test skip/ignore (I-4). Keep the diff limited to src/image/mod.rs + Cargo.toml + Cargo.lock — `git diff --name-only` must show ONLY those. Commit on this branch with: `feat(image): decode first GIF frame and wire into sniffer (t0261)`. Comments and identifiers in English. IMPORTANT: commit your work before finishing (do not leave changes uncommitted). End with a one-paragraph summary and the names of the tests you added. If the spec is ambiguous or conflicts with real browser behavior, do NOT decide on your own — leave a `// TODO(spec):` and report it.'
