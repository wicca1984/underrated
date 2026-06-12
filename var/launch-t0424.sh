#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0424
LOG=/workspaces/toy-browser/var/log/t0424.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything you need is in the local source files and this prompt. Reading local files is fine; network/web search is forbidden and wastes time.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code — note: test code MAY use unwrap/panic as the existing tests do).

Task t0424 — Add ONE new integration test that proves an `<img loading="lazy">` is STILL fetched, decoded, and blitted through the engine pipeline (i.e. the engine treats `loading="lazy"` as eager because there is no viewport/scroll lazy mechanism). Touch ONLY the file `tests/oracle_snapshot_test.rs`. Do NOT edit any file under src/, and do NOT edit any fixture HTML.

WHY this is new (do not duplicate existing coverage): `tests/oracle_snapshot_test.rs` already has end-to-end engine tests for PNG (`test_b3_relative_url_image_pipeline`) and GIF (`test_gif_image_pipeline_end_to_end`). `src/engine/mod.rs` already covers srcset/picture selection. But there is NO test anywhere that proves an image carrying `loading="lazy"` is still fetched+decoded+blitted through the engine. Read `fetch_and_decode_images` in `src/engine/mod.rs` first and CONFIRM it does not skip images based on the `loading` attribute (it currently fetches every `<img src>` eagerly). Your test guards that documented decision ("loading=lazy treated as eager"). If, contrary to this, you find the engine actually DOES skip lazy images, do NOT modify src/ — instead leave a `// TODO(spec):` comment, assert the actually-observed behavior, and clearly report the discrepancy in your final summary.

Model your test EXACTLY on the existing `test_gif_image_pipeline_end_to_end` in `tests/oracle_snapshot_test.rs` (read it in full first). Reuse the SAME MockLoader / `underrated::loader::ResourceLoader` pattern, the SAME render entry point (`underrated::engine::render_page(html, &base_url, &loader, 800.0)`), and the SAME `underrated::paint::build_display_list(...)` + `DisplayItem::Image { src, decoded, .. }` assertions. Read EXACT current signatures from source before using them — do not assume.

Reuse the existing valid 1x1 PNG used by the relative-url test (find how `test_b3_relative_url_image_pipeline` obtains its PNG bytes and copy that exact approach — either its base64 const decoded via `underrated::loader::decode_base64`, or its byte source). Confirm `underrated::image::decode_image(&png_bytes)` returns `Some` with width==1 && height==1 as a sanity precondition.

Steps:
1. Decode the 1x1 PNG fixture bytes (same source as the relative-url test); assert decoder sees a 1x1 image (precondition).
2. Define a MockLoader implementing `underrated::loader::ResourceLoader` that records requested URLs into a `std::cell::RefCell<Vec<String>>` and returns `Ok(png_bytes.clone())` for the expected image url (else `Err(LoadError::NotFound)` or the appropriate variant — read `src/loader/mod.rs` for the exact `LoadError` variants and the exact trait method signature). Implement the method that `<img>` fetching actually calls (verify by reading `fetch_and_decode_images`).
3. Render HTML = `<html><body><img id="g" src="http://example.com/lazy.png" loading="lazy" style="width:1px;height:1px;"></body></html>` with base url `underrated::url::Url::parse("http://example.com/").unwrap()` and viewport width 800.0, using `render_page`.
4. Build the display list and ASSERT:
   a. The MockLoader recorded a request for `http://example.com/lazy.png` (proves the lazy image WAS fetched eagerly, not skipped).
   b. The display list contains exactly one `DisplayItem::Image`, and its `decoded` is `Some` with `width == 1 && height == 1` (proves fetch->decode->blit succeeded despite loading="lazy").
Put a short `//` comment above the test naming what it guards (e.g. "guards: loading=\"lazy\" images are still fetched+decoded+blitted (treated as eager, no viewport lazy mechanism)"). Name the test `test_lazy_loading_image_pipeline_end_to_end`.

You MAY temporarily add `eprintln!` of recorded urls and decoded dims, run `cargo test test_lazy_loading_image_pipeline_end_to_end -- --nocapture` to discover the real behavior, then encode robust assertions and REMOVE the eprintln! before committing. Keep the test deterministic (no timing, no real network).

Run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "test(oracle): verify loading=lazy <img> is still fetched, decoded and blitted through engine pipeline (t0424)"
Then print "T0424 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
