#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0320
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

Task: t0320 (milestone MS-MVP-JS, task J-0a — prerequisite for wiring HTTP redirect-following into the form-submit navigation path). Implement a redirect-aware load 'hop' method on the \`ResourceLoader\` trait, plus deterministic network-free unit tests. ENTIRELY inside the LOADER module.
Target module: src/loader/mod.rs ONLY. Do NOT modify src/engine, src/script, src/layout, src/url, src/net, lib.rs, any other module's mod.rs, or any file under tests/. Do NOT touch other worktrees.

WHY (context — read before coding):
- t0318 added a generic, network-free redirect follower \`follow_redirects<F>(start: &Url, fetch: F)\` where \`F: FnMut(&Url) -> Result<(RedirectMeta, LoaderResponse), LoadError>\`, plus the \`RedirectMeta { status: u16, location: Option<String> }\` struct and \`MAX_REDIRECTS\`. See src/loader/mod.rs around line 185-225.
- BUT the \`ResourceLoader\` trait's existing \`load_request(...)\` returns only a \`LoaderResponse\` (no HTTP status, no Location header). So nothing can currently DRIVE \`follow_redirects\`: there is no trait method that yields \`(RedirectMeta, LoaderResponse)\`. THIS TASK closes exactly that gap. A later task (not yours) will call your new method from \`engine::navigate\`.

WHAT TO ADD (precise spec — implement exactly this, do not redesign):
1. Add a NEW defaulted method to \`pub trait ResourceLoader\` (the trait around src/loader/mod.rs:393, which already has defaulted methods \`load_rich\` and \`load_request\` — mirror their style and doc-comment density):
   \`\`\`rust
   /// Performs a single load \"hop\", returning HTTP-level redirect metadata
   /// (status code + optional Location header) alongside the response, so that
   /// callers can drive [\`follow_redirects\`]. The default implementation performs
   /// an ordinary [\`ResourceLoader::load_request\`] and reports a terminal 200
   /// response with no Location, preserving existing non-redirecting behavior.
   /// Loaders able to surface real HTTP status and headers override this.
   fn load_request_hop(
       &self,
       url: &Url,
       method: HttpMethod,
       body: &[u8],
       content_type: Option<&str>,
   ) -> Result<(RedirectMeta, LoaderResponse), LoadError> {
       let resp = self.load_request(url, method, body, content_type)?;
       Ok((RedirectMeta { status: 200, location: None }, resp))
   }
   \`\`\`
   Keep the exact method name \`load_request_hop\` and the exact signature above (this is the public IF a later task depends on — do NOT rename or change argument order/types). Do NOT add any new variant to LoadError or any new public type. Do NOT modify \`follow_redirects\`, \`RedirectMeta\`, \`load_request\`, \`load_rich\`, or \`load\`.

TDD — write tests FIRST in the existing \`#[cfg(test)] mod tests\` (src/loader/mod.rs:501), then confirm green. All tests MUST be network-free and deterministic (no real I/O, no files outside any existing test fixtures). Required cases:
  a. DEFAULT behavior: define a tiny mock loader that implements ONLY \`load(&self, _url) -> Ok(b\"hello\".to_vec())\` (so it inherits the default \`load_request_hop\`). Call \`load_request_hop\` and assert the returned RedirectMeta is \`{ status: 200, location: None }\` and the response bytes are \`b\"hello\"\`. This proves existing loaders keep non-redirecting behavior.
  b. DRIVING follow_redirects through an OVERRIDING mock loader: define a mock loader that overrides \`load_request_hop\` to return, on the FIRST distinct URL, \`(RedirectMeta { status: 302, location: Some(\"/final\".into()) }, <some response>)\`, and on the redirected URL, \`(RedirectMeta { status: 200, location: None }, <terminal response with distinctive bytes e.g. b\"FINAL\">)\`. Drive it via \`follow_redirects(&start_url, |u| loader.load_request_hop(u, HttpMethod::Get, b\"\", None))\` and assert the returned LoaderResponse has the terminal bytes \`b\"FINAL\"\` (i.e. the redirect was followed and Location resolved relative to start). Build \`Url\`s via the crate's existing URL construction/parse used elsewhere in these tests.
  c. NON-redirect status passes straight through: a mock whose \`load_request_hop\` returns \`{ status: 200, location: None }\` with bytes \`b\"OK\"\`; driving it through \`follow_redirects\` yields \`b\"OK\"\` with no extra hops.
Use the same Url construction idiom already present in this file's tests (grep the test module for how Url is built — do not invent a new constructor). Keep assertions concrete (assert_eq! on bytes / status / location).

When done: run \`cargo test\`, \`cargo clippy --all-targets -- -D warnings\`, \`cargo fmt --check\`, \`cargo doc --no-deps\` — ALL must be green. This is a logic-only (non-rendering) task, so NO PNG is required. Then \`git add -A && git commit\` with message exactly:
  feat(loader): add ResourceLoader::load_request_hop for redirect-aware loading (t0320)
Then print \`git log -1 --oneline\`, run \`git status --porcelain\` and confirm the working tree is clean. Do NOT push or open a PR (the orchestrator handles that). If the spec is genuinely ambiguous or real-browser behavior conflicts, do NOT decide alone — leave a \`// TODO(spec):\` and report. Finish with a short English summary of what changed and which files you touched (must be only src/loader/mod.rs)." -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta
