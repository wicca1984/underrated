#!/usr/bin/env bash
# Launcher for Gemini worker t0286 — P-1: mechanize "centered"/"max-width" oracle assertions.
# Target: tests/oracle_snapshot_test.rs ONLY. Dispatched via setsid (memory: worker-dispatch-must-setsid).
set -euo pipefail

WT=/workspaces/wt/t0286
LOG=/workspaces/toy-browser/var/worker-logs/t0286.log

read -r -d '' PROMPT <<'EOF' || true
You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow ALL of it (especially I-1..I-7).

Task: t0286 — P-1 "Mechanize the centering / max-width judgment" for the Google smoke oracle test.
The goal is to STOP relying on loose self-judgment of "looks centered" by adding hard, machine-checked
structural assertions to the existing oracle snapshot test, then applying them to the Google mock fixture.

Read first (verify every claim against the actual code before editing):
- docs/SPEC.md and docs/ARCHITECTURE.md under /workspaces/underrated-meta/ (oracle / snapshot sections).
- src/oracle/mod.rs — `pub fn export_snapshot(html, css, width, height) -> serde_json::Value`. It renders
  the page and returns a normalized JSON tree. Each element node has `node["rect"]["x"]`, `["y"]`,
  `["width"]`, `["height"]` as numbers (f64), plus `node["tag"]`, `node["type"]`, `node["attrs"]`, `node["children"]`.
- tests/oracle_snapshot_test.rs — the ONLY file you may modify. It already has helpers:
  `find_element_by_tag(node, tag) -> Option<&Value>`, `find_elements_by_tag(node, tag, &mut results)`,
  `load_fixture_snapshot(filename) -> Value`, and the existing test `test_fixture_07_google_mock` (~line 360).
- tests/oracle/fixtures/07_google_mock.html — the Google mock. It contains `.logo` (width 272, margin auto),
  `.search-box` (width 400, margin auto), and two `<button>` elements inside `.buttons`. Viewport is 800x600.

Target module: tests/oracle_snapshot_test.rs ONLY. Do NOT touch any other file (no src/, no fixtures, no other
worktree). Do NOT change export_snapshot or any production code. This is a TEST-ONLY task.

What to implement (in tests/oracle_snapshot_test.rs):
1. Two new test-helper fns (place them near the other helpers like find_element_by_tag):
   - `fn assert_centered(node: &Value, viewport_width: f64, tolerance_px: f64)`:
       Reads x = node["rect"]["x"], w = node["rect"]["width"] (both as f64). Computes center = x + w/2.0.
       Asserts `(center - viewport_width / 2.0).abs() <= tolerance_px`, with a message that prints the tag,
       the computed center, the expected center (viewport_width/2), and the tolerance.
   - `fn assert_max_width(node: &Value, viewport_width: f64, ratio: f64)`:
       Reads w = node["rect"]["width"] as f64. Asserts `w <= viewport_width * ratio`, with a message that
       prints the tag, w, and the limit (viewport_width*ratio).
   Use `.as_f64()` with a clear `.unwrap_or_else(|| panic!(...))` style for missing fields (this is a #[cfg(test)]
   / integration test file, so panics/unwrap on the rect fields are acceptable — I-6 forbids unwrap only in
   NON-test production code). Match the existing code style in the file (look at assert_structural_invariants).
   Helper that finds a node by its `class` attribute will be needed: either reuse find_elements_by_tag + filter
   on node["attrs"]["class"], or add a small `fn find_element_by_class<'a>(node, class) -> Option<&'a Value>`.

2. Extend the EXISTING test `test_fixture_07_google_mock` (do NOT delete or weaken any of its current
   assertions — keep the script-size and non-white-pixel checks intact). After the existing assertions, add:
   - Locate `.search-box` (the centered search box, width 400 in an 800px viewport) and call
     `assert_centered(&search_box, 800.0, 4.0)` — its center must be within ±4px of viewport center.
   - Locate `.logo` and `assert_centered(&logo, 800.0, 4.0)` as well (also margin:auto centered).
   - Locate the two `<button>` elements (find_elements_by_tag(&snapshot, "button", ...)). For EACH button,
     call `assert_max_width(button, 800.0, 0.4)` — a shrink-to-fit button label+padding must be well under
     40% of viewport width. (Buttons currently shrink-to-fit via calculate_shrink_to_fit_width in src/layout.)

   IMPORTANT — verify before asserting: first run the test once and inspect actual rect values if any
   assertion fails. If `.search-box` or `.logo` does NOT come out centered within 4px, DO NOT loosen the
   tolerance to force green and DO NOT edit production code. Instead, leave the assertion at ±4px, mark the
   test `#[ignore]` is NOT allowed; instead add a `// TODO(spec):` note above the failing assertion describing
   the observed vs expected center, and STOP and report it in your final summary as a real layout regression
   (this is a genuine finding the orchestrator needs). Only commit GREEN if the assertions actually pass.
   (Expectation: with margin:auto + text-align:center already implemented, centering SHOULD pass. The buttons
   SHOULD pass max_width(0.4). If they don't, that is a real bug to report, not to paper over.)

Done when: `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check` ALL pass
(run from the worktree root). Keep ALL existing tests green; do NOT delete or alter any foreign test or
assertion to force green (hard violation).

Commit (you MUST git add + git commit before finishing — uncommitted work is lost):
  `test(oracle): mechanize centered / max-width assertions on google mock smoke (t0286)`

End with a short summary: the helper fn names you added, which nodes you asserted on, and the ACTUAL
measured center-x / width values for .search-box, .logo, and each button (so the orchestrator can verify).
EOF

cd "$WT"
exec gemini -p "$PROMPT" \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  < /dev/null > "$LOG" 2>&1
