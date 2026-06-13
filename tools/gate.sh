#!/usr/bin/env bash
set -euo pipefail

# Accept an optional target directory to gate
if [ -n "${1:-}" ]; then
  echo "Changing directory to $1..."
  cd "$1"
fi

echo "=== Running cargo fmt ==="
cargo fmt --all --check

echo "=== Running cargo clippy ==="
cargo clippy --all-targets -- -D warnings

echo "=== Running cargo test ==="
cargo test

echo "=== Running cargo doc ==="
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items

echo "=== Running render smoke gate ==="
bash "$(dirname "$0")/smoke.sh"

echo "All gates passed!"
