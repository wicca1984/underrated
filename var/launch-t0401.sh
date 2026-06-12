#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0401
LOG=/workspaces/toy-browser/var/log/t0401.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code).

Task t0401 — recognize the `clear` CSS property and its keyword values. Touch ONLY src/css/values.rs. Do NOT edit any other file/module. Leave a `// TODO(spec): ...` in values.rs if something truly needs another module.

Background (read before coding):
- Read src/css/values.rs. The companion property `float` was JUST added (commit for t0396). Find the three sites that handle `float` and mirror them EXACTLY for `clear`:
  1. `is_known_layout_property(name: &str) -> bool` (~line 300): a `matches!(...)` list that now includes `"float"`. Add `"clear"`.
  2. `is_valid_property_value(name, value) -> bool` (~line 395): a `"float" => match value { CssValue::Keyword(kw) => matches!(kw.to_ascii_lowercase().as_str(), "none" | "left" | "right"), _ => false }` arm. Add a `"clear"` arm. For clear the valid keywords are `none | left | right | both`.
  3. `parse_property_value(...)` (~line 537): a `"float" => { if let CssValue::Keyword(kw) = &val { match kw.to_ascii_lowercase().as_str() { "none" | "left" | "right" => Some(val), _ => None } } else { None } }` arm. Add a `"clear"` arm with keywords `none | left | right | both`.
- This is purely keyword recognition/validation/parsing — no layout/paint behavior. Mirror the `float` code precisely (same structure, same `to_ascii_lowercase()` handling); only the property name and the keyword set differ (clear adds `both`).

Scope for THIS task (single file, src/css/values.rs):
1. Add `clear` to `is_known_layout_property`.
2. Add the `clear` arm to `is_valid_property_value` (keywords: none, left, right, both).
3. Add the `clear` arm to `parse_property_value` (keywords: none, left, right, both).
4. Panic-free: no unwrap/expect/panicking indexing in non-test code.

Tests — add to the existing `#[cfg(test)] mod tests` in src/css/values.rs (do NOT modify/delete existing tests; mirror the `test_flex_wrap_and_float` test added in t0396 for setup style):
- `is_known_layout_property("clear")` is true.
- `is_valid_property_value("clear", Keyword("both"))` and for left/right/none are true; an invalid keyword (e.g. "middle") is false.
- `parse_property_value("clear", ...)` returns `Some(Keyword("both"))` for `both` (and left/right/none), and `None` for an invalid keyword.
Use whatever existing token-construction helpers the surrounding tests use (e.g. the `token(...)` / `CssToken::Ident` helper used by test_flex_wrap_and_float); do not invent new infrastructure.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(css): recognize the clear property and its keyword values (t0401)"
Then print "T0401 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
