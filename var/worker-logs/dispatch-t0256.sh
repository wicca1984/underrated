#!/usr/bin/env bash
set -euo pipefail
# Robust auth: var/.env is the source of truth (bashrc can be wiped on rebuild).
set -a
[ -f /workspaces/underrated-meta/var/.env ] && . /workspaces/underrated-meta/var/.env
set +a
cd /workspaces/wt/t0256
exec gemini \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  -p 'You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories (/workspaces/underrated-meta/AGENTS.md) and follow ALL of it, especially invariants I-1..I-7.

Task: t0256 — Extend ordered-list (`<ol>`) marker numbering to support the alphabetic and roman `list-style-type` values: `decimal` (already the default), `lower-alpha` / `lower-latin` (a, b, c, ...), `upper-alpha` / `upper-latin` (A, B, C, ...), `lower-roman` (i, ii, iii, ...), and `upper-roman` (I, II, III, ...). Real wiki/encyclopedia pages use these ordered-list styles, so this improves text fidelity for the NewTargets/Wiki milestone.

Target module: src/layout/mod.rs (touch ONLY src/layout/mod.rs and its inline tests). Do NOT modify src/style, src/css, src/dom, src/paint or any other module — REUSE the existing ComputedStyle/CssValue API and the existing marker-emitting code. Read those modules as needed.

Scope — STRICTLY ordered-list NUMBERING (the marker text string) only:
- IN SCOPE: the `<ol>` decimal-index marker path (around src/layout/mod.rs:441, which currently does `format!("{}.", index)` using `get_li_decimal_index`). Convert the integer `index` to a string according to the element''s `list-style-type` keyword, then keep the existing `"{marker}."` formatting.
- OUT OF SCOPE (do NOT touch): the `<ul>` bullet path (disc/`*`), and the `circle` / `square` bullet glyphs. Those require a paint-side fill primitive (see the existing `// TODO(spec): disc marker needs a paint-side fill primitive` note) and would cross module boundaries — leave them exactly as-is and keep the existing `// TODO(spec): support other list-style-type ...` line updated to mention only the still-unsupported bullet shapes.
- OUT OF SCOPE: `list-style-position`, `list-style-image` — leave their existing TODO(spec) lines.

Reuse / facts (verified, do NOT reimplement):
- The cascade already stores `list-style-type` generically; at layout time `style.get("list-style-type") -> Some(CssValue::Keyword(s))` (e.g. "lower-alpha"). Absent/unknown keyword => behave as `decimal`.
- `get_li_decimal_index(dom, node, list_node) -> usize` (src/layout/mod.rs:815) already returns the 1-based ordinal for the `<li>`. Reuse it unchanged for ALL numbering systems.

Spec (https://www.w3.org/TR/CSS22/generate.html#lists, CSS Lists Level 3 counter styles):
- `decimal`: 1, 2, 3, ... (unchanged).
- `lower-alpha` / `lower-latin`: bijective base-26, a..z, then aa, ab, ... (n=1->"a", 26->"z", 27->"aa"). `upper-alpha` / `upper-latin`: same, uppercase.
- `lower-roman`: standard roman numerals lowercased (1->"i", 4->"iv", 9->"ix", 40->"xl", 90->"xc", 400->"cd", 900->"cm"). `upper-roman`: uppercase. For values <= 0, fall back to the decimal representation (roman has no zero/negative) — do NOT panic.
- All existing `"{N}."` trailing-dot formatting is preserved (e.g. lower-alpha index 1 renders as `a.`).

Approach (test-first / TDD):
1. Add two small pure helper fns in src/layout/mod.rs: `fn to_alpha(n: usize, upper: bool) -> String` (bijective base-26) and `fn to_roman(n: usize, upper: bool) -> String` (with the <=0 decimal fallback). No unwrap/expect, no panic.
2. In the `<ol>` marker branch, read `list-style-type`, map the keyword to the chosen numbering and produce the marker text; default to decimal for absent/unknown.
3. Keep ALL existing tests green (especially `test_list_item_markers`) and add new ones.

Acceptance (must all be green) — add inline unit tests in src/layout/mod.rs mirroring `test_list_item_markers`:
  - pure helpers: `to_alpha(1,false)=="a"`, `to_alpha(26,false)=="z"`, `to_alpha(27,false)=="aa"`, `to_alpha(1,true)=="A"`; `to_roman(4,false)=="iv"`, `to_roman(9,false)=="ix"`, `to_roman(40,false)=="xl"`, `to_roman(1990,true)=="MCMXC"`; `to_roman(0,false)=="0"` (decimal fallback).
  - an `<ol style="list-style-type: lower-alpha">` with 3 `<li>` emits marker texts `a.`, `b.`, `c.` (assert on the marker LayoutBox text, like the existing test does).
  - an `<ol style="list-style-type: upper-roman">` with 3 `<li>` emits `I.`, `II.`, `III.`.
  - regression: a plain `<ol>` (no list-style-type) still emits `1.`, `2.`, `3.`; a `<ul>` still emits the existing `*` marker.
  - cargo test
  - cargo clippy --all-targets -- -D warnings
  - cargo fmt --check
Done when all three pass. No unwrap/expect in non-test code (I-6). No unsafe (forbidden). No test skip/ignore (I-4). Keep the diff limited to src/layout/mod.rs — `git diff --name-only` must show ONLY src/layout/mod.rs.
Commit on this branch with: `feat(layout): support alpha/roman ordered-list marker numbering (t0256)`. Comments and identifiers in English.
IMPORTANT: commit your work before finishing (do not leave changes uncommitted). End with a one-paragraph summary and the names of the tests you added.
If the spec is ambiguous or conflicts with real browser behavior, do NOT decide on your own — leave a `// TODO(spec):` and report it.'
