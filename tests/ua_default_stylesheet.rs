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

#[test]
fn test_ua_default_stylesheet_text_semantics() {
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

    let html = "\
        <s>s-text</s> \
        <strike>strike-text</strike> \
        <del>del-text</del> \
        <u>u-text</u> \
        <ins>ins-text</ins> \
        <mark>mark-text</mark> \
        <center>center-text</center> \
        <address>address-text</address>\
    ";
    let page = underrated::engine::render_page(html, &base_url, &DummyLoader, 800.0);

    let doc = page.dom.document();
    let mut s_style = None;
    let mut strike_style = None;
    let mut del_style = None;
    let mut u_style = None;
    let mut ins_style = None;
    let mut mark_style = None;
    let mut center_style = None;
    let mut address_style = None;

    for id in page.dom.descendants(doc) {
        if let Some(underrated::dom::NodeData::Element { name, .. }) = page.dom.data(id) {
            match name.as_str() {
                "s" => s_style = Some(page.styles.get(&id).expect("s should have styles")),
                "strike" => {
                    strike_style = Some(page.styles.get(&id).expect("strike should have styles"))
                }
                "del" => del_style = Some(page.styles.get(&id).expect("del should have styles")),
                "u" => u_style = Some(page.styles.get(&id).expect("u should have styles")),
                "ins" => ins_style = Some(page.styles.get(&id).expect("ins should have styles")),
                "mark" => mark_style = Some(page.styles.get(&id).expect("mark should have styles")),
                "center" => {
                    center_style = Some(page.styles.get(&id).expect("center should have styles"))
                }
                "address" => {
                    address_style = Some(page.styles.get(&id).expect("address should have styles"))
                }
                _ => {}
            }
        }
    }

    let s_s = s_style.expect("s element should be found");
    assert_eq!(
        s_s.get("text-decoration"),
        Some(&CssValue::Keyword("line-through".to_string()))
    );

    let strike_s = strike_style.expect("strike element should be found");
    assert_eq!(
        strike_s.get("text-decoration"),
        Some(&CssValue::Keyword("line-through".to_string()))
    );

    let del_s = del_style.expect("del element should be found");
    assert_eq!(
        del_s.get("text-decoration"),
        Some(&CssValue::Keyword("line-through".to_string()))
    );

    let u_s = u_style.expect("u element should be found");
    assert_eq!(
        u_s.get("text-decoration"),
        Some(&CssValue::Keyword("underline".to_string()))
    );

    let ins_s = ins_style.expect("ins element should be found");
    assert_eq!(
        ins_s.get("text-decoration"),
        Some(&CssValue::Keyword("underline".to_string()))
    );

    let mark_s = mark_style.expect("mark element should be found");
    assert_eq!(
        mark_s.get("background-color"),
        Some(&CssValue::Color(underrated::css::values::Color::Rgba(
            255, 255, 0, 255
        )))
    );
    assert_eq!(
        mark_s.get("color"),
        Some(&CssValue::Color(underrated::css::values::Color::Rgba(
            0, 0, 0, 255
        )))
    );

    let center_s = center_style.expect("center element should be found");
    assert_eq!(
        center_s.get("text-align"),
        Some(&CssValue::Keyword("center".to_string()))
    );

    let address_s = address_style.expect("address element should be found");
    assert_eq!(
        address_s.get("display"),
        Some(&CssValue::Keyword("block".to_string()))
    );
    assert_eq!(
        address_s.get("font-style"),
        Some(&CssValue::Keyword("italic".to_string()))
    );
}
