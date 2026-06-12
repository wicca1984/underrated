#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0425
LOG=/workspaces/toy-browser/var/log/t0425.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything you need is in the local source files and this prompt. Reading local files is fine; network/web search is forbidden and wastes time.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code — test code MAY use unwrap/panic as the existing tests do).

Task t0425 — Implement `Dom::set_text_content` (the WHATWG `Node.textContent` setter) and have it mark the node layout-dirty. Touch ONLY the file `src/dom/text.rs` (where the existing `text_content` getter lives). Do NOT edit `src/dom/mod.rs`, `src/dom/mutate.rs`, `src/dom/dirty.rs`, or any other file.

Read these first to learn the EXACT existing API (do not assume signatures):
- `src/dom/text.rs` (the `text_content` getter is already here; add the setter in the same `impl Dom` block / same file).
- `src/dom/mod.rs` for `create_node(&mut self, data: NodeData) -> NodeId`, `append_child(&mut self, parent, child)`, and the `NodeData` enum (esp. `NodeData::Text(String)` and how to detect Element nodes).
- `src/dom/dirty.rs` for `pub fn mark_dirty(&mut self, node: NodeId)` — you will call this.
- `src/dom/mutate.rs` for how children are detached/cleared (reuse the existing pattern; e.g. clearing a parent's `children` and nulling each child's `parent` link). Do NOT edit mutate.rs — just mirror its safe approach inside text.rs.

Spec behavior to implement (https://dom.spec.whatwg.org/#dom-node-textcontent setter):
- `pub fn set_text_content(&mut self, node: NodeId, text: &str)`.
- No-op (do nothing, no panic) for invalid/stale ids. For a Text node, replace its stored string with `text`. For an Element (or Document/other container) node: remove ALL existing children (detach them — clear the parent's children list and set each removed child's `parent` to `None`, matching the no-panic style of `remove_child` in mutate.rs), then, IF `text` is non-empty, create ONE new `NodeData::Text(text)` node and append it as the sole child. If `text` is empty, leave the element with no children (per spec, empty string → no text node).
- After any mutation that changed the tree or text, call `self.mark_dirty(node)` so a future batched relayout can pick it up. (For an invalid id, do nothing — no mark_dirty.)
- I-6: NO unwrap/expect/panicking indexing in the implementation. Use `if let`/`get`/`get_mut`. Use iterative logic only.

Add unit tests in the existing `#[cfg(test)] mod tests` of `src/dom/text.rs` (add the module if absent, mirroring the test style used elsewhere in src/dom, e.g. `src/dom/dirty.rs`):
1. Setting text content on an element with existing children replaces them all with a single Text node, and `text_content(node)` returns exactly the new string.
2. Setting text content to "" on an element with children removes all children (children list empty) and `text_content` returns "".
3. Setting text content on a Text node replaces its data (round-trip via `text_content`).
4. After `set_text_content`, the node is dirty (`is_dirty(node)` true / `has_dirty()` true).
5. Calling with an invalid `NodeId` is a no-op and does not mark anything dirty.
Read how other src/dom tests build a `Dom` and nodes (e.g. `dom.create_node(...)`, `dom.append_child(...)`) and reuse that exact pattern.

Run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(dom): implement Node.textContent setter (set_text_content) with layout-dirty marking (t0425)"
Then print "T0425 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
