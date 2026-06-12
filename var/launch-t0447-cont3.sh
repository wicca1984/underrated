#!/usr/bin/env bash
# Single-worker continuation of t0447 big-bang migration. SINGLETON ONLY.
set -euo pipefail
cd /workspaces/wt/t0447

read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Write the code, run the checks, fix until green, then commit. This is the CONTINUATION of a LARGE multi-file task already ~90% done -- keep going across MANY edit/build/fix cycles without stopping. Do NOT stop until `cargo build`, `cargo clippy`, and `cargo test` all pass and you have committed. If you run low on turns, COMMIT partial progress first, then continue.

CRITICAL: Do NOT use web search or any web tool. Everything is in local files. Network/web search is forbidden.

You are the ONLY worker on this worktree (/workspaces/wt/t0447, branch agent/t0447-css-bigbang-migration, base feature/css-arch, latest commit a36180a). Work and respond in English. Read AGENTS.md (via --include-directories) and obey I-1..I-7. NEVER use unwrap()/expect() in non-test code (I-6). NEVER add an external crate -- std only. DO NOT delete or skip tests to fake green; UPDATE old-API tests to the typed API instead.

CONTEXT: A big-bang migration (t0447) replacing the legacy HashMap `ComputedStyle` with the categorized Arc-shared `CategorizedComputedStyle` is ~90% done and COMMITTED. The cascade and most consumers are migrated.

REMAINING: exactly 40 MECHANICAL compile errors:
  - 34x E0599 "no method named `get` found for `&CategorizedComputedStyle`"
  - 5x  E0614 "type `f32` cannot be dereferenced"
  - 1x  E0308 "mismatched types"
concentrated in src/paint/mod.rs and src/layout/mod.rs (plus possibly table.rs/flex.rs).

HOW TO FIX EACH:
1. E0599 `.get("prop")`: the legacy `style.get("prop") -> Option<&CssValue>` is GONE. Convert each `style.get("prop")` to direct typed field access on the right category struct per src/style/categorized.rs. Category structs: inherited_text, inherited_list, inherited_table, inherited_ui, inherited_effects, reset_box, reset_surround, reset_background, reset_flex, reset_table, reset_effects. Examples already in this branch: `matches!(style.get("margin-left"), Some(CssValue::Keyword(kw)) if kw=="auto")` became `style.reset_surround.margin_left == -1`; a `width` definite-check became `style.reset_box.width != -1`. READ src/style/categorized.rs for the exact field name, type (String / i32 / u32 / f32 / Option<CssValue>), and the sentinel for "auto"/unset (numeric fields use -1; see each struct's Default). Rewrite the surrounding `if let Some(CssValue::Keyword(s)) = ...` / `match` to read the typed field directly. NO `.get("..")`/`.insert("..")` string-key access may survive outside src/style/.
2. E0614 deref: old code matched `&CssValue` and did `*v`. The field is now already a typed f32 (e.g. reset_effects.opacity, reset_flex.flex_grow/flex_shrink). Remove the stray `*` and read the typed field directly.
3. E0308: fix per the compiler hint (likely a borrow `&color` or String-vs-&str mismatch).

REQUIRED READING before editing: src/style/categorized.rs (every category struct, fields, types, Default sentinels, setters), src/style/mod.rs (cascade output), and /workspaces/underrated-meta/docs/spec/0001-computed-style-layout.md section 2.1 for the property->category.field mapping. Use grep to find each remaining `.get("` / `.insert("` / `*` deref site.

PROCEDURE (iterate, do not give up after one cycle):
  - `cargo build` -> read first errors -> fix -> repeat until it builds.
  - `cargo fmt`.
  - `cargo clippy --all-targets -- -D warnings` -> fix every warning.
  - `cargo test` -> fix or UPDATE tests asserting on the old string API to the typed API. DO NOT delete tests. Rendering tests must still pass.
  - When build+clippy+test are all green: `git add -A && git commit -m "feat(css): finish big-bang migration to CategorizedComputedStyle, delete legacy HashMap ComputedStyle (t0447)"`.
  - COMMIT before finishing (commit partial progress too). Report the final `cargo test` summary line.
EOF

exec gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta
