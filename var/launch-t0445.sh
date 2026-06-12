#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0445
LOG=/workspaces/toy-browser/var/log/t0445.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Write the code, run the checks, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything you need is in the local source files and this prompt. Reading local files is fine; network/web search is forbidden and wastes time.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7. NEVER use unwrap()/expect() in non-test code (I-6). NEVER add a new external crate dependency to Cargo.toml — use only std.

Task t0445 — MS-CSS-Architecture item 3 (画像全般 / image support). Add **BMP decoding** to the image module. This is a self-contained, additive, single-module task touching ONLY `src/image/mod.rs`.

CONTEXT — read first:
  - `src/image/mod.rs` already defines:
      `pub struct DecodedImage { pub width: u32, pub height: u32, pub rgba: Vec<u8> }`  (rgba is tightly-packed RGBA8, row-major, top-to-bottom)
    and `pub fn decode_png`, `decode_jpeg`, `decode_gif`, and the dispatcher
      `pub fn decode_image(bytes: &[u8]) -> Option<DecodedImage>` which sniffs magic bytes (PNG `\x89PNG...`, JPEG `\xFF\xD8\xFF`, GIF `GIF8`) and returns `None` for unknown formats.
  - PNG/JPEG/GIF use external crates already in Cargo.toml. For BMP you must write a PURE-std decoder — do NOT add any crate.

STEP 1 — Implement `pub fn decode_bmp(bytes: &[u8]) -> Option<DecodedImage>` in `src/image/mod.rs`.
  Support the common Windows BMP (`BITMAPINFOHEADER`) uncompressed cases that real pages use:
    - File header: 14 bytes. Bytes 0..2 == b"BM". Bytes 10..14 (LE u32) = pixel data offset.
    - DIB header: read its size at bytes 14..18 (LE u32); handle BITMAPINFOHEADER (40 bytes) and tolerate larger headers (V4/V5) by trusting the declared size and the offset.
    - width = i32 LE at 18..22, height = i32 LE at 22..26. height may be NEGATIVE (top-down rows); positive = bottom-up rows (the common case) — handle BOTH by flipping appropriately so output is always top-to-bottom.
    - bpp (bits per pixel) = u16 LE at 28..30. Support **24-bit (BGR)** and **32-bit (BGRA)** uncompressed (compression field u32 LE at 30..34 must be 0 = BI_RGB; for 32bpp also accept 3 = BI_BITFIELDS only if you implement masks, otherwise return None for anything you do not handle — do NOT panic). Return `None` (not a panic) for unsupported bpp/compression (e.g. 8-bit palettized, RLE), so the dispatcher cleanly falls through.
    - Rows are padded to a multiple of 4 bytes (stride = ((width*bpp/8 + 3) / 4) * 4). Read pixels starting at the declared data offset.
    - BMP channel order is **BGR(A)**; convert to RGBA8 in output. For 24-bit, alpha = 0xFF. For 32-bit, use the stored alpha byte.
  GUARD every slice index / arithmetic against malformed input: validate lengths before indexing, use checked arithmetic or explicit bounds checks, and return `None` on any inconsistency. No unwrap()/expect()/panic on bad data. Use `u32::from_le_bytes` / `i32::from_le_bytes` / `u16::from_le_bytes` on fixed-size sub-slices obtained via `.get(a..b)?`.

STEP 2 — Wire it into the dispatcher `decode_image`: add a branch `else if bytes.starts_with(b"BM") { decode_bmp(bytes) }` in the correct position (BMP magic is just "BM", so keep it among the format checks; order does not conflict with the other magics).

STEP 3 — Add `#[cfg(test)]` unit tests in the existing tests module of `src/image/mod.rs`:
  - Construct a tiny 2x2 (or 2x1) 24-bit BMP byte vector BY HAND in the test (build the 14-byte file header + 40-byte BITMAPINFOHEADER + bottom-up padded pixel rows with known BGR values). Decode it and assert width/height and that specific output pixels are the expected RGBA (proving BGR->RGBA channel swap AND bottom-up row flip are correct).
  - Add a 32-bit BGRA case asserting the alpha byte is preserved.
  - Add a malformed-input test (e.g. truncated bytes, or bpp=8) asserting `decode_bmp` returns `None` and does NOT panic.
  - Add a test that `decode_image` routes a valid BMP to a successful decode.
  In tests, unwrap()/expect() is allowed.

CONSTRAINTS / SCOPE GUARD:
  - `git status` before commit must show ONLY changes to `src/image/mod.rs`. NOTHING else. If anything else changed, revert it.
  - No new Cargo dependency. std only. No unwrap()/expect()/panic in non-test code.
  - Do not modify decode_png/jpeg/gif or any other module.

VERIFY before committing (all must pass):
  - `cargo fmt --all`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test image:: -- --nocapture`
  - `cargo build`

COMMIT on branch agent/t0445-image-bmp-decode with message:
  feat(image): add pure-std BMP (BITMAPINFOHEADER 24/32-bit) decoding (t0445)
Then STOP. Report: which BMP bpp/compression cases you support, the test assertions added, and confirm git status shows only src/image/mod.rs changed.
EOF
echo "$PROMPT" | setsid gemini -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta > "$LOG" 2>&1 &
echo "launched t0445 pid=$!"
