#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0420
LOG=/workspaces/toy-browser/var/log/t0420.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

CRITICAL: Do NOT use web search or any web tool. Everything you need is in the local source files and this prompt. Reading local files is fine; network/web search is forbidden and wastes time.

You are a Gemini worker on `underrated` (an independent web browser engine in Rust, edition 2024). Work and respond in English. First read the AGENTS.md passed via --include-directories and follow I-1..I-7 (esp. I-3 no cross-worktree access, I-5 one-module, I-6 no unwrap/expect/panicking-index in non-test code — note: test code MAY use unwrap/panic as the existing tests do).

Task t0420 — extend the deterministic oracle snapshot tests to cover the currently-UNTESTED VERTICAL block-flow stacking order of the real Google homepage fixture `08_google_real.html`. Touch ONLY the file `tests/oracle_snapshot_test.rs`. Do NOT edit any file under src/, and do NOT edit any fixture HTML. If a spec-correct assertion would fail because of an engine layout bug, do NOT fix the engine and do NOT weaken the test into a no-op — instead leave a `// TODO(spec): <describe the discrepancy>` comment, assert the actually-observed behavior with an explanatory comment, and clearly report the discrepancy in your final summary.

Context (read before coding):
- The EXISTING test `test_fixture_08_google_real()` (around line 568) is the C-1 regression floor: it already asserts HORIZONTAL facts — the search input (name="q", class="lst") is centered, the two `<input class="lsb">` submit buttons (btnG "Google 検索", btnI "I'm Feeling Lucky") flow side-by-side and are centered AS A PAIR, and every `<input type="hidden">` has rect width 0. Do NOT modify or delete it. Your NEW test must cover DIFFERENT, currently-unasserted facts: the VERTICAL stacking order of the centered homepage column.
- The fixture's visible centered column (inside `<center>`) flows top-to-bottom: the logo `<img id="hplogo" alt="Google">`, then the search `<input name="q" class="lst">`, then the button pair (`<input class="lsb">` x2), then the footer `<span id="footer">` containing several `<a>` links and a copyright `<p>`.
- Reuse the EXISTING private helpers at the top of `tests/oracle_snapshot_test.rs`. Confirm the exact names by reading the file; they include: `load_fixture_snapshot(filename)`, `find_element_by_attr(node, key, val)`, `find_element_by_class(node, class)`, `find_elements_by_class(node, class, &mut results)`, `find_element_by_tag(node, tag)`, `find_elements_by_tag(node, tag, &mut results)`. Read rects via `node["rect"]["y"].as_f64().unwrap()` etc., matching the existing style.

Add a SINGLE new `#[test] fn test_fixture_08_google_vertical_stack()` (appended right after `test_fixture_08_google_real`) with a short `//` comment above it naming the uncovered area it guards (vertical block-flow order of the homepage column). It must:
1. Load the snapshot via `load_fixture_snapshot("08_google_real.html")`.
2. Locate the search input (`find_element_by_attr(&snapshot, "name", "q")` or fallback `find_element_by_class(&snapshot, "lst")`); read its y and height. Assert positive height.
3. Locate the two `lsb` buttons (`find_elements_by_class(&snapshot, "lsb", &mut v)`; expect exactly 2). Compute the pair's MIN top-y (`buttons_top`) and MAX bottom-y. Assert each has positive height.
4. Locate the footer (`find_element_by_attr(&snapshot, "id", "footer")`); read its y. Assert positive height.
5. VERTICAL ORDER (the core new facts), using a small tolerance (e.g. 0.5px):
   - search input sits strictly above the button pair: `q.y + q.height <= buttons_top + 0.5` (no vertical overlap).
   - the button pair sits strictly above the footer: `buttons_bottom <= footer.y + 0.5`.
6. LOGO ORDER (guard against the known B-3 relative-URL-image gap): locate the logo via `find_element_by_attr(&snapshot, "id", "hplogo")`. If it is found AND its rect height > 0.0, assert it sits above the search input: `logo.y + logo.height <= q.y + 0.5`. If the logo is absent or has zero height, do NOT fail — instead add a `// TODO(spec):` note that the relative-URL logo did not render (known B-3) and skip that sub-assertion. Report which branch happened in your final summary.

Before writing assertions, you MAY temporarily add `eprintln!` of the observed rects and run `cargo test test_fixture_08_google_vertical_stack -- --nocapture` to discover real coordinates, then encode robust RELATIONAL assertions (relationships and tolerances, not brittle magic numbers). Remove any temporary eprintln! before committing. Keep the test deterministic (no timing, no network).

Run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green:
  git add -A && git commit -m "test(oracle): cover Google homepage vertical block-flow stacking order (t0420)"
Then print "T0420 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
