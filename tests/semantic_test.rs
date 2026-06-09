use underrated::dom::{Dom, NodeData};
use underrated::semantic::{
    AxNode, SemanticNode, accessible_name, ax_tree, build_semantic_view, role, to_markdown,
};

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

#[test]
fn test_ax_tree_export() {
    let mut dom = Dom::new();
    let doc = dom.document();

    // 1. Create a container (main)
    let main = dom.create_node(NodeData::Element {
        name: "main".into(),
        attrs: vec![],
    });
    dom.append_child(doc, main);

    // 2. h2 (heading)
    let h2 = dom.create_node(NodeData::Element {
        name: "h2".into(),
        attrs: vec![],
    });
    let h2_text = dom.create_node(NodeData::Text("Welcome Header".into()));
    dom.append_child(h2, h2_text);
    dom.append_child(main, h2);

    // 3. button (with aria-label)
    let btn = dom.create_node(NodeData::Element {
        name: "button".into(),
        attrs: vec![("aria-label".into(), "Custom Button".into())],
    });
    let btn_text = dom.create_node(NodeData::Text("Ignored Inner".into()));
    dom.append_child(btn, btn_text);
    dom.append_child(main, btn);

    // 4. link (a with href)
    let link = dom.create_node(NodeData::Element {
        name: "a".into(),
        attrs: vec![("href".into(), "https://example.com".into())],
    });
    let link_text = dom.create_node(NodeData::Text("Visit Us".into()));
    dom.append_child(link, link_text);
    dom.append_child(main, link);

    // 5. Presentation role: element itself is pruned, children are promoted
    let presentational_div = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("role".into(), "presentation".into())],
    });
    let promo_btn = dom.create_node(NodeData::Element {
        name: "button".into(),
        attrs: vec![],
    });
    let promo_text = dom.create_node(NodeData::Text("Promoted".into()));
    dom.append_child(promo_btn, promo_text);
    dom.append_child(presentational_div, promo_btn);
    dom.append_child(main, presentational_div);

    // 6. None role: same as presentation
    let none_div = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("role".into(), "none".into())],
    });
    let promo_link = dom.create_node(NodeData::Element {
        name: "a".into(),
        attrs: vec![("href".into(), "#".into())],
    });
    let promo_link_text = dom.create_node(NodeData::Text("Promo Link".into()));
    dom.append_child(promo_link, promo_link_text);
    dom.append_child(none_div, promo_link);
    dom.append_child(main, none_div);

    // 7. Hidden: aria-hidden="true" (entire subtree pruned)
    let hidden_subtree = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("aria-hidden".into(), "true".into())],
    });
    let hidden_btn = dom.create_node(NodeData::Element {
        name: "button".into(),
        attrs: vec![],
    });
    let hidden_text = dom.create_node(NodeData::Text("Hidden".into()));
    dom.append_child(hidden_btn, hidden_text);
    dom.append_child(hidden_subtree, hidden_btn);
    dom.append_child(main, hidden_subtree);

    // 8. Hidden: display: none style (entire subtree pruned)
    let display_none_subtree = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("style".into(), "display: none;".into())],
    });
    let hidden_btn2 = dom.create_node(NodeData::Element {
        name: "button".into(),
        attrs: vec![],
    });
    dom.append_child(display_none_subtree, hidden_btn2);
    dom.append_child(main, display_none_subtree);

    // 9. Hidden: visibility: hidden style (entire subtree pruned)
    let visibility_hidden_subtree = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("style".into(), "visibility: hidden".into())],
    });
    let hidden_btn3 = dom.create_node(NodeData::Element {
        name: "button".into(),
        attrs: vec![],
    });
    dom.append_child(visibility_hidden_subtree, hidden_btn3);
    dom.append_child(main, visibility_hidden_subtree);

    // 10. Doctype and comment nodes (ignored entirely)
    let doctype = dom.create_node(NodeData::Doctype {
        name: "html".into(),
        public_id: "".into(),
        system_id: "".into(),
    });
    dom.append_child(doc, doctype);

    let comment = dom.create_node(NodeData::Comment("This is a comment".into()));
    dom.append_child(main, comment);

    // Export AXTree explicitly using AxNode to avoid unused import warning.
    let tree: AxNode = ax_tree(&dom);

    // Expected structure:
    // Document (role: None, name: "")
    //   main (role: "main", name: "Welcome HeaderIgnored InnerVisit UsPromotedPromo Link")
    //     h2 (role: "heading", name: "Welcome Header")
    //       "Welcome Header" (role: None, name: "Welcome Header")
    //     button (role: "button", name: "Custom Button")
    //       "Ignored Inner" (role: None, name: "Ignored Inner")
    //     link (role: "link", name: "Visit Us")
    //       "Visit Us" (role: None, name: "Visit Us")
    //     button (role: "button", name: "Promoted")
    //       "Promoted" (role: None, name: "Promoted")
    //     link (role: "link", name: "Promo Link")
    //       "Promo Link" (role: None, name: "Promo Link")

    assert_eq!(tree.role, None);
    assert_eq!(tree.name, String::new());
    assert_eq!(tree.children.len(), 1);

    let main_node = &tree.children[0];
    assert_eq!(main_node.role, Some("main".into()));
    // Under existing accessible_name implementation, container's fallback is text_content of its descendants
    assert!(main_node.name.contains("Welcome Header"));
    assert!(main_node.name.contains("Ignored Inner"));
    assert_eq!(main_node.children.len(), 5);

    // Verify Heading
    let h2_node = &main_node.children[0];
    assert_eq!(h2_node.role, Some("heading".into()));
    assert_eq!(h2_node.name, "Welcome Header");
    assert_eq!(h2_node.children.len(), 1);
    assert_eq!(h2_node.children[0].role, None);
    assert_eq!(h2_node.children[0].name, "Welcome Header");

    // Verify Custom Button
    let btn_node = &main_node.children[1];
    assert_eq!(btn_node.role, Some("button".into()));
    assert_eq!(btn_node.name, "Custom Button");
    assert_eq!(btn_node.children.len(), 1);
    assert_eq!(btn_node.children[0].role, None);
    assert_eq!(btn_node.children[0].name, "Ignored Inner");

    // Verify Link
    let link_node = &main_node.children[2];
    assert_eq!(link_node.role, Some("link".into()));
    assert_eq!(link_node.name, "Visit Us");
    assert_eq!(link_node.children.len(), 1);
    assert_eq!(link_node.children[0].role, None);
    assert_eq!(link_node.children[0].name, "Visit Us");

    // Verify Promoted Button (from role="presentation" container)
    let promo_btn_node = &main_node.children[3];
    assert_eq!(promo_btn_node.role, Some("button".into()));
    assert_eq!(promo_btn_node.name, "Promoted");
    assert_eq!(promo_btn_node.children.len(), 1);
    assert_eq!(promo_btn_node.children[0].role, None);
    assert_eq!(promo_btn_node.children[0].name, "Promoted");

    // Verify Promoted Link (from role="none" container)
    let promo_link_node = &main_node.children[4];
    assert_eq!(promo_link_node.role, Some("link".into()));
    assert_eq!(promo_link_node.name, "Promo Link");
}

