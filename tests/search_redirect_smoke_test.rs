//! End-to-end integration test for search form submission with redirects.
//! This verifies MVP milestone MS-MVP completion-condition 3 with redirects:
//! form submit -> HTTP 302 redirect -> result page fetch -> DOM build -> render -> raster.

use std::collections::HashMap;
use underrated::dom::{Dom, NodeData};
use underrated::encoding::InputStream;
use underrated::engine::navigate;
use underrated::forms::{FormState, Method, submit};
use underrated::html::parse_document;
use underrated::infra::NodeId;
use underrated::loader::{HttpMethod, LoadError, LoaderResponse, RedirectMeta, ResourceLoader};
use underrated::url::Url;

/// A mock resource loader specifically designed to test redirect-following behavior.
struct RedirectMockLoader {
    responses: HashMap<String, (RedirectMeta, LoaderResponse)>,
}

impl ResourceLoader for RedirectMockLoader {
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

    fn load_request_hop(
        &self,
        url: &Url,
        _method: HttpMethod,
        _body: &[u8],
        _content_type: Option<&str>,
    ) -> Result<(RedirectMeta, LoaderResponse), LoadError> {
        let serialized_url = url.serialize();
        if let Some(entry) = self.responses.get(&serialized_url) {
            Ok(entry.clone())
        } else {
            Err(LoadError::NotFound)
        }
    }
}

/// Helper function to locate a node by its HTML element tag name.
fn find_element_by_tag(dom: &Dom, tag_name: &str) -> Option<NodeId> {
    let doc = dom.document();
    for node_id in dom.descendants(doc) {
        if let Some(NodeData::Element { name, .. }) = dom.data(node_id)
            && name.eq_ignore_ascii_case(tag_name)
        {
            return Some(node_id);
        }
    }
    None
}

/// Helper function to locate an input element by its "name" attribute value.
fn find_input_by_name(dom: &Dom, name_attr: &str) -> Option<NodeId> {
    let doc = dom.document();
    for node_id in dom.descendants(doc) {
        if let Some(NodeData::Element { name, attrs }) = dom.data(node_id)
            && name.eq_ignore_ascii_case("input")
            && attrs
                .iter()
                .any(|(k, v)| k.eq_ignore_ascii_case("name") && v == name_attr)
        {
            return Some(node_id);
        }
    }
    None
}

/// Verification for MS-MVP milestone acceptance gate:
/// HTML form submit -> NavigationRequest -> engine::navigate following an HTTP 302 redirect ->
/// final 200 result page DOM build -> render/raster.
#[test]
fn test_search_submit_redirect_result_smoke() {
    // 1. Build a GET search form and parse it into a DOM.
    let form_html = r#"
        <form action="/search" method="get">
            <input name="q">
            <input type="submit" value="Search">
        </form>
    "#;
    let input_stream = InputStream::from_utf8(form_html.as_bytes());
    let dom = parse_document(input_stream);

    // Locate form and input control NodeIds.
    let form_id = find_element_by_tag(&dom, "form").expect("Form element not found");
    let input_id = find_input_by_name(&dom, "q").expect("Input element with name='q' not found");

    // 2. Set q to a value carrying spaces to verify correct percent-encoding (space -> "+").
    let mut form_state = FormState::new();
    form_state.set_value(input_id, "rust lang");

    // Call forms::submit to produce a NavigationRequest.
    let req = submit(&dom, form_id, &form_state).expect("Form submit failed");

    // Assert the navigation request properties for GET.
    assert_eq!(req.method, Method::Get);
    assert!(req.url.contains("q=rust+lang"), "URL was: {}", req.url);

    // 3. Implement mock loader with two mapped hops.
    let search_url = "https://example.com/search?q=rust+lang";
    let results_url = "https://example.com/results?q=rust+lang";

    let hop1_meta = RedirectMeta {
        status: 302,
        location: Some(results_url.to_string()),
    };
    let hop1_resp = LoaderResponse {
        bytes: b"Redirecting...".to_vec(),
        content_type: "text/html".to_string(),
        charset: Some("utf-8".to_string()),
    };

    let hop2_meta = RedirectMeta {
        status: 200,
        location: None,
    };
    let hop2_resp = LoaderResponse {
        bytes: b"<html><body><a id=\"r1\" href=\"/p\">Result One</a></body></html>".to_vec(),
        content_type: "text/html".to_string(),
        charset: Some("utf-8".to_string()),
    };

    let mut responses = HashMap::new();
    responses.insert(search_url.to_string(), (hop1_meta, hop1_resp));
    responses.insert(results_url.to_string(), (hop2_meta, hop2_resp));

    let mock_loader = RedirectMockLoader { responses };
    let base_url = Url::parse("https://example.com/").expect("Failed to parse base URL");

    // 4. Call engine::navigate. Because the first hop is a 302, navigate MUST transparently
    // follow the redirect to the results page.
    let page = navigate(&req, &base_url, &mock_loader, 800.0);

    // 5. Assert the final result page was built from hop 2 (NOT the redirect placeholder):
    // Find the <a> and assert its text_content == "Result One".
    let a_id = find_element_by_tag(&page.dom, "a")
        .expect("Rendered <a> element not found in navigated page");
    let a_text = page.dom.text_content(a_id);
    assert_eq!(a_text, "Result One");

    // 6. Paint-level (raster) assertions mirroring the template.
    // Build display list and rasterize the navigated page.
    let display_list = underrated::paint::build_display_list(&page.layout, &page.dom, &page.styles);
    let canvas = underrated::raster::rasterize(&display_list, 800, 600);

    // Assert that the canvas contains more than one distinct color (the result actually painted).
    let mut unique_colors = std::collections::HashSet::new();
    for &pixel in &canvas.pixels {
        unique_colors.insert(pixel);
    }
    assert!(
        unique_colors.len() > 1,
        "Canvas must contain more than one distinct color, but found only {}",
        unique_colors.len()
    );
}
