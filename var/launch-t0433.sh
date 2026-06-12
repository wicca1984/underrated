#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0433
LOG=/workspaces/toy-browser/var/log/t0433.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything you need is in the local source files and this prompt. Reading local files is fine; network/web search is forbidden and wastes time.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-5 one-module; I-6 NO unwrap/expect in library/src code; test code MAY use unwrap/panic as the existing tests do).

Task t0433 — Add a deterministic DEEPLY-NESTED-DOM profiling/regression fixture. Create exactly ONE new file: `tests/deep_nesting_perf_test.rs`. Do NOT edit any other file (no changes under src/, no changes to other tests).

WHY this is new (do not duplicate): `tests/perf_regression_test.rs` ALREADY exists, but it builds a representative FLEX page that is broad and shallow. The current milestone (MS-NewTargets-Verify & Perf) needs a fixture that specifically stresses DEEP nesting — large real pages (Wiki ~1MB) currently time out, and the suspected hotpaths are O(N^2)-or-worse layout/style/paint over deeply nested subtrees and over very large sibling lists. Your fixture must surface deep-recursion / deep-nesting cost, which the existing flat-ish flex fixture does not.

READ FIRST: open `tests/perf_regression_test.rs` and mirror its structure exactly:
  - the `PerfTestLoader` struct implementing `underrated::loader::ResourceLoader` (its `load` returns `Err(LoadError::NotFound)`),
  - the use of `underrated::engine::render_page_to_canvas`, `underrated::url::Url`, `std::time::Instant`,
  - the way it constructs a `Url`, builds the HTML string, calls the render entry point, and asserts an elapsed-time budget.
Use the SAME public entry point (`render_page_to_canvas`) and the SAME loader pattern so this stays a shipping-path test. Look at the exact signature `render_page_to_canvas` is called with in the existing test and copy it verbatim (same argument order/types, same viewport width).

YOUR FIXTURE — write a `fn generate_deeply_nested_fixture() -> String` that DETERMINISTICALLY builds:
  - a chain of deeply nested block elements (e.g. ~1500 nested `<div>`s, each opening then later closing) wrapping a small text leaf, AND
  - within it, one node with a large deterministic sibling list (e.g. ~2000 sibling `<span>text</span>` elements),
  so both deep recursion and wide sibling iteration are exercised. Pick fixed counts (named `const` so the test is deterministic and self-documenting); do NOT use randomness or time-based values. Add a `<style>` with a few simple block rules (margin/padding/font-size) so style + layout + paint all run. Keep the HTML fully self-contained (no external resources; the loader returns NotFound anyway).

Add ONE `#[test] fn deep_nesting_render_within_budget()` that:
  - records `Instant::now()`, calls `render_page_to_canvas(...)` on the generated fixture, measures elapsed,
  - asserts elapsed is under a GENEROUS debug-build budget (follow the existing test's rationale; pick a ceiling in the same spirit — e.g. a few seconds — generous enough to avoid CI flakiness but still catch catastrophic/exponential regressions). Add a `//` comment justifying the chosen budget, mirroring the existing test's justification comment.
  - Also assert the render produced a non-empty canvas (e.g. canvas width/height > 0) so the test fails loudly if rendering silently no-ops. Inspect what `render_page_to_canvas` returns in the existing test and assert on it the same way.
  - Add a `// TODO(spec):` note that this budget is a generous baseline pending an official perf gate spec (mirror the existing file's TODO(spec) note).

CONSTRAINTS:
  - Keep I-6: NO `unwrap`/`expect` in non-test code. This is a test file, so `unwrap`/`expect`/`panic` ARE allowed exactly as the existing perf test uses them.
  - WARNING: deep recursion can blow the stack on Windows CI (there is a stack-overflow regression guard job). Do NOT pick an absurd nesting depth that would overflow the engine's recursion on a normal stack; ~1500 deep is a reasonable target. If unsure, mirror depths the codebase already tolerates (search src/ and tests/ for existing recursion-depth guards or comments). Keep the depth comfortably within what the engine already handles.

Run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test --test deep_nesting_perf_test`. Then run the full `cargo test` to confirm nothing else broke. If all green:
  git add -A && git commit -m "test(perf): add deterministic deeply-nested-DOM render budget fixture (t0433)"
Then print "T0433 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
