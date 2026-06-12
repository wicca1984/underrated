#!/usr/bin/env bash
# Launcher for Gemini worker t0294 — ParentNode.replaceChildren() DOM binding (MS-MVP-JS idle-fill).
# Target: src/script/mod.rs ONLY. Dispatched via setsid (memory: worker-dispatch-must-setsid).
set -euo pipefail

WT=/workspaces/wt/t0294
LOG=/workspaces/toy-browser/var/worker-logs/t0294.log

read -r -d '' PROMPT <<'EOF' || true
You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow ALL of it (especially I-1..I-7).

Task: t0294 — implement the DOM binding `ParentNode.replaceChildren(...nodes)` (per WHATWG DOM). It is currently
NOT implemented (grep confirms 0 hits for `replaceChildren`). This continues the recently-merged ChildNode/
ParentNode family (`remove`, `replaceWith`, `append`, `prepend`).

Target file: src/script/mod.rs ONLY (production code + its in-file `#[cfg(test)] mod tests`). Do NOT touch any
other file (no other src/ module, no other test file, no fixtures, no Cargo.toml, no other worktree).
1 task = 1 module (I-5). Production Rust code must NOT use `unwrap`/`expect` (I-6); test code may.

Read first and VERIFY every claim against the ACTUAL code before editing (do not trust this prompt blindly):
- The `append` and `prepend` bindings are installed via `Object.defineProperty(node, '...', { value: function(...args){...} })`
  inside the per-node binding-injection JS string (search for `Object.defineProperty(node, 'append'`, around
  line 1298). They are pure-JS wrappers built on existing primitives: `this.appendChild(n)`,
  `this.insertBefore(n, refNode)`, `this.firstChild`, `document.createTextNode(str)`, and the `arg.__key__`
  test that distinguishes a Node from a string. Match that EXACT style and surrounding indentation.

Implementation (pure JS in that same injection block, immediately after `prepend`):
  Add `Object.defineProperty(node, 'replaceChildren', { value: function(...args) { ... }, enumerable: false,
  configurable: true, writable: true })`.
  Semantics (WHATWG): first coerce each argument to a node EXACTLY like `append` does (string ->
  `document.createTextNode(arg)`; object with `__key__` -> the node itself; otherwise
  `throw new TypeError("Argument must be a Node or a string")`). Do the coercion/validation of ALL args BEFORE
  mutating, then remove ALL existing children of `this` (e.g. `while (this.firstChild) { this.removeChild(this.firstChild); }`),
  then append the new nodes in order via `this.appendChild(n)`. Calling with no args clears all children.
  Add a short comment: `// TODO(spec): ParentNode.replaceChildren() v1 — Node/DOMString args; DocumentFragment expansion out of scope.`

Tests (add to the in-file tests module — do NOT delete, weaken, `#[ignore]`, or alter ANY existing test). Build
inputs the SAME way the existing DOM-write tests do (look at `test_dom_childnode_before_after` and the
append/prepend tests for the harness: how they build the script env, run JS, and read back `dom.children` /
`dom.text_content`). Add ONE test `fn test_dom_parentnode_replace_children()` that:
  1) Builds a parent with some initial children, runs JS `parent.replaceChildren(a, "txt", b)`, and asserts the
     parent's children are EXACTLY the new set in order (old children gone, new ones present, text node created
     for the string arg).
  2) Asserts `parent.replaceChildren()` with no args removes all children (parent becomes empty).
Reuse existing test helpers/patterns; do not invent a new harness.

If any primitive/helper name does not match this description, TRUST THE CODE, not this prompt, and adapt — but
keep the WHATWG intent: validate-all-then-replace, no-arg clears.

Done when (run from the worktree root /workspaces/wt/t0294):
  `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check` ALL pass.

Commit (you MUST `git add -A && git commit` before finishing — uncommitted work is lost; the worktree may be
force-removed after you exit):
  `feat(script): implement ParentNode.replaceChildren() DOM binding (t0294)`

End with a short summary: where you inserted the binding, the exact validate-then-mutate order you used, and the
children counts your test observed before/after replaceChildren and after the no-arg clear.
EOF

cd "$WT"
exec gemini -p "$PROMPT" \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  < /dev/null > "$LOG" 2>&1
