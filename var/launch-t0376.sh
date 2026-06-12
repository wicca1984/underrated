#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0376
LOG=/workspaces/toy-browser/var/log/t0376.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

Task t0376 — add a deterministic, headless E2E oracle test for the search interaction flow.

Target module: tests/ (create ONE new file, e.g. tests/e2e_search_flow.rs; touch ONLY the tests/ directory — do not modify any src/ module).

Goal: a regression guard for the MVP-Live interaction (home page -> locate search <input> -> focus -> type a query -> press Enter -> navigate to result page) that runs WITHOUT a window, by driving the EXISTING pure functions directly. No winit, no real network.

Use the public crate API (read the source to confirm exact signatures/paths before calling):
- Render a small local fixture HTML containing a `<form>` with a text `<input name="q">` and a submit control, into a `Page` (find the public render entry the other integration tests in tests/ already use — mirror their setup; many use a `ResourceLoader` impl).
- `crate::layout::hit_test(root, x, y) -> Option<NodeId>` to locate the input by clicking its rect coordinates (read the laid-out box rect to pick a point inside it).
- `crate::shell::ShellInputManager` to focus the hit node and set its text buffer to a query (e.g. "rust lang").
- `crate::forms::FormState` to carry the typed value for the input's name.
- `crate::engine::navigate_from_enter(dom, focused, &form_state, &base, &loader, viewport_width) -> Option<Page>` to simulate Enter. Provide a deterministic local `ResourceLoader` impl whose `load_request` returns a canned result-page HTML (so the result page is fully deterministic, no network).
- Assert on the resulting `Page`: e.g. the navigation produced `Some(page)`, the submitted URL/method matches expectations, and the result page DOM/text contains a marker string from your canned response. Prefer asserting structural facts over a pixel snapshot; if you do a render assertion, assert on text fragments, not raw colors.

Hard constraints (AGENTS.md I-1..I-7):
- Touch ONLY the tests/ directory; do NOT edit src/. If a needed API is private, leave a `// TODO(spec): expose <X>` comment and assert what you can through the public surface instead of changing src.
- NO `unwrap()`/`expect()` is allowed in test code, but prefer explicit assertions; if you must unwrap in a test, that is acceptable per project test conventions — mirror how existing tests in tests/ handle Option/Result.
- Do NOT skip or #[ignore] tests. The test must actually run and pass.
- Make it fully deterministic: no real DNS/HTTP, no Date/time/random dependence.
- Keep `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` green.

Before writing, READ an existing integration test under tests/ (e.g. one that constructs a Page or uses a ResourceLoader) and copy its setup idioms exactly so paths/types compile on the first try.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If green, commit:
  git add -A && git commit -m "test(e2e): deterministic search interaction oracle (home->type->Enter->result) (t0376)"
Then print "T0376 DONE" as the last line.
EOF
exec gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
