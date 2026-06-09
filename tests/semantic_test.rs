use underrated::dom::{Dom, NodeData};
use underrated::semantic::{SemanticNode, accessible_name, build_semantic_view, role, to_markdown};

#[test]
fn test_semantic_view_and_markdown() {
    let mut dom = Dom::new();
    let doc = dom.document();

    // <h1>Title</h1>
    let h1 = dom.create_node(NodeData::Element {
        name: "h1".into(),
        attrs: vec![],
    });
    let h1_text = dom.create_node(NodeData::Text("Title".into()));
    dom.append_child(h1, h1_text);
    dom.append_child(doc, h1);

    // <p>A p containing a <a href="https://example.com">link</a>.</p>
    let p = dom.create_node(NodeData::Element {
        name: "p".into(),
        attrs: vec![],
    });
    let p_text1 = dom.create_node(NodeData::Text("A p containing a ".into()));
    let a = dom.create_node(NodeData::Element {
        name: "a".into(),
        attrs: vec![("href".into(), "https://example.com".into())],
    });
    let a_text = dom.create_node(NodeData::Text("link".into()));
    let p_text2 = dom.create_node(NodeData::Text(".".into()));
    dom.append_child(a, a_text);
    dom.append_child(p, p_text1);
    dom.append_child(p, a);
    dom.append_child(p, p_text2);
    dom.append_child(doc, p);

    // <ul>
    //   <li>item 1</li>
    //   <li>item 2</li>
    // </ul>
    let ul = dom.create_node(NodeData::Element {
        name: "ul".into(),
        attrs: vec![],
    });
    let li1 = dom.create_node(NodeData::Element {
        name: "li".into(),
        attrs: vec![],
    });
    let li1_text = dom.create_node(NodeData::Text("item 1".into()));
    dom.append_child(li1, li1_text);
    let li2 = dom.create_node(NodeData::Element {
        name: "li".into(),
        attrs: vec![],
    });
    let li2_text = dom.create_node(NodeData::Text("item 2".into()));
    dom.append_child(li2, li2_text);
    dom.append_child(ul, li1);
    dom.append_child(ul, li2);
    dom.append_child(doc, ul);

    let view = build_semantic_view(&dom);

    // Verify SemanticNodes
    assert_eq!(view.roots.len(), 3);

    // Heading
    assert_eq!(
        view.roots[0],
        SemanticNode::Heading {
            level: 1,
            text: "Title".into()
        }
    );

    // Section (because of the link in p)
    match &view.roots[1] {
        SemanticNode::Section(children) => {
            assert_eq!(children.len(), 3);
            assert_eq!(children[0], SemanticNode::Text("A p containing a ".into()));
            assert_eq!(
                children[1],
                SemanticNode::Link {
                    text: "link".into(),
                    href: "https://example.com".into()
                }
            );
            assert_eq!(children[2], SemanticNode::Text(".".into()));
        }
        _ => panic!(
            "Expected Section for complex paragraph, got {:?}",
            view.roots[1]
        ),
    }

    // List
    match &view.roots[2] {
        SemanticNode::List(items) => {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0], SemanticNode::ListItem("item 1".into()));
            assert_eq!(items[1], SemanticNode::ListItem("item 2".into()));
        }
        _ => panic!("Expected List, got {:?}", view.roots[2]),
    }

    // Verify Markdown
    let md = to_markdown(&view);
    let expected_md =
        "# Title\n\nA p containing a [link](https://example.com).\n\n- item 1\n- item 2";
    assert_eq!(md, expected_md);
}

