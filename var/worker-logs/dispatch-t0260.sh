#!/usr/bin/env bash
set -euo pipefail
# Robust auth: var/.env is the source of truth (bashrc can be wiped on rebuild).
set -a
[ -f /workspaces/underrated-meta/var/.env ] && . /workspaces/underrated-meta/var/.env
set +a
cd /workspaces/wt/t0260
exec gemini \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  -p 'You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories (/workspaces/underrated-meta/AGENTS.md) and follow ALL of it, especially invariants I-1..I-7.

Task: t0260 — Bind the DOM methods `element.matches(selector)` and `element.closest(selector)` in the JavaScript runtime prelude. Real search-result pages (e.g. Google) rely heavily on these for event delegation (`e.target.closest(...)`). This advances MS-MVP-JS (JS support for the Google results page).

Target module: src/script/mod.rs (touch ONLY src/script/mod.rs and its inline tests). Do NOT add or change any Rust `bridge_*` host function, and do NOT modify src/dom, src/style, src/layout or any other module. This task is implementable PURELY inside the existing JavaScript prelude string by reusing host functions that ALREADY exist. Read other modules read-only as needed.

Reuse / facts (verified — do NOT reinvent, do NOT add new Rust bridges):
- The JS prelude builds every DOM node in a `getOrCreateNode(key)` factory in src/script/mod.rs. Around src/script/mod.rs:760-818 there is an object-literal block of node METHODS (method shorthand: `appendChild`, `removeChild`, `setAttribute`, `getAttribute`, `hasAttribute`, `removeAttribute`, `toggleAttribute`, `cloneNode`, `getAttributeNames`). Inside these methods, `this` is the node and `this.__key__` is its opaque node key. Add the two new methods INTO this same object literal (e.g. right after `getAttributeNames()`), following the identical style. Mind the commas between method entries.
- `document.querySelectorAll(selector)` is ALREADY defined in the prelude (around src/script/mod.rs:1040) and returns a JS array of node objects, each carrying `__key__`. `document.querySelectorAll` matches against the WHOLE document — which is exactly the correct scope for `element.matches`.
- The `parentElement` accessor is ALREADY defined (returns the parent element node object or null/undefined). `nodeType` is ALREADY defined; element nodes have `nodeType === 1`.

Semantics (DOM standard) — implement EXACTLY:
- `element.matches(selector)`: returns `true` if `element` would be selected by `selector` when evaluated against its document, else `false`. Implement by running `document.querySelectorAll(String(selector))` and returning whether any returned node has the same `__key__` as `this.__key__`. Never throw on a normal selector; if the underlying query returns nothing, return `false`.
- `element.closest(selector)`: starting at `element` itself and walking up through `parentElement`, return the FIRST element (including `this`) whose `matches(selector)` is true; return `null` if none. Only consider element nodes (`nodeType === 1`); stop when there is no parent.
- Known limitation: because matching is document-rooted, `matches`/`closest` on a node that is NOT attached to the document returns `false`/`null`. That is acceptable for this task — add a `// TODO(spec):` comment in the prelude noting that detached-node matching and `:scope` are not yet supported.

Approach:
1. In the node methods object literal, add:
     matches(selector) {
         const sel = String(selector);
         const all = document.querySelectorAll(sel);
         for (let i = 0; i < all.length; i++) {
             if (all[i] && all[i].__key__ === this.__key__) return true;
         }
         return false;
     },
     closest(selector) {
         const sel = String(selector);
         let el = this;
         while (el && el.nodeType === 1) {
             if (el.matches(sel)) return el;
             el = el.parentElement;
         }
         return null;
     }
   Adjust to match the surrounding indentation/quote style exactly, and keep commas valid.
2. Add a `// TODO(spec):` note (in the Rust source near the prelude) about detached-node / `:scope` limitations.
3. Do NOT register any new host function on `__dom_bridge__`. Do NOT touch the Rust bridge registration `.function(...)` chain. No unwrap/expect/panic/unsafe in non-test code (I-6).

Acceptance (must all be green) — add inline Rust unit tests in src/script/mod.rs mirroring the EXISTING querySelector tests (search for `fn test_eval_with_dom_query_selector` around src/script/mod.rs:3611; build a DOM with `eval_with_dom`):
  - matches positive: a `<div class="x"><p id="t">hi</p></div>`; `document.getElementById("t").matches("div.x > p")` returns `true` (or use a selector your query engine supports, e.g. `"p"` / `".x"` — verify against existing querySelector tests which selectors parse).
  - matches negative: the same `<p>` `.matches("a")` returns `false`.
  - closest hit: from the inner `<p>`, `.closest("div.x")` (or `"div"`) returns the ancestor `<div>` (assert via its `.className` or `.tagName`).
  - closest self: an element `.closest(<selector-it-matches>)` returns the element itself.
  - closest miss: `.closest("table")` from the `<p>` returns `null` (assert the JS expression `=== null` or that the result is falsy).
  Use ONLY selectors that the existing query engine already supports (cross-check the existing querySelector tests to avoid testing an unsupported selector grammar).
  - cargo test
  - cargo clippy --all-targets -- -D warnings
  - cargo fmt --check
Done when all three pass. No unwrap/expect in non-test code (I-6). No unsafe (forbidden). No test skip/ignore (I-4). Keep the diff limited to src/script/mod.rs — `git diff --name-only` must show ONLY src/script/mod.rs. Commit on this branch with: `feat(script): bind Element matches/closest in DOM prelude (t0260)`. Comments and identifiers in English. IMPORTANT: commit your work before finishing (do not leave changes uncommitted). End with a one-paragraph summary and the names of the tests you added. If the spec is ambiguous or conflicts with real browser behavior, do NOT decide on your own — leave a `// TODO(spec):` and report it.'
