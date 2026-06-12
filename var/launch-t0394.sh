#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0394
LOG=/workspaces/toy-browser/var/log/t0394.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code).

Task t0394 — implement correct matching for the link pseudo-classes `:link` and `:any-link`. Touch ONLY src/selector/matching.rs. Do NOT edit any other file/module (the parser already produces `Component::PseudoClass("link")` / `Component::PseudoClass("any-link")` generically, so NO parser change is needed). Leave a `// TODO(spec): ...` in matching.rs if something truly needs another module.

Background (read before coding):
- Read src/selector/matching.rs. Find the `Component::PseudoClass(name)` arm (~line 182). Simple-name pseudo-classes are handled in a `match name.to_ascii_lowercase().as_str()` block (~line 295) with arms like `"first-of-type" => is_first_of_type(dom, node)`, `"empty" => is_empty(dom, node)`, `"root" => is_root(dom, node)`, and a fallthrough `_ => true`.
- PROBLEM: `:link` and `:any-link` currently fall into `_ => true`, so they incorrectly match EVERY element. They must only match hyperlink elements.
- Per the HTML/CSS spec (https://html.spec.whatwg.org/multipage/semantics-other.html#selector-link and CSS Selectors L4 `:any-link`): `:link` and `:any-link` match `a`, `area`, and `link` elements that HAVE an `href` attribute. (We do not track visited state, so treat `:link` == `:any-link`: any link with an href. Do NOT implement `:visited`.)
- Look at how existing helpers obtain the element tag name and attributes: see `is_first_of_type` / `is_root` / `matches_compound` and the `NodeData::Element { tag, attrs, .. }` shape, plus any existing `get_attribute`-style helper or attribute scan already used in this file. MIRROR that exact style — do not invent new infrastructure.

Scope for THIS task (single file):
1. Add a private helper `fn is_link(dom: &Dom, node: NodeId) -> bool` (mirror the style/signature of the existing `is_root`/`is_empty` helpers in this file). It returns true iff `node` is an element whose tag (compared ASCII-case-insensitively) is one of `a`, `area`, `link` AND it has an `href` attribute present (value may be empty; presence is enough). Non-elements return false.
2. In the simple-name pseudo-class match block, add arms BEFORE the `_ => true` fallthrough:
   `"link" => is_link(dom, node),`
   `"any-link" => is_link(dom, node),`
3. Panic-free: no unwrap/expect/panicking indexing in non-test code; use Option combinators / `matches!` / iterators.

Tests — add to the existing `#[cfg(test)] mod tests` in src/selector/matching.rs (do NOT modify/delete existing tests; mirror an existing matching test such as the `:not`/`is_first_of_type` ones for setup style — build a small Dom, then assert `matches(&parse_selector_list("...").unwrap(), &dom, node)` true/false):
- `<a href="x">t</a>` matches `:link` and `:any-link` (true).
- `<area href="x">` matches `:any-link` (true).
- `<a>t</a>` (no href) does NOT match `:link` (false).
- `<div href="x">t</div>` (wrong tag) does NOT match `:link` (false).
- A compound like `a:link` matches an `<a href>` and a bare `<a>` does not.
Use whatever existing Dom-construction + selector-parse + match helpers the surrounding tests use; do not invent new infrastructure.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(selector): match :link and :any-link against href-bearing a/area/link (t0394)"
Then print "T0394 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
