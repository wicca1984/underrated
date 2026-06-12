//! Deeply-nested DOM performance/regression test gate for the entire render pipeline.
//! This verifies that parsing, styling, layout, painting, and rasterization of a deeply nested
//! and wide DOM tree completes within a reasonable budget under a debug build.
//!
//! Rationale for chosen budget:
//! - Debug builds are significantly slower due to the lack of optimizations.
//! - CI environments may run on shared/constrained hardware, introducing variability.
//! - Thus, a generous ceiling of 5000ms is used. This is high enough to avoid
//!   flakiness under constrained CI environments, while still acting as a
//!   dependable backstop against catastrophic regressions (e.g. infinite loops,
//!   quadratic layout, selector matching over deeply nested subtrees, or long sibling chains).
//!
//! // TODO(spec): Since the official performance gate specification is not yet
//! // detailed, this test establishes a 5000ms generous baseline. Future specifications
//! // may refine this budget or establish separate release-build bounds.

use std::time::Instant;
use underrated::engine::render_page_to_canvas;
use underrated::loader::{LoadError, ResourceLoader};
use underrated::url::Url;

/// The nesting depth of the DOM hierarchy. We pick a value that is deep enough
/// to exercise style matching, HTML tree construction, and layout recursion guards,
/// without blowing the stack on platforms with a small stack size (e.g., 1MB).
const NESTING_DEPTH: usize = 1200;

/// The number of sibling elements at the leaf level of the hierarchy.
/// This exercises sibling list traversal and layout of wide subtrees.
const SIBLING_COUNT: usize = 2000;

struct PerfTestLoader;

impl ResourceLoader for PerfTestLoader {
    fn load(&self, _url: &Url) -> Result<Vec<u8>, LoadError> {
        Err(LoadError::NotFound)
    }
}

/// Deterministically builds a HTML page with deep block nesting and a wide sibling list
/// at the bottom, and CSS styling to exercise the full render pipeline.
fn generate_deeply_nested_fixture() -> String {
    let mut html = String::new();
    html.push_str("<!DOCTYPE html><html><head><style>");
    html.push_str("
        body { font-size: 16px; margin: 0; padding: 10px; background-color: #f0f0f0; }
        div { margin: 0px; padding: 0px; font-size: 14px; }
        .nest-spaced { margin: 1px; padding: 1px; font-size: 12px; }
        .sibling-leaf { display: inline-block; width: 15px; height: 15px; background-color: #28a745; margin: 1px; }
    ");
    html.push_str("</style></head><body>");

    // Open deeply nested div tags
    for i in 0..NESTING_DEPTH {
        if i % 100 == 0 {
            html.push_str(&format!("<div class=\"nest-spaced\" id=\"div-{}\">", i));
        } else {
            html.push_str(&format!("<div id=\"div-{}\">", i));
        }
    }

    // Leaf sibling list
    html.push_str("<span class=\"sibling-leaf\">Leaf</span>");
    for i in 0..SIBLING_COUNT {
        html.push_str(&format!("<span class=\"sibling-leaf\">s{}</span>", i));
    }

    // Close all opened div tags
    for _ in 0..NESTING_DEPTH {
        html.push_str("</div>");
    }

    html.push_str("</body></html>");
    html
}

#[test]
fn deep_nesting_render_within_budget() {
    let html = generate_deeply_nested_fixture();
    let base_url = Url::parse("https://example.com/").expect("Failed to parse base URL");
    let loader = PerfTestLoader;

    // Run the pipeline once to warm up fonts or other lazy-static initialization.
    // This minimizes variance/flakiness on first-run execution overhead.
    let _warmup = render_page_to_canvas(&html, &base_url, &loader, 1024, 768);

    // Measure the actual pipeline execution duration
    let start = Instant::now();
    let canvas = render_page_to_canvas(&html, &base_url, &loader, 1024, 768);
    let duration = start.elapsed();

    println!("Deep nesting render pipeline took: {:?}", duration);

    // Verify that the output canvas has the expected size and is non-empty
    assert_eq!(canvas.width, 1024);
    assert_eq!(canvas.height, 768);
    assert_eq!(canvas.pixels.len(), (1024 * 768) as usize);

    // Performance budget: 5000ms
    let budget_ms = 5000;
    let duration_ms = duration.as_millis();

    assert!(
        duration_ms < budget_ms,
        "Performance regression detected! Deep nesting render pipeline took {}ms, which exceeds the budget of {}ms",
        duration_ms,
        budget_ms
    );
}
