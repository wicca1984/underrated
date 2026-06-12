#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0326
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

Task: t0326 (milestone MS-NewTargets, parallel lane — inline layout correctness). Implement the forced line break produced by the HTML \`<br>\` element during inline layout. Today \`<br>\` is parsed as a void element (src/html/tree.rs) but inline layout (src/layout/inline.rs) has NO handling for it, so \`a<br>b\` currently renders \`a\` and \`b\` on the SAME line instead of breaking. Real pages (Wikipedia, addresses, forms) rely on \`<br>\`. NOTE: the existing 'Force a line break!' code around line 296 handles embedded \`\\n\` characters in preserved-whitespace text — that is a DIFFERENT mechanism and does NOT cover the \`<br>\` ELEMENT. You must add explicit handling: when inline layout encounters an element whose tag name is \`br\` (case-insensitive), finish the current line box and start a new one (a hard break), then continue laying out subsequent inline content on the new line. The \`<br>\` itself contributes a break, not a visible glyph; the new line must use the current line height.

SCOPE — STRICT. Modify exactly ONE production file: src/layout/inline.rs. Do NOT modify any other file under src/ (do NOT touch html/, css/, paint/, engine/, style/). Do NOT modify, delete, weaken, rename, or \`#[ignore]\` ANY existing test anywhere. Do NOT touch lib.rs or any module registration. \`git diff --name-only origin/main...HEAD\` MUST list ONLY src/layout/inline.rs (plus, if you choose, NEW fixture/PNG files under var/ which are allowed but optional). If you find the change genuinely requires touching another module, STOP and instead leave a \`// TODO(spec):\` note and report the blocker — do not expand scope.

BEFORE WRITING — read and understand:
- src/layout/inline.rs in full, especially the main inline-run walk, how child elements are descended into, where line boxes are created (\`create_line_box_adjusted\`), and the existing forced-break at ~line 296.
- How an inline element node is matched by tag name elsewhere (e.g. \`NodeData::Element { name, .. }\` with \`name.eq_ignore_ascii_case(\"br\")\`).

REQUIREMENTS:
1. Add a Rust unit test in src/layout/inline.rs proving that \`<body>a<br>b</body>\` lays out \`a\` and \`b\` on two separate line boxes (assert the body/block container has 2 line boxes, or that the second fragment's y-offset is greater than the first). Also assert that text WITHOUT \`<br>\` (\`<body>a b</body>\`) still stays on ONE line (no regression to normal inline wrapping).
2. Handle multiple consecutive \`<br><br>\` producing multiple breaks (empty line between), and \`<br>\` at the start/end gracefully.
3. Do NOT regress existing whitespace-collapsing / wrapping / newline behavior.

GATES (all must pass before you commit):
- \`cargo test\` (entire suite) green.
- \`cargo clippy --all-targets -- -D warnings\` clean (NO \`unwrap\`/\`expect\` in non-test production code — I-6).
- \`cargo fmt --check\` clean (run \`cargo fmt\` first).
- \`cargo doc --no-deps\` clean.

VERIFIED-IN-WINDOW (required, UI-affecting): create a fixture \`var/t0326-br.html\` containing e.g. \`<html><body>line one<br>line two<br><br>line four</body></html>\` and render it through the SHIPPING path to a PNG at \`var/t0326-br.png\` using the existing example (\`cargo run --example render_local_png -- var/t0326-br.html var/t0326-br.png\` — confirm the exact example name/args by reading examples/). Confirm the PNG is a non-trivial size and that the lines are visually stacked. Keep both files.

COMMIT: after all gates pass and the PNG exists, \`git add -A\` then \`git commit -m \"feat(layout): break inline flow on the <br> element (t0326)\"\`. Then run \`git diff --name-only origin/main...HEAD\` and confirm ONLY the allowed files are listed. Do NOT push and do NOT open a PR — the orchestrator will review, gate, and merge. Report a concise summary of what you changed and the gate results when done." \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta
