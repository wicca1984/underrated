#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0397
LOG=/workspaces/toy-browser/var/log/t0397.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code).

Task t0397 — implement the `InSelect` insertion mode in the HTML tree builder. Touch ONLY src/html/tree.rs. Do NOT edit any other file/module. Leave a `// TODO(spec): ...` in tree.rs if something truly needs another module.

Background (read before coding):
- Read src/html/tree.rs. The `InsertionMode::InSelect` enum variant (~line 33) is DEFINED but currently (a) never entered and (b) dispatched as a stub: `InsertionMode::InSelect => self.handle_in_body(token), // TODO(spec)` (~line 104).
- The file has ~13 dedicated `handle_*` functions. The most recently added one, `handle_in_column_group` (see commit ee4e12c), is the reference template for style/structure/error-handling. MIRROR it exactly — do not invent new infrastructure.
- Spec reference: https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inselect

Scope for THIS task (single file, src/html/tree.rs):
1. Add `fn handle_in_select(&mut self, token: Token)` (place it next to the other handlers, e.g. after `handle_in_template`). Implement the spec's "in select" insertion mode rules, mirroring the existing handlers' patterns for inserting characters/comments, error handling, and stack manipulation:
   - Character token that is U+0000 NULL: parse error, ignore.
   - Any other character token: insert the character.
   - Comment: insert a comment.
   - DOCTYPE: parse error, ignore.
   - Start tag `option`: if current node is an `option`, pop it; then insert an `option` element.
   - Start tag `optgroup`: if current node is `option`, pop it; if current node is `optgroup`, pop it; then insert an `optgroup` element.
   - End tag `optgroup`: per spec (pop an `option` first if appropriate, then pop the `optgroup` if current node is `optgroup`; otherwise parse error/ignore).
   - End tag `option`: if current node is `option`, pop it; otherwise parse error/ignore.
   - End tag `select`: if a `select` is in select scope, pop elements until a `select` has been popped, and reset the insertion mode appropriately (mirror however existing handlers leave/reset modes — if a full "reset the insertion mode appropriately" algorithm does not already exist in this file, set the mode back to the mode used before entering select, e.g. `InBody`, and leave a `// TODO(spec): reset insertion mode appropriately` note).
   - Start tag `select`: parse error; act as the `select` end tag (close the select) — do NOT open a nested select.
   - EOF: handle like the other handlers do for EOF (mirror handle_in_body's EOF behavior, e.g. stop parsing / delegate).
   - Anything else: parse error, ignore the token.
   Keep it focused; where the precise spec sub-step depends on machinery not present in this file, leave a `// TODO(spec): ...` rather than reaching into another module.
2. Wire the dispatch (~line 104): replace the stub with `InsertionMode::InSelect => self.handle_in_select(token),`.
3. Wire ENTRY into the mode: in `handle_in_body`, a start tag `select` must insert the `<select>` element and switch the insertion mode to `InSelect` (mirror how handle_in_body switches modes for other elements like table/colgroup). If handle_in_body already inserts `<select>` but does not switch the mode, add the mode switch. Keep this change minimal and within tree.rs.
4. Panic-free: no unwrap/expect/panicking indexing in non-test code; use Option combinators / `matches!` / iterators.

Tests — add to the existing `#[cfg(test)] mod tests` in src/html/tree.rs (do NOT modify/delete existing tests; mirror an existing tree-builder test for setup style — parse a small HTML snippet and assert on the resulting DOM tree shape):
- `<select><option>a</option><option>b</option></select>` produces a `select` element with two `option` children (no improper nesting).
- `<select><option>a<option>b</select>` (unclosed options) produces two sibling `option` elements under `select` (the second `option` start tag implicitly closes the first).
- `<select><optgroup><option>a</option></optgroup></select>` nests `option` under `optgroup` under `select`.
Use whatever existing parse + DOM-inspection helpers the surrounding tests use; do not invent new infrastructure.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(html): implement the in-select insertion mode (t0397)"
Then print "T0397 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
