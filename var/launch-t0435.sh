#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0435
LOG=/workspaces/toy-browser/var/log/t0435.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything you need is in the local source files and this prompt. Reading local files is fine; network/web search is forbidden and wastes time.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-5 one-module; I-6 NO unwrap/expect in library/src code; test code MAY use unwrap/panic).

Task t0435 — PERFORMANCE FIX (milestone MS-NewTargets-Perf). Eliminate a per-node O(rules) recomputation in style resolution. This is a behavior-preserving refactor scoped to EXACTLY ONE FILE: `src/style/mod.rs`. Do NOT touch any other file.

THE PROBLEM (read the code to confirm before editing):
In `src/style/mod.rs`, `compute_node_style(...)` computes a value called `ua_rules_count` like this (recomputed for EVERY DOM node):

    let ua_rules_count = stylesheet
        .rules
        .iter()
        .position(|rule| {
            if let Rule::Qualified(qr) = rule {
                let s = serialize_component_values(&qr.prelude);
                s.replace(" ", "") == "head,style,script,meta,link,title"
            } else {
                false
            }
        })
        .map(|pos| pos + 1)
        .unwrap_or(0);

This value depends ONLY on `stylesheet` (not on `node`), yet `compute_node_style` is called once per DOM node from `compute_styles_with_viewport`. For a large page (e.g. ~1MB Wikipedia with many rules) this is O(nodes * rules) string serialization and is a real hotspot.

THE FIX (hoist the invariant out of the per-node loop):
  1. Compute `ua_rules_count` EXACTLY ONCE inside `compute_styles_with_viewport`, BEFORE the `while let Some(node) = stack.pop()` traversal loop (it only needs `stylesheet`).
  2. Add a parameter `ua_rules_count: usize` to `compute_node_style(...)` and pass the precomputed value in at the call site.
  3. Remove the now-redundant per-node computation block from `compute_node_style`, using the parameter instead. The rest of `compute_node_style` (the `for decl in &mut matched_declarations { if decl.source_order >= ua_rules_count ... }` adjustment and `collect_presentational_hints(dom, node, ua_rules_count, ...)`) must keep working IDENTICALLY using the passed-in value.

CONSTRAINTS:
  - Behavior MUST be identical: same `ua_rules_count` value, same cascade results. This is purely moving an invariant computation out of the loop. Do NOT change cascade/specificity/sorting logic.
  - ONLY edit `src/style/mod.rs`. If `compute_node_style` has other callers in this file, update them too (all within this one file). Do NOT change any public function signature that is used outside this file — `compute_styles` and `compute_styles_with_viewport` signatures MUST stay the same. `compute_node_style` is a private `fn`, so adding a parameter to it is fine.
  - NO unwrap/expect/panic in non-test code (I-6).

VERIFY behavior preservation:
  - Run the FULL existing test suite; every test that exercises styling/cascade/UA-default must still pass unchanged. Do NOT modify, delete, skip, or `#[ignore]` any existing test (I-4). You MAY add one small new `#[cfg(test)]` test in this file asserting that styles computed for a small multi-node document are unchanged (optional but encouraged).

Run: `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test` (full suite). If ALL green:
  git add -A && git commit -m "perf(style): hoist invariant ua_rules_count out of per-node style computation (t0435)"
Then print "T0435 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
