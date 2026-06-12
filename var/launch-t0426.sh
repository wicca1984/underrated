#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0426
LOG=/workspaces/toy-browser/var/log/t0426.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything you need is in the local source files and this prompt. Reading local files is fine; network/web search is forbidden and wastes time.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code — test code MAY use unwrap/panic as the existing tests do).

Task t0426 — Wire the existing DOM mutation APIs in `src/dom/mutate.rs` to mark the affected node layout-dirty, so a future batched relayout (the already-implemented `take_dirty`/`flush_dirty` machinery) can react to attribute and tree edits. Touch ONLY the file `src/dom/mutate.rs`. Do NOT edit `src/dom/mod.rs`, `src/dom/dirty.rs`, `src/dom/text.rs`, or any other file.

Read first to learn EXACT signatures (do not assume):
- `src/dom/mutate.rs` in full. It already defines `set_attribute`, `remove_attribute`, `remove_child`, `insert_before`, and possibly `append_child`/others.
- `src/dom/dirty.rs` for `pub fn mark_dirty(&mut self, node: NodeId)`, `is_dirty`, `has_dirty`, `take_dirty`.

Changes (mark dirty ONLY when a mutation actually changed something — mirror each method's existing "did it change" detection; do NOT mark dirty on no-ops / invalid ids):
- `set_attribute`: after successfully adding or overwriting the attribute on an Element, call `self.mark_dirty(node)`. (If the node is not an element / invalid id, no change → do not mark.)
- `remove_attribute`: if an attribute was actually present and removed, mark that node dirty. (Use `Vec::retain` return-aware logic: compare length before/after, or check `iter().any(...)` first, to detect a real removal. No-op removal → no mark.)
- `remove_child`: it already computes a `removed: bool`. When `removed` is true, mark the PARENT node dirty (its child list changed). Reuse the existing `removed` flag; do not add a second retain pass.
- `insert_before`: when an insertion actually occurs (mirror its existing success detection, e.g. the `inserted` flag), mark the PARENT node dirty.
- If `mutate.rs` also contains `append_child` or a `set_text`-like tree edit, apply the same rule (mark the mutated/parent node dirty on real change). Do NOT touch methods that live in OTHER files (e.g. `set_text_content` belongs to text.rs — out of scope here).

Constraints: I-6 (no unwrap/expect/panicking index; keep the existing graceful no-panic style — these methods must still tolerate invalid/stale ids). Keep changes minimal and localized; do not refactor unrelated code.

Add unit tests in the `#[cfg(test)] mod tests` of `src/dom/mutate.rs` (add the module if absent, mirroring the test style in `src/dom/dirty.rs`):
1. `set_attribute` on a valid element marks it dirty (`is_dirty` true).
2. `set_attribute` on an INVALID node id does NOT mark anything dirty (`has_dirty()` false).
3. `remove_attribute` of an existing attribute marks the element dirty; removing a NON-existent attribute does NOT mark dirty.
4. `remove_child` of an actual child marks the PARENT dirty; removing a non-child is a no-op (no dirty).
5. `insert_before`/append of a child marks the PARENT dirty.
Build the `Dom` and nodes using the same helpers the existing tests use (`create_node`, `append_child`, etc.). Between sub-assertions you may call `take_dirty()`/`clear_dirty()` to reset state — read dirty.rs for those.

Run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(dom): mark nodes layout-dirty on attribute and tree mutations (t0426)"
Then print "T0426 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
