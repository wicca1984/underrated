#!/usr/bin/env bash
# Launcher for Gemini worker t0281 — Element.matches() / Element.closest() DOM bindings (src/script/mod.rs).
# Dispatched via setsid so it survives the orchestrator tick (memory: worker-dispatch-must-setsid).
set -euo pipefail

WT=/workspaces/wt/t0281
LOG=/workspaces/toy-browser/var/worker-logs/t0281.log

read -r -d '' PROMPT <<'EOF' || true
You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow ALL of it (especially I-1..I-7).

Task: t0281 — Implement `Element.matches(selectorString)` and `Element.closest(selectorString)` DOM bindings in the boa JS host.
Read: docs/ARCHITECTURE.md and docs/SPEC.md (script / DOM bindings) under /workspaces/underrated-meta/.
Target module: src/script/mod.rs ONLY. Do NOT touch any other module or any file outside src/script/mod.rs (lib.rs additions are forbidden here), or any other worktree. You MAY call existing public selector-matching helpers in src/dom/query.rs, but do NOT modify src/dom/query.rs.

Background (already true in the code — verify before changing):
- `bridge_query_selector` / `bridge_query_selector_all` already exist in src/script/mod.rs and reuse selector matching from src/dom/query.rs (look at exactly how they parse the selector string and resolve a node key to a NodeId via `with_dom(|dom, key_to_node| ...)`). MIRROR that existing pattern — do not invent a new selector parser.
- JS-side element objects carry a `__key__` and methods are attached both on the per-node prototype block (search for where `node.contains = function(...)` and `Object.defineProperty(node, 'childNodes', ...)` are installed) and, where applicable, on `document`.

Goal — implement WHATWG semantics:
- `element.matches(selector)` -> returns Boolean: true iff `element` itself matches the CSS selector. Does NOT search descendants.
- `element.closest(selector)` -> returns the closest ancestor-or-self Element that matches the selector (walk: self, then parentElement chain), or `null` if none. Returns an element object (same shape query_selector returns), NOT a key string.
- Both: if the selector string fails to parse, follow the SAME behavior the existing query_selector bridge uses for invalid selectors (inspect it — likely returns false/null rather than throwing). Match that exactly; do not diverge.

Implementation notes:
- Add `bridge_matches` and `bridge_closest` native fns next to `bridge_query_selector`, registered with `.function(NativeFunction::from_fn_ptr(bridge_matches), JsString::from("matches"), 1)` etc. in the same builder block where `bridge_query_selector` is registered.
- Attach `element.matches = function(sel) { return bridge.matches(this.__key__, sel); }` and `element.closest = function(sel) { ... }` in the SAME per-node prototype JS block where `node.contains` was just added. For `closest`, the bridge should return the matched node's key (or null); reuse the SAME JS helper the query_selector path uses to turn a returned key into an element object (find how query_selector wraps a key into a JS element and reuse it verbatim) so `closest` returns a real element with `.id`, `.matches`, etc. If query_selector returns a wrapped element directly from the bridge, mirror that; if it returns a key the JS wraps, mirror that. Be consistent with the existing path.
- Inspect the actual selector-matching helper signature in src/dom/query.rs (e.g. a function that tests whether a single NodeId matches a parsed selector). If only `query_selector_from`-style descendant search exists, implement `matches` by checking whether the element is among results anchored at itself, OR better, find/use the single-node match predicate. Do NOT modify src/dom/query.rs — if it lacks a usable public predicate, implement matching locally in src/script/mod.rs by parsing the selector the same way and comparing against the single node.
- No `unwrap()`/`expect()` in non-test code (I-6). Use the same error/`unwrap_or_default` patterns the neighboring bridge fns use.

Approach: test-first (TDD).
Acceptance (must be green) — add unit tests in the existing `#[cfg(test)]` module of src/script/mod.rs using the `eval_with_dom` harness (copy the style of `test_node_contains`):
- Build a small DOM (e.g. div#a > span.b.highlight > … plus a sibling). Then assert via JS:
  - `getElementById('b-span').matches('span.highlight')` => "true"
  - `getElementById('b-span').matches('div')` => "false"
  - `getElementById('b-span').matches('#b-span')` => "true"
  - `getElementById('b-span').closest('div')` resolves to the ancestor div (e.g. compare its `.id` to "a") => the expected id string
  - `getElementById('b-span').closest('.no-such-class')` => "null" (assert the JS stringifies to "null")
  - `closest` returns self when self matches: `getElementById('a').closest('#a').id` => "a"
- Keep ALL existing script tests green (do not weaken or delete any existing assertion or test — deleting/altering foreign tests to force green is a hard violation).

Done when: `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check` ALL pass.
Commit (you MUST git add + git commit before finishing — uncommitted work is lost):
  `feat(script): implement Element.matches() and Element.closest() DOM bindings (t0281)`
Comments and identifiers in English.
If the spec is ambiguous or conflicts with real browser behavior, do NOT guess — leave a `// TODO(spec):` and report it.
End with a short summary of exactly what changed and the test names you added.
EOF

cd "$WT"
exec gemini -p "$PROMPT" \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  < /dev/null > "$LOG" 2>&1
