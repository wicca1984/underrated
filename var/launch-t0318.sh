#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0318
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

Task: t0318 (milestone MS-MVP-JS, sub-task J-1) — implement an HTTP 3xx redirect-following PRIMITIVE plus deterministic unit tests, entirely inside the LOADER module.
Read: docs/SPEC.md and docs/ARCHITECTURE.md (under /workspaces/underrated-meta/) for the loader/net layering.
Target module: src/loader/mod.rs (and src/loader/http.rs if strictly needed). Touch ONLY src/loader/. Do NOT modify src/engine, src/net, src/url, lib.rs, any other module's mod.rs, tests/ outside the loader's own #[cfg(test)] module, or other worktrees.

HARD CONSTRAINT — DO NOT BREAK OTHER MODULES (this enforces one-task-one-module):
The public struct \`LoaderResponse\` (src/loader/mod.rs) is constructed with plain struct literals (no \`..rest\`) in OTHER modules: src/net/mod.rs:235, src/engine/mod.rs:714, src/engine/mod.rs:1787, and tests/form_submit_nav_test.rs:42. Therefore you MUST NOT add, remove, or rename any field of \`LoaderResponse\` — doing so would force edits to engine/net/tests and is FORBIDDEN. Implement redirect following WITHOUT changing the shape of \`LoaderResponse\`.

DESIGN (self-contained, network-free, single-module):
1. In src/loader/mod.rs, add a new error variant \`TooManyRedirects\` to the existing \`LoadError\` enum (this enum lives in this same module, so it is in scope). Update any exhaustive \`match LoadError\` arms that live INSIDE src/loader/ only; if an exhaustive match exists in another module, prefer a non-breaking approach (the variant is additive — only modules that exhaustively match without a wildcard would break; verify with \`cargo build\` and if an OUTSIDE module breaks, STOP and leave a \`// TODO(spec):\` note instead of editing it, then report).
2. Add a small internal value type to represent one HTTP hop's redirect-relevant metadata, e.g.:
     pub struct RedirectMeta { pub status: u16, pub location: Option<String> }
   (status is the HTTP status code; location is the raw \`Location\` header value if present). Keep it \`#[derive(Debug, Clone)]\`.
3. Add a reusable, generic, network-free redirect-following function in src/loader/mod.rs, e.g.:
     pub const MAX_REDIRECTS: usize = 10;
     pub fn follow_redirects<F>(start: &Url, mut fetch: F) -> Result<LoaderResponse, LoadError>
     where F: FnMut(&Url) -> Result<(RedirectMeta, LoaderResponse), LoadError>
   Behavior:
     - Start at \`start\`. Call \`fetch(&current_url)\` to get \`(meta, resp)\`.
     - If \`meta.status\` is a redirect status (301, 302, 303, 307, 308) AND \`meta.location\` is Some, resolve the Location against the current URL using \`crate::url::resolve(&current_url, &location)\` (this is an existing public helper at src/url/mod.rs:550 — CALL it, do not reimplement, do not modify src/url). If resolve returns None, return the current \`resp\` as-is (cannot follow). Otherwise set current_url = resolved and loop.
     - Cap the number of hops at MAX_REDIRECTS; if exceeded, return \`Err(LoadError::TooManyRedirects)\`.
     - For any non-redirect status (or redirect without a Location), return \`Ok(resp)\` (the final response).
   This function performs NO real I/O itself — all fetching is delegated to the \`fetch\` closure, which is exactly what makes it deterministically testable and reusable by the engine/E2E layer later (J-0/J-2 will pass a real or mock fetch closure).
4. (Optional, only if trivial and within src/loader/http.rs) expose the numeric status + Location from the real ureq path as a helper returning \`RedirectMeta\`, so a future caller can build the closure. ureq already follows redirects transparently for live network, so do NOT rip that out; this helper is only to surface metadata. If wiring this cleanly requires touching other modules, SKIP it and leave a \`// TODO(spec): surface status/Location for engine-driven redirect following\`.

TDD — write tests FIRST in the existing \`#[cfg(test)] mod tests\` of src/loader/mod.rs, then implement until green. Use closures as the mock fetch (NO network, NO files). Required cases:
  a. Single 302 with absolute Location -> second fetch returns 200 body; assert \`follow_redirects\` returns the FINAL body/response.
  b. 302 with a RELATIVE Location (e.g. \`/results?q=x\`) correctly resolves against the start URL via crate::url::resolve before the second fetch (assert the closure was called with the resolved absolute URL — capture seen URLs in a Vec<String> via a RefCell or a mutable closure).
  c. A redirect chain longer than MAX_REDIRECTS returns \`Err(LoadError::TooManyRedirects)\` (e.g. a closure that always returns 302 to a new URL).
  d. A 200 response on the first hop passes through unchanged (no extra fetch).
  e. A 3xx WITHOUT a Location header returns the 3xx response as-is (no loop, no error).
  Build each \`LoaderResponse\` in tests using its existing fields ONLY (bytes/content_type/charset — do not invent fields).

When done: run \`cargo test\`, \`cargo clippy --all-targets -- -D warnings\`, \`cargo fmt --check\`, \`cargo doc --no-deps\` — ALL must be green. This is a non-rendering (logic-only) task, so NO PNG is required.
Then \`git add -A && git commit\` with message exactly:
  feat(loader): add network-free 3xx redirect-following primitive with unit tests (t0318)
Then print \`git log -1 --oneline\`, run \`git status --porcelain\` and confirm the working tree is clean. Do NOT push or open a PR (the orchestrator handles that).
If the spec is ambiguous or real browser behavior conflicts, do NOT decide alone — leave \`// TODO(spec):\` and report. Finish with a short English summary of what changed and which files you touched (must be only src/loader/)." \
  -m gemini-3.5-flash --approval-mode yolo -o stream-json --include-directories /workspaces/underrated-meta \
  > /workspaces/underrated-meta/var/worker-logs/t0318.log 2>&1