#[test]
fn test_semantic_view_form_elements_and_pruning() {
    let mut dom = Dom::new();
    let doc = dom.document();

    // <form action="/search" method="post">
    let form = dom.create_node(NodeData::Element {
        name: "form".into(),
        attrs: vec![
            ("action".into(), "/search".into()),
            ("method".into(), "post".into()),
        ],
    });
    dom.append_child(doc, form);

    // 1. Associated label: <label for="username">User Name</label>
    let label1 = dom.create_node(NodeData::Element {
        name: "label".into(),
        attrs: vec![("for".into(), "username".into())],
    });
    let label1_text = dom.create_node(NodeData::Text("User Name".into()));
    dom.append_child(label1, label1_text);
    dom.append_child(form, label1);

    // <input id="username" type="text" value="alice">
    let input1 = dom.create_node(NodeData::Element {
        name: "input".into(),
        attrs: vec![
            ("id".into(), "username".into()),
            ("type".into(), "text".into()),
            ("value".into(), "alice".into()),
        ],
    });
    dom.append_child(form, input1);

    // 2. Nested label: <label>Password <input type="password" value="pass123"></label>
    let label2 = dom.create_node(NodeData::Element {
        name: "label".into(),
        attrs: vec![],
    });
    let label2_text = dom.create_node(NodeData::Text("Password".into()));
    dom.append_child(label2, label2_text);

    let input2 = dom.create_node(NodeData::Element {
        name: "input".into(),
        attrs: vec![
            ("type".into(), "password".into()),
            ("value".into(), "pass123".into()),
        ],
    });
    dom.append_child(label2, input2);
    dom.append_child(form, label2);

    // 3. Checkbox with aria-label: <input type="checkbox" checked aria-label="Subscribe">
    let input_check = dom.create_node(NodeData::Element {
        name: "input".into(),
        attrs: vec![
            ("type".into(), "checkbox".into()),
            ("checked".into(), "".into()),
            ("aria-label".into(), "Subscribe".into()),
        ],
    });
    dom.append_child(form, input_check);

    // 4. Radio without label/aria-label, fallback to name: <input type="radio" name="option_choice">
    let input_radio = dom.create_node(NodeData::Element {
        name: "input".into(),
        attrs: vec![
            ("type".into(), "radio".into()),
            ("name".into(), "option_choice".into()),
        ],
    });
    dom.append_child(form, input_radio);

    // 5. Button: <button type="submit">Search</button>
    let btn = dom.create_node(NodeData::Element {
        name: "button".into(),
        attrs: vec![("type".into(), "submit".into())],
    });
    let btn_text = dom.create_node(NodeData::Text("Search".into()));
    dom.append_child(btn, btn_text);
    dom.append_child(form, btn);

    // 6. Select dropdown: <select aria-label="Selection"><option>A</option><option selected>B</option></select>
    let select = dom.create_node(NodeData::Element {
        name: "select".into(),
        attrs: vec![("aria-label".into(), "Selection".into())],
    });
    let opt1 = dom.create_node(NodeData::Element {
        name: "option".into(),
        attrs: vec![],
    });
    let opt1_text = dom.create_node(NodeData::Text("A".into()));
    dom.append_child(opt1, opt1_text);
    let opt2 = dom.create_node(NodeData::Element {
        name: "option".into(),
        attrs: vec![("selected".into(), "".into())],
    });
    let opt2_text = dom.create_node(NodeData::Text("B".into()));
    dom.append_child(opt2, opt2_text);
    dom.append_child(select, opt1);
    dom.append_child(select, opt2);
    dom.append_child(form, select);

    // 7. Hidden element that should be pruned: <div style="display: none;"><input type="text" value="secret"></div>
    let hidden_div = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![("style".into(), "display: none;".into())],
    });
    let hidden_input = dom.create_node(NodeData::Element {
        name: "input".into(),
        attrs: vec![
            ("type".into(), "text".into()),
            ("value".into(), "secret".into()),
        ],
    });
    dom.append_child(hidden_div, hidden_input);
    dom.append_child(form, hidden_div);

    let view = build_semantic_view(&dom);

    // Verify SemanticNodes
    assert_eq!(view.roots.len(), 1);

    if let SemanticNode::Form {
        action,
        method,
        children,
    } = &view.roots[0]
    {
        assert_eq!(action, "/search");
        assert_eq!(method, "post");

        // children should contain:
        // 1. Text("User Name") (from label1 text)
        // 2. Input { label: "User Name", input_type: "text", value: "alice", ... }
        // 3. Text("Password") (from label2 text)
        // 4. Input { label: "Password", input_type: "password", value: "pass123", ... }
        // 5. Input { label: "Subscribe", input_type: "checkbox", checked: true, ... }
        // 6. Input { label: "option_choice", input_type: "radio", checked: false, ... }
        // 7. Button { label: "Search", button_type: "submit" }
        // 8. Select { label: "Selection", selected: Some("B"), options: ["A", "B"] }
        // Note: The hidden input MUST be pruned.

        assert_eq!(children.len(), 8);
        assert_eq!(children[0], SemanticNode::Text("User Name".into()));
        assert_eq!(
            children[1],
            SemanticNode::Input {
                label: "User Name".into(),
                input_type: "text".into(),
                value: "alice".into(),
                checked: false,
            }
        );
        assert_eq!(children[2], SemanticNode::Text("Password".into()));
        assert_eq!(
            children[3],
            SemanticNode::Input {
                label: "Password".into(),
                input_type: "password".into(),
                value: "pass123".into(),
                checked: false,
            }
        );
        assert_eq!(
            children[4],
            SemanticNode::Input {
                label: "Subscribe".into(),
                input_type: "checkbox".into(),
                value: "on".into(),
                checked: true,
            }
        );
        assert_eq!(
            children[5],
            SemanticNode::Input {
                label: "option_choice".into(),
                input_type: "radio".into(),
                value: "on".into(),
                checked: false,
            }
        );
        assert_eq!(
            children[6],
            SemanticNode::Button {
                label: "Search".into(),
                button_type: "submit".into(),
            }
        );
        assert_eq!(
            children[7],
            SemanticNode::Select {
                label: "Selection".into(),
                selected: Some("B".into()),
                options: vec!["A".into(), "B".into()],
            }
        );
    } else {
        panic!("Expected root to be a Form node, got {:?}", view.roots[0]);
    }

    // Verify Markdown Output
    let md = to_markdown(&view);
    let expected_md = "[Form: /search (post)]\nUser Name\n\nUser Name: [alice]\n\nPassword\n\nPassword: [pass123]\n\n[x] Subscribe\n\n( ) option_choice\n\n[Button: Search]\n\nSelection: [B] v";
    assert_eq!(md, expected_md);
}
