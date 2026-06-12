#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0406
LOG=/workspaces/toy-browser/var/log/t0406.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code).

Task t0406 — recognize the CSS `table-layout` property and its keyword values. Touch ONLY src/css/values.rs. Do NOT edit any other file/module. Leave a `// TODO(spec): ...` if something truly needs another module.

Background (read before coding) — this is a pure property-RECOGNITION task, identical in shape to how `clear`, `float`, and `visibility` are already handled in src/css/values.rs:
- `pub fn is_known_layout_property(name: &str) -> bool` (around line 292) lists layout property names in a `matches!(...)`. The `clear` and `float` entries are the model.
- `pub fn is_valid_property_value(name: &str, value: &CssValue) -> bool` has one match arm per property. Study the `"float" => match value { CssValue::Keyword(kw) => matches!(kw.to_ascii_lowercase().as_str(), "none" | "left" | "right"), _ => false }` arm (around line 408) and the `"clear"` arm right after it.
- The parse dispatch (around line 560) has matching `"float" =>` / `"clear" =>` arms returning `Some(val)` only for valid keywords, else `None`.

The valid keyword values for `table-layout` are exactly: `auto` | `fixed`.

Scope for THIS task (single file, src/css/values.rs):
1. Add `"table-layout"` to the `is_known_layout_property` `matches!` list (mirror the `"clear"` entry placement/style).
2. Add a `"table-layout" => match value { CssValue::Keyword(kw) => matches!(kw.to_ascii_lowercase().as_str(), "auto" | "fixed"), _ => false }` arm to `is_valid_property_value`, mirroring the `"clear"` arm exactly.
3. Add the corresponding parse arm `"table-layout" => { if let CssValue::Keyword(kw) = &val { match kw.to_ascii_lowercase().as_str() { "auto" | "fixed" => Some(val), _ => None } } else { None } }` mirroring the existing `"clear"` parse arm.
4. Panic-free: no unwrap/expect/panicking indexing in non-test code.

Tests — add to the existing `#[cfg(test)] mod tests` in src/css/values.rs (do NOT modify/delete existing tests; mirror the `clear` test block exactly — search for `is_known_layout_property("clear")` and `parse_property_value("clear", ...)`):
- `assert!(is_known_layout_property("table-layout"));`
- `is_valid_property_value("table-layout", ...)` true for `auto` and `fixed`, false for an invalid keyword (e.g. `bogus`) and for a non-keyword value.
- `parse_property_value("table-layout", ...)` returns `Some` for `auto`/`fixed`, `None` for an invalid keyword.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(css): recognize the table-layout property and its keyword values (t0406)"
Then print "T0406 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
