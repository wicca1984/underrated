#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0328
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

Task: t0328 (milestone MS-AI-Proxy, SPEC S-97). Create a NEW same-crate module \`src/bff/\` that fixes the SECURITY BOUNDARY for a browser-internal 'local BFF' (Backend-For-Frontend): the browser main process injects provider credentials and proxies requests to external AI services, and a web page NEVER receives secret material. THIS SLICE IS A SECURITY-NEUTRAL SCAFFOLD ONLY: NO real credential storage, NO real network egress. Real backends are explicitly deferred with \`// TODO(spec):\` markers. Do NOT add ANY new third-party dependency (no HTTP client, no keyring crate) — I-1 self-implement boundary and this slice does no I/O.

CREATE exactly: \`src/bff/mod.rs\`, and register it with ONE new line in \`src/lib.rs\`: \`pub mod bff;\` inserted in alphabetical order (between \`pub mod ascii;\` and \`pub mod css;\`).

PUBLIC IF — implement EXACTLY these (this is the blessed S-97 surface; do not add/rename public items):
- \`pub struct AiRequest { pub provider: String, pub path: String, pub body: Vec<u8> }\` — opaque page-supplied payload; contains NO secret.
- \`pub struct AiResponse { pub status: u16, pub body: Vec<u8> }\` — external AI response; contains NO credential.
- \`pub enum BffError { UnknownProvider(String), MissingCredential(String), NotImplemented }\` (derive Debug; PartialEq is fine).
- \`pub trait SecretStore { fn credential(&self, provider: &str) -> Option<String>; }\` — abstracts WHERE credentials live. Document that the real backend (OS secure store) is deferred: \`// TODO(spec): real secure-store backend (OS keychain) — not in this scaffold\`.
- \`pub struct InMemorySecretStore\` implementing \`SecretStore\`, holding a map of provider->credential, with a constructor to insert entries. Document loudly that it is for tests/scaffolding ONLY and must not hold production secrets.
- \`pub struct LocalBff<S: SecretStore>\` with \`pub fn new(store: S) -> Self\` and \`pub fn forward(&self, req: &AiRequest) -> Result<AiResponse, BffError>\`.

\`forward\` BEHAVIOR (no I/O):
1. Determine whether \`req.provider\` is a known/allowed provider. Define a small fixed allow-list of known provider ids as a private const (e.g. \`\"anthropic\"\`). If unknown -> \`Err(BffError::UnknownProvider(req.provider.clone()))\`.
2. Look up the credential via \`self.store.credential(&req.provider)\`. If \`None\` -> \`Err(BffError::MissingCredential(req.provider.clone()))\`.
3. If both present: the credential WOULD be injected into the outbound request here, but real egress is not wired. Leave a \`// TODO(spec): inject credential into outbound request and perform egress to the external AI service\` and return \`Err(BffError::NotImplemented)\`.
CRITICAL SECURITY INVARIANT: the credential String must NEVER be placed into the returned value, into \`AiResponse\`, into \`BffError\`, or logged. \`forward\` must not expose secrets to the caller.

TESTS (in \`src/bff/mod.rs\` under \`#[cfg(test)] mod tests\`): cover the four invariants — (1) unknown provider -> \`UnknownProvider\`; (2) known provider but no credential in the store -> \`MissingCredential\`; (3) known provider WITH a credential present -> \`NotImplemented\` (egress stub); (4) assert the boundary: after a successful-credential \`forward\` returning an error, the returned \`BffError\` value's Debug string does NOT contain the secret credential value (use a recognizable secret like \"SECRET-XYZ\" and assert the formatted error/result does not contain it).

SCOPE — STRICT. \`git diff --name-only origin/main...HEAD\` MUST list ONLY \`src/bff/mod.rs\` and \`src/lib.rs\` (the single registration line). Do NOT touch any other file, do NOT touch net/, engine/, loader/, script/. Do NOT modify, delete, weaken, rename, or \`#[ignore]\` ANY existing test anywhere. Do NOT add dependencies to Cargo.toml. If you believe you must do any of these, STOP, leave a \`// TODO(spec):\` note, and report the blocker — do not expand scope.

GATES (all must pass before commit):
- \`cargo test\` (entire suite) green.
- \`cargo clippy --all-targets -- -D warnings\` clean (NO \`unwrap\`/\`expect\` in non-test production code — I-6).
- \`cargo fmt\` then \`cargo fmt --check\` clean.
- \`cargo doc --no-deps\` clean (all public items must have doc comments).

This task is NOT UI/rendering — NO verified-in-window PNG is required.

COMMIT: after ALL gates pass, \`git add -A\` then \`git commit -m \"feat(bff): scaffold local AI-proxy module with credential-boundary types (t0328)\"\`. Then run \`git diff --name-only origin/main...HEAD\` and confirm ONLY \`src/bff/mod.rs\` and \`src/lib.rs\` are listed. Do NOT push and do NOT open a PR — the orchestrator reviews, gates, and merges. Report a concise summary: the public items created, how the security invariant is enforced/tested, and gate results." \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta
