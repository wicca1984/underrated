#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0227
exec gemini \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  -p 'You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories (/workspaces/underrated-meta/AGENTS.md) and follow ALL of it, especially invariants I-1..I-7.

Task: t0227 — Expose document.getElementsByTagName(tagName) and document.getElementsByClassName(className) as JavaScript DOM bindings. These are extremely common in real-world page scripts (including Google search result pages) and are the natural next step after querySelector/querySelectorAll (t0226).

Target module: src/script/ (touch ONLY src/script/mod.rs and its tests). Do NOT modify src/dom, src/selector, src/css or any other module — REUSE the existing Dom query API. Read those modules as needed.

Background / reuse (already implemented, do NOT reimplement):
- src/dom/query.rs already provides `Dom::query_selector_all(&self, selector: &str) -> Vec<NodeId>`.
- src/script/mod.rs already binds `document.querySelectorAll(selector)` via a `bridge.querySelectorAll` host function (see `bridge_query_selector_all`, around lines 265-276 for the JS side and ~679 for the Rust host fn). It returns a JS Array of node proxy objects keyed into `__node_registry__`.

Approach (test-first / TDD):
1. Implement the two new bindings by delegating to the EXISTING query path:
   - `document.getElementsByTagName(tag)`  -> equivalent to query_selector_all with selector = the tag name itself (e.g. "div"). Special-case the wildcard "*" by passing "*" through to query_selector_all (only if query_selector_all already supports "*"; if it does NOT, pass it through and let it return whatever it returns — do NOT add selector features here; if "*" is unsupported, leave a `// TODO(spec):` noting the universal-selector gap and do not special-case it).
   - `document.getElementsByClassName(cls)` -> equivalent to query_selector_all with selector = "." + cls (a class selector). If `cls` contains multiple space-separated tokens (e.g. "a b"), join them as a compound class selector ".a.b" (matches elements having ALL the classes), mirroring the DOM spec.
   - Both must return a JS Array of the SAME node-proxy shape that querySelectorAll already returns (reuse the exact same proxy-construction JS/host code path — ideally have the two new host bridge functions build the selector string and then call the same internal helper that bridge_query_selector_all uses, so the proxies are identical). Do NOT duplicate proxy-construction logic if you can factor the shared part into one helper.
   - Case-insensitivity: getElementsByTagName tag matching in HTML is ASCII-case-insensitive. If the underlying selector engine already lowercases/normalizes tag names, lowercase the incoming tag before building the selector. If unsure whether the engine is case-sensitive, lowercase the tag for the tag-name path (HTML tag names are conventionally lowercase in the parsed DOM) and add a brief comment.
2. Wire the JS side in `setup_experimental_dom` exactly like querySelectorAll: register the host bridge function(s) and define `document.getElementsByTagName` / `document.getElementsByClassName` JS wrappers that call the bridge and return the array of proxies.

Acceptance (must all be green) — add unit tests in src/script/mod.rs mirroring the existing `test_eval_with_dom_query_selector_all` test:
  - test getElementsByTagName returns the correct count and the textContent/attributes of returned nodes for a small DOM containing multiple <p> elements.
  - test getElementsByClassName returns elements matching a single class, and (if you implemented multi-token) elements matching a compound class set.
  - an empty-result case returns an empty array (length 0), not null/undefined.
  - cargo test
  - cargo clippy --all-targets -- -D warnings
  - cargo fmt --check
Done when all three pass. No unwrap/expect in non-test code (I-6). No unsafe (forbidden). No test skip/ignore (I-4). Keep the diff limited to src/script/mod.rs and its tests — `git diff --name-only` must show ONLY src/script/mod.rs.
Commit on this branch with: `feat(script): expose document.getElementsByTagName/getElementsByClassName JS bindings (t0227)`. Comments and identifiers in English.
IMPORTANT: commit your work before finishing (do not leave changes uncommitted). End with a one-paragraph summary and the names of the tests you added.
If the spec is ambiguous or conflicts with real browser behavior, do NOT decide on your own — leave a `// TODO(spec):` and report it.'
