#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0427
LOG=/workspaces/toy-browser/var/log/t0427.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything you need is in the local source files and this prompt. Reading local files is fine; network/web search is forbidden and wastes time.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-5 one-module; test code MAY use unwrap/panic as the existing tests do).

Task t0427 — Add ONE new integration test proving that a responsive `<img srcset>` is resolved to a candidate URL that is then fetched, decoded, and blitted through the FULL engine pipeline. Touch ONLY the file `tests/oracle_snapshot_test.rs`. Do NOT edit any file under src/ and do NOT edit any fixture HTML.

WHY this is new (do not duplicate): `tests/oracle_snapshot_test.rs` already has end-to-end engine tests for PNG (`test_b3_relative_url_image_pipeline`), GIF (`test_gif_image_pipeline_end_to_end`), and loading=lazy (`test_lazy_loading_image_pipeline_end_to_end`), but NONE for `srcset` candidate selection through the engine pipeline. `src/engine/mod.rs` has unit coverage of srcset/picture selection, but there is no end-to-end oracle test proving the SELECTED srcset candidate is the URL actually fetched+decoded+blitted. Your test guards that.

Read these IN FULL before writing anything:
- `test_lazy_loading_image_pipeline_end_to_end` and `test_gif_image_pipeline_end_to_end` in `tests/oracle_snapshot_test.rs` — copy their MockLoader / `underrated::loader::ResourceLoader` pattern EXACTLY, the same render entry point (`underrated::engine::render_page(html, &base_url, &loader, 800.0)` — confirm the exact signature from source), and the same `underrated::paint::build_display_list(...)` + `DisplayItem::Image { src, decoded, .. }` assertions.
- `src/engine/mod.rs` `fetch_and_decode_images` and the srcset-parsing/selection helper it uses — DISCOVER the engine's actual candidate-selection rule (which candidate it picks for viewport width 800 with no DPR info — likely the first/last/width-matched candidate). Do NOT assume; read the code and pick a srcset whose SELECTED url is unambiguous and recorded by your MockLoader.
- `src/loader/mod.rs` for the EXACT `ResourceLoader` trait method signature and `LoadError` variants.

Steps:
1. Obtain a valid 1x1 PNG the same way `test_b3_relative_url_image_pipeline` does (reuse its base64 const + `underrated::loader::decode_base64`, or its exact byte source). Assert `underrated::image::decode_image(&png_bytes)` returns `Some` with width==1 && height==1 (precondition).
2. Define a MockLoader (RefCell<Vec<String>> of requested urls) returning `Ok(png_bytes.clone())` for the candidate url you EXPECT the engine to select, and an appropriate `Err(LoadError::...)` for any other url. Implement the exact trait method `<img>` fetching calls.
3. Render HTML containing an `<img>` with a `srcset` listing at least two candidates (e.g. `srcset="http://example.com/small.png 200w, http://example.com/large.png 800w"` plus a `src` fallback to a THIRD distinct url) at base url `http://example.com/` and viewport width 800.0 via `render_page`. Choose the srcset values so that, given the engine's real selection rule you read in step's source review, exactly one candidate is selected — and it is NOT the `src` fallback (this proves srcset was honored over src).
4. Build the display list and ASSERT:
   a. The MockLoader recorded a request for the SELECTED srcset candidate url (and did NOT record the un-selected candidates / the src fallback — assert the recorded set matches expectation, proving srcset resolution drove the fetch).
   b. The display list contains exactly one `DisplayItem::Image` whose `decoded` is `Some` with width==1 && height==1.
You MAY temporarily `eprintln!` the recorded urls, run `cargo test <name> -- --nocapture` to confirm which candidate the engine truly picks, then encode robust assertions and REMOVE the eprintln! before committing. If the engine's real selection differs from your initial guess, ADJUST the assertions to the observed real behavior (do NOT modify src/). Keep deterministic (no real network, no timing).
Put a short `//` comment above the test naming what it guards. Name it `test_srcset_image_pipeline_end_to_end`.

Run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "test(oracle): verify srcset candidate is selected, fetched, decoded and blitted through engine pipeline (t0427)"
Then print "T0427 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
