#!/usr/bin/env bash
# t0450 — add WebP image decoding (decode_webp) to src/image/mod.rs. Base: origin/main.
set -euo pipefail
cd /workspaces/wt/t0450

read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Write the code, run the checks, fix until green, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything is in local files. Network/web search is forbidden (cargo may fetch crates from crates.io — that is allowed and is NOT web search).

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. Read AGENTS.md (passed via --include-directories) and obey I-1..I-7. NEVER use unwrap()/expect() in non-test code (I-6). DO NOT delete or skip tests to fake green.

You are the ONLY worker on this worktree: /workspaces/wt/t0450, branch agent/t0450-webp-decode, base origin/main (commit 6dcf80f). Touch ONLY these two files: src/image/mod.rs and Cargo.toml. DO NOT touch anything else.

TASK — add WebP decoding, mirroring the existing PNG/JPEG/GIF/BMP decoders.
Context (already implemented in src/image/mod.rs):
  - `pub struct DecodedImage { pub width: u32, pub height: u32, pub rgba: Vec<u8> }` (RGBA8, row-major).
  - decoders: `decode_png`, `decode_jpeg`, `decode_gif`, `decode_bmp`, each returns `Option<DecodedImage>`.
  - `pub fn decode_image(bytes: &[u8]) -> Option<DecodedImage>` sniffs magic bytes and dispatches.
  - existing crate deps in Cargo.toml: jpeg-decoder, gif, png.

STEPS:
1. Add a pure-Rust WebP decoder crate to Cargo.toml. Use `image-webp` (the dedicated WebP codec extracted from the `image` crate, pure Rust, no C/libwebp dependency). Pick a recent compatible version (e.g. image-webp = "0.2"). Do NOT add the full `image` crate. Do NOT add any C-linked / libwebp-sys crate.
2. Implement `pub fn decode_webp(bytes: &[u8]) -> Option<DecodedImage>` in src/image/mod.rs using `image_webp::WebPDecoder`. Read the decoder's dimensions and decode into an RGBA8 buffer (convert from RGB to RGBA by setting alpha=255 if the decoder yields RGB). Return None on any decoder error — propagate errors with `?` inside a helper that returns Result and map to Option, or match and return None; do NOT unwrap()/expect() in non-test code.
3. In `decode_image`, add a branch BEFORE the fallthrough: WebP files start with ASCII "RIFF" at bytes[0..4] and "WEBP" at bytes[8..12]. Detect `bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP"` and route to decode_webp.
4. ADD a test in the existing `#[cfg(test)] mod tests`. Preferred: if `image-webp` exposes a lossless encoder (image_webp::WebPEncoder / encoder API), round-trip a small 2x2 RGBA image (encode then decode_webp) and assert width/height/rgba like the existing PNG `test_round_trip`. If no encoder is available in the crate, instead embed a known-valid minimal lossless WebP as a base64 const, decode it, and assert width/height. ALSO add a test that decode_image routes a "RIFF....WEBP" header to the webp path and that decode_webp returns None on truncated/garbage input. Do NOT weaken or delete existing tests.

PROCEDURE (iterate until all green):
  - cargo build
  - cargo fmt
  - cargo clippy --all-targets -- -D warnings   (fix every warning)
  - cargo test                                   (all pass)
  - git add -A && git commit -m "feat(image): add WebP (RIFF/WEBP) decoding via image-webp (t0450)"
  COMMIT before finishing (commit partial progress too). Report the final cargo test summary line.
EOF

exec gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta
