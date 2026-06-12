#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0412
LOG=/workspaces/toy-browser/var/log/t0412.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code).

Task t0412 — match the `:read-only` and `:read-write` form-state pseudo-classes. Touch ONLY src/selector/matching.rs. Do NOT edit any other file/module. If something truly needs another module, leave a `// TODO(spec): ...` and report instead.

Context (read before coding) — this mirrors EXACTLY how `:disabled` / `:enabled` (and `:required` / `:optional`) are already implemented in src/selector/matching.rs:
- In the pseudo-class match block (around the arms `"checked" => is_checked(dom, node)`, `"disabled" => is_disabled(dom, node)`, `"enabled" => is_enabled(dom, node)`, and the already-present `"required"`/`"optional"` arms), add two new arms: `"read-only" => is_read_only(dom, node)` and `"read-write" => is_read_write(dom, node)`.
- There is a helper `fn is_form_associated(name: &str) -> bool` and helpers `fn is_disabled(dom, node)` / `fn is_enabled(dom, node)` that match `NodeData::Element { name, attrs }` and inspect `attrs` with `ascii::eq_ignore_ascii_case`. Mirror that style exactly.

Spec-scoped semantics (HTML/CSS Selectors Level 4, scoped to the common case):
- `:read-write` matches an `<input>` or `<textarea>` element that is mutable: i.e. it is NOT `disabled` AND NOT `readonly`. (For `<input>`, also require it to be a type that is normally mutable — to keep scope tight, treat any `<input>` or `<textarea>` without `readonly`/`disabled` as read-write; leave a `// TODO(spec):` for the per-input-type editability table such as `type=checkbox/radio/hidden/button` which are technically read-only.)
- `:read-only` matches an `<input>` or `<textarea>` element that is NOT read-write (i.e. has `readonly` or `disabled`). Non-form elements are NOT matched by either pseudo-class in this scoped implementation; leave a `// TODO(spec):` noting that `contenteditable` and the general "any non-editable element is :read-only" rule are out of scope.

Implement two helper functions near `is_disabled`/`is_enabled`:
- `fn is_read_write(dom: &Dom, node: NodeId) -> bool`: element is `<input>` or `<textarea>` AND attrs do NOT contain `readonly` AND do NOT contain `disabled` (case-insensitive).
- `fn is_read_only(dom: &Dom, node: NodeId) -> bool`: element is `<input>` or `<textarea>` AND (attrs contain `readonly` OR contain `disabled`).
Use a small local check for `<input>`/`<textarea>` (do NOT reuse `is_form_associated`, which is broader — select/option/fieldset are not in scope here).

Panic-free: no unwrap/expect/panicking indexing in non-test code.

Tests — add to the existing `#[cfg(test)] mod tests` in src/selector/matching.rs (do NOT modify or delete any existing test; mirror the setup style of the existing `:disabled`/`:required` tests that build Element nodes with `attrs: vec![...]`):
- `<input>` with no readonly/disabled matches `:read-write` and NOT `:read-only`.
- `<input readonly>` matches `:read-only` and NOT `:read-write`.
- `<input disabled>` matches `:read-only` and NOT `:read-write`.
- `<textarea>` (no attrs) matches `:read-write`.
- A `<div>` matches NEITHER `:read-only` nor `:read-write`.
- Case-insensitive attribute name (e.g. `READONLY`) is honored.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(selector): match :read-only and :read-write form-state pseudo-classes (t0412)"
Then print "T0412 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
