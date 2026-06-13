#!/usr/bin/env bash
set -euo pipefail

# Locate repo root (directory containing Cargo.toml) relative to the script
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# Helper function to perform float comparisons using awk
compare_float() {
    # Usage: compare_float <val> <op> <limit>
    # Returns 0 (success/true) if the comparison holds, 1 otherwise
    awk -v val="$1" -v op="$2" -v limit="$3" 'BEGIN {
        if (op == ">") {
            if (val > limit) exit 0; else exit 1;
        } else if (op == "<") {
            if (val < limit) exit 0; else exit 1;
        } else if (op == ">=") {
            if (val >= limit) exit 0; else exit 1;
        } else if (op == "<=") {
            if (val <= limit) exit 0; else exit 1;
        }
        exit 2
    }'
}

# Define corpus
fixtures=(
    "tests/oracle/fixtures/01_single_block_text.html"
    "tests/oracle/fixtures/09_wiki_article.html"
    "tests/oracle/fixtures/10_news_article.html"
    "tests/oracle/fixtures/08_google_real.html"
)

for fixture in "${fixtures[@]}"; do
    echo "Rendering $fixture with smoke_render..."
    # If the example run fails, bash set -e will abort immediately
    output=$(cargo run -q --example smoke_render -- "$fixture")

    # Extract metrics
    elements=$(echo "$output" | grep "dom       :" | sed -E 's/.*: ([0-9]+) elements.*/\1/')
    pct=$(echo "$output" | grep "raster    :" | sed -E 's/.*\(([0-9.]+)%\).*/\1/')
    colors=$(echo "$output" | grep "raster    :" | sed -E 's/.*, ([0-9]+) distinct colors.*/\1/')

    if [[ -z "$elements" || -z "$pct" || -z "$colors" ]]; then
        echo "ERROR: Failed to parse stats from smoke_render output for $fixture"
        echo "Output was:"
        echo "$output"
        exit 1
    fi

    # Set thresholds based on fixture
    case "$fixture" in
        "tests/oracle/fixtures/01_single_block_text.html")
            # 01_single_block_text.html observed: 7 elements, 100.0% non-black, 2 colors
            el_floor=3
            pct_floor=50.0
            pct_ceiling=100.1
            color_floor=1
            ;;
        "tests/oracle/fixtures/09_wiki_article.html")
            # 09_wiki_article.html observed: 29 elements, 98.6% non-black, 5 colors
            el_floor=14
            pct_floor=50.0
            pct_ceiling=99.5
            color_floor=2
            ;;
        "tests/oracle/fixtures/10_news_article.html")
            # 10_news_article.html observed: 22 elements, 99.1% non-black, 4 colors
            el_floor=11
            pct_floor=50.0
            pct_ceiling=99.5
            color_floor=2
            ;;
        "tests/oracle/fixtures/08_google_real.html")
            # 08_google_real.html observed: 83 elements, 99.9% non-black, 240 colors
            el_floor=41
            pct_floor=50.0
            pct_ceiling=99.95
            color_floor=120
            ;;
        *)
            echo "Unknown fixture: $fixture"
            exit 1
            ;;
    esac

    # Perform comparisons
    # DOM elements: must be > floor
    if ! compare_float "$elements" ">" "$el_floor"; then
        echo "FAIL: $fixture elements threshold violated (observed: $elements, floor: $el_floor)"
        exit 1
    fi
    echo "  PASS: elements ($elements > $el_floor)"

    # non-black %: must be within a band: > floor AND < ceiling
    if ! compare_float "$pct" ">" "$pct_floor"; then
        echo "FAIL: $fixture non-black % floor threshold violated (observed: $pct%, floor: $pct_floor%)"
        exit 1
    fi
    if ! compare_float "$pct" "<" "$pct_ceiling"; then
        echo "FAIL: $fixture non-black % ceiling threshold violated (observed: $pct%, ceiling: $pct_ceiling%)"
        exit 1
    fi
    echo "  PASS: non-black % ($pct_floor% < $pct% < $pct_ceiling%)"

    # distinct colors: must be > floor
    if ! compare_float "$colors" ">" "$color_floor"; then
        echo "FAIL: $fixture distinct colors threshold violated (observed: $colors, floor: $color_floor)"
        exit 1
    fi
    echo "  PASS: distinct colors ($colors > $color_floor)"
done

echo "SMOKE GATE PASSED"
exit 0
