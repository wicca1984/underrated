#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0391
LOG=/workspaces/toy-browser/var/log/t0391.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code).

Task t0391 — recognize and validate the `overflow-x` and `overflow-y` longhand properties. Touch ONLY src/css/values.rs. Do NOT edit any other file/module. Leave a `// TODO(spec): ...` in values.rs if something truly needs another module.

Background (read before coding):
- Read src/css/values.rs in full, focusing on how the existing `overflow` (shorthand-ish single property) is handled. Specifically:
  * `is_known_layout_property(name)` (around line 293-296) lists `"overflow"` among known layout property names.
  * `is_valid_property_value(name, value)` (around line 320) has an `"overflow" => match value { CssValue::Keyword(kw) => matches!(kw.as_str(), ...), CssValue::Overflow(_) => true, _ => false }` arm.
  * The value parser (around line 420) has an `"overflow" => { ... }` arm that converts the keyword token into a `CssValue::Overflow(...)`.
- The valid keyword set for overflow is: `visible`, `hidden`, `scroll`, `auto` (and `clip` if the existing code already supports it — check; if not present, do NOT add `clip`, just match the existing set exactly).
- `overflow-x` and `overflow-y` are the per-axis longhands and accept exactly the same single keyword values as `overflow`. Reference: https://developer.mozilla.org/docs/Web/CSS/overflow-x

Scope for THIS task (parse-only; layout/paint clipping is a SEPARATE task — do NOT touch layout/paint):
1. Add `"overflow-x"` and `"overflow-y"` to `is_known_layout_property` alongside `"overflow"`.
2. In `is_valid_property_value`, make `"overflow-x"` and `"overflow-y"` accept the SAME values as `"overflow"` (reuse the exact same match arm logic — e.g. by adding them to a shared arm `"overflow" | "overflow-x" | "overflow-y" => ...`).
3. In the value parser, make `"overflow-x"` and `"overflow-y"` parse the keyword the SAME way `"overflow"` does, producing `CssValue::Overflow(...)` (or whatever the existing overflow arm produces). Prefer extending the existing match arm pattern to `"overflow" | "overflow-x" | "overflow-y"` so there is no duplicated logic.
4. Do NOT implement the two-value `overflow: <x> <y>` shorthand expansion in this task (that belongs in the style longhand-expansion module). If you notice it is missing, leave a precise `// TODO(spec): expand two-value overflow shorthand into overflow-x/overflow-y in style::expand` and move on. Do NOT edit src/style.
5. Panic-free: no unwrap/expect/panicking indexing in non-test code.

Tests — add to the existing `#[cfg(test)] mod tests` in values.rs (do NOT modify/delete existing tests; mirror the existing overflow parse tests around lines 1182-1199):
- `overflow-x: hidden` parses to the same `CssValue::Overflow(...)` variant that `overflow: hidden` produces.
- `overflow-y: scroll` parses correctly.
- `overflow-x: auto` and `overflow-y: visible` parse correctly.
- `is_known_layout_property("overflow-x")` and `("overflow-y")` return true.
- `is_valid_property_value("overflow-x", <Overflow value>)` returns true; an invalid keyword (e.g. `overflow-x: banana`) is rejected the same way `overflow: banana` is.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(css): recognize overflow-x and overflow-y longhand properties (t0391)"
Then print "T0391 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
