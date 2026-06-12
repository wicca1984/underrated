#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0408
LOG=/workspaces/toy-browser/var/log/t0408.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code).

Task t0408 — add the `:required` and `:optional` form-state pseudo-classes to the selector matcher. Touch ONLY src/selector/matching.rs. Do NOT edit any other file/module. Leave a `// TODO(spec): ...` if something truly needs another module.

Background (read before coding):
- src/selector/matching.rs already matches `:checked`, `:disabled`, `:enabled` via helper fns `is_checked`, `is_disabled`, `is_enabled` (around lines 587-635) and dispatches them in the pseudo-class `match name.to_ascii_lowercase().as_str()` block (around lines 284-303).
- `is_disabled`/`is_enabled` use the existing helper `is_form_associated(name)` and iterate `attrs` with `ascii::eq_ignore_ascii_case`. Mirror that exact style.

Scope for THIS task (single file, src/selector/matching.rs):
1. Add a helper `fn is_required(dom: &Dom, node: NodeId) -> bool`: true iff the node is an `Element` whose tag is one of input/select/textarea AND whose attrs contain a `required` attribute (presence-only, case-insensitive). Use `ascii::eq_ignore_ascii_case` exactly like `is_disabled`.
2. Add a helper `fn is_optional(dom: &Dom, node: NodeId) -> bool`: true iff the node is an `Element` whose tag is one of input/select/textarea AND whose attrs do NOT contain a `required` attribute (mirror `is_enabled`). Note: only those three tags qualify (a `<div>` is neither :required nor :optional).
3. Wire both into the pseudo-class dispatch match, next to the `"disabled"`/`"enabled"` arms:
   - `"required" => is_required(dom, node),`
   - `"optional" => is_optional(dom, node),`

Panic-free: no unwrap/expect/panicking indexing in non-test code.

Tests — add to the existing `#[cfg(test)] mod tests` in src/selector/matching.rs (do NOT modify/delete existing tests; mirror the existing `:disabled`/`:enabled` tests for parse + match setup):
- `<input required>` matches `input:required` and does NOT match `input:optional`.
- `<input>` (no required) matches `input:optional` and does NOT match `input:required`.
- `<select required>` matches `:required`; `<textarea>` matches `:optional`.
- A `<div required>` matches NEITHER `:required` nor `:optional`.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "feat(selector): match :required and :optional form-state pseudo-classes (t0408)"
Then print "T0408 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
