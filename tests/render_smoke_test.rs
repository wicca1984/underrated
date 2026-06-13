use std::collections::BTreeSet;
use underrated::dom::NodeData;
use underrated::engine;
use underrated::loader::{HttpMethod, LoadError, LoaderResponse, ResourceLoader};
use underrated::url::Url;

struct DummyLoader;

impl ResourceLoader for DummyLoader {
    fn load(&self, _url: &Url) -> Result<Vec<u8>, LoadError> {
        Err(LoadError::NotFound)
    }

    fn load_request(
        &self,
        _url: &Url,
        _method: HttpMethod,
        _body: &[u8],
        _content_type: Option<&str>,
    ) -> Result<LoaderResponse, LoadError> {
        Err(LoadError::NotFound)
    }
}

fn run_fixture(fixture_name: &str) -> Result<(usize, f64, usize), String> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let file_path = format!("{}/tests/oracle/fixtures/{}", manifest_dir, fixture_name);
    let html = std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read fixture {}: {}", file_path, e))?;

    let url =
        Url::parse("https://example.com/").map_err(|e| format!("Failed to parse URL: {:?}", e))?;
    let loader = DummyLoader;

    let page = engine::render_page(&html, &url, &loader, 800.0);
    let doc = page.dom.document();
    let mut element_count = 0;
    for id in page.dom.descendants(doc) {
        if let Some(NodeData::Element { .. }) = page.dom.data(id) {
            element_count += 1;
        }
    }

    let canvas = engine::render_page_to_canvas(&html, &url, &loader, 800, 600);
    let total = canvas.pixels.len();
    if total == 0 {
        return Err("Canvas pixels cannot be empty".to_string());
    }

    let non_black_count = canvas
        .pixels
        .iter()
        .filter(|&&p| (p & 0x00FF_FFFF) != 0)
        .count();
    let ratio = non_black_count as f64 / total as f64;

    let distinct_colors: BTreeSet<u32> = canvas.pixels.iter().map(|&p| p & 0x00FF_FFFF).collect();

    eprintln!(
        "FIXTURE: {}\n  observed elements: {}\n  observed non-black ratio: {:.6} ({} / {})\n  observed distinct colors: {}\n",
        fixture_name,
        element_count,
        ratio,
        non_black_count,
        total,
        distinct_colors.len()
    );

    Ok((element_count, ratio, distinct_colors.len()))
}

#[test]
fn test_smoke_google_mock() {
    let (elements, ratio, colors) =
        run_fixture("07_google_mock.html").expect("Failed to render 07_google_mock.html");

    // observed healthy: 11 elements (2026-06-13); floor at 5
    assert!(
        elements >= 5,
        "elements count too low: {} (expected >= 5)",
        elements
    );

    assert!(ratio > 0.0, "ratio must be strictly > 0.0 (got {})", ratio);
    assert!(ratio < 1.0, "ratio must be strictly < 1.0 (got {})", ratio);

    // observed healthy: 7 distinct colors (2026-06-13); floor at 3
    assert!(
        colors >= 3,
        "distinct color count too low: {} (expected >= 3)",
        colors
    );
}

#[test]
fn test_smoke_wiki_article() {
    let (elements, ratio, colors) =
        run_fixture("09_wiki_article.html").expect("Failed to render 09_wiki_article.html");

    // observed healthy: 29 elements (2026-06-13); floor at 14
    assert!(
        elements >= 14,
        "elements count too low: {} (expected >= 14)",
        elements
    );

    assert!(ratio > 0.0, "ratio must be strictly > 0.0 (got {})", ratio);
    assert!(ratio < 1.0, "ratio must be strictly < 1.0 (got {})", ratio);

    // observed healthy: 5 distinct colors (2026-06-13); floor at 2
    assert!(
        colors >= 2,
        "distinct color count too low: {} (expected >= 2)",
        colors
    );
}

#[test]
fn test_smoke_news_article() {
    let (elements, ratio, colors) =
        run_fixture("10_news_article.html").expect("Failed to render 10_news_article.html");

    // observed healthy: 22 elements (2026-06-13); floor at 11
    assert!(
        elements >= 11,
        "elements count too low: {} (expected >= 11)",
        elements
    );

    assert!(ratio > 0.0, "ratio must be strictly > 0.0 (got {})", ratio);
    assert!(ratio < 1.0, "ratio must be strictly < 1.0 (got {})", ratio);

    // observed healthy: 4 distinct colors (2026-06-13); floor at 2
    assert!(
        colors >= 2,
        "distinct color count too low: {} (expected >= 2)",
        colors
    );
}
