#!/usr/bin/env bash
# Launcher for Gemini worker t0292 — B-3 relative-URL image blit verification (oracle test).
# Target: tests/oracle_snapshot_test.rs ONLY. Dispatched via setsid (memory: worker-dispatch-must-setsid).
set -euo pipefail

WT=/workspaces/wt/t0292
LOG=/workspaces/toy-browser/var/worker-logs/t0292.log

read -r -d '' PROMPT <<'EOF' || true
You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow ALL of it (especially I-1..I-7).

Task: t0292 — add a regression/verification test proving that an `<img>` with a RELATIVE-URL `src` (e.g.
`/images/logo.png`) is correctly resolved against the page's base URL, fetched through the injected
ResourceLoader, decoded, and BLITTED to the canvas. This locks down the Google-logo case (MS-Regression-Google
B-3). This is a TEST-ONLY task: do NOT change any production code. The production resolution path already
works (engine::fetch_and_decode_images resolves `src` against the page base via `load_image_safely_with_loader`
and stores the decoded image with `dom.add_image`); the only reason it appeared "broken" before is that the
offline render harness used a loader that always returns NotFound. Your job is to prove the full chain works
with a stub loader that actually returns image bytes.

Target file: tests/oracle_snapshot_test.rs ONLY. Do NOT touch any other file (no src/ module, no other test,
no fixtures, no Cargo.toml, no other worktree). 1 task = 1 module (I-5). Test-side `unwrap`/`expect`/`assert!`
is allowed (I-6 forbids unwrap only in non-test PRODUCTION code).

Read first (verify every claim against the ACTUAL code before writing — do not trust this description blindly):
- This file already contains `#[test]` functions that call
  `underrated::engine::render_page_to_canvas(&html, &base_url, &Loader, width, height)` and define a local
  `struct DummyLoader` implementing `underrated::loader::ResourceLoader` (search for `impl ... ResourceLoader`
  in this file around lines 405 and 488). COPY that loader's method signatures EXACTLY (both `load` and
  `load_request`) — do not invent new ones.
- `underrated::image::encode_png(&canvas) -> Vec<u8>` encodes a `underrated::raster::Canvas` (pixels are
  `Vec<u32>` in 0xAARRGGBB) into PNG bytes. `underrated::raster::Canvas::new(width, height)` exists. Use these
  to MANUFACTURE the image bytes inside the test — do NOT add a binary fixture file.
- `underrated::loader::ResourceLoader` is a trait with `load(&self, &Url)` returning
  `Result<Vec<u8>, underrated::loader::LoadError>`. The engine calls `loader.load(resolved_url)` for images.

Write ONE new test `fn test_b3_relative_url_image_blits()` that:
  1. Builds a Canvas of a small known size (e.g. 40x20), fills EVERY pixel with an opaque, distinctive color
     that is neither white nor black — use opaque BLUE `0xFF0000FF`. Encode it: `let png = encode_png(&canvas);`
     assert `!png.is_empty()`.
  2. Defines a `struct StubLoader { png: Vec<u8> }` implementing `ResourceLoader`. Its `load` returns
     `Ok(self.png.clone())` ONLY when the requested URL serializes to EXACTLY
     `https://www.example.com/images/logo.png` (compare via the same URL string accessor the other tests /
     code use — inspect how `Url` exposes its string form, e.g. a `serialize()` method or `to_string`; find
     the real API, do not guess), and returns `Err(underrated::loader::LoadError::NotFound)` otherwise.
     `load_request` returns `Err(NotFound)`.
  3. HTML: a minimal page whose ONLY meaningful element is
     `<img src="/images/logo.png" width="40" height="20">` (RELATIVE src, no `<base>` tag). Put it in normal
     flow (e.g. inside `<body>`). Render with
     `base_url = underrated::url::Url::parse("https://www.example.com/").unwrap()` and the StubLoader, viewport
     e.g. 200x100.
  4. Assert the rendered canvas actually contains BLUE pixels: count pixels equal to the blue value
     (0xFF0000FF, allowing for the exact value since it is opaque and unscaled) and assert the count is > 0
     (ideally roughly the 40x20 = 800 area, but a strict `> 100` is enough to prove the blit happened and is
     not a stray dot). This proves: relative `src` -> resolved against `https://www.example.com/` ->
     `https://www.example.com/images/logo.png` -> fetched via loader -> decoded -> blitted.
  5. ALSO add a negative control assertion in the SAME test: render the SAME html+base with a loader that
     always returns `NotFound` (you may reuse a local `DummyLoader` pattern) and assert the canvas has ZERO
     blue pixels — proving the blue only appears when the loader actually serves the bytes (i.e. the test is
     not vacuously passing).

If anything about the URL string API or the loader trait signature does not match this description, TRUST THE
CODE, not this prompt, and adapt — but keep the test's intent (relative-URL image resolves+fetches+blits).

  // TODO(spec): B-3 verification — proves relative-URL <img> resolves against page base, fetches via loader,
  // and blits. Real external fetch (HttpLoader/network) and placeholder-on-failure rendering are out of scope.
Leave that TODO(spec) marker as a comment above the new test.

Do NOT delete, weaken, `#[ignore]`, or alter any existing test or assertion (hard violation). Add only your
one new test (plus the local StubLoader struct it needs).

Done when (run from the worktree root /workspaces/wt/t0292):
  `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check` ALL pass.

Commit (you MUST `git add -A && git commit` before finishing — uncommitted work is lost; the worktree may be
force-removed after you exit):
  `test(oracle): verify relative-URL <img> resolves against page base and blits (t0292)`

End with a short summary: the URL string API you used in the StubLoader match, the blue-pixel count your
positive assertion observed, and confirmation that the negative-control (NotFound loader) path has zero blue.
EOF

cd "$WT"
exec gemini -p "$PROMPT" \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  < /dev/null > "$LOG" 2>&1
