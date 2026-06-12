#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0419
LOG=/workspaces/toy-browser/var/log/t0419.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything you need is in the local source files and this prompt. Reading local files is fine; network/web search is forbidden and wastes time.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code — note: test code MAY use unwrap/panic as the existing tests do).

Task t0419 — extend the deterministic oracle snapshot tests to cover the currently-UNTESTED structural layout of fixture `10_news_article.html` (the header/news-content block containment and the in-list links). Touch ONLY the file `tests/oracle_snapshot_test.rs`. Do NOT edit any file under src/, and do NOT edit any fixture HTML. If a spec-correct assertion would fail because of an engine layout bug, do NOT fix the engine and do NOT weaken the test into a no-op — instead leave a `// TODO(spec): <describe the discrepancy>` comment, assert the actually-observed behavior with an explanatory comment, and clearly report the discrepancy in your final summary.

Context (read before coding):
- The fixture `tests/oracle/fixtures/10_news_article.html` has `<div class="container">` (width 600px) containing a `<header>` (with bottom border + padding + margin) holding `<h1 class="site-title">`, followed by `<div class="news-content">` holding `<h2>`, a `<p class="byline">`, three body `<p>` elements, and a `<ul>` with three `<li>` each containing one `<a>`.
- The EXISTING test `test_fixture_10_news_article()` (around line 971) already asserts h1/h2 positive size, that the 4 paragraphs are in top-to-bottom order, that there are 3 list items with positive size, and container width == 600. Do NOT modify or delete it. Your NEW test must cover DIFFERENT, currently-unasserted facts (header-vs-content block flow, containment, and link layout).
- Reuse the existing private helpers at the top of `tests/oracle_snapshot_test.rs`: `load_fixture_snapshot(filename)`, `find_element_by_class(node, class)`, `find_elements_by_class(node, class, &mut results)`, `find_element_by_tag(node, tag)`, and `find_elements_by_tag(node, tag, &mut results)`. Match the exact style of `test_fixture_10_news_article` (e.g. `node["rect"]["y"].as_f64().unwrap()`).

Add a SINGLE new `#[test] fn test_fixture_10_news_header_and_link_layout()` (appended near the existing fixture-10 test) that:
1. Loads the snapshot via `load_fixture_snapshot("10_news_article.html")`.
2. Locates the `<header>` element (via `find_element_by_tag(&snapshot, "header")`) and the `news-content` div (via `find_element_by_class`). Read both rects. Assert each has positive width and height.
3. BLOCK FLOW: assert the header sits strictly above the news-content — `header.y + header.height <= news_content.y + 0.5` (allow a tiny tolerance). They must not vertically overlap.
4. HEADER CONTAINMENT: locate `site-title` (the h1, via `find_element_by_class`). Assert it is contained within the header box horizontally and vertically: `site_title.x >= header.x - 0.5`, `(site_title.x + site_title.width) <= (header.x + header.width) + 0.5`, `site_title.y >= header.y - 0.5`, and `(site_title.y + site_title.height) <= (header.y + header.height) + 0.5`.
5. CONTENT CONTAINMENT: collect the `<p>` elements inside news-content (via `find_elements_by_tag(news_content, "p", &mut v)`). Assert there are 4. For each, assert horizontal containment within news-content: `p.x >= news_content.x - 0.5` and `(p.x + p.width) <= (news_content.x + news_content.width) + 0.5`.
6. LINK LAYOUT: collect the three `<li>` elements (via `find_elements_by_tag` on the `<ul>`), and for each `<li>` find its `<a>` (via `find_element_by_tag(li, "a")`). Assert each `<a>` exists, has positive width and height, and is contained within its `<li>` horizontally: `a.x >= li.x - 0.5` and `(a.x + a.width) <= (li.x + li.width) + 0.5`.

Before writing assertions, you MAY temporarily add `eprintln!` of the observed rects and run `cargo test test_fixture_10_news_header_and_link_layout -- --nocapture` to discover real coordinates, then encode robust RELATIONAL assertions (relationships and tolerances, not brittle magic numbers). Remove any temporary eprintln! before committing. Keep the test deterministic (no timing, no network). Add a short `//` comment above the test naming the uncovered area it guards.

Run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "test(oracle): cover news-article header flow, containment, and in-list links (t0419)"
Then print "T0419 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
