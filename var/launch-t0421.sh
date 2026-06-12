#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0421
LOG=/workspaces/toy-browser/var/log/t0421.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything you need is in the local source files and this prompt. Reading local files is fine; network/web search is forbidden and wastes time.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code — note: test code MAY use unwrap/panic as the existing tests do).

Task t0421 — Add ONE new integration test that proves the B-3 RELATIVE-URL image pipeline: a relative `<img src="logo.png">` is resolved against the page base URL, fetched via the injected loader, decoded, and emitted as a blittable image in the display list. Touch ONLY the file `tests/oracle_snapshot_test.rs`. Do NOT edit any file under src/, and do NOT edit any fixture HTML.

WHY this is new (do not duplicate existing coverage): the in-crate test `test_remote_img_fetch_decode_blit_s90` in `src/engine/mod.rs` already covers an ABSOLUTE url (`http://example.com/image.png`) and a `data:` URI. It does NOT cover a RELATIVE src resolved against the page base — which is exactly the known B-3 gap. Your test must use a RELATIVE src and prove base-relative resolution end-to-end through the public API.

There is a stub comment near line 665 of `tests/oracle_snapshot_test.rs`:
  `// TODO(spec): B-3 verification — proves relative-URL <img> resolves against page base, fetches via loader, and blits.`
Replace/extend that area with the real test (keep a short `//` comment above the test naming what it guards). Append the new `#[test]` fn right after that stub.

All needed items are PUBLIC from the `underrated` crate (this is an integration test in `tests/`, so use the `underrated::` path prefix, NOT `crate::`):
- `underrated::engine::render_page(html: &str, base_url: &underrated::url::Url, loader: &impl underrated::loader::ResourceLoader, viewport_width: f64) -> Page`
- `underrated::url::Url::parse(&str)` and `underrated::url::resolve(base, rel)`
- `underrated::raster::Canvas` (pub fields `width`, `height`, `pixels: Vec<u32>` of 0xAARRGGBB; `Canvas::new(w,h)`)
- `underrated::image::encode_png(&Canvas) -> Vec<u8>` and `underrated::image::DecodedImage { width, height, rgba }`
- `underrated::paint::build_display_list(&page.layout, &page.dom, &page.styles) -> underrated::paint::DisplayList` (tuple struct `DisplayList(pub Vec<DisplayItem>)`)
- `underrated::paint::DisplayItem::Image { src, decoded, rect, .. }` where `decoded: Option<DecodedImage>`

Model your scaffolding on `test_remote_img_fetch_decode_blit_s90` (read it in `src/engine/mod.rs` first to copy the PNG-generation pattern), but adapt for the integration crate and a RELATIVE url. Steps:
1. Generate a small valid PNG: build a `Canvas::new(2, 2)`, set its 4 `pixels` to distinct ARGB colors, then `let png = underrated::image::encode_png(&canvas);`.
2. Define a `MockLoader` struct implementing `underrated::loader::ResourceLoader` that:
   - records every requested URL string into a `std::cell::RefCell<Vec<String>>` (so the test can later assert WHICH absolute url was requested), and
   - returns `Ok(png.clone())` for the EXPECTED resolved absolute url, and an appropriate `Err(...)` (e.g. `LoadError::NotFound`) otherwise.
   Read the EXACT `ResourceLoader` trait signature (methods, `LoadError`, `LoaderResponse`, `HttpMethod`) from `src/loader/mod.rs` and the existing `DummyLoader` impl already in `tests/oracle_snapshot_test.rs` (around line 450) — match them precisely. Implement the image-fetch method to return the PNG and the http(s) method to whatever the existing DummyLoader pattern uses (it can record + delegate, or simply record the url and return NotFound for the http method if image fetch goes through the image-loading path — VERIFY which loader method `render_page` actually calls for `<img>` by reading `fetch_and_decode_images` / `load_image_safely_with_loader` in `src/engine/mod.rs`).
3. Render: HTML = `<html><body><img id="logo" src="logo.png" style="width:2px;height:2px;"></body></html>`; base = `underrated::url::Url::parse("http://example.com/dir/").unwrap()`. The expected resolved url is therefore `http://example.com/dir/logo.png` — compute it via `underrated::url::resolve(&base, "logo.png")` rather than hardcoding, and confirm it equals that string.
4. Call `render_page(html, &base, &loader, 800.0)`, then `build_display_list(...)`.
5. ASSERT (the new facts):
   a. The MockLoader recorded a request for the base-resolved ABSOLUTE url (`http://example.com/dir/logo.png`) — this proves relative→base resolution happened.
   b. The display list contains exactly one `DisplayItem::Image`, and its `decoded` is `Some` with `width == 2 && height == 2` and the first RGBA pixel matching the color you set — this proves fetch+decode+blit succeeded for the relative url.

If the engine does NOT currently resolve/blit the relative url (i.e. an assertion would fail because of a real engine bug), do NOT fix the engine and do NOT weaken the test into a no-op. Instead leave a `// TODO(spec): <describe the discrepancy>` and assert the actually-observed behavior with an explanatory comment, and clearly report the discrepancy in your final summary. (Per known root-cause notes the relative path SHOULD work, so a green spec-correct assertion is expected; report if not.)

You MAY temporarily add `eprintln!` of recorded urls and decoded dims, run `cargo test <your_test_name> -- --nocapture` to discover the real behavior, then encode robust assertions and REMOVE the eprintln! before committing. Keep the test deterministic (no timing, no real network).

Run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "test(oracle): verify relative-URL <img> base resolution, fetch, decode and blit (t0421)"
Then print "T0421 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
