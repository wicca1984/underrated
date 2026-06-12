#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0414
LOG=/workspaces/toy-browser/var/log/t0414.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code).

Task t0414 — implement element-only sibling navigation on the DOM. Touch ONLY src/dom/query.rs. Do NOT edit any other file/module. If something truly needs another module, leave a `// TODO(spec): ...` and report instead.

Context (read before coding) — src/dom/query.rs already has `impl Dom { ... }` with read-only query methods like `get_element_by_id` and `query_selector`. Add the new methods to this same `impl Dom` block. You will reuse EXISTING public accessors on `Dom` (defined in src/dom/mod.rs): `pub fn parent(&self, node: NodeId) -> Option<NodeId>`, `pub fn children(&self, node: NodeId) -> &[NodeId]`, and `pub fn data(&self, node: NodeId) -> Option<&NodeData>`. An element node is one whose data matches `NodeData::Element { .. }`. There is NO `next_sibling`/`previous_sibling` accessor — compute siblings via parent + children + index.

What to implement (two methods on `impl Dom` in src/dom/query.rs):
1. `pub fn next_element_sibling(&self, node: NodeId) -> Option<NodeId>`:
   - Find `node`'s parent; among the parent's `children`, locate `node`'s position, then return the FIRST following sibling whose data is `NodeData::Element { .. }` (skipping Text/Comment/etc. nodes).
   - Returns `None` if the node has no parent, is not found among its parent's children, or has no following element sibling.
2. `pub fn previous_element_sibling(&self, node: NodeId) -> Option<NodeId>`:
   - Same, but the NEAREST PRECEDING sibling that is an element (walk backwards from `node`'s position).
   - Returns `None` if no such sibling exists.

Keep it panic-free: no unwrap/expect/panicking indexing in non-test code (use `iter().position(...)`, `.get(..)`, iterators). Document each public method with a `///` doc comment in the same concise style as the other methods in query.rs.

Tests — add a new `#[cfg(test)] mod tests` section (or extend the existing one if present) in src/dom/query.rs. Build a small tree using the existing Dom construction API (`create_node(NodeData::Element { name, attrs })`, `create_node(NodeData::Text(...))`, `append_child(parent, child)` — mirror how other dom tests build trees). Cover:
- A parent with children [Element a, Text, Element b, Element c]: `next_element_sibling(a)` == Some(b) (skips the Text node); `next_element_sibling(b)` == Some(c); `next_element_sibling(c)` == None.
- `previous_element_sibling(c)` == Some(b); `previous_element_sibling(b)` == Some(a) (skipping the Text node when relevant); `previous_element_sibling(a)` == None.
- A node with no parent (e.g. a freshly created, unattached node) => both return None.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(dom): add element-only next/previous sibling navigation (t0414)"
Then print "T0414 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
