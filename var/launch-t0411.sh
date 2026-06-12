#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0411
LOG=/workspaces/toy-browser/var/log/t0411.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code).

Task t0411 — recognize the CSS `cursor` property and its keyword values. Touch ONLY src/css/values.rs (and, if and only if a corresponding test module lives there, add tests in the same file). Do NOT edit any other file/module. If something truly needs another module, leave a `// TODO(spec): ...` and report instead.

Context (read before coding) — this mirrors EXACTLY how `visibility` and `clear` were already added:
- `src/css/values.rs` has `pub fn is_known_layout_property(name: &str) -> bool` whose `matches!(...)` list currently ends with `"clear" | "visibility"`. Add `"cursor"` to that list.
- The same file has `pub fn is_valid_property_value(name: &str, value: &CssValue) -> bool`. Add a new match arm for `"cursor"` right next to the existing `"visibility" => match value { CssValue::Keyword(kw) => matches!(kw.to_ascii_lowercase().as_str(), ...), _ => false }` arm.

What to implement:
1. Add `"cursor"` to `is_known_layout_property`.
2. Add a `"cursor" => match value { CssValue::Keyword(kw) => matches!(kw.to_ascii_lowercase().as_str(), <KEYWORDS>), _ => false }` arm in `is_valid_property_value`, where <KEYWORDS> is the common CSS `cursor` keyword set:
   "auto" | "default" | "none" | "context-menu" | "help" | "pointer" | "progress" | "wait" | "cell" | "crosshair" | "text" | "vertical-text" | "alias" | "copy" | "move" | "no-drop" | "not-allowed" | "grab" | "grabbing" | "e-resize" | "n-resize" | "ne-resize" | "nw-resize" | "s-resize" | "se-resize" | "sw-resize" | "w-resize" | "ew-resize" | "ns-resize" | "nesw-resize" | "nwse-resize" | "col-resize" | "row-resize" | "all-scroll" | "zoom-in" | "zoom-out"
3. Leave a `// TODO(spec):` marker noting that `url(...)` custom cursor images and comma-separated cursor fallback lists are out of scope (keyword values only for now).

Panic-free: no unwrap/expect/panicking indexing in non-test code.

Tests — add to the existing `#[cfg(test)] mod tests` in src/css/values.rs (do NOT modify or delete any existing test; mirror the setup style of the existing visibility/clear tests if present, otherwise the nearest keyword-property test):
- `is_known_layout_property("cursor")` is true.
- `is_valid_property_value("cursor", &CssValue::Keyword("pointer".into()))` is true; also test "auto", "not-allowed", "grab" => true.
- An unknown keyword like "bogus" => false, and a non-keyword value (e.g. a number) => false.
- Case-insensitivity: "Pointer" / uppercase => true.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(css): recognize the cursor property and its keyword values (t0411)"
Then print "T0411 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
