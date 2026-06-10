//! End-to-end smoke test for form submission navigation and rendering.
//! This verifies MVP milestone MS-MVP completion-condition 3:
//! form submit -> result page fetch -> render.

use std::collections::HashMap;
use underrated::dom::{Dom, NodeData};
use underrated::encoding::InputStream;
use underrated::engine::navigate;
use underrated::forms::{FormState, Method, submit};
use underrated::html::parse_document;
use underrated::infra::NodeId;
use underrated::loader::{HttpMethod, LoadError, LoaderResponse, ResourceLoader};
use underrated::url::Url;

/// A simple mock resource loader for form submission tests.
struct TestMockLoader {
    responses: HashMap<String, Vec<u8>>,
}

impl ResourceLoader for TestMockLoader {
    fn load(&self, _url: &Url) -> Result<Vec<u8>, LoadError> {
        Err(LoadError::NotFound)
    }

    fn load_request(
        &self,
        url: &Url,
        method: HttpMethod,
        body: &[u8],
        _content_type: Option<&str>,
    ) -> Result<LoaderResponse, LoadError> {
        let serialized_url = url.serialize();
        let key = match method {
            HttpMethod::Get => format!("GET:{}", serialized_url),
            HttpMethod::Post => {
                let body_str = String::from_utf8_lossy(body);
                format!("POST:{}:{}", serialized_url, body_str)
            }
        };

        if let Some(bytes) = self.responses.get(&key) {
            Ok(LoaderResponse {
                bytes: bytes.clone(),
                content_type: "text/html".to_string(),
                charset: Some("utf-8".to_string()),
            })
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

#[test]
fn test_get_search_form_submit_nav_smoke() {
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

    // 2. Set the input value to a query carrying spaces to test form-encoding (space -> "+").
    let mut form_state = FormState::new();
    form_state.set_value(input_id, "rust lang");

    // Call forms::submit to produce a NavigationRequest.
    let req = submit(&dom, form_id, &form_state).expect("Form submit failed");

    // Assert the navigation request properties.
    assert_eq!(req.method, Method::Get);
    assert!(req.url.contains("q=rust+lang"), "URL was: {}", req.url);

    // 3. Implement mock responses for the expected search URL.
    let mut responses = HashMap::new();
    let expected_url = "https://example.com/search?q=rust+lang";
    let mock_html = b"<html><body><a id=\"r1\" href=\"/p\">Result One</a></body></html>";
    responses.insert(format!("GET:{}", expected_url), mock_html.to_vec());

    let mock_loader = TestMockLoader { responses };
    let base_url = Url::parse("https://example.com/").expect("Failed to parse base URL");

    // 4. Call engine::navigate and assert the returned page's DOM contains the rendered result.
    let page = navigate(&req, &base_url, &mock_loader, 800.0);

    // Assert that the page's DOM rendered the <a> element from our mock response.
    let a_id = find_element_by_tag(&page.dom, "a")
        .expect("Rendered <a> element not found in navigated page");
    let a_text = page.dom.text_content(a_id);
    assert_eq!(a_text, "Result One");
}

#[test]
fn test_post_search_form_submit_nav_smoke() {
    // 1. Build a POST search form and parse it into a DOM.
    let form_html = r#"
        <form action="/search" method="post">
            <input name="q">
            <input type="submit" value="Search">
        </form>
    "#;
    let input_stream = InputStream::from_utf8(form_html.as_bytes());
    let dom = parse_document(input_stream);

    // Locate form and input control NodeIds.
    let form_id = find_element_by_tag(&dom, "form").expect("Form element not found");
    let input_id = find_input_by_name(&dom, "q").expect("Input element with name='q' not found");

    // 2. Set the input value to a query carrying spaces.
    let mut form_state = FormState::new();
    form_state.set_value(input_id, "rust lang");

    // Call forms::submit to produce a NavigationRequest.
    let req = submit(&dom, form_id, &form_state).expect("Form submit failed");

    // Assert the navigation request properties for POST.
    assert_eq!(req.method, Method::Post);
    assert_eq!(req.url, "/search");
    assert_eq!(req.body, "q=rust+lang");
    assert_eq!(
        req.content_type,
        Some("application/x-www-form-urlencoded".to_string())
    );

    // 3. Implement mock responses for the expected POST search request.
    let mut responses = HashMap::new();
    let expected_url = "https://example.com/search";
    let mock_html = b"<html><body><a id=\"r1\" href=\"/p\">Post Result One</a></body></html>";
    responses.insert(
        format!("POST:{}:q=rust+lang", expected_url),
        mock_html.to_vec(),
    );

    let mock_loader = TestMockLoader { responses };
    let base_url = Url::parse("https://example.com/").expect("Failed to parse base URL");

    // 4. Call engine::navigate and assert the returned page's DOM contains the rendered POST result.
    let page = navigate(&req, &base_url, &mock_loader, 800.0);

    // Assert that the page's DOM rendered the <a> element with POST-specific text content.
    let a_id = find_element_by_tag(&page.dom, "a")
        .expect("Rendered <a> element not found in navigated page");
    let a_text = page.dom.text_content(a_id);
    assert_eq!(a_text, "Post Result One");
}
