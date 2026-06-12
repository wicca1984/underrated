#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0403
LOG=/workspaces/toy-browser/var/log/t0403.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code).

Task t0403 — recognize the `visibility` CSS property and its keyword values. Touch ONLY src/css/values.rs. Do NOT edit any other file/module. Leave a `// TODO(spec): ...` in values.rs if something truly needs another module.

Background (read before coding):
- Read src/css/values.rs. The property `clear` was JUST added (commit t0401) and `float` before it. Find the three sites that handle `clear`/`float` and mirror them EXACTLY for `visibility`:
  1. `is_known_layout_property(name: &str) -> bool` (~line 292-307): a `matches!(...)` list that includes `"float"` and `"clear"`. Add `"visibility"`.
  2. `is_valid_property_value(name, value) -> bool` (~line 408-420): a `"clear" => match value { CssValue::Keyword(kw) => matches!(kw.to_ascii_lowercase().as_str(), "none" | "left" | "right" | "both"), _ => false }` arm. Add a `"visibility"` arm. For visibility the valid keywords are `visible | hidden | collapse`.
  3. `parse_property_value(...)` (~line 560-575): the `"clear" => { if let CssValue::Keyword(kw) = &val { match kw.to_ascii_lowercase().as_str() { ... => Some(val), _ => None } } else { None } }` arm. Add a `"visibility"` arm with keywords `visible | hidden | collapse`.
- This is purely keyword recognition/validation/parsing — no layout/paint behavior. Mirror the `clear` code precisely (same structure, same `to_ascii_lowercase()` handling); only the property name and the keyword set differ.

Scope for THIS task (single file, src/css/values.rs):
1. Add `visibility` to `is_known_layout_property`.
2. Add the `visibility` arm to `is_valid_property_value` (keywords: visible, hidden, collapse).
3. Add the `visibility` arm to `parse_property_value` (keywords: visible, hidden, collapse).
4. Panic-free: no unwrap/expect/panicking indexing in non-test code.

Tests — add to the existing `#[cfg(test)] mod tests` in src/css/values.rs (do NOT modify/delete existing tests; mirror the `test_flex_wrap_and_float` / clear test for setup style and the token helper they use):
- `is_known_layout_property("visibility")` is true.
- `is_valid_property_value("visibility", Keyword("hidden"))` and for visible/collapse are true; an invalid keyword (e.g. "gone") is false.
- `parse_property_value("visibility", ...)` returns `Some(Keyword("hidden"))` for `hidden` (and visible/collapse), and `None` for an invalid keyword.
Use whatever existing token-construction helpers the surrounding tests use; do not invent new infrastructure.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(css): recognize the visibility property and its keyword values (t0403)"
Then print "T0403 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
