use underrated::css::values::CssValue;
use underrated::css::values::LengthUnit;

#[test]
fn test_ua_default_stylesheet_margins() {
    let base_url = underrated::url::Url::parse("http://localhost/").unwrap();

    struct DummyLoader;
    impl underrated::loader::ResourceLoader for DummyLoader {
        fn load(
            &self,
            _url: &underrated::url::Url,
        ) -> Result<Vec<u8>, underrated::loader::LoadError> {
            Err(underrated::loader::LoadError::NotFound)
        }
        fn load_request(
            &self,
            _url: &underrated::url::Url,
            _method: underrated::loader::HttpMethod,
            _body: &[u8],
            _content_type: Option<&str>,
        ) -> Result<underrated::loader::LoaderResponse, underrated::loader::LoadError> {
            Err(underrated::loader::LoadError::NotFound)
        }
    }

    let html = "<p>Hello World</p>";
    let page = underrated::engine::render_page(html, &base_url, &DummyLoader, 800.0);

    // Assert the <p> resolved style margin-top is a Length of 1.0 em.
    let doc = page.dom.document();
    let mut p_style = None;
    for id in page.dom.descendants(doc) {
        if let Some(underrated::dom::NodeData::Element { name, .. }) = page.dom.data(id)
            && name == "p"
        {
            p_style = page.styles.get(&id);
            break;
        }
    }

    let p_s = p_style.expect("p should have styles");
    assert_eq!(
        p_s.get("margin-top"),
        Some(&CssValue::Length(1.0, LengthUnit::Em))
    );
}

#[test]
fn test_ua_default_stylesheet_list_indentation() {
    let base_url = underrated::url::Url::parse("http://localhost/").unwrap();

    struct DummyLoader;
    impl underrated::loader::ResourceLoader for DummyLoader {
        fn load(
            &self,
            _url: &underrated::url::Url,
        ) -> Result<Vec<u8>, underrated::loader::LoadError> {
            Err(underrated::loader::LoadError::NotFound)
        }
        fn load_request(
            &self,
            _url: &underrated::url::Url,
            _method: underrated::loader::HttpMethod,
            _body: &[u8],
            _content_type: Option<&str>,
        ) -> Result<underrated::loader::LoaderResponse, underrated::loader::LoadError> {
            Err(underrated::loader::LoadError::NotFound)
        }
    }

    let html = "<body style=\"margin: 0;\"><ul><li>First</li></ul></body>";
    let page = underrated::engine::render_page(html, &base_url, &DummyLoader, 800.0);

    // Recursively find the <li> layout box and assert its left edge x is 40.0
    fn find_layout_box_by_tag<'a>(
        layout: &'a underrated::layout::LayoutBox,
        dom: &underrated::dom::Dom,
        tag: &str,
    ) -> Option<&'a underrated::layout::LayoutBox> {
        if let Some(node_id) = layout.node
            && let Some(underrated::dom::NodeData::Element { name, .. }) = dom.data(node_id)
            && name == tag
        {
            return Some(layout);
        }
        for child in &layout.children {
            if let Some(found) = find_layout_box_by_tag(child, dom, tag) {
                return Some(found);
            }
        }
        None
    }

    let li_box = find_layout_box_by_tag(&page.layout, &page.dom, "li")
        .expect("li element should have a layout box");

    assert_eq!(li_box.rect.origin.x, 40.0);

    // Also assert the <ul> itself receives the UA margin: 1em 0 rule.
    let doc = page.dom.document();
    let mut ul_style = None;
    for id in page.dom.descendants(doc) {
        if let Some(underrated::dom::NodeData::Element { name, .. }) = page.dom.data(id)
            && name == "ul"
        {
            ul_style = page.styles.get(&id);
            break;
        }
    }
    let ul_s = ul_style.expect("ul should have styles");
    assert_eq!(
        ul_s.get("margin-top"),
        Some(&CssValue::Length(1.0, LengthUnit::Em))
    );
    assert_eq!(
        ul_s.get("margin-bottom"),
        Some(&CssValue::Length(1.0, LengthUnit::Em))
    );
}

#[test]
fn test_ua_default_stylesheet_th() {
    let base_url = underrated::url::Url::parse("http://localhost/").unwrap();

    struct DummyLoader;
    impl underrated::loader::ResourceLoader for DummyLoader {
        fn load(
            &self,
            _url: &underrated::url::Url,
        ) -> Result<Vec<u8>, underrated::loader::LoadError> {
            Err(underrated::loader::LoadError::NotFound)
        }
        fn load_request(
            &self,
            _url: &underrated::url::Url,
            _method: underrated::loader::HttpMethod,
            _body: &[u8],
            _content_type: Option<&str>,
        ) -> Result<underrated::loader::LoaderResponse, underrated::loader::LoadError> {
            Err(underrated::loader::LoadError::NotFound)
        }
    }

    let html = "<table><tr><th>Header</th></tr></table>";
    let page = underrated::engine::render_page(html, &base_url, &DummyLoader, 800.0);

    let doc = page.dom.document();
    let mut th_style = None;
    for id in page.dom.descendants(doc) {
        if let Some(underrated::dom::NodeData::Element { name, .. }) = page.dom.data(id)
            && name == "th"
        {
            th_style = page.styles.get(&id);
            break;
        }
    }

    let th_s = th_style.expect("th should have styles");
    assert_eq!(
        th_s.get("font-weight"),
        Some(&CssValue::Keyword("bold".to_string()))
    );
    assert_eq!(
        th_s.get("text-align"),
        Some(&CssValue::Keyword("center".to_string()))
    );
}
