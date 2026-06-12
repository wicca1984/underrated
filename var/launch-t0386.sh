#!/usr/bin/env bash
set -euo pipefail
cd /workspaces/wt/t0386
LOG=/workspaces/toy-browser/var/log/t0386.log
mkdir -p /workspaces/toy-browser/var/log
read -r -d '' PROMPT <<'EOF' || true
EXECUTE IMMEDIATELY. Do not ask for confirmation. Do not stop at a plan. Implement, build, test, then commit.

Task t0386 — expand the charset transport-label table in `sniff_charset` to recognize the full set of WHATWG encoding labels/aliases that map onto the four existing `Charset` variants. Touch ONLY files under src/encoding/. Do NOT edit html/, dom/, paint/, layout/, style/, engine/, css/, main.rs, or any other module. If something genuinely requires another module, leave a `// TODO(spec): ...` comment in src/encoding/charset.rs and stop.

Background (read before coding):
- Read src/encoding/charset.rs. `pub fn sniff_charset(bytes, transport_label)` has a `match label.to_ascii_lowercase().as_str()` that currently only handles "utf-8", "utf-16le", "utf-16be", "windows-1252", with a `// TODO(spec): Full label table` fallthrough.
- The `Charset` enum has exactly four variants: Utf8, Utf16Le, Utf16Be, Windows1252. You are NOT adding new variants or new decoders — only mapping more aliases onto these four.
- Reference: the WHATWG Encoding Standard "encodings" label table (https://encoding.spec.whatwg.org/#concept-encoding-get). Map the labels for the relevant encodings to the closest of the four supported variants. windows-1252 is the practical superset used for legacy/ascii/latin1.

Implement (minimal, idiomatic, matching surrounding code) in src/encoding/charset.rs:
1. Replace the small match arms with a comprehensive ASCII-case-insensitive, whitespace-trimmed label lookup. Group labels by target variant. At minimum cover:
   - Charset::Utf8 ← "utf-8", "utf8", "unicode-1-1-utf-8", "unicode11utf8", "unicode20utf8", "x-unicode20utf8"
   - Charset::Utf16Le ← "utf-16le", "utf-16", "utf-16le", "csunicode", "iso-10646-ucs-2", "ucs-2", "unicode", "unicodefeff"  (note: WHATWG maps the bare "utf-16" label to UTF-16LE)
   - Charset::Utf16Be ← "utf-16be", "unicodefffe"
   - Charset::Windows1252 ← "windows-1252", "ansi_x3.4-1968", "ascii", "us-ascii", "iso-8859-1", "iso8859-1", "iso_8859-1", "latin1", "l1", "cp1252", "cp819", "ibm819", "x-cp1252", "us-ascii"
   Trim surrounding ASCII whitespace from the label before matching (per spec, leading/trailing whitespace is stripped).
2. Keep the BOM sniffing, meta prescan, and default-Windows1252 fallback exactly as-is. Only the transport-label arm changes. Leave a narrower `// TODO(spec):` noting that non-UTF/non-1252 legacy encodings (e.g. shift_jis, euc-jp, gbk) are decoded as windows-1252 because no dedicated decoder exists yet.
3. Panic-free (AGENTS.md I-6): no unwrap()/expect()/panicking indexing in non-test code.

Add unit tests in the existing `#[cfg(test)] mod tests` block in src/encoding/charset.rs (copy the existing test pattern of calling `sniff_charset(b"...", Some("label"))`):
- `test_label_utf8_aliases`: "UTF8" and "unicode-1-1-utf-8" both -> Charset::Utf8 (assert case-insensitivity with mixed case).
- `test_label_latin1_alias`: "iso-8859-1" and "latin1" -> Charset::Windows1252.
- `test_label_ascii_alias`: "us-ascii" -> Charset::Windows1252.
- `test_label_utf16_bare`: "utf-16" -> Charset::Utf16Le.
- `test_label_whitespace_trimmed`: "  utf-8  " -> Charset::Utf8.
- `test_label_unknown_falls_through_to_default`: an unknown label with no meta and no BOM -> Charset::Windows1252.

When done: run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`. If all green, commit:
  git add -A && git commit -m "feat(encoding): expand WHATWG charset label aliases in sniff_charset (t0386)"
Then print "T0386 DONE" as the last line.
EOF
exec setsid gemini -p "$PROMPT" -m gemini-3.5-flash --approval-mode yolo -o stream-json \
  --include-directories /workspaces/underrated-meta >>"$LOG" 2>&1
