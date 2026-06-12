#!/usr/bin/env bash
set -euo pipefail
# Robust auth: var/.env is the source of truth (bashrc can be wiped on rebuild).
set -a
[ -f /workspaces/underrated-meta/var/.env ] && . /workspaces/underrated-meta/var/.env
set +a
cd /workspaces/wt/t0290
exec gemini \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  -p 'You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories (/workspaces/underrated-meta/AGENTS.md) and follow ALL of it, especially invariants I-1..I-7.

Task: t0290 — Implement the DOM `ParentNode.append()` and `ParentNode.prepend()` JavaScript bindings. These are the ParentNode-mixin counterparts to the already-implemented ChildNode `before()`/`after()` bindings (t0285).
  - `parent.append(...nodes)`: inserts the given nodes/strings as the LAST children of the parent (at the end of its child list).
  - `parent.prepend(...nodes)`: inserts the given nodes/strings as the FIRST children of the parent (at the start of its child list).
Each accepts a variadic list where each argument is either a Node or a string; strings are converted to Text nodes (use the same string->Text conversion the existing before/after bindings use). Neither is currently implemented (verified: there is no `append`/`prepend` method on the node wrapper object; do NOT confuse with the existing `appendChild`).

Scope: touch ONLY `src/script/mod.rs` (including its inline `#[cfg(test)] mod tests`). `git diff --name-only` MUST show ONLY `src/script/mod.rs`. Do NOT modify dom, layout, style, or any other module/file, do NOT add to lib.rs, and do NOT touch any other worktree.

Reuse / facts (verified — read these in src/script/mod.rs BEFORE writing):
- The DIRECT mirror is the existing `before` / `after` ChildNode bindings, defined PURELY in the JS wrapper (NO dedicated Rust bridge fn). They live inside `getOrCreateNode` as `Object.defineProperty(node, ''before'', { value: function(){ ... }, enumerable:false, configurable:true, writable:true })` at ~lines 1269 and 1283. Read them carefully and follow the EXACT same shape, including how they (a) accept variadic args, (b) convert string args to Text nodes, and (c) reuse `insertBefore` / `appendChild`.
- The JS wrapper already exposes `appendChild(child)`, `insertBefore(newNode, refNode)`, `firstChild` and `childNodes`/`children` on the node object. So `append`/`prepend` can be implemented ENTIRELY in JS by reusing these existing methods/getters — you do NOT need a new Rust bridge fn.
  - `append`: for each argument in order, convert string->Text if needed, then `this.appendChild(node)`.
  - `prepend`: insert before the current `this.firstChild`. Reuse `this.insertBefore(node, this.firstChild)` for each argument IN ORDER (insertBefore with a null/absent ref appends; with firstChild it prepends — verify firstChild handling matches the existing before/after code so order is preserved: prepending [x, y] before child a yields [x, y, a]).

Implement (JS-wrapper-only, mirroring before/after):
1. Inside `getOrCreateNode`, right after the `after` defineProperty block (~line 1283-1295), add two `Object.defineProperty(node, ''append''|''prepend'', { value: function(){ ... }, enumerable:false, configurable:true, writable:true })` blocks. Use the SAME variadic + string->Text conversion idiom as before/after.
2. Do these for the node wrapper only (mirror how before/after were added to the node wrapper).
3. Leave exactly one `// TODO(spec): ParentNode.append()/prepend() v1 — Node and string (->Text) args only; DocumentFragment expansion and other edge cases out of scope.` comment above the new blocks.
4. Do NOT alter, weaken, or remove any existing binding, test, or assertion (deleting/altering foreign tests to force green is a hard violation, I-4). Keep all public interfaces stable except for the additive `append`/`prepend` bindings.

Do NOT use unwrap/expect/panic/unsafe in non-test code (I-6). No new dependencies (I-1). Do NOT change iterative code back to recursive.

Acceptance — add inline integration test(s) in `src/script/mod.rs` (reuse the existing JS-evaluation test harness; mirror `test_dom_childnode_before_after` at ~line 5450 which builds a small DOM, runs a JS snippet, then asserts the resulting DOM tree via `dom.children(...)` / `dom.text_content(...)`):
- append: build parent with children [a, b], call `parent.append(c)` (an element) from JS, assert parent children become [a, b, c] in order.
- prepend: build parent with children [a, b], call `parent.prepend(z)` from JS, assert parent children become [z, a, b] in order.
- string conversion: call `parent.append(''hi'')` and assert a Text node with content `hi` is now the last child.
Do NOT weaken or remove any existing test (especially `test_dom_childnode_before_after`).

Done when ALL of these pass in this worktree (run from the worktree root /workspaces/wt/t0290):
  - cargo test
  - cargo clippy --all-targets -- -D warnings
  - cargo fmt --check
Comments and identifiers in English.
Commit (you MUST git add + git commit BEFORE finishing — uncommitted work is lost when the worktree is removed): `git add -A && git commit -m "feat(script): implement ParentNode.append()/prepend() DOM bindings (t0290)"`.
End with a short English summary of exactly what changed in src/script/mod.rs (the new `append`/`prepend` defineProperty blocks and the `// TODO(spec):` you left), the test name(s) you added, and confirm you committed.'
