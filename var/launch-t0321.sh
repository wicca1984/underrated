#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0321
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
First, read the entire AGENTS.md passed via --include-directories and follow all of it (esp. I-1..I-7). One task = one module.

Task: t0321 (milestone MS-MVP-JS, task J-0b — wire HTTP redirect-following into the form-submit navigation path). Make \`engine::navigate\` transparently follow 3xx redirects by driving the existing \`follow_redirects\` helper through the \`ResourceLoader::load_request_hop\` trait method. ENTIRELY inside the ENGINE module.
Target module: src/engine/mod.rs ONLY. Do NOT modify src/loader, src/script, src/layout, src/url, src/net, lib.rs, any other module's mod.rs, or any file under tests/. Do NOT touch other worktrees.

WHY (context — read before coding):
- t0318 added \`pub fn follow_redirects<F>(start: &Url, mut fetch: F) -> Result<LoaderResponse, LoadError>\` where \`F: FnMut(&Url) -> Result<(RedirectMeta, LoaderResponse), LoadError>\` in src/loader/mod.rs (~line 198). It loops following 301/302/303/307/308 Location headers, resolving each relative to the current URL, up to MAX_REDIRECTS.
- t0320 added the trait method \`ResourceLoader::load_request_hop(&self, url, method, body, content_type) -> Result<(RedirectMeta, LoaderResponse), LoadError>\` (src/loader/mod.rs ~line 428). Its DEFAULT impl just calls \`load_request\` and reports a terminal \`RedirectMeta { status: 200, location: None }\`, so loaders that do not override it keep non-redirecting behavior.
- BUT \`engine::navigate\` (src/engine/mod.rs ~line 339) still calls \`loader.load_request(...)\` DIRECTLY (around line 355), so redirects are NOT followed today. THIS TASK closes exactly that gap.

WHAT TO CHANGE (precise — implement exactly this, do not redesign):
1. In \`engine::navigate\`, replace the direct \`loader.load_request(&resolved_url, method, ...)\` call (the \`let response = match loader.load_request(...) { Ok(res) => res, Err(_) => return render_page(\"\", base, loader, viewport_width) };\` block, ~lines 355-363) with a call to \`crate::loader::follow_redirects\` driven by \`load_request_hop\`, e.g.:
   \`\`\`rust
   let response = match crate::loader::follow_redirects(&resolved_url, |url| {
       loader.load_request_hop(url, method, req.body.as_bytes(), req.content_type.as_deref())
   }) {
       Ok(res) => res,
       Err(_) => return render_page(\"\", base, loader, viewport_width),
   };
   \`\`\`
   Keep the SAME error fallback (render an empty page) and keep \`method\` as the already-computed \`crate::loader::HttpMethod\`. Everything AFTER obtaining \`response\` (charset sniff, BOM offset, decode, \`render_page(&decoded_html, &resolved_url, loader, viewport_width)\`) stays exactly as is. Do NOT change the function signature, the \`navigate_from_click\` wrapper, or any other function.
2. NOTE on base URL after redirect: \`follow_redirects\` returns only the \`LoaderResponse\`, not the final post-redirect URL, so the result page still resolves relative links against the ORIGINAL \`resolved_url\`. This is a known limitation. Leave EXACTLY this comment immediately above the \`render_page(&decoded_html, ...)\` final line:
   \`// TODO(spec): after a redirect chain, relative URLs on the result page should resolve against the final hop URL, but follow_redirects does not surface it yet (needs a loader IF extension — future task).\`
   Do NOT attempt to change the loader IF to return the final URL (out of scope; would cross modules).

TDD — add tests in the existing \`#[cfg(test)] mod tests\` block of src/engine/mod.rs (after the existing \`test_navigate_get_and_post_forms\`). Study that test and its \`MockLoader\` first (grep for \`struct MockLoader\` in this file) to reuse the exact idiom for building Url / NavigationRequest / scanning the resulting DOM text. All tests MUST be network-free and deterministic. Required NEW test(s):
  a. REDIRECT IS FOLLOWED: define a small mock loader (local to the test fn, or extend the pattern) that OVERRIDES \`load_request_hop\`: on the first/original URL it returns \`(RedirectMeta { status: 302, location: Some(\"/final\".into()) }, <any LoaderResponse>)\`; on the redirected URL (\"/final\" resolved against base) it returns \`(RedirectMeta { status: 200, location: None }, <LoaderResponse whose bytes are HTML like b\"<html><body><h1>Redirected result</h1></body></html>\">)\`. Build the LoaderResponse via the same struct fields used elsewhere in this file's tests (grep \`LoaderResponse {\`). Call \`navigate(&get_request, &base_url, &loader, 800.0)\` and assert the resulting page's DOM contains the text \"Redirected result\" (scan descendants for \`NodeData::Text\` exactly like \`test_navigate_get_and_post_forms\` does). This proves the 302 was transparently followed end-to-end through navigate.
  b. NON-redirect path still works: a mock returning a terminal 200 (or simply relying on the existing MockLoader that only implements \`load_request\` and inherits the default hop) navigates straight to the result — assert the result DOM contains the expected text. (You may simply confirm the existing \`test_navigate_get_and_post_forms\` still passes, but ALSO add at least one explicit assertion that a non-redirecting hop yields the page directly.)
Keep assertions concrete (scan for the exact result text). Do NOT weaken or delete any existing test.

When done: run \`cargo test\`, \`cargo clippy --all-targets -- -D warnings\`, \`cargo fmt --check\`, \`cargo doc --no-deps\` — ALL must be green. This is a logic-only (non-rendering) wiring task, so NO PNG is required (the smoke/render verification is a separate follow-up task J-0). Then \`git add -A && git commit\` with message exactly:
  feat(engine): follow 3xx redirects in navigate via load_request_hop (t0321)
Then print \`git log -1 --oneline\`, run \`git status --porcelain\` and confirm the working tree is clean. Do NOT push or open a PR (the orchestrator handles that). If the spec is genuinely ambiguous or real-browser behavior conflicts, do NOT decide alone — leave a \`// TODO(spec):\` and report. Finish with a short English summary of what changed and which files you touched (must be only src/engine/mod.rs)." -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta
