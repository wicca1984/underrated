#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0432
LOG=/workspaces/toy-browser/var/log/t0432.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything you need is in the local source files and this prompt. Reading local files is fine; network/web search is forbidden and wastes time.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-5 one-module; I-6 NO unwrap/expect in library/src code; test code MAY use unwrap/panic as the existing tests do).

Task t0432 — Exclude the children (content) of a `<template>` element from the layout tree. Touch ONLY `src/layout/mod.rs` (plus tests in that same file). Do NOT edit any other file under src/.

WHY this is new (do not duplicate): the HTML parser ALREADY parses `<template>` (an `InTemplate` insertion mode exists in `src/html/tree.rs`) and inserts the template's content as ordinary child nodes in the DOM. Per the HTML spec, a `<template>` element's contents are an inert DocumentFragment that MUST NOT be rendered. Currently nothing excludes them, so template content is laid out and painted. Your job: make layout skip a `<template>` element's children so its content does not appear.

READ FIRST: in `src/layout/mod.rs` find `pub(crate) fn get_layoutable_children(dom: &Dom, styles: &HashMap<NodeId, ComputedStyle>, node: NodeId) -> Vec<NodeId>` (around line 649). It walks `dom.children(node)` and collects layoutable children (skipping absolute/fixed-positioned nodes, `display:none` elements, etc.).

YOUR CHANGE — at the very TOP of `get_layoutable_children`, before the loop: if `node` itself is an Element whose tag name is `template`, return an empty `Vec::new()` immediately (a template's content is inert and never laid out). Use `dom.data(node)` and pattern-match `Some(NodeData::Element { name, .. }) if name == "template"`. Add a short `//` comment citing the spec reason (template contents are an inert DocumentFragment, not rendered). Compare the tag name case-insensitively only if the existing code elsewhere does; the parser lowercases tag names, so a plain `name == "template"` match is fine — keep it consistent with how other tag-name checks in this file are written (search for existing `name == "table"` style matches and mirror them).

- Keep I-6: NO `unwrap`/`expect` in non-test code. No new panics.
- Do NOT change the DOM, the HTML parser, or any other module. The fix is purely "do not descend into a template's children during layout".

Tests — add to the existing `#[cfg(test)]` module in `src/layout/mod.rs`. Add ONE `#[test] fn test_template_children_excluded_from_layout()` with a short `//` comment naming what it guards. Build a small fixture, e.g. `<div><template><p>HIDDEN</p></template><span>SHOWN</span></div>`, run it through the same layout entry point the neighboring tests use (search the existing tests in this file for the exact helper, e.g. a `layout`/`build_layout_tree`/`render`-style helper + `parse_html`/`parse_stylesheet`), and assert that the laid-out box tree contains the `<span>` / "SHOWN" content but does NOT contain any box for the `<p>` / "HIDDEN" template content. Mirror the existing tests' construction style exactly. Keep all existing tests passing unchanged.

Run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(layout): exclude <template> element contents from layout tree (t0432)"
Then print "T0432 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
