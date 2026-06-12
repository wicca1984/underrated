#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0404
LOG=/workspaces/toy-browser/var/log/t0404.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code).

Task t0404 — implement the `:checked`, `:disabled`, and `:enabled` form-state pseudo-classes. Touch ONLY src/selector/matching.rs. Do NOT edit any other file/module. Leave a `// TODO(spec): ...` if something truly needs another module.

Background (read before coding):
- In src/selector/matching.rs there is a pseudo-class match block (around lines 284-300) that maps names to matchers:
      "hover" => get_node_state(node).hover,
      ...
      "empty" => is_empty(dom, node),
      "root" => is_root(dom, node),
      "link" => is_link(dom, node),
      "any-link" => is_link(dom, node),
      n if n.contains('(') => false,
      _ => true,
  The fallthrough `_ => true` means `:checked`/`:disabled`/`:enabled` currently INCORRECTLY match every element. Add explicit arms before the fallthrough.
- Existing helpers like `is_empty(dom, node)`, `is_root(dom, node)`, `is_link(dom, node)` show the helper style and how to read DOM attributes/element name. The DOM is accessed via `dom` (type `&Dom`); use `dom.data(node)` to get `NodeData::Element { name, attrs, .. }` and check tag name / attributes. There may be a `dom.get_attribute(node, "...")` helper — grep for how `is_link` checks `href` to mirror attribute access.

Spec to implement (Selectors Level 4 form pseudo-classes, simplified to attribute presence since this engine has no live form state):
- Add three private helper fns near the other `is_*` helpers: `is_checked`, `is_disabled`, `is_enabled`.
- `is_checked(dom, node)`: true iff node is an element AND has the `checked` attribute present (attribute presence; value irrelevant). Applies to input/option-like elements; presence of the `checked` attribute is sufficient for this engine.
- `is_disabled(dom, node)`: true iff node is an element of a form-associated kind (button, input, select, textarea, optgroup, option, fieldset) AND has the `disabled` attribute present.
- `is_enabled(dom, node)`: true iff node is a form-associated element of the kinds above AND does NOT have the `disabled` attribute present. (i.e. enabled is the form-element complement of disabled — a non-form element is neither enabled nor disabled, so `is_enabled` returns false for, say, a `<div>`.)
- Wire them into the match block:
      "checked" => is_checked(dom, node),
      "disabled" => is_disabled(dom, node),
      "enabled" => is_enabled(dom, node),
  placed before `n if n.contains('(') => false,`.

IMPORTANT: read the file to use the EXACT existing API for reading the element tag name and attribute presence (mirror `is_link`). Use a match on `dom.data(node)` returning `Some(NodeData::Element { name, attrs, .. })`. No unwrap/expect/panicking indexing in non-test code. Use `.iter().any(|(k, _)| k == "disabled")` style for attribute-presence checks if that matches how attrs are stored (verify the attr tuple shape from neighboring code).

Tests — add to the existing `#[cfg(test)] mod tests` in src/selector/matching.rs (do NOT modify/delete existing tests; mirror the style of existing pseudo-class tests, e.g. the first-child / empty tests, including how they build a DOM and call the matching entry point):
- `<input checked>` matches `:checked`; `<input>` (no attr) does not.
- `<button disabled>` matches `:disabled` and does NOT match `:enabled`; `<button>` matches `:enabled` and not `:disabled`.
- A `<div>` matches neither `:disabled` nor `:enabled`.
Use the existing selector-parse + match helpers other tests in this file use; do not invent new infrastructure.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(selector): match :checked/:disabled/:enabled form-state pseudo-classes (t0404)"
Then print "T0404 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
