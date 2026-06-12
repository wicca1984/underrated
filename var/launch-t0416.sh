#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0416
LOG=/workspaces/toy-browser/var/log/t0416.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code).

Task t0416 — recognize the CSS `direction` property and its keyword values. Touch ONLY src/css/values.rs. Do NOT edit any other file/module. If something truly needs another module, leave a `// TODO(spec): ...` and report instead.

This is a RECOGNITION-ONLY task (parse + validate the property and its keywords). Do NOT wire it into layout/paint consumption — mirror exactly how the existing `visibility`, `clear`, and `float` properties are handled in src/css/values.rs.

The `direction` property has exactly two valid keyword values: `ltr` and `rtl` (ASCII case-insensitive).

Make THREE edits in src/css/values.rs, mirroring the existing `visibility` handling:
1. In `pub fn is_known_layout_property(name: &str) -> bool`: add `| "direction"` to the `matches!` list (alongside "visibility", "clear", etc.).
2. In `pub fn is_valid_property_value(...)`: add a new match arm
   `"direction" => match value { CssValue::Keyword(kw) => matches!(kw.to_ascii_lowercase().as_str(), "ltr" | "rtl"), _ => false }`,
   placed near the existing "visibility" arm (before the final `_ => true,`).
3. In `pub fn parse_property_value(...)`: add a new match arm
   `"direction" => { if let CssValue::Keyword(kw) = &val { match kw.to_ascii_lowercase().as_str() { "ltr" | "rtl" => Some(val), _ => None } } else { None } }`,
   placed near the existing "visibility" arm (before the final `_ => Some(val),`).

Tests — in the existing `#[cfg(test)] mod tests` at the bottom of src/css/values.rs, add a `#[test] fn test_direction_property()` mirroring the existing `test_visibility_property`:
- assert `is_known_layout_property("direction")` is true.
- assert `is_valid_property_value("direction", &CssValue::Keyword("ltr".into()))` and `"rtl"` are true; `"sideways"` is false.
- assert `parse_property_value("direction", &[token(CssToken::Ident("ltr".into()))])` == `Some(CssValue::Keyword("ltr".into()))`; same for "rtl"; and that an invalid keyword (e.g. "sideways") yields `None`. Use the same `token(...)` test helper the other tests in this file use.

Keep it panic-free: no unwrap/expect/panicking indexing in non-test code. Document nothing new is required beyond matching the surrounding style.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(css): recognize the direction property and its keyword values (t0416)"
Then print "T0416 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
