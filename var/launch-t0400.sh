#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0400
LOG=/workspaces/toy-browser/var/log/t0400.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code).

Task t0400 — implement the `InHeadNoscript` insertion mode in the HTML tree builder. Touch ONLY src/html/tree.rs. Do NOT edit any other file/module. Leave a `// TODO(spec): ...` in tree.rs if something truly needs another module.

Background (read before coding):
- Read src/html/tree.rs. The `InsertionMode::InHeadNoscript` enum variant is DEFINED but currently dispatched as a stub: `InsertionMode::InHeadNoscript => self.handle_in_body(token), // TODO(spec)` (~line 93).
- The file has many dedicated `handle_*` functions (e.g. `handle_in_head` ~line 227, and the recently added `handle_in_select` / `handle_in_column_group`). MIRROR them exactly for style/structure/error-handling/stack manipulation — do not invent new infrastructure.
- Spec reference: https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inheadnoscript

Scope for THIS task (single file, src/html/tree.rs):
1. Add `fn handle_in_head_noscript(&mut self, token: Token)` (place it next to `handle_in_head`). Implement the spec's "in head noscript" insertion mode rules, mirroring existing handlers:
   - DOCTYPE: parse error, ignore.
   - Start tag `html`: process using the "in body" rules (delegate to handle_in_body).
   - End tag `noscript`: pop the current `noscript` node off the stack of open elements (current node should be `noscript`; its parent should be `head`), then switch the insertion mode to `InHead`.
   - Whitespace character token, Comment, and start tags `basefont`, `bgsound`, `link`, `meta`, `noframes`, `style`: process using the "in head" rules (delegate to handle_in_head).
   - End tag `br`: act as the "anything else" case below (do NOT ignore).
   - Start tag `head` or `noscript`: parse error, ignore.
   - Anything else (including EOF and any other end tag): parse error; pop the current `noscript` node, switch the insertion mode to `InHead`, then reprocess the token (re-dispatch the same token through the tree builder, mirroring however other handlers reprocess after a mode switch).
   Where a precise spec sub-step depends on machinery not present in this file, leave a `// TODO(spec): ...` rather than reaching into another module.
2. Wire the dispatch (~line 93): replace the stub with `InsertionMode::InHeadNoscript => self.handle_in_head_noscript(token),`.
3. Wire ENTRY into the mode: in `handle_in_head`, a start tag `noscript` (when scripting is disabled — this engine has no scripting flag active during parse, so treat it as disabled) must insert the `<noscript>` element and switch the insertion mode to `InHeadNoscript`. Mirror how handle_in_head switches modes for other elements. Keep this change minimal and within tree.rs. If handle_in_head already handles `noscript` differently, adjust minimally so the new mode is entered.
4. Panic-free: no unwrap/expect/panicking indexing in non-test code; use Option combinators / `matches!`.

Tests — add to the existing `#[cfg(test)] mod tests` in src/html/tree.rs (do NOT modify/delete existing tests; mirror an existing tree-builder test — parse a small HTML snippet and assert on the resulting DOM tree shape):
- `<head><noscript><link></noscript></head>` produces a `noscript` element under `head` containing the `link` element.
- `<head><noscript><p>x</p></noscript></head>`: the `<p>` is NOT allowed in head-noscript, so it triggers the "anything else" path — the `noscript` is closed and the `<p>` ends up in the body (assert `p` is not a child of `noscript`).
- `<head><noscript></noscript></head>` produces an empty `noscript` under `head`.
Use whatever existing parse + DOM-inspection helpers the surrounding tests use; do not invent new infrastructure.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(html): implement the in-head-noscript insertion mode (t0400)"
Then print "T0400 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
