#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0323
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

Task: t0323 (milestone MS-MVP-JS, critical path J-0 — the ACCEPTANCE smoke test). Add a deterministic, network-free integration test that exercises the full chain: HTML search form Submit -> NavigationRequest -> engine::navigate following an HTTP 3xx REDIRECT -> final 200 result page DOM build -> render/raster. This becomes the milestone's acceptance gate.

SCOPE — STRICT. Create exactly ONE NEW FILE: tests/search_redirect_smoke_test.rs. Do NOT modify any file under src/ (the production code already supports redirect-following — see below). Do NOT modify, delete, weaken, or rename ANY existing test or any other file. Do NOT touch lib.rs or other worktrees. \`git diff --name-only origin/main...HEAD\` MUST list ONLY tests/search_redirect_smoke_test.rs.

PRODUCTION CODE THAT ALREADY EXISTS (do NOT change it — just call it from the test):
- \`underrated::forms::submit(&dom, form_id, &form_state) -> Option<NavigationRequest>\` produces the request from a submitted form.
- \`underrated::engine::navigate(&req, &base_url, &loader, viewport_width) -> Page\` ALREADY follows 3xx redirects internally via \`underrated::loader::follow_redirects\`, which calls \`ResourceLoader::load_request_hop\` on the loader. So to test redirect-following you implement a MOCK loader whose \`load_request_hop\` returns a redirect on the first hop.
- \`underrated::loader::ResourceLoader\` trait. Its \`load_request_hop(&self, url, method, body, content_type) -> Result<(RedirectMeta, LoaderResponse), LoadError>\` is what \`follow_redirects\` drives. \`RedirectMeta { status: u16, location: Option<String> }\`. A redirect is followed when \`status\` is one of 301|302|303|307|308 AND \`location\` is Some. \`LoaderResponse { bytes: Vec<u8>, content_type: String, charset: Option<String> }\`.
- Confirm ALL of these exact names/fields/signatures by reading src/loader/mod.rs (grep: \`fn load_request_hop\`, \`struct RedirectMeta\`, \`struct LoaderResponse\`, \`fn follow_redirects\`) and src/engine/mod.rs (\`pub fn navigate\`) BEFORE writing. Do not invent fields.

TEMPLATE TO MIRROR (copy its structure/imports/helpers): tests/form_submit_nav_test.rs. It already builds a search form, calls forms::submit, sets a FormState value, calls engine::navigate, and asserts the result DOM + rasterizes. Reuse its \`find_element_by_tag\` / \`find_input_by_name\` helpers and its raster-assertion style. The ONLY new thing your test adds is the redirect hop in the mock loader.

WHAT THE NEW TEST MUST DO (\`#[test] fn test_search_submit_redirect_result_smoke()\`):
1. Build a GET search form (action=\"/search\", an \`<input name=\"q\">\`, a submit), parse with \`parse_document\`.
2. Set q to a value, call forms::submit to get the NavigationRequest; assert it is a GET whose URL contains the encoded query.
3. Implement a mock loader that overrides \`load_request_hop\` (NOT just load_request) with TWO mapped hops, keyed by the serialized URL:
   - Hop 1: the submitted search URL (e.g. https://example.com/search?q=...) returns \`RedirectMeta { status: 302, location: Some(\"https://example.com/results?q=...\".into()) }\` with an EMPTY/placeholder LoaderResponse body.
   - Hop 2: the redirect-target URL (https://example.com/results?q=...) returns \`RedirectMeta { status: 200, location: None }\` with a LoaderResponse whose bytes are the result HTML, e.g. \`<html><body><a id=\\\"r1\\\" href=\\\"/p\\\">Result One</a></body></html>\`, content_type \"text/html\", charset Some(\"utf-8\").
   Use a HashMap<String, ...> keyed on \`url.serialize()\` exactly like the template. For unmapped URLs return \`Err(LoadError::NotFound)\`. (If \`follow_redirects\` resolves a RELATIVE Location against the current URL, you may key on the absolute resolved URL — verify by reading follow_redirects; prefer ABSOLUTE Location strings to avoid ambiguity.)
4. Call engine::navigate(&req, &base_url, &mock_loader, 800.0). Because the first hop is a 302, navigate MUST transparently follow to the results page.
5. ASSERT the FINAL result page was built from hop 2 (NOT the redirect placeholder): find the \`<a>\` and assert its text_content == \"Result One\". This proves the redirect was followed end-to-end.
6. Add a paint-level assertion mirroring the template: build_display_list + rasterize(800,600) and assert the canvas has more than one distinct color (i.e. the result actually painted).

Also add a SECOND smaller assertion or a note that a non-redirecting (status 200 directly) submit still works is NOT required — keep scope to the redirect acceptance path; one focused test fn is enough (you may add a brief second \`#[test]\` only if it stays in this same new file and adds no src changes).

All assertions MUST be deterministic and network-free. NO real I/O. Do NOT use unwrap/expect in production code (this is a test file, so expect() in the test body is acceptable per AGENTS.md test conventions — mirror the template which uses .expect with messages).

When done: run \`cargo test --test search_redirect_smoke_test\`, then full \`cargo test\`, \`cargo clippy --all-targets -- -D warnings\`, \`cargo fmt --check\`, \`cargo doc --no-deps\` — ALL must be green. This is a logic/integration test, so NO PNG is required. Then \`git add -A && git commit\` with message EXACTLY:
  test(engine): add Submit->redirect->result acceptance smoke test (t0323)
Then print \`git log -1 --oneline\`, run \`git status --porcelain\` (must be clean), and \`git diff --name-only origin/main...HEAD\` (must show ONLY tests/search_redirect_smoke_test.rs). Do NOT push or open a PR (the orchestrator handles that). If the redirect API does not behave as described (e.g. navigate does NOT follow redirects, or follow_redirects/load_request_hop is absent), do NOT modify src to make it pass and do NOT fake it — instead leave a \`// TODO(spec):\` note in the test, report the exact mismatch, and stop. Finish with a short English summary of what you verified and that the only file touched is tests/search_redirect_smoke_test.rs." -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta < /dev/null