#[test]
fn test_simple_paragraph_and_formatting() {
    let mut dom = Dom::new();
    let doc = dom.document();

    // <p>Simple <strong>bold</strong> text.</p>
    let p = dom.create_node(NodeData::Element {
        name: "p".into(),
        attrs: vec![],
    });
    let t1 = dom.create_node(NodeData::Text("Simple ".into()));
    let strong = dom.create_node(NodeData::Element {
        name: "strong".into(),
        attrs: vec![],
    });
    let t2 = dom.create_node(NodeData::Text("bold".into()));
    let t3 = dom.create_node(NodeData::Text(" text.".into()));
    dom.append_child(strong, t2);
    dom.append_child(p, t1);
    dom.append_child(p, strong);
    dom.append_child(p, t3);
    dom.append_child(doc, p);

    let view = build_semantic_view(&dom);
    assert_eq!(view.roots.len(), 1);
    assert_eq!(
        view.roots[0],
        SemanticNode::Paragraph("Simple bold text.".into())
    );

    let md = to_markdown(&view);
    assert_eq!(md, "Simple bold text.");
}

#[test]
fn test_aria_role_and_accessible_name() {
    let mut dom = Dom::new();

    // 1. Implicit ARIA role mappings
    // Button
    let btn = dom.create_node(NodeData::Element {
        name: "button".into(),
        attrs: vec![],
    });
    assert_eq!(role(&dom, btn), Some("button".into()));

    // Link with href
    let link_with_href = dom.create_node(NodeData::Element {
        name: "a".into(),
        attrs: vec![("href".into(), "#".into())],
    });
    assert_eq!(role(&dom, link_with_href), Some("link".into()));

    // Link without href (should have NO implicit role)
    let link_no_href = dom.create_node(NodeData::Element {
        name: "a".into(),
        attrs: vec![],
    });
    assert_eq!(role(&dom, link_no_href), None);

    // Heading (h2)
    let h2 = dom.create_node(NodeData::Element {
        name: "h2".into(),
        attrs: vec![],
    });
    assert_eq!(role(&dom, h2), Some("heading".into()));

    // Image
    let img = dom.create_node(NodeData::Element {
        name: "img".into(),
        attrs: vec![],
    });
    assert_eq!(role(&dom, img), Some("img".into()));

    // Landmark elements: nav, main, aside, header, footer, article, section, form
    let nav = dom.create_node(NodeData::Element {
        name: "nav".into(),
        attrs: vec![],
    });
    assert_eq!(role(&dom, nav), Some("navigation".into()));

    let main = dom.create_node(NodeData::Element {
        name: "main".into(),
        attrs: vec![],
    });
    assert_eq!(role(&dom, main), Some("main".into()));

    let aside = dom.create_node(NodeData::Element {
        name: "aside".into(),
        attrs: vec![],
    });
    assert_eq!(role(&dom, aside), Some("complementary".into()));

    let header = dom.create_node(NodeData::Element {
        name: "header".into(),
        attrs: vec![],
    });
    assert_eq!(role(&dom, header), Some("banner".into()));

    let footer = dom.create_node(NodeData::Element {
        name: "footer".into(),
        attrs: vec![],
    });
    assert_eq!(role(&dom, footer), Some("contentinfo".into()));

    let article = dom.create_node(NodeData::Element {
        name: "article".into(),
        attrs: vec![],
    });
    assert_eq!(role(&dom, article), Some("article".into()));

    let section = dom.create_node(NodeData::Element {
        name: "section".into(),
        attrs: vec![],
    });
    assert_eq!(role(&dom, section), Some("region".into()));

    let form = dom.create_node(NodeData::Element {
        name: "form".into(),
        attrs: vec![],
    });
    assert_eq!(role(&dom, form), Some("form".into()));

    // Lists
    let ul = dom.create_node(NodeData::Element {
        name: "ul".into(),
        attrs: vec![],
    });
    assert_eq!(role(&dom, ul), Some("list".into()));

    let li = dom.create_node(NodeData::Element {
        name: "li".into(),
        attrs: vec![],
    });
    assert_eq!(role(&dom, li), Some("listitem".into()));

    // Input by type
    let input_checkbox = dom.create_node(NodeData::Element {
        name: "input".into(),
        attrs: vec![("type".into(), "checkbox".into())],
    });
    assert_eq!(role(&dom, input_checkbox), Some("checkbox".into()));

    // Input type radio with case insensitivity
    let input_radio = dom.create_node(NodeData::Element {
        name: "input".into(),
        attrs: vec![("type".into(), "RADIO".into())],
    });
    assert_eq!(role(&dom, input_radio), Some("radio".into()));

    let input_text = dom.create_node(NodeData::Element {
        name: "input".into(),
        attrs: vec![("type".into(), "text".into())],
    });
    assert_eq!(role(&dom, input_text), Some("textbox".into()));

    let input_no_type = dom.create_node(NodeData::Element {
        name: "input".into(),
        attrs: vec![],
    });
    assert_eq!(role(&dom, input_no_type), Some("textbox".into()));

    // 2. Honoring explicit role attribute
    // Single explicit role
    let custom_role = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("role".into(), "search".into())],
    });
    assert_eq!(role(&dom, custom_role), Some("search".into()));

    // Space-separated explicit roles: first recognized/specified token
    let multi_role = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("role".into(), "checkbox button".into())],
    });
    assert_eq!(role(&dom, multi_role), Some("checkbox".into()));

    // Explicit role with whitespace padding
    let space_role = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("role".into(), "   navigation   ".into())],
    });
    assert_eq!(role(&dom, space_role), Some("navigation".into()));

    // 3. Accessible name computation
    // From aria-label
    let label_btn = dom.create_node(NodeData::Element {
        name: "button".into(),
        attrs: vec![("aria-label".into(), "Submit Form".into())],
    });
    assert_eq!(accessible_name(&dom, label_btn), "Submit Form".to_string());

    // From alt attribute (for img)
    let alt_img = dom.create_node(NodeData::Element {
        name: "img".into(),
        attrs: vec![("alt".into(), "Company Logo".into())],
    });
    assert_eq!(accessible_name(&dom, alt_img), "Company Logo".to_string());

    // From text content fallback
    let text_btn = dom.create_node(NodeData::Element {
        name: "button".into(),
        attrs: vec![],
    });
    let btn_text = dom.create_node(NodeData::Text("Click Me".into()));
    dom.append_child(text_btn, btn_text);
    assert_eq!(accessible_name(&dom, text_btn), "Click Me".to_string());

    // aria-label takes precedence over alt and text content
    let prec_btn = dom.create_node(NodeData::Element {
        name: "img".into(),
        attrs: vec![
            ("aria-label".into(), "Label Name".into()),
            ("alt".into(), "Alt Name".into()),
        ],
    });
    let img_text = dom.create_node(NodeData::Text("Text Name".into()));
    dom.append_child(prec_btn, img_text);
    assert_eq!(accessible_name(&dom, prec_btn), "Label Name".to_string());

    // alt takes precedence over text content
    let alt_prec_btn = dom.create_node(NodeData::Element {
        name: "img".into(),
        attrs: vec![("alt".into(), "Alt Name".into())],
    });
    let img_text2 = dom.create_node(NodeData::Text("Text Name".into()));
    dom.append_child(alt_prec_btn, img_text2);
    assert_eq!(accessible_name(&dom, alt_prec_btn), "Alt Name".to_string());

    // 4. Invalid id / non-element node safety
    let mut dom_other = Dom::new();
    let mut foreign = dom_other.document();
    for _ in 0..100 {
        foreign = dom_other.create_node(NodeData::Element {
            name: "x".into(),
            attrs: vec![],
        });
    }
    // Verify that foreign is indeed an invalid ID for dom
    assert!(dom.data(foreign).is_none());
    assert_eq!(role(&dom, foreign), None);
    assert_eq!(accessible_name(&dom, foreign), "".to_string());

    let text_node = dom.create_node(NodeData::Text("Hello".into()));
    assert_eq!(role(&dom, text_node), None);
    assert_eq!(accessible_name(&dom, text_node), "Hello".to_string());
}
