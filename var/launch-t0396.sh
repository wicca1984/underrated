#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0396
LOG=/workspaces/toy-browser/var/log/t0396.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code).

Task t0396 — recognize the `flex-wrap` and `float` CSS keyword properties in the value parser. Touch ONLY src/css/values.rs. Do NOT edit any other file/module. Leave a `// TODO(spec): ...` in values.rs if something truly needs another module.

Background (read before coding):
- Read src/css/values.rs. Mirror EXACTLY how the recently-added longhand keyword properties (e.g. `overflow-x`/`overflow-y`, `align-items`) are recognized and validated. There are THREE coordinated sites to update, around these locations:
  * `is_known_layout_property()` (~line 304): a match listing recognized property names. Add `"flex-wrap"` and `"float"`.
  * `is_valid_property_value()` (~line 377-385): per-property keyword validation. Add cases accepting the keyword set for each.
  * `parse_property_value()` (~line 522): the parsing case that turns the recognized property into its `CssValue` (these are simple `CssValue::Keyword` values, like the existing keyword properties — do NOT invent a new enum).
- Accepted keyword values:
  * `flex-wrap`: `nowrap`, `wrap`, `wrap-reverse`.
  * `float`: `none`, `left`, `right`.
- IMPORTANT: layout already CONSUMES these as `CssValue::Keyword` (src/layout/flex.rs reads `flex-wrap`; src/layout/float.rs reads `float`). Your job is ONLY to make the css value parser recognize/validate/produce them so they survive parsing. Do NOT touch layout — it already reads them. Mirror the EXACT style of the existing keyword properties; do not invent new infrastructure or enum variants.

Scope for THIS task (single file, src/css/values.rs):
1. Add `"flex-wrap"` and `"float"` to the known-property recognition.
2. Add keyword validation for each (accept only the listed keywords; reject others).
3. Produce `CssValue::Keyword` for valid values, exactly like the sibling keyword properties.
4. Panic-free: no unwrap/expect/panicking indexing in non-test code; use Option combinators / `matches!` / iterators.

Tests — add to the existing `#[cfg(test)] mod tests` in src/css/values.rs (do NOT modify/delete existing tests; mirror the existing property-recognition tests such as the overflow-x/overflow-y ones for setup style):
- `flex-wrap: wrap`, `flex-wrap: nowrap`, `flex-wrap: wrap-reverse` are recognized and produce the expected `CssValue::Keyword`.
- `float: left`, `float: right`, `float: none` are recognized and produce the expected `CssValue::Keyword`.
- An invalid value (e.g. `flex-wrap: banana` or `float: up`) is rejected exactly like other invalid keyword values are in the surrounding tests.
Use whatever existing parse + assertion helpers the surrounding tests use; do not invent new infrastructure.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(css): recognize flex-wrap and float keyword properties (t0396)"
Then print "T0396 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
