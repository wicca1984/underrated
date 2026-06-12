#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0398
LOG=/workspaces/toy-browser/var/log/t0398.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code).

Task t0398 — implement the structural pseudo-classes `:first-child`, `:last-child`, `:nth-child(An+B)`, and `:nth-last-child(An+B)` in the selector matcher. Touch ONLY src/selector/matching.rs. Do NOT edit any other file/module. Leave a `// TODO(spec): ...` in matching.rs if something truly needs another module.

Background (read before coding):
- Read src/selector/matching.rs. It ALREADY supports the *-of-type* structural pseudo-classes: `first-of-type`, `last-of-type`, `only-of-type`, `nth-of-type(...)`, `nth-last-of-type(...)`, plus `only-child`. See the simple-pseudo match arm (~line 299) and the `nth-...-of-type(` parsing branches (~line 254). It also has An+B parsing for the nth-*-of-type forms — REUSE that exact An+B parser/helper; do NOT write a new one.
- The difference between `*-of-type` and `*-child`: of-type counts only siblings with the SAME element tag name; child counts ALL element siblings regardless of tag name. Mirror the existing `is_first_of_type` / `nth-of-type` helpers but count every element sibling instead of filtering by tag name.

Scope for THIS task (single file, src/selector/matching.rs):
1. Add `:first-child` — matches if the element is the first element child of its parent.
2. Add `:last-child` — matches if it is the last element child.
3. Add `:nth-child(An+B)` — matches if its 1-based index among element siblings (counting from the start) satisfies An+B. Reuse the SAME An+B parsing used by `nth-of-type`.
4. Add `:nth-last-child(An+B)` — same but counting from the end.
5. Wire these into the same match/parse sites the of-type variants use (the `name.starts_with("nth-...(")` branches for the functional forms, and the simple-pseudo match arm for first-child/last-child). Add small private helper fns mirroring the of-type helpers (e.g. `is_first_child`, `is_last_child`, and a child-index helper) — count only Element-node siblings (skip text/comment nodes), consistent with how the of-type helpers enumerate siblings.
6. Panic-free: no unwrap/expect/panicking indexing in non-test code; use Option combinators / iterators.

Tests — add to the existing `#[cfg(test)] mod tests` in src/selector/matching.rs (do NOT modify/delete existing tests; mirror the existing nth-of-type / first-of-type tests for setup style — build a small DOM and assert which nodes match via `parse_selector_list(...)`):
- `:first-child` and `:last-child` select the first/last element child regardless of tag (e.g. mixed `<h1>`,`<p>`,`<p>`,`<span>` children).
- `p:nth-child(2)` matches a `p` only when it is the 2nd element child overall (NOT the 2nd p).
- `:nth-child(odd)` / `:nth-child(2n)` select the expected positions.
- `:nth-last-child(1)` equals `:last-child`.
Use whatever existing parse + DOM helpers the surrounding tests use; do not invent new infrastructure.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(selector): match :first-child/:last-child/:nth-child structural pseudo-classes (t0398)"
Then print "T0398 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
