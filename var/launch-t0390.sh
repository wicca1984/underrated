#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0390
LOG=/workspaces/toy-browser/var/log/t0390.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code).

Task t0390 — evaluate media-conditioned `sizes` so the correct source size is picked per viewport. Touch ONLY src/html/srcset.rs. Do NOT edit any other file/module. Leave a `// TODO(spec): ...` in srcset.rs if something truly needs another module.

Background (read before coding):
- Read src/html/srcset.rs in full. `pub fn resolve_sizes(sizes: Option<&str>, viewport_width: u32) -> u32` currently has `// TODO(spec): Complete evaluation of media-conditioned sizes is left for the future.` and only parses the FIRST comma part, ignoring its media condition.
- HTML `sizes` syntax: a comma-separated list of `<source-size>` entries. Each entry (except optionally the LAST) is `<media-condition> <source-size-value>`. The last entry may be just `<source-size-value>` with no media condition (the default). The first entry whose media condition matches the viewport wins; its `<source-size-value>` is the result. If none match, fall back to `viewport_width`. Reference: https://html.spec.whatwg.org/multipage/images.html#sizes-attribute

Scope for THIS task (self-contained, px lengths only — same simplification as current code):
1. Split `sizes` on commas into entries. For each entry in order:
   - Separate an optional leading media condition (a parenthesized expression like `(max-width: 600px)` or `(min-width: 900px)`) from the trailing source-size-value.
   - The source-size-value is the LAST whitespace-separated token; support only `<number>px` (round to u32), consistent with existing code. Ignore non-px values (treat that entry's value as unusable / skip).
   - A media condition matches when evaluated against `viewport_width`:
     * `(max-width: Npx)` matches iff `viewport_width <= N`.
     * `(min-width: Npx)` matches iff `viewport_width >= N`.
     * An entry with NO parenthesized condition (the default/last) always matches.
     * Any other / unparseable condition: treat as NON-matching (do not crash).
   - Return the source-size-value of the FIRST entry whose condition matches AND whose value is a valid px length.
2. If no entry matches, return `viewport_width` (unchanged fallback). Keep `None` -> `viewport_width`.
3. Write small private helpers (e.g. `fn parse_px(tok: &str) -> Option<u32>`, `fn condition_matches(cond: &str, vw: u32) -> bool`). Panic-free: no unwrap/expect/panicking indexing in non-test code; use Option combinators.
4. Do NOT change `parse_srcset` or `select_candidate`.

Tests — add to the existing `#[cfg(test)] mod tests` in srcset.rs (do NOT modify/delete existing tests; the existing `test_resolve_sizes_basic` must still pass, including its `(max-width: 600px) 200px, 100px` case which at vw=1280 yields 100):
- `(max-width: 600px) 200px, 100px` at vw=500 -> 200 (condition matches).
- `(max-width: 600px) 200px, 100px` at vw=1280 -> 100 (falls through to default).
- `(min-width: 900px) 400px, 50px` at vw=1000 -> 400.
- `(min-width: 900px) 400px, 50px` at vw=300 -> 50 (default).
- `(max-width: 600px) 200px` at vw=1280 (no default entry, no match) -> 1280 (viewport fallback).
- multiple conditions: `(max-width:400px) 100px, (max-width:800px) 300px, 600px` at vw=700 -> 300 (first matching is the second entry).

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(html): evaluate media-conditioned sizes in resolve_sizes (t0390)"
Then print "T0390 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
