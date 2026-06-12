#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0447
LOG=/workspaces/toy-browser/var/log/t0447-cont.log
mkdir -p /workspaces/toy-browser/var/log
if [ -f /workspaces/underrated-meta/var/.env ]; then set -a; . /workspaces/underrated-meta/var/.env; set +a; fi
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Write the code, run the checks, fix until green, then commit. This is a CONTINUATION of a LARGE multi-file task already ~85% done — keep going across many edit/build/fix cycles; do not stop until `cargo build` and `cargo test` pass.

CRITICAL: Do NOT use web search or any web tool. Everything is in local files. Network/web search is forbidden.

You are a Gemini worker on `underrated` (independent web browser engine in Rust, edition 2024). Work and respond in English. Read AGENTS.md (via --include-directories) and obey I-1..I-7. NEVER use unwrap()/expect() in non-test code (I-6). NEVER add an external crate — std only.

CONTEXT: A big-bang migration (t0447) to the categorized Arc-shared style type is ALREADY mostly done on this branch (agent/t0447-css-bigbang-migration, base feature/css-arch). The legacy HashMap `ComputedStyle` has been replaced by `CategorizedComputedStyle` (see src/style/categorized.rs and src/style/mod.rs). The cascade and several consumers are already migrated and COMMITTED (commit 878a7fe is WIP). Your job is to FINISH it: make `cargo build`, `cargo clippy`, and `cargo test` all pass, then commit.

There are ~43 remaining compile errors, all MECHANICAL, concentrated in these files:
  src/layout/mod.rs (17), src/layout/inline.rs (9), src/paint/mod.rs (8), src/layout/flex.rs (6), src/layout/table.rs (4).

Error categories and how to fix each:
  1. E0614 "type `f32` cannot be dereferenced" (~35 occurrences): old code matched a `&CssValue` and did `*v` to deref. The style field is now ALREADY a typed value (f32/i32/String) on a category struct. Remove the stray `*` and read the typed field directly. e.g. `Some(CssValue::Number(v)) => *v` patterns over a style lookup become a direct read of `style.<category>.<field>`.
  2. E0432 "unresolved import `crate::style::ComputedStyle`" (5): the legacy type is gone. Change the import (and the type in signatures) to `crate::style::CategorizedComputedStyle`.
  3. E0599 "no method named `get` found for `&CategorizedComputedStyle`" (2): convert the remaining `style.get("prop")` string lookups to typed field access `style.<category>.<field>` per ADR 0001 section 2.1 property->category table. NO `.get("..")` / `.insert("..")` string-key access may survive outside src/style/.
  4. E0308 mismatched types in src/paint/mod.rs ~line 1141 (`scale_color_alpha(color, ..)`): borrow it — `scale_color_alpha(&color, ..)`.

REQUIRED READING: src/style/categorized.rs (category structs + fields + setters), src/style/mod.rs (the new cascade output type), and /workspaces/underrated-meta/docs/spec/0001-computed-style-layout.md section 2.1 for the property->category.field mapping. Use grep to find each remaining `.get("`/`.insert("`/`*` deref site.

PROCEDURE (iterate, do not give up after one cycle):
  - `cargo build` → read the first errors → fix → repeat until it builds.
  - `cargo fmt`.
  - `cargo clippy --all-targets -- -D warnings` → fix every warning.
  - `cargo test` → fix or UPDATE tests that asserted on the old string API to the typed API. DO NOT delete tests to fake green. Rendering tests must still pass.
  - When build+clippy+test are all green: `git add -A && git commit -m "feat(css): finish big-bang migration to CategorizedComputedStyle, delete legacy HashMap ComputedStyle (t0447)"`.
  - COMMIT before finishing. Report the final `cargo test` summary line.
EOF
exec setsid gemini -p "$PROMPT" \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  < /dev/null >> "$LOG" 2>&1
