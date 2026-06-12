#!/usr/bin/env bash
set -euo pipefail
# Robust auth: var/.env is the source of truth (bashrc can be wiped on rebuild).
set -a
[ -f /workspaces/underrated-meta/var/.env ] && . /workspaces/underrated-meta/var/.env
set +a
cd /workspaces/wt/t0287
exec gemini \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  -p 'You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories (/workspaces/underrated-meta/AGENTS.md) and follow ALL of it, especially invariants I-1..I-7.

Task: t0287 — Implement the DOM `ChildNode.remove()` JavaScript binding. `node.remove()` removes the node from its parent (i.e. it is equivalent to `if (node.parentNode) node.parentNode.removeChild(node);`). It is the canonical third method of the ChildNode mixin alongside the already-implemented `before()` and `after()` (added in t0285). `ChildNode.remove()` is currently NOT implemented (verified: there is no `remove` method on the node wrapper object; only `removeChild`, `removeAttribute`, and the unrelated `classList.remove(token)` exist — do NOT confuse it with those).

Scope: touch ONLY `src/script/mod.rs` (including its inline `#[cfg(test)] mod tests`). `git diff --name-only` MUST show ONLY `src/script/mod.rs`. Do NOT modify dom, layout, style, or any other module/file, do NOT add to lib.rs, and do NOT touch any other worktree.

Reuse / facts (verified — read these in src/script/mod.rs BEFORE writing):
- The DIRECT mirror is the existing `before` / `after` ChildNode bindings, defined PURELY in the JS wrapper (NO dedicated Rust bridge fn). They live inside `getOrCreateNode` as `Object.defineProperty(node, ''before'', { value: function(newNode){ ... }, enumerable:false, configurable:true, writable:true })` at ~lines 1269 and 1283. They reuse `this.parentNode.insertBefore(...)`. Read them carefully and follow the EXACT same shape.
- The JS wrapper already exposes `removeChild(child)` on the node object (~line 960) and on `document` (~line 1349). `parentNode` is exposed as a getter on the node wrapper. So `remove()` can be implemented ENTIRELY in JS by reusing the existing `parentNode` getter and `removeChild` method — you do NOT need a new Rust bridge fn.

Implement (JS-wrapper-only, mirroring before/after):
1. Inside `getOrCreateNode`, right after the `after` defineProperty block (~line 1283-1295), add an `Object.defineProperty(node, ''remove'', { value: function(){ ... }, enumerable:false, configurable:true, writable:true })`. The function body must be exactly the ChildNode.remove() semantics:
   `if (!this.parentNode) return; this.parentNode.removeChild(this);`
   Per the DOM spec, calling `remove()` on a node that has no parent is a silent no-op (do NOT throw). Take NO arguments.
2. Do NOT add `remove` to the `document` object (the document has no parent; ChildNode.remove does not apply to it) — matching how before/after were NOT added to document.
3. Leave exactly one `// TODO(spec): ChildNode.remove() v1 — single-node removal only; DocumentFragment / cross-document host edge cases out of scope.` comment above the new block.
4. Do NOT alter, weaken, or remove any existing binding, test, or assertion (deleting/altering foreign tests to force green is a hard violation, I-4). Keep all public interfaces stable except for the additive `remove` binding.

Do NOT use unwrap/expect/panic/unsafe in non-test code (I-6). No new dependencies (I-1). Do NOT change iterative code back to recursive.

Acceptance — add an inline integration test in `src/script/mod.rs` (reuse the existing JS-evaluation test harness; mirror `test_dom_childnode_before_after` at ~line 5450 which builds a small DOM, runs a JS snippet, then asserts the resulting DOM tree via `dom.children(...)` / `dom.text_content(...)`):
- Removing a middle child detaches it: build parent with children [a, b, c], call `b.remove()` from JS, then assert parent now has exactly children [a, c] (count 2, in order) and that `b` is no longer among them.
- Calling `remove()` on a node with no parent is a silent no-op (does NOT throw): e.g. create a detached element and call `el.remove()` — the JS must evaluate without error.
Do NOT weaken or remove any existing test (especially `test_dom_childnode_before_after` and `test_dom_write_insert_before_and_remove_child`).

Done when ALL of these pass in this worktree (run from the worktree root /workspaces/wt/t0287):
  - cargo test
  - cargo clippy --all-targets -- -D warnings
  - cargo fmt --check
Comments and identifiers in English.
Commit (you MUST git add + git commit BEFORE finishing — uncommitted work is lost when the worktree is removed): `git add -A && git commit -m "feat(script): implement ChildNode.remove() DOM binding (t0287)"`.
End with a short English summary of exactly what changed in src/script/mod.rs (the new `remove` defineProperty block and the `// TODO(spec):` you left), the test name(s) you added, and confirm you committed.'
