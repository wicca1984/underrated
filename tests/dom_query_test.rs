use underrated::dom::{Dom, NodeData};
use underrated::infra::NodeId;

fn setup_dom() -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.document();

    // <html>
    let html = dom.create_node(NodeData::Element {
        name: "html".into(),
        attrs: vec![],
    });
    dom.append_child(doc, html);

    //   <body class="main">
    let body = dom.create_node(NodeData::Element {
        name: "body".into(),
        attrs: vec![("class".into(), "main".into())],
    });
    dom.append_child(html, body);

    //     <div id="container" class="foo bar">
    let div = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![
            ("id".into(), "container".into()),
            ("class".into(), "foo bar".into()),
        ],
    });
    dom.append_child(body, div);

    //       <p class="text" id="p1">Hello</p>
    let p1 = dom.create_node(NodeData::Element {
        name: "p".into(),
        attrs: vec![("class".into(), "text".into()), ("id".into(), "p1".into())],
    });
    dom.append_child(div, p1);

    //       <span class="text">World</span>
    let span = dom.create_node(NodeData::Element {
        name: "span".into(),
        attrs: vec![("class".into(), "text".into())],
    });
    dom.append_child(div, span);

    (dom, div)
}

#[test]
fn test_get_element_by_id() {
    let (dom, _div) = setup_dom();

    assert!(dom.get_element_by_id("container").is_some());
    assert!(dom.get_element_by_id("p1").is_some());
    assert!(dom.get_element_by_id("non-existent").is_none());

    let container = dom.get_element_by_id("container").unwrap();
    if let Some(NodeData::Element { name, .. }) = dom.data(container) {
        assert_eq!(name, "div");
    } else {
        panic!("Should be an element");
    }
}

#[test]
fn test_query_selector() {
    let (dom, _div) = setup_dom();
    let doc = dom.document();

    assert!(dom.query_selector(doc, "#container").is_some());
    assert!(dom.query_selector(doc, ".text").is_some());
    assert!(dom.query_selector(doc, "div > p").is_some());
    assert!(dom.query_selector(doc, "section").is_none());
    assert!(dom.query_selector(doc, "invalid selector").is_none());

    let first_text = dom.query_selector(doc, ".text").unwrap();
    if let Some(NodeData::Element { name, attrs }) = dom.data(first_text) {
        assert_eq!(name, "p");
        assert!(attrs.iter().any(|(n, v)| n == "id" && v == "p1"));
    } else {
        panic!("Should be an element");
    }
}

#[test]
fn test_query_selector_all() {
    let (dom, _div) = setup_dom();
    let doc = dom.document();

    let texts = dom.query_selector_all(doc, ".text");
    assert_eq!(texts.len(), 2);

    // Document order: p then span
    if let Some(NodeData::Element { name: name1, .. }) = dom.data(texts[0]) {
        assert_eq!(name1, "p");
    }
    if let Some(NodeData::Element { name: name2, .. }) = dom.data(texts[1]) {
        assert_eq!(name2, "span");
    }

    assert_eq!(dom.query_selector_all(doc, "section").len(), 0);
    assert_eq!(dom.query_selector_all(doc, "invalid selector").len(), 0);
}

#[test]
fn test_subtree_query_selector() {
    let (dom, div) = setup_dom();

    // Querying from div (root is div)
    // The descendant with class .text should be found.
    let first_text_under_div = dom.query_selector(div, ".text").unwrap();
    if let Some(NodeData::Element { name, attrs }) = dom.data(first_text_under_div) {
        assert_eq!(name, "p");
        assert!(attrs.iter().any(|(n, v)| n == "id" && v == "p1"));
    } else {
        panic!("Should be an element");
    }

    // Since 'div' itself is not a descendant of 'div' (descendants excludes root),
    // querying for '#container' from div root should return None.
    assert!(dom.query_selector(div, "#container").is_none());
}

#[test]
fn test_acceptance_cases() {
    let mut dom = Dom::new();
    let doc = dom.document();

    // Setup elements for S-67 acceptance: 'div.x > p' and querySelectorAll('a')
    let div_x = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("class".into(), "x".into())],
    });
    dom.append_child(doc, div_x);

    let p = dom.create_node(NodeData::Element {
        name: "p".into(),
        attrs: vec![],
    });
    dom.append_child(div_x, p);

    let a1 = dom.create_node(NodeData::Element {
        name: "a".into(),
        attrs: vec![("href".into(), "url1".into())],
    });
    dom.append_child(p, a1);

    let a2 = dom.create_node(NodeData::Element {
        name: "a".into(),
        attrs: vec![("href".into(), "url2".into())],
    });
    dom.append_child(div_x, a2);

    // 1. querySelector('div.x > p') matches the p node
    let matched_p = dom.query_selector(doc, "div.x > p");
    assert_eq!(matched_p, Some(p));

    // 2. querySelectorAll('a') returns both anchors in order
    let anchors = dom.query_selector_all(doc, "a");
    assert_eq!(anchors, vec![a1, a2]);

    // 3. invalid / empty selectors return None / []
    assert_eq!(dom.query_selector(doc, ""), None);
    assert_eq!(dom.query_selector(doc, "div > > p"), None);
    assert_eq!(dom.query_selector_all(doc, "").len(), 0);
    assert_eq!(dom.query_selector_all(doc, "div > > p").len(), 0);
}
