#!/usr/bin/env bash
set -e

echo "Running oracle snapshot tests..."
cargo test --test oracle_snapshot_test

echo "All tests passed!"
