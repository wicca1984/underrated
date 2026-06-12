#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0393
LOG=/workspaces/toy-browser/var/log/t0393.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code).

Task t0393 — implement the HTML "in column group" insertion mode properly. Touch ONLY src/html/tree.rs. Do NOT edit any other file/module. Leave a `// TODO(spec): ...` in tree.rs if something truly needs another module.

Background (read before coding):
- Read src/html/tree.rs. The dispatch table `fn process_token` (~line 100) currently routes `InsertionMode::InColumnGroup => self.handle_in_table(token), // TODO(spec)`. This is WRONG: once inside a `<colgroup>` the parser must use the dedicated "in column group" rules, not the generic in-table rules. Currently `handle_in_table`'s `"col"` branch would spuriously open a NEW colgroup while already inside one.
- See `handle_in_table` (~line 677) for the house style of a handler: it matches on `Token::StartTag { name, attrs, .. }`, `Token::EndTag { name, .. }`, `Token::Character(_)`, `Token::Comment`, `Token::Doctype`, `Token::Eof`. Reuse the SAME helpers it uses: `create_and_insert_element(name.clone(), attrs.clone())`, `self.stack_of_open_elements`, `self.insert_character`/comment helpers (find the exact names already used by other handlers — e.g. how `handle_in_table` and `handle_in_body` insert characters/comments), and `self.handle_in_body(token)` / `self.handle_in_head(token)` delegation.
- Note `handle_in_table`'s `"colgroup"` start tag (~line 692) creates the colgroup, pushes it, and sets `InsertionMode::InColumnGroup`; the bare `"col"` start tag (~line 698) synthesizes a `<colgroup>` first then reprocesses — your new handler is where the subsequent `<col>` tokens land.

Spec to implement — §13.2.6.4.12 "The 'in column group' insertion mode" (https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-incolgroup). Add a new method `fn handle_in_column_group(&mut self, token: Token)` and route `InsertionMode::InColumnGroup` to it in `process_token` (replace the TODO line). Rules:
- A whitespace-only character token (U+0009, U+000A, U+000C, U+000D, U+0020): insert the character (use the same char-insertion path other handlers use).
- A comment token: insert a comment.
- A DOCTYPE token: parse error; ignore.
- A start tag `"html"`: process using the "in body" rules (`self.handle_in_body(token)`).
- A start tag `"col"`: insert an HTML element for the token, then immediately pop it off the stack of open elements (a col is void/self-closing — do not leave it open). Ignore the self-closing flag acknowledgement (no flag tracking exists; just don't push it).
- An end tag `"colgroup"`: if the current node is NOT a `colgroup` element, this is a parse error; ignore the token. Otherwise pop the current node (the colgroup) off the stack and switch the insertion mode to `InsertionMode::InTable`.
- An end tag `"col"`: parse error; ignore.
- A start/end tag `"template"` (start) or `"template"` (end): process using the "in head" rules (`self.handle_in_head(token)`).
- An EOF token: process using the "in body" rules (`self.handle_in_body(token)`).
- Anything else (any other token): act as the "anything else" clause — if the current node is NOT a colgroup element, parse error; ignore the token. Otherwise pop the current node (colgroup), switch insertion mode to `InsertionMode::InTable`, and REPROCESS the current token (`self.process_token(token)`).
- To check "current node is a colgroup": peek the last element on `self.stack_of_open_elements` and compare its element name to "colgroup" using whatever DOM/name accessor the existing handlers use (mirror how e.g. `is_in_table_scope` or pop helpers read element names). Panic-free: use `.last()` + `if let`, never indexing/unwrap.

Constraints:
- Single file only (src/html/tree.rs). Do NOT change the `Token`/`InsertionMode` enums or any other module.
- Panic-free in non-test code: no unwrap/expect/panicking indexing; use `?`/Option combinators/`.last()`.
- Do NOT modify or delete any existing test or handler. Do NOT weaken existing behavior — `<table><colgroup><col></colgroup>...` must still build the same tree shape, and a `<col>` directly in a table (auto-colgroup path) must keep working.

Tests — add a `#[cfg(test)] mod` test (or extend the existing one) in src/html/tree.rs that parses snippets and asserts tree structure using the SAME parse + node-inspection helpers the existing tests in this file use (read them first; do not invent new infra):
- `<table><colgroup><col><col></colgroup><tr><td>x</table>`: the colgroup contains exactly two `col` children, and the colgroup is a child of the table; the `td`/`tr` are siblings under the table (col elements are NOT nested inside each other).
- `<table><col></table>` (bare col, auto-colgroup): a `colgroup` is created containing one `col`.
- `<table><colgroup>text<col></colgroup>` : stray non-whitespace text inside colgroup triggers the "anything else" clause (colgroup is popped, text handled in table context) — assert the col count / that parsing does not panic and the colgroup closed.

Regression guard (MANDATORY): the html5lib-tests submodule under tests/html5lib-tests must stay green. Run the full `cargo test` (which includes the html5lib tree-construction harness) and ensure no previously-passing case regresses.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(html): implement the in-column-group insertion mode (t0393)"
Then print "T0393 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
