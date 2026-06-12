#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0430
LOG=/workspaces/toy-browser/var/log/t0430.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything you need is in the local source files and this prompt. Reading local files is fine; network/web search is forbidden and wastes time.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-5 one-module; I-6 NO unwrap/expect in library/src code; test code MAY use unwrap/panic as the existing tests do).

Task t0430 — Add a NEW deterministic "dynamic-DOM" oracle snapshot fixture (modeling a JS-driven site like YouTube whose visible content is built by inline script at load time) and a test that verifies the POST-JS layout result. Touch ONLY:
  - a new fixture file `tests/oracle/fixtures/11_dynamic_dom.html`
  - the test file `tests/oracle_snapshot_test.rs` (add ONE new `#[test]` fn)
Do NOT edit any other file. Do NOT edit any file under src/.

WHY this is valuable & how it works (study before writing):
- Read `tests/oracle_snapshot_test.rs` in full. Note the helper `load_fixture_snapshot(filename)` (around line ~232): it reads `tests/oracle/fixtures/<filename>`, calls `underrated::oracle::export_snapshot(&html, "", 800, 600)`, runs `assert_structural_invariants`, and returns a serde_json snapshot tree. Note helpers `find_element_by_tag`, `find_elements_by_tag` and how existing tests (e.g. `test_fixture_06_vertical_stack`, `test_fixture_01_single_block_text`) assert on `snapshot["tag"]`, child structure, and per-element `rect` (x/y/width/height).
- Read `src/oracle/mod.rs` `export_snapshot`: it calls `crate::engine::render_page(...)`, which runs the FULL pipeline INCLUDING inline `<script>` execution, then snapshots the resulting (post-JS) DOM + layout. So a fixture whose body is populated by an inline script will, after rendering, expose the script-created elements in the snapshot. CONFIRM this by reading render_page if unsure; if (and only if) script execution turns out NOT to run in this path, fall back to asserting on the static skeleton and add a `// TODO(spec):` note — but FIRST try the dynamic-DOM approach.
- Look at the existing fixtures `tests/oracle/fixtures/06_vertical_stack.html` and `09_wiki_article.html` for the HTML conventions used (doctype, minimal CSS, structure).

Build the fixture `11_dynamic_dom.html` (keep it SMALL, fully self-contained, NO network/external resources, fully deterministic):
- A minimal HTML document with an empty container, e.g. `<div id="app"></div>`.
- An inline `<script>` that, at load, deterministically creates and appends a fixed set of child elements into `#app` — e.g. a loop that appends exactly 3 `<div class="card">` items, each containing a known text node (use `document.createElement`, `appendChild`, `textContent` / `innerHTML` — use whichever DOM APIs are already supported; check `src/script/mod.rs` bindings if needed, but do NOT edit it). Give each card simple block CSS (e.g. width/height via a `<style>` block) so the layout rects are predictable and stack vertically.
- The script must be deterministic (no Date/Math.random/network) so the snapshot is stable across runs.

Add ONE `#[test] fn test_fixture_11_dynamic_dom()` to `tests/oracle_snapshot_test.rs`:
- Call `load_fixture_snapshot("11_dynamic_dom.html")`.
- Assert the snapshot root `tag` is `html`.
- Find the `#app` container and assert it now contains exactly 3 script-created `.card` div children (use `find_elements_by_tag` and/or attribute checks) — i.e. prove the inline script ran and mutated the DOM before snapshotting.
- Assert the cards have non-negative rects and stack in increasing `y` (vertical order) per the existing rect-assertion style.
- Put a short `//` comment above the test naming what it guards (post-JS dynamic DOM layout).

Run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "test(oracle): add dynamic-DOM (post-JS) snapshot fixture and verify script-built layout (t0430)"
Then print "T0430 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
