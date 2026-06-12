#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0388
LOG=/workspaces/toy-browser/var/log/t0388.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

Task t0388 — implement invalidation of a `srcset` attribute that mixes width (`w`) and pixel-density (`x`) descriptors. Touch ONLY src/html/srcset.rs. Do NOT edit dom/, paint/, layout/, style/, engine/, css/, main.rs, or any other module/file. If something genuinely requires another module, leave a `// TODO(spec): ...` comment in src/html/srcset.rs and stop.

Background (read before coding):
- Read src/html/srcset.rs in full. `pub fn parse_srcset(srcset: &str) -> Vec<ImageCandidate>` builds a `Vec<ImageCandidate>`. Each candidate has `w_descriptor: Option<u32>` (Some => width/`w` descriptor) and a `density: f32` (used for `x` descriptors and the bare/no-descriptor default of 1.0).
- There is a `// TODO(spec): Strict validation and invalidation of mixed 'w' and 'x' descriptors is left for the future.` at the top of the loop.
- HTML spec "parse a srcset attribute": a srcset must not mix width descriptors with pixel-density (or default-density) descriptors. Such a list is inconsistent and yields no usable candidate set (the element falls back to its `src`). Reference: https://html.spec.whatwg.org/multipage/images.html#parsing-a-srcset-attribute

Definition for THIS task:
- A candidate is "width-type" iff `w_descriptor.is_some()`.
- A candidate is "density-type" iff `w_descriptor.is_none()` (this covers BOTH explicit `x` descriptors AND the bare no-descriptor default, which is an implicit 1x density candidate).
- A srcset is INCONSISTENT when, among the successfully-parsed candidates, BOTH at least one width-type and at least one density-type candidate are present.

Implement (minimal, idiomatic, matching surrounding code) in src/html/srcset.rs:
1. After the existing parse loop fills `candidates`, add a consistency check: if `candidates` contains both a width-type and a density-type candidate, the whole srcset is invalid — return `Vec::new()` (empty). Otherwise return `candidates` as before.
2. Replace the stale `// TODO(spec): Strict validation and invalidation of mixed 'w' and 'x' descriptors...` comment, since mixing is now handled. You may leave a narrower `// TODO(spec):` for finer per-descriptor parse-error reporting if you wish, but the mixing case must be done.
3. Do NOT change `select_candidate` or `resolve_sizes`. Do NOT alter the per-candidate parsing logic — only add the post-loop consistency gate. Keep behavior identical for all-width and all-density (incl. bare default) lists.
4. Panic-free (AGENTS.md I-6): no unwrap()/expect()/panicking indexing in non-test code. Use iterator `.any(...)` for the checks.

Add unit tests in the existing `#[cfg(test)] mod tests` block in src/html/srcset.rs (these MUST pass alongside the existing tests, which you must NOT modify or delete — verify `test_parse_srcset_basic`, `_w_descriptors`, `_no_descriptor`, `_empty_garbage` still pass):
- `test_parse_srcset_mixed_w_and_x_invalid`: `parse_srcset("a.png 480w, b.png 2x")` -> empty Vec.
- `test_parse_srcset_mixed_w_and_default_invalid`: `parse_srcset("a.png 480w, b.png")` -> empty Vec (bare default is a density candidate).
- `test_parse_srcset_all_w_still_valid`: `parse_srcset("a.png 480w, b.png 960w")` -> 2 candidates (regression guard, unchanged).
- `test_parse_srcset_all_x_still_valid`: `parse_srcset("a.png 1x, b.png 2x")` -> 2 candidates (regression guard, unchanged).

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green, commit:
  git add -A && git commit -m "feat(html): invalidate srcset with mixed w/x descriptors (t0388)"
Then print "T0388 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
