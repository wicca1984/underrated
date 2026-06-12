#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0384
LOG=/workspaces/toy-browser/var/log/t0384.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

Task t0384 — implement a fragment-unioning bounding rect query in the layout tree (groundwork for Element.getBoundingClientRect). Touch ONLY files under src/layout/. Do NOT edit dom/, paint/, style/, engine/, script/, html/, main.rs, or any other module. If something genuinely requires another module, leave a `// TODO(spec): ...` comment in the closest src/layout/ file and stop.

Background (read before coding):
- Read src/layout/mod.rs. There is an existing `pub fn find_box_rect(root: &LayoutBox, node: NodeId) -> Option<Rect>` (around line 36). It returns the absolute `Rect` of the FIRST `LayoutBox` whose `node == Some(node)` in DFS pre-order. There is a `// TODO(spec): getBoundingClientRect should eventually union all fragments of an element.` comment at ~line 35.
- A single DOM element (e.g. an inline element that wraps across lines, or any element) may correspond to MULTIPLE `LayoutBox` entries sharing the same `NodeId` (one per fragment/word). `find_box_rect` only returns the first; getBoundingClientRect must return the union (bounding box) of ALL of them.
- Look at how `find_box_rect` walks the tree and how a `LayoutBox`'s absolute rect is computed (note any accumulated offset logic). Reuse that exact traversal/offset logic — do NOT invent a new coordinate scheme.
- Inspect the `Rect` type (fields like x/y/width/height or origin/size) so the union math matches the existing API.

Implement (minimal, idiomatic, matching surrounding code) in src/layout/mod.rs:
1. Add `pub fn bounding_client_rect(root: &LayoutBox, node: NodeId) -> Option<Rect>` that traverses the layout tree exactly like `find_box_rect`, collecting the absolute rect of EVERY `LayoutBox` whose node matches, and returns the union rectangle (min left, min top, max right, max bottom) as a `Rect`. Returns `None` if no box matches.
2. The union: left = min of all lefts, top = min of all tops, right = max of all rights, bottom = max of all bottoms; width = right-left, height = bottom-top.
3. Keep `find_box_rect` as-is (other callers may rely on it). Resolve/remove the stale `// TODO(spec): ... union all fragments ...` comment now that union exists, or update it to point at the new function.
4. Panic-free (AGENTS.md I-6): no unwrap()/expect()/panicking indexing in non-test code. Use iterators / fold with explicit Option handling.

Add unit tests in the existing `#[cfg(test)] mod tests` block in src/layout/mod.rs (find how other layout tests build a `LayoutBox` tree or run `layout_document` on a small DOM — copy that pattern):
- `test_bounding_rect_single_fragment`: an element with one box returns that box's rect (same as find_box_rect).
- `test_bounding_rect_unions_multiple_fragments`: construct (or lay out) a node that produces two boxes with the same NodeId at different positions/sizes; assert the returned rect is their union (covers both), and that it differs from `find_box_rect` (which returns only the first).
- `test_bounding_rect_absent_node`: a NodeId with no matching box returns `None`.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green, commit:
  git add -A && git commit -m "feat(layout): union all fragments in bounding_client_rect for getBoundingClientRect (t0384)"
Then print "T0384 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
