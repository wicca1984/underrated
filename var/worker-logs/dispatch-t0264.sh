#!/usr/bin/env bash
set -euo pipefail
# Robust auth: var/.env is the source of truth (bashrc can be wiped on rebuild).
set -a
[ -f /workspaces/underrated-meta/var/.env ] && . /workspaces/underrated-meta/var/.env
set +a
cd /workspaces/wt/t0264
exec gemini \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  -p 'You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories (/workspaces/underrated-meta/AGENTS.md) and follow ALL of it, especially invariants I-1..I-7.

Task: t0264 — Honor the HTML ordered-list numbering attributes `<ol start>`, `<ol reversed>`, and `<li value>` when computing list-item marker numbers. Today ordered-list markers are numbered by their 1-based DOM position only, so real articles/Wikipedia lists that use `start`, `value`, or `reversed` render with wrong numbers. This advances MS-NewTargets (Wiki).

Target module: src/layout/mod.rs (touch ONLY this file — both the layout code and the inline `#[cfg(test)] mod tests`). Do NOT modify any other file. `git diff --name-only` must show ONLY: src/layout/mod.rs.

Reuse / facts (verified — do NOT reinvent):
- The ordinal is computed by `fn get_li_decimal_index(dom: &Dom, li_node: NodeId, list_node: NodeId) -> usize` (~line 889). It currently calls `find_li_descendants(dom, list_node, list_node, &mut lis)` to collect the list items in tree order, then returns `position(li_node) + 1`. `find_li_descendants` already returns ONLY the direct list items of `list_node` (it skips nested ul/ol), in document order — REUSE it; do NOT rewrite list traversal.
- The caller is the `ol` branch of the marker code (~line 440): it does `let index = get_li_decimal_index(dom, node, list_node);` then formats `index` via a match on `list-style-type` keyword: `lower-alpha`/`lower-latin` → `to_alpha(index,false)`, `upper-*` → `to_alpha(index,true)`, `lower-roman` → `to_roman(index,false)`, `upper-roman` → `to_roman(index,true)`, else `index.to_string()`; finally `format!("{formatted}.")`.
- `to_alpha(n: usize, ...)` and `to_roman(n: usize, ...)` are existing private fns that expect n >= 1.
- Attribute access: `dom.get_attribute(node_id, "name") -> Option<&str>` (defined in src/dom/mutate.rs). The `ol` element is `list_node`; each `<li>` is the item node.
- Element nodes are `crate::dom::NodeData::Element { name, attrs, .. }` matched via `dom.data(node)`.

Semantics — implement EXACTLY the HTML ordered-list ordinal algorithm (https://html.spec.whatwg.org/multipage/grouping-content.html#the-ol-element):
- `reversed` = the `list_node` (ol) has a `reversed` attribute present (value ignored; presence only).
- Parse an optional integer `start` from the ol`s `start` attribute (trim, `i64::from_str` / `str::parse::<i64>().ok()`).
- Starting value: if `start` parsed → use it; else if `reversed` → use the COUNT of direct list items (from `find_li_descendants`); else `1`.
- Walk the direct list items in document order, maintaining a running counter `current` initialized to the starting value. For EACH item in order:
  - if that item (`<li>`) has a valid integer `value` attribute, set `current = value` (the `value` overrides the running counter for this item and all following items continue from it).
  - the item`s ordinal = `current`.
  - then step: `current += if reversed { -1 } else { 1 }`.
  - stop once you reach `li_node` and return ITS ordinal.
- Change `get_li_decimal_index` to return `i64` (ordinals can be zero/negative with `reversed`, `start="0"`, or `value="-3"`). Keep the fn private; this is internal — no other module calls it.
- In the `ol` formatting match: decimal stays `index.to_string()`. For alpha/roman, only call `to_alpha`/`to_roman` when `index >= 1` (cast to usize); when `index < 1`, fall back to `index.to_string()` (browsers render decimal for non-positive roman/alpha). Do this without any unwrap/expect/panic and without casting a negative i64 to usize.

Keep it small and safe. Add a `// spec: https://html.spec.whatwg.org/multipage/grouping-content.html#the-ol-element` comment near `get_li_decimal_index`. Remove the now-satisfied `// TODO(spec): support start, value, and reversed attributes for list numbering` line (the other list TODOs about circle/square/list-style-image/inside stay). Leave a `// TODO(spec):` only if you find a genuine ambiguity you must not decide (e.g. interaction with `list-style-type` per-item resets) — do NOT invent behavior.

Acceptance — add inline unit tests mirroring the EXISTING ordered-list test in the same `#[cfg(test)] mod tests` block (find the test asserting `li_*_marker.text.as_deref() == Some("a.")` / `Some("I.")` near line 1200; REUSE its exact helper pattern: `dom.create_node(NodeData::Element{ name, attrs })`, `dom.append_child`, `parse_stylesheet`, `compute_styles`, `layout_document`, and read the marker via the li box`s LAST child `.text.as_deref()`). To set attributes, build the element with `attrs: vec![("start".into(), "5".into())]` etc. — inspect how `attrs` entries are typed in NodeData::Element and match it exactly. Cover:
  1. `<ol start="5">` with 3 items → markers "5.", "6.", "7.".
  2. `<ol reversed>` with 3 items (no start) → markers "3.", "2.", "1.".
  3. `<ol>` with second `<li value="10">` → markers "1.", "10.", "11.".
  4. Plain `<ol>` (no attrs) with 2 items → markers "1.", "2." (no regression).

Done when ALL of these pass:
  - cargo test
  - cargo clippy --all-targets -- -D warnings
  - cargo fmt --check
No unwrap/expect/panic/unsafe in non-test code (I-6). No `unsafe` anywhere. No test skip/ignore (I-4). Keep the diff limited to src/layout/mod.rs — `git diff --name-only` must show ONLY that file. Commit on this branch with: `feat(layout): honor ol start/reversed and li value in list numbering (t0264)`. Comments and identifiers in English. IMPORTANT: commit your work before finishing (do not leave changes uncommitted). End with a one-paragraph summary and the names of the tests you added. If the spec is ambiguous or conflicts with real browser behavior, do NOT decide on your own — leave a `// TODO(spec):` and report it.'
