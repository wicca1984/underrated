#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0423
LOG=/workspaces/toy-browser/var/log/t0423.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything you need is in the local source files and this prompt. Reading local files is fine; network/web search is forbidden and wastes time.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code — note: test code MAY use unwrap/panic as the existing tests do).

Task t0423 — Strengthen HTML Tree Construction insertion-mode conformance with NEW edge-case tests for the "in select" and "in table" insertion modes. Touch ONLY the file `src/html/tree.rs` (add `#[test]` functions inside the existing `#[cfg(test)] mod tests` block; do NOT change any non-test code in this file or any other file).

WHY this is new (do not duplicate existing coverage): the existing tests already cover: `test_in_select_nested_options`, `test_in_select_unclosed_options`, `test_in_select_optgroup`, `test_simple_table`, `test_table_with_implicit_tbody`, `test_table_foster_parenting`, `test_html5lib_table_1/2`, `test_robustness_table_implicit`. Do NOT re-create those. Add tests for DISTINCT, currently-uncovered spec edge cases (see list below). First READ the existing tests and the tree-construction code paths for the "in select" and "in table" insertion modes so your assertions reflect THIS engine's public parse output (use the SAME parse/serialize/query helpers the existing tests use — discover them by reading the test module; do not invent new helpers).

Add tests for these spec edge cases (pick the ones that this engine actually implements; for any case where the engine's behavior diverges from the HTML5 spec, do NOT change the engine — instead assert the ACTUALLY-OBSERVED tree and put a `// TODO(spec): <expected per spec vs observed>` comment above that test, and note it in your summary):
1. "in select": a nested `<select>` start tag acts as a `</select>` (it closes the current select rather than nesting another). Build `<select><option>a</option><select><option>b` and assert the resulting structure (whether a second select nests or the first closes — assert what THIS engine does, annotate vs spec).
2. "in select": an `<input>` (and/or `<textarea>`/`<keygen>`) start tag inside `<select>` is handled specially per spec (it acts to close the select). Assert observed structure for `<select><option>a</option><input>`.
3. "in table": foster-parenting of stray character/text content — text directly inside `<table>` (e.g. `<table>hello<tr><td>x`) is foster-parented out before the table. Assert the stray text node lands before/around the table per this engine, distinct from the existing element foster-parenting test.
4. "in table": a `<caption>` element and/or a `<colgroup>`/`<col>` is placed correctly relative to the table sections. Assert observed structure for `<table><caption>cap</caption><tr><td>x`.

Keep each test small and focused, with a short `//` comment above it naming the exact edge case it guards. Use only deterministic in-memory parsing (no IO, no network, no timing). Mirror the assertion style (tag-name / nesting / text queries) of the existing `test_in_select_*` and `test_*table*` tests in this same module.

You MAY temporarily add `eprintln!`/`dbg!` of the serialized tree, run `cargo test <your_test_name> -- --nocapture` to discover the real parse output, then encode robust assertions and REMOVE any debug prints before committing.

Run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "test(html): cover in-select and in-table insertion-mode edge cases (t0423)"
Then print "T0423 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
