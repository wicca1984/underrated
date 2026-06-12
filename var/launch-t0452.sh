#!/usr/bin/env bash
# t0452 — verified-in-window E2E search-render example (examples/e2e_search_render.rs). Base: origin/main 8043b67.
set -euo pipefail
cd /workspaces/wt/t0452

read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Write the code, run the checks, fix until green, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything is in local files. Network/web search is forbidden (cargo may fetch crates from crates.io — that is allowed and is NOT web search).

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. Read AGENTS.md (passed via --include-directories) and obey I-1..I-7. NEVER use unwrap()/expect() in non-test code (I-6); this is an example binary with a `main()` that returns nothing, so handle errors with `match`/`if let`/`eprintln!` + `std::process::exit`, NOT unwrap/expect. DO NOT delete or skip tests to fake green.

You are the ONLY worker on this worktree: /workspaces/wt/t0452, branch agent/t0452-e2e-search-render, base origin/main (commit 8043b67). Create exactly ONE new file: examples/e2e_search_render.rs. DO NOT modify any other file (no src/, no existing examples, no tests). Adding a new example file does NOT require touching Cargo.toml — Cargo auto-discovers files in examples/.

GOAL — produce verified-in-window PNG evidence that the full MVP search interaction loop (focus an input -> type a query -> press Enter -> submit -> navigate -> render the result page) works through the SHIPPING render path. This example renders to PNG exactly the way the GUI ships pixels.

REFERENCE EXISTING CODE (read these first to copy the exact public APIs and patterns):
- examples/render_local_png.rs — shows the PNG output path: `engine::render_page_to_canvas(...)`, `image::encode_png(&canvas)`, and writing bytes to a file under var/. Copy its DummyLoader/arg-parse style only as needed.
- tests/e2e_search_flow.rs — shows the EXACT search-flow driver: building a home page with a `<form action="/search" method="get">` and an `<input name="q">`, `engine::render_page(...)`, `find_box_rect`/`hit_test` to locate the input rect, `shell::ShellInputManager` (`handle_click`, `focused_element`, `set_text_buffer`, `text_buffer`), `forms::FormState::set_value`, and `engine::navigate_from_enter(...)` which returns the result `Page`. Reuse the same helper logic (you may inline small helpers like find_input_by_name / find_box_rect into the example).
- src/engine/mod.rs around line 553-562 — `render_page_to_canvas` internally does:
      let display_list = crate::paint::build_display_list(&page.layout, &page.dom, &page.styles);
      crate::raster::rasterize(&display_list, width, height)
  Use the PUBLIC equivalents from the example: `underrated::paint::build_display_list(&page.layout, &page.dom, &page.styles)` then `underrated::raster::rasterize(&display_list, width, height)` to rasterize a `Page` you already have (navigate_from_enter returns a Page, NOT HTML, so you must rasterize it via build_display_list + rasterize — render_page_to_canvas only takes HTML and cannot be used for the result Page).

WHAT THE EXAMPLE MUST DO (in `fn main()`):
1. Define an offline mock ResourceLoader (implement `underrated::loader::ResourceLoader`, both `load` and `load_request`) that returns a canned search-result page for the submitted GET URL. Mirror the mock in tests/e2e_search_flow.rs: it must answer `GET https://example.com/search?q=rust+lang` with bytes like `b"<html><body><h1>Search results: rust lang</h1></body></html>"`. For any other URL return `LoadError::NotFound`. NO real network.
2. Home page HTML: a deterministic search form (copy the `home_html` string from tests/e2e_search_flow.rs verbatim, including the inline CSS that forces a non-zero input rect: `input { display: block; width: 200px; height: 40px; }`).
3. base_url = `https://example.com/`, viewport width 800, height 600 (use u32 for raster width/height, f32 for layout viewport_width as the existing APIs require — match the signatures you see in the reference files).
4. Render the initial home page: `engine::render_page(home_html, &base_url, &mock, 800.0)` -> initial Page. Rasterize it (build_display_list + rasterize 800x600) and save PNG to `var/e2e-search-home.png` via `image::encode_png`.
5. Locate the `<input name="q">` node, find its layout box rect, compute the center point, `hit_test` to get the node id, drive `ShellInputManager`: handle_click(center), assert/confirm focused_element() == input id, `set_text_buffer(input_id, "rust lang")`.
6. `FormState::set_value(input_id, "rust lang")`.
7. `engine::navigate_from_enter(&initial.dom, input_id, &form_state, &base_url, &mock, 800.0)` -> Option<Page>. On None, `eprintln!` and exit non-zero. On Some(result_page): rasterize it (build_display_list + rasterize 800x600) and save PNG to `var/e2e-search-result.png`.
8. Print to stdout the two PNG paths written and a one-line confirmation that the result page's <h1> text content equals "Search results: rust lang" (look it up via `result_page.dom` text_content of the h1, as the test does), so the run self-verifies the loop end to end.

CORRECTNESS NOTES:
- This is an EXAMPLE (cargo run --example e2e_search_render), not a test, so it has no #[test]. It must compile under `cargo clippy --all-targets -- -D warnings` (examples are built by --all-targets) and `cargo fmt --check`.
- Do not use unwrap()/expect() in the example body — examples are non-test code for I-6. Use match / if let Some / `let Some(x) = .. else { eprintln!(..); std::process::exit(1) }`.
- Ensure the `var/` directory exists before writing (create parent dir like render_local_png does).
- Keep it self-contained and offline-deterministic.

PROCEDURE (iterate until all green):
  - cargo build --examples
  - cargo run --example e2e_search_render        (must write both PNGs and print the h1 confirmation)
  - cargo fmt
  - cargo clippy --all-targets -- -D warnings    (fix every warning)
  - cargo test                                   (all existing tests still pass — you added none, but confirm nothing broke)
  - git add -A && git commit -m "test(example): add verified-in-window E2E search-render harness (t0452)"
  COMMIT before finishing (commit partial progress too). Report the final cargo test summary line, the two PNG paths, and the exact h1 confirmation line your run printed.
EOF

exec gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta < /dev/null
