#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0417
LOG=/workspaces/toy-browser/var/log/t0417.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything you need is in the local source files and this prompt. Reading local files is fine; network/web search is forbidden and wastes time.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code — note: test code MAY use unwrap/panic as the existing tests do).

Task t0417 — extend the deterministic oracle snapshot tests to cover the currently-UNTESTED Wikipedia "infobox" internal layout. Touch ONLY the file `tests/oracle_snapshot_test.rs`. Do NOT edit any file under src/, and do NOT edit the fixture HTML. If a spec-correct assertion would fail because of an engine layout bug, do NOT fix the engine and do NOT weaken the test into a no-op — instead leave a `// TODO(spec): <describe the discrepancy>` comment, assert the actually-observed behavior with an explanatory comment, and clearly report the discrepancy in your final summary.

Context (read before coding):
- The fixture `tests/oracle/fixtures/09_wiki_article.html` renders a Wikipedia-style article. Inside `<div class="container">` there are two inline-block columns: `<div class="main-content">` (width 480px) and `<div class="infobox">` (width 240px + 10px padding + 1px border each side => 262px box). The infobox contains, in order: one `<div class="infobox-title">` and three `<div class="infobox-row">` elements.
- The existing test `test_fixture_09_wiki_article()` already asserts the infobox/main-content WIDTHS and basic heading/paragraph/list facts. It does NOT assert anything about the infobox's internal vertical stacking, nor that the infobox rows are contained within the infobox box. That internal layout is the uncovered area you are adding coverage for.
- Reuse the existing private helpers already defined at the top of `tests/oracle_snapshot_test.rs`: `load_fixture_snapshot(filename)`, `find_element_by_class(node, class)`, `find_elements_by_class(node, class, &mut results)`, `find_element_by_tag`, and `find_elements_by_tag`. Match the style of the existing `test_fixture_09_wiki_article` function exactly (e.g. `node["rect"]["y"].as_f64().unwrap()`).

What to add (a SINGLE new `#[test]` function, e.g. `fn test_fixture_09_wiki_infobox_internal_layout()`, appended near the existing fixture-09 test — do NOT modify or delete the existing test):
1. Load the snapshot via `load_fixture_snapshot("09_wiki_article.html")`.
2. Locate the `infobox` element via `find_element_by_class`. Read its rect (x, y, width, height). Assert height > 0.
3. Locate the `infobox-title` via `find_element_by_class`, and collect the three `infobox-row` elements via `find_elements_by_class`. Assert exactly 3 rows are found.
4. Assert VERTICAL STACKING inside the infobox: the title's `y` is strictly less than the first row's `y` (title sits above rows), and the three rows are in strictly increasing `y` order (row[i].y < row[i+1].y, allowing a small >= with a 0.1 tolerance only if needed — prefer strict `<` and only relax with a documented tolerance if the observed values require it). Each of these block-level divs must NOT horizontally overlap its sibling (they stack, not side-by-side): assert each next element's `y` is at or below the previous element's bottom (prev.y + prev.height - 0.5).
5. Assert CONTAINMENT: each of the title and the three rows has `x >= infobox.x - 0.5` and `(x + width) <= (infobox.x + infobox.width) + 0.5` (rows live inside the infobox content box, within tolerance for padding/border rounding).
6. Assert each row and the title has positive width and positive height.

Before writing assertions with specific numeric thresholds, you MAY temporarily add `eprintln!` of the observed rects and run `cargo test test_fixture_09_wiki_infobox_internal_layout -- --nocapture` to discover the real coordinates, then encode robust relational assertions (relationships and tolerances, NOT brittle exact magic numbers except where the existing test already uses them). Remove any temporary eprintln! before committing.

Keep the new test deterministic (no timing, no network). Add a short `//` comment above the test explaining what uncovered area it guards.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "test(oracle): cover wiki infobox internal vertical stacking and containment (t0417)"
Then print "T0417 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
