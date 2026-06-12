#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0325
# Auth: prefer canonical var/.env, fall back to bashrc export.
if grep -q '^export GEMINI_API_KEY=' /workspaces/underrated-meta/var/.env 2>/dev/null; then
  eval "$(grep -m1 '^export GEMINI_API_KEY=' /workspaces/underrated-meta/var/.env)"
elif grep -q '^GEMINI_API_KEY=' /workspaces/underrated-meta/var/.env 2>/dev/null; then
  export "$(grep -m1 '^GEMINI_API_KEY=' /workspaces/underrated-meta/var/.env)"
else
  eval "$(grep -m1 '^export GEMINI_API_KEY=' ~/.bashrc)"
fi
mkdir -p /workspaces/underrated-meta/var/worker-logs
exec gemini -p "You are a Gemini worker on \`underrated\` (an independent web browser engine in Rust, edition 2024). Work and respond in English.
First, read the entire AGENTS.md passed via --include-directories and follow all of it (esp. I-1..I-7). One task = one module/concern.

Task: t0325 (milestone MS-NewTargets, parallel lane — UA stylesheet correctness). Add the missing \`display: inline\` default rules for HTML inline-level elements to the user-agent default stylesheet. Today the UA default CSS declares block elements explicitly but NEVER declares the common inline elements (span, a, b, strong, i, em, ...). Because the engine's layout treats an element as inline-level ONLY when its computed \`display\` is explicitly inline/inline-block (see src/layout/mod.rs \`is_inline_level\`, which returns false when no display rule applies), a BARE \`<span>Hi</span>\` or \`<a>link</a>\` currently lays out as a BLOCK on its own line instead of flowing inline with surrounding text. This is wrong per CSS and breaks real-world rich text (e.g. Wikipedia). Fix the UA stylesheet so these elements default to \`display: inline\`.

SCOPE — STRICT. Modify exactly ONE production file: src/engine/mod.rs (the \`UA_DEFAULT_CSS\` const string near the top, lines ~19-66). Do NOT modify any other file under src/ (do NOT touch layout/, paint/, style/, css/). Do NOT modify, delete, weaken, rename, or \`#[ignore]\` ANY existing test anywhere. Do NOT touch lib.rs or any module registration. \`git diff --name-only origin/main...HEAD\` MUST list ONLY src/engine/mod.rs (plus, if you choose, a NEW html fixture under var/ which is fine but not required).

BEFORE WRITING — read and confirm:
- Read \`UA_DEFAULT_CSS\` in src/engine/mod.rs in full. Note it already styles a/b/strong/i/em with text-decoration/font-weight/font-style but sets NO display for them, and explicitly declares block elements on the \`div, p, h1..h6, ul, ol, li, section, ...\` line.
- Read \`is_inline_level\` in src/layout/mod.rs (do NOT modify it) to confirm it matches \`CssValue::Keyword(\"inline\"|\"inline-block\")\` or \`CssValue::Display(DisplayValue::Inline|InlineBlock)\`. Your new rule must produce one of those, which a plain \`display: inline\` declaration already does via the existing parser — confirm by grepping how \`display: inline\` / \`display: block\` are parsed in the existing UA CSS path (the block elements already work, so \`inline\` will too).

WHAT TO ADD (inside UA_DEFAULT_CSS only): add a single new rule line declaring \`display: inline\` for the standard inline-level HTML elements. Use this exact set (it must NOT include any element already defaulted to block, and must NOT include replaced/form elements that are inline-block like input/button):
   a, b, strong, i, em, span, code, small, big, sub, sup, abbr, cite, q, s, strike, del, u, ins, mark, label, tt, kbd, samp, var, dfn, time, bdi, bdo, wbr, font { display: inline; }
Place it logically (e.g. right after the block-elements line). Keep the existing a/b/i/em/etc. decoration rules intact — they layer on top and must remain. Match the surrounding string-literal formatting EXACTLY (each physical line ends with \`\\n\\\` inside the Rust raw-by-backslash continuation string; mirror line 22's style).

TESTS (add to the existing \`#[cfg(test)] mod tests\` in src/engine/mod.rs, OR to the existing layout/paint test pattern if engine tests can't easily assert layout — prefer engine's own test module; mirror the helpers already there):
- Add a test that builds a small document like \`<body>foo<span>bar</span>baz</body>\`, runs it through the shipping render path far enough to compute styles+layout (mirror how existing engine tests in src/engine/mod.rs drive render_page / styles), and asserts the \`<span>\` is laid out INLINE — i.e. the span's box shares a line with the surrounding text rather than starting a new block line. Concretely assert that \`is_inline_level\`-equivalent behavior holds: the computed \`display\` for the span resolves to inline (assert the span's ComputedStyle \`display\` == inline keyword/value), AND/OR that foo/bar/baz fragments share the same line-box y-coordinate. Use whichever assertion the existing test infrastructure makes cleanest — read a neighboring engine/layout test first and copy its mechanism. Do NOT add real I/O; use the existing DummyLoader pattern.
- Keep it minimal and deterministic.

VERIFIED-IN-WINDOW (REQUIRED — this task changes rendering): produce a PNG via the SHIPPING path and save it under var/.
  1. Create var/t0325-inline.html containing a clear case, e.g.:
     <html><body><p>start <a href=\"#\">link</a> <b>bold</b> <span>span</span> end</p><p>X<em>Y</em>Z</p></body></html>
  2. Render it: \`cargo run --example render_local_png -- var/t0325-inline.html --out var/t0325-inline.png\`
  3. Confirm the PNG is non-trivial (\`ls -l var/t0325-inline.png\` shows a reasonable size). Eyeball intent: the inline elements must now flow ON THE SAME LINE as surrounding text (\`start link bold span end\` on one line), not stacked vertically.
  Report the exact PNG path in your summary.

When done, run ALL of these and ensure GREEN:
  cargo test
  cargo clippy --all-targets -- -D warnings
  cargo fmt --check
  cargo doc --no-deps
Then \`git add -A && git commit\` with message EXACTLY:
  fix(engine): default common inline HTML elements to display:inline in UA stylesheet (t0325)
Then print \`git log -1 --oneline\`, run \`git status --porcelain\` (must be clean), and \`git diff --name-only origin/main...HEAD\` (must show ONLY src/engine/mod.rs and optionally the var/ fixture/PNG). Do NOT push or open a PR (the orchestrator handles that).
If any assumption is wrong (e.g. adding the rule breaks an existing test because some element in the set was relied upon as block), do NOT delete or weaken that test and do NOT touch other src files — instead REMOVE only the offending element from your inline set, re-run, and note it; if still blocked, leave a \`// TODO(spec):\` note, report the exact failure, and stop. Finish with a short English summary of what you verified, the PNG path, and confirm the ONLY production file touched is src/engine/mod.rs." -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta < /dev/null
