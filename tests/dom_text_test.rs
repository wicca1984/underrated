use underrated::dom::{Dom, NodeData};

#[test]
fn test_text_content_basic() {
    let mut dom = Dom::new();
    let doc = dom.document();

    // <div>a<span>b</span>c<!--x--></div>
    let div = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![],
    });
    dom.append_child(doc, div);

    let text_a = dom.create_node(NodeData::Text("a".into()));
    dom.append_child(div, text_a);

    let span = dom.create_node(NodeData::Element {
        name: "span".into(),
        attrs: vec![],
    });
    dom.append_child(div, span);

    let text_b = dom.create_node(NodeData::Text("b".into()));
    dom.append_child(span, text_b);

    let text_c = dom.create_node(NodeData::Text("c".into()));
    dom.append_child(div, text_c);

    let comment = dom.create_node(NodeData::Comment("x".into()));
    dom.append_child(div, comment);

    assert_eq!(dom.text_content(div), "abc");
    assert_eq!(dom.text_content(text_a), "a");
    assert_eq!(dom.text_content(span), "b");
    assert_eq!(dom.text_content(comment), "");
}

#[test]
fn test_text_content_empty() {
    let mut dom = Dom::new();
    let div = dom.create_node(NodeData::Element {
        name: "div".into(),
        attrs: vec![],
    });
    assert_eq!(dom.text_content(div), "");
}

#[test]
fn test_text_content_document() {
    let mut dom = Dom::new();
    let doc = dom.document();
    let text = dom.create_node(NodeData::Text("hello".into()));
    dom.append_child(doc, text);
    assert_eq!(dom.text_content(doc), "hello");
}
