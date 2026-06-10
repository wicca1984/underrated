//! Performance regression test gate for the entire render pipeline.
//! This verifies that parsing, styling, layout, painting, and rasterization
//! of a representative, self-contained HTML/CSS page completes within a
//! reasonable performance budget under a debug build.
//!
//! Rationale for chosen budget:
//! - Debug builds are significantly slower due to the lack of optimizations.
//! - CI environments may run on shared/constrained hardware, introducing variability.
//! - Thus, a generous ceiling of 2000ms is used. This is high enough to avoid
//!   flakiness under constrained CI environments, while still acting as a
//!   dependable backstop against catastrophic regressions (e.g. infinite loops
//!   or exponential layout/selector matching algorithms).
//!
//! // TODO(spec): Since the official performance gate specification is not yet
//! // detailed, this test establishes a 2000ms generous baseline. Future specifications
//! // may refine this budget or establish separate release-build bounds.

use std::time::Instant;
use underrated::engine::render_page_to_canvas;
use underrated::loader::{LoadError, ResourceLoader};
use underrated::url::Url;

struct PerfTestLoader;

impl ResourceLoader for PerfTestLoader {
    fn load(&self, _url: &Url) -> Result<Vec<u8>, LoadError> {
        Err(LoadError::NotFound)
    }
}

fn generate_large_fixture() -> String {
    let mut html = String::new();
    html.push_str("<!DOCTYPE html><html><head><style>");
    html.push_str("
        body { font-size: 16px; margin: 0; padding: 10px; background-color: #fafafa; }
        .container { display: flex; flex-direction: column; width: 800px; }
        .row { display: flex; flex-direction: row; margin: 5px; padding: 5px; border: 1px solid #ccc; }
        .box { width: 100px; height: 50px; background-color: #007bff; margin: 2px; }
        .text-node { font-size: 14px; color: #333; }
        .nested-1 { padding: 4px; background-color: #e2e2e2; }
        .nested-2 { padding: 2px; background-color: #bcbcbc; }
        .nested-3 { display: inline-block; width: 40px; height: 20px; background-color: #28a745; }
        @media (max-width: 1024px) {
            .box { background-color: #dc3545; }
        }
    ");
    html.push_str("</style></head><body>");
    html.push_str("<div class=\"container\">");

    // Programmatically generate ~750 DOM nodes (50 rows * 15 nodes per row)
    // to provide a substantial and representative benchmark workload.
    for i in 0..50 {
        html.push_str(&format!("<div class=\"row\" id=\"row-{}\">", i));
        for j in 0..3 {
            html.push_str(&format!(
                "<div class=\"box nested-1\"><div class=\"nested-2\"><span class=\"nested-3\"></span>Box {}-{}</div></div>",
                i, j
            ));
        }
        html.push_str("<p class=\"text-node\">Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.</p>");
        html.push_str("</div>");
    }

    html.push_str("</div>");
    html.push_str("</body></html>");
    html
}

#[test]
fn test_render_pipeline_performance_gate() {
    let html = generate_large_fixture();
    let base_url = Url::parse("https://example.com/").expect("Failed to parse base URL");
    let loader = PerfTestLoader;

    // Run the pipeline once to warm up any potential lazy static/etc (e.g. fonts).
    // This reduces flakiness due to first-run initialization overhead.
    let _warmup = render_page_to_canvas(&html, &base_url, &loader, 1024, 768);

    // Measure the actual pipeline execution duration
    let start = Instant::now();
    let canvas = render_page_to_canvas(&html, &base_url, &loader, 1024, 768);
    let duration = start.elapsed();

    println!("Full render pipeline took: {:?}", duration);

    // Verify that the output canvas has the expected size
    assert_eq!(canvas.width, 1024);
    assert_eq!(canvas.height, 768);

    // Performance budget: 2000ms
    let budget_ms = 2000;
    let duration_ms = duration.as_millis();

    assert!(
        duration_ms < budget_ms,
        "Performance regression detected! Render pipeline took {}ms, which exceeds the budget of {}ms",
        duration_ms,
        budget_ms
    );
}
