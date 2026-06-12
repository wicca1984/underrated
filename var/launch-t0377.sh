#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0377
LOG=/workspaces/toy-browser/var/log/t0377.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

Task t0377 — add IMAGE-DECODE DIAGNOSTICS to src/engine (B-3' logo diagnosis: diagnose, DO NOT guess-fix).

Target module: src/engine/mod.rs ONLY (touch ONLY this file; do not touch image/, dom/, loader/, or any other module or worktree).

Background: The logo / some <img> images do not render. The function `fetch_and_decode_images` (src/engine/mod.rs ~line 167) resolves each <img>'s URL, loads bytes via `load_image_safely_with_loader`, then calls `crate::image::decode_image(&bytes)`. Today, when the load fails OR the decode returns None (e.g. an unsupported format like WebP or SVG), the image is SILENTLY dropped (the `if let ... && let Some(decoded) = decode_image(...)` chain just falls through). This makes it impossible to tell WHY an image is missing. Your job is to make the failure observable and to classify the format — NOT to add a new decoder.

Implement (all inside src/engine/mod.rs, scoped to the image-fetch path around lines 269-275):
1. Restructure the terminal `if let` chain so each outcome is logged to STDERR via `eprintln!`:
   - load failed  -> `eprintln!("[img] fetch failed url={chosen_url}")`
   - loaded N bytes but decode_image returned None -> `eprintln!("[img] decode failed url={chosen_url} bytes={n} sniff={fmt}")`
   - decoded OK -> `eprintln!("[img] decoded url={chosen_url} bytes={n}")` (keep the existing `dom.add_image(...)` call on this branch).
2. Add a small pure helper `fn sniff_image_format(bytes: &[u8]) -> &'static str` (private, in this module) that classifies the bytes by magic number and returns one of: "png", "jpeg", "gif", "bmp", "webp", "svg", "unknown". Magic numbers:
   - PNG: starts with 89 50 4E 47
   - JPEG: starts with FF D8 FF
   - GIF: starts with "GIF8"
   - BMP: starts with "BM"
   - WebP: bytes 0..4 == "RIFF" AND bytes 8..12 == "WEBP"
   - SVG: leading ASCII (after optional whitespace/BOM) contains "<svg" or starts with "<?xml" — a simple case-insensitive substring check on the first ~256 bytes is fine.
   - otherwise "unknown".
   Use this helper to fill the `sniff={fmt}` field in the decode-failed log so we learn whether the undecodable logo is WebP/SVG/etc.
3. Leave a `// TODO(spec): track unsupported image formats (webp/svg) for a decode-support decision` comment next to the decode-failed branch. Do NOT add any new decoder and do NOT change `crate::image::decode_image`.

Add a unit test in the existing `#[cfg(test)] mod tests` block in src/engine/mod.rs:
- `fn test_sniff_image_format()` asserting sniff on small byte fixtures: a PNG magic slice -> "png", a JPEG magic slice -> "jpeg", a RIFF....WEBP slice -> "webp", a "<svg ...>" byte slice -> "svg", and random bytes -> "unknown".

Hard constraints (AGENTS.md I-1..I-7):
- Touch ONLY src/engine/mod.rs. If something genuinely requires another module, leave a `// TODO(spec): ...` and stop there — do not edit other files.
- NO `unwrap()`/`expect()` in non-test code (I-6) — use match/if let/slicing guards (check bytes.len() before indexing ranges).
- Do NOT skip, #[ignore], or delete any existing test. Keep all existing #[test] in src/engine/mod.rs.
- Keep `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` green.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green, commit:
  git add -A && git commit -m "feat(engine): add image-decode diagnostics and format sniff for <img> failures (t0377)"
Then print "T0377 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
