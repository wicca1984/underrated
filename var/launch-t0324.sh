#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0324
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

Task: t0324 (milestone MS-MVP-JS, critical path J-2 — HTML Form Submit wiring). Add ENGINE-LEVEL wiring for IMPLICIT FORM SUBMISSION via the Enter key. Today \`engine::navigate_from_click\` resolves a submit-button CLICK into a navigated result Page. The missing piece is the keyboard path: pressing Enter while a form control (e.g. the search text input) is focused must submit that control's owning form and navigate to the result page. You will add the engine helper that does this.

SCOPE — STRICT. Modify exactly ONE production file: src/engine/mod.rs. Do NOT modify any other file under src/ (forms/, loader/, event/, shell/, dom/ etc. already provide everything you need — see below). Do NOT modify, delete, weaken, rename, or \`#[ignore]\` ANY existing test anywhere. Do NOT touch lib.rs, other worktrees, or any module's mod-registration. \`git diff --name-only origin/main...HEAD\` MUST list ONLY src/engine/mod.rs.

PRODUCTION CODE THAT ALREADY EXISTS (do NOT change it — just CALL it; confirm every exact name/signature by grepping before writing):
- \`engine::navigate_from_click(dom, clicked, values, base, loader, viewport_width) -> Option<Page>\` in src/engine/mod.rs — MIRROR this function's structure, signature style, doc-comment style, and error handling. Read it first.
- \`engine::navigate(&req, base, loader, viewport_width) -> Page\` — dispatches a NavigationRequest and renders (already follows redirects).
- \`underrated::forms::find_form_for_button(dom, node) -> Option<NodeId>\` (pub, in src/forms/mod.rs) — despite the name it is GENERIC: given ANY control NodeId it returns the owning form via the \`form\` attribute or the nearest ancestor \`<form>\`. Use it to find the form that owns the focused control. Confirm it is \`pub\` and its exact signature.
- \`underrated::forms::submit(dom, form, values) -> Option<NavigationRequest>\` (pub) — builds the NavigationRequest from a form NodeId + FormState. Use this (NOT submit_from_button — there may be no button in the Enter path).
- \`underrated::forms::FormState\` — the values container, same type \`navigate_from_click\` already takes.
Read src/forms/mod.rs (grep: \`pub fn find_form_for_button\`, \`pub fn submit\`, \`pub fn submit_from_button\`) and the existing \`navigate_from_click\` in src/engine/mod.rs BEFORE writing. Do not invent names or fields.

WHAT TO ADD (in src/engine/mod.rs only):
1. A new pub fn, mirroring navigate_from_click, e.g.:
   \`pub fn navigate_from_enter(dom: &Dom, focused: NodeId, values: &FormState, base: &Url, loader: &dyn ResourceLoader, viewport_width: f32) -> Option<Page>\`
   Use the SAME fully-qualified path style for types that navigate_from_click uses (e.g. \`crate::dom::Dom\`, \`crate::infra::NodeId\`, \`crate::forms::FormState\`, \`Url\`, \`&dyn ResourceLoader\`) — match the existing function exactly.
   Behavior: resolve the focused control's owning form via \`crate::forms::find_form_for_button(dom, focused)?\`; build the request via \`crate::forms::submit(dom, form, values)?\`; return \`Some(navigate(&req, base, loader, viewport_width))\`. Returns \`None\` when the focused node is not associated with any form (mirroring navigate_from_click returning None when the click doesn't trigger a submission). Never panics (I-6) — NO unwrap/expect in the function body; use \`?\`.
   Write a doc-comment in the same voice as navigate_from_click's, explaining the implicit-submission semantics (Enter in a form control submits the owning form).
2. NOTE on HTML default-button semantics: a fully spec-compliant implicit submission selects a 'default button' submitter and includes its name/value. For this MVP task, submitting the form directly via forms::submit (no explicit submitter) is acceptable for the search-box case. Leave a single \`// TODO(spec): implicit submission should select the form's default submit button as submitter\` comment above the new fn — do NOT try to implement default-button selection in this task (out of scope, would touch forms).

TESTS (add to the existing \`#[cfg(test)] mod tests\` in src/engine/mod.rs — mirror the style already there, reuse its DummyLoader / mock-loader helpers and its DOM-building helpers):
- test_navigate_from_enter_submits_owning_form: build a GET search form with an \`<input name=\"q\">\` inside a \`<form action=\"/search\">\`; get the input NodeId; set a FormState value for it; provide a mock loader that returns a result page (e.g. \`<html><body><a>Hit</a></body></html>\`) for the expected search URL; call navigate_from_enter with the INPUT as the focused node; assert it returns Some(Page) whose DOM contains the expected result text (mirror how existing engine tests assert on the navigated Page's DOM/text_content).
- test_navigate_from_enter_no_form_returns_none: build an \`<input>\` that is NOT inside any form; call navigate_from_enter on it; assert it returns None.
  (If you need a mock loader that maps a specific URL, mirror exactly the mock-loader pattern already used by tests in src/engine/mod.rs or in tests/form_submit_nav_test.rs — read one first. Do NOT add real I/O. expect() inside test bodies is fine per AGENTS.md test conventions.)

When done, run ALL of these and ensure GREEN:
  cargo test --lib engine
  cargo test
  cargo clippy --all-targets -- -D warnings
  cargo fmt --check
  cargo doc --no-deps
This task is engine logic wiring (no new visible rendering primitive), so NO PNG is required.
Then \`git add -A && git commit\` with message EXACTLY:
  feat(engine): wire implicit Enter-key form submission via navigate_from_enter (t0324)
Then print \`git log -1 --oneline\`, run \`git status --porcelain\` (must be clean), and \`git diff --name-only origin/main...HEAD\` (must show ONLY src/engine/mod.rs). Do NOT push or open a PR (the orchestrator handles that).
If any assumed API does not exist or behaves differently (e.g. find_form_for_button is not pub, or submit's signature differs), do NOT modify other src files to force it and do NOT fake it — leave a \`// TODO(spec):\` note, report the exact mismatch, and stop. Finish with a short English summary of what you verified and confirm the ONLY file touched is src/engine/mod.rs." -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta < /dev/null
