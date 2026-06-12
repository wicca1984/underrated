#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0422
LOG=/workspaces/toy-browser/var/log/t0422.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything you need is in the local source files and this prompt. Reading local files is fine; network/web search is forbidden and wastes time.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code — note: test code MAY use unwrap/panic as the existing tests do).

Task t0422 — Add ONE new integration test that proves the GIF image pipeline end-to-end: a `<img>` whose loader returns GIF bytes is fetched via the injected loader, decoded, and emitted as a blittable image (DisplayItem::Image with Some(decoded) of the right dimensions) in the display list. Touch ONLY the file `tests/oracle_snapshot_test.rs`. Do NOT edit any file under src/, and do NOT edit any fixture HTML.

WHY this is new (do not duplicate existing coverage): the engine pipeline is already covered for PNG (relative-url, `test_relative_*` in `tests/oracle_snapshot_test.rs`) and for JPEG (`test_remote_jpeg_fetch_decode_blit` in `src/engine/mod.rs`). There is NO end-to-end engine test that drives the GIF path (fetch -> sniff `GIF8` -> `decode_gif` -> blit). Your test adds exactly that GIF coverage. GIF decoding itself is already implemented and unit-tested in `src/image/mod.rs` (see `test_decode_gif_minimal` and the `GIF_BASE64` fixture) — you are testing the ENGINE pipeline, not re-testing the decoder.

A reusable 1x1 GIF fixture already exists in `src/image/mod.rs`:
  const GIF_BASE64: &str = "R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7";
This is a valid 1x1 GIF. In your test, obtain the raw bytes via `underrated::loader::decode_base64(GIF_BASE64).unwrap()` (copy the const string literal into your test). Confirm `underrated::image::decode_image(&gif_bytes)` returns `Some` with `width == 1 && height == 1` as a sanity precondition before wiring the loader.

All needed items are PUBLIC from the `underrated` crate (this is an integration test in `tests/`, so use the `underrated::` path prefix, NOT `crate::`). Read the EXACT current signatures from the source before using them — do not assume:
- The render entry point used by the existing oracle tests in THIS file (search `tests/oracle_snapshot_test.rs` for how it renders a page with a custom loader — reuse the SAME function and the SAME `ResourceLoader`/`DummyLoader` patterns already present around line 450). Match the trait method names, `LoadError`, `LoaderResponse`/`HttpMethod` EXACTLY as defined in `src/loader/mod.rs`.
- `underrated::loader::decode_base64(&str) -> Option<Vec<u8>>`
- `underrated::image::decode_image(&[u8]) -> Option<DecodedImage>` and `DecodedImage { width, height, rgba }`
- `underrated::paint::build_display_list(...) -> DisplayList` and `DisplayItem::Image { src, decoded, rect, .. }` with `decoded: Option<DecodedImage>` — read the EXACT variant fields in `src/paint/mod.rs`.

Model your scaffolding on the existing relative-url PNG test already in `tests/oracle_snapshot_test.rs` (read it first) and on `test_remote_jpeg_fetch_decode_blit` in `src/engine/mod.rs` (read it to copy the MockLoader-returns-image-bytes pattern), but return GIF bytes instead and assert GIF dimensions. Steps:
1. Decode the GIF fixture bytes as above; assert the decoder sees a 1x1 image (precondition).
2. Define a MockLoader implementing `underrated::loader::ResourceLoader` that records requested URLs into a `std::cell::RefCell<Vec<String>>` and returns `Ok(gif_bytes.clone())` for the expected image url (and an appropriate `Err(LoadError::...)` otherwise). VERIFY which loader method `<img>` fetching actually calls by reading `fetch_and_decode_images` / `load_image_safely_with_loader` in `src/engine/mod.rs`, and implement that method to return the GIF.
3. Render HTML = `<html><body><img id="g" src="http://example.com/pic.gif" style="width:1px;height:1px;"></body></html>` with base url `underrated::url::Url::parse("http://example.com/").unwrap()` and a viewport width of 800.0, using the same render fn the other tests in this file use.
4. Build the display list and ASSERT:
   a. The MockLoader recorded a request for `http://example.com/pic.gif` (proves the GIF url was fetched).
   b. The display list contains exactly one `DisplayItem::Image`, and its `decoded` is `Some` with `width == 1 && height == 1` (proves sniff->decode_gif->blit succeeded for the GIF format).

If the engine does NOT currently decode/blit the GIF (an assertion would fail due to a real engine bug), do NOT fix the engine and do NOT weaken the test into a no-op. Instead leave a `// TODO(spec): <describe the discrepancy>` comment, assert the actually-observed behavior with an explanatory comment, and clearly report the discrepancy in your final summary. (GIF decode is implemented and the sniffer recognizes `GIF8`, so a green spec-correct assertion is expected; report if not.)

You MAY temporarily add `eprintln!` of recorded urls and decoded dims, run `cargo test <your_test_name> -- --nocapture` to discover the real behavior, then encode robust assertions and REMOVE the eprintln! before committing. Keep the test deterministic (no timing, no real network). Put a short `//` comment above the test naming what it guards.

Run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "test(oracle): verify GIF <img> fetch, decode and blit through engine pipeline (t0422)"
Then print "T0422 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
