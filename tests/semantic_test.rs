use underrated::dom::{Dom, NodeData};
use underrated::semantic::{SemanticNode, build_semantic_view, to_markdown};

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
