use crate::ascii;
use crate::dom::{Dom, NodeData};
use crate::infra::NodeId;
use crate::selector::{
    AttrOp, Combinator, ComplexSelector, Component, CompoundSelector, SelectorList,
};

/// Matches a selector list against a DOM node.
/// Returns true if any of the complex selectors in the list match the node.
// spec: https://drafts.csswg.org/selectors-4/#match-against-element
pub fn matches(list: &SelectorList, dom: &Dom, node: NodeId) -> bool {
    list.0.iter().any(|sel| matches_complex(sel, dom, node))
}

/// Matches a complex selector against a DOM node.
// spec: https://drafts.csswg.org/selectors-4/#match-against-element
pub fn matches_complex(sel: &ComplexSelector, dom: &Dom, node: NodeId) -> bool {
    if sel.parts.is_empty() {
        return false;
    }

    // Match the rightmost compound selector first.
    let last_part_idx = sel.parts.len() - 1;
    let (_, compound) = &sel.parts[last_part_idx];

    if !matches_compound(compound, dom, node) {
        return false;
    }

    // If there are no more parts, it's a match.
    if last_part_idx == 0 {
        return true;
    }

    // Match the rest of the selector against ancestors/siblings.
    matches_rest(
        &sel.parts[..last_part_idx],
        dom,
        node,
        sel.parts[last_part_idx].0,
    )
}

fn matches_rest(
    parts: &[(Combinator, CompoundSelector)],
    dom: &Dom,
    node: NodeId,
    comb: Combinator,
) -> bool {
    match comb {
        Combinator::Descendant => {
            let mut current = dom.parent(node);
            while let Some(ancestor) = current {
                if matches_complex_at_part(parts, dom, ancestor) {
                    return true;
                }
                current = dom.parent(ancestor);
            }
            false
        }
        Combinator::Child => {
            if let Some(parent) = dom.parent(node) {
                matches_complex_at_part(parts, dom, parent)
            } else {
                false
            }
        }
        Combinator::NextSibling => {
            if let Some(prev) = get_previous_sibling(dom, node) {
                matches_complex_at_part(parts, dom, prev)
            } else {
                false
            }
        }
        Combinator::SubsequentSibling => {
            let mut current = get_previous_sibling(dom, node);
            while let Some(sibling) = current {
                if matches_complex_at_part(parts, dom, sibling) {
                    return true;
                }
                current = get_previous_sibling(dom, sibling);
            }
            false
        }
    }
}

/// Matches a complex selector (represented as a slice of parts) against a node.
fn matches_complex_at_part(
    parts: &[(Combinator, CompoundSelector)],
    dom: &Dom,
    node: NodeId,
) -> bool {
    if parts.is_empty() {
        return false;
    }

    let last_idx = parts.len() - 1;
    let (_, compound) = &parts[last_idx];

    if !matches_compound(compound, dom, node) {
        return false;
    }

    if last_idx == 0 {
        return true;
    }

    matches_rest(&parts[..last_idx], dom, node, parts[last_idx].0)
}

fn matches_compound(compound: &CompoundSelector, dom: &Dom, node: NodeId) -> bool {
    if compound.components.is_empty() {
        return false;
    }
    compound
        .components
        .iter()
        .all(|comp| matches_component(comp, dom, node))
}

fn matches_component(comp: &Component, dom: &Dom, node: NodeId) -> bool {
    let data = match dom.data(node) {
        Some(NodeData::Element { name, attrs }) => (name, attrs),
        _ => return false, // Non-element nodes never match.
    };
    let (tag_name, attrs) = data;

    match comp {
        Component::Type(name) => ascii::eq_ignore_ascii_case(tag_name, name),
        Component::Universal => true,
        Component::Id(id) => attrs.iter().any(|(n, v)| n == "id" && v == id),
        Component::Class(class) => attrs
            .iter()
            .any(|(n, v)| n == "class" && v.split(ascii::is_html_whitespace).any(|c| c == class)),
        Component::Attribute { name, op, value } => {
            let attr_val = attrs.iter().find(|(n, _)| n == name).map(|(_, v)| v);
            match (attr_val, op, value) {
                (Some(_), None, _) => true, // Presence only
                (Some(v), Some(op), Some(val)) => match op {
                    AttrOp::Exact => v == val,
                    AttrOp::Includes => v.split(ascii::is_html_whitespace).any(|c| c == val),
                    AttrOp::DashMatch => {
                        v == val
                            || (v.starts_with(val) && v.as_bytes().get(val.len()) == Some(&b'-'))
                    }
                    AttrOp::Prefix => v.starts_with(val),
                    AttrOp::Suffix => v.ends_with(val),
                    AttrOp::Substring => v.contains(val),
                },
                _ => false,
            }
        }
        Component::PseudoClass(name) => {
            // Other functional pseudo-classes are not yet implemented.
            match name.as_str() {
                n if n.contains('(') => false,
                _ => true, // Match any pseudo-class by name for now as per SPEC.
            }
        }
        Component::PseudoElement(_) => true, // Match any pseudo-element by name for now.
        Component::NthChild(a, b) => {
            if let Some(parent) = dom.parent(node) {
                let children = dom.children(parent);
                // Only count elements
                let mut element_index = 0;
                for &child in children {
                    if child == node {
                        let i = element_index + 1; // 1-indexed
                        if *a == 0 {
                            return i == *b;
                        }
                        let diff = i - *b;
                        if *a > 0 {
                            return diff >= 0 && diff % *a == 0;
                        } else {
                            return diff <= 0 && diff % *a == 0;
                        }
                    }
                    if matches!(dom.data(child), Some(NodeData::Element { .. })) {
                        element_index += 1;
                    }
                }
            }
            false
        }
        Component::Not(compound) => !matches_compound(compound, dom, node),
        Component::FirstChild => {
            if let Some(parent) = dom.parent(node) {
                let children = dom.children(parent);
                for &child in children {
                    if matches!(dom.data(child), Some(NodeData::Element { .. })) {
                        return child == node;
                    }
                }
            }
            false
        }
        Component::LastChild => {
            if let Some(parent) = dom.parent(node) {
                let children = dom.children(parent);
                for &child in children.iter().rev() {
                    if matches!(dom.data(child), Some(NodeData::Element { .. })) {
                        return child == node;
                    }
                }
            }
            false
        }
    }
}

fn get_previous_sibling(dom: &Dom, node: NodeId) -> Option<NodeId> {
    let parent = dom.parent(node)?;
    let children = dom.children(parent);
    let idx = children.iter().position(|&id| id == node)?;
    if idx > 0 {
        Some(children[idx - 1])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selector::parse_selector_list;

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

        //       <p class="text" title="hello">Hello</p>
        let p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![
                ("class".into(), "text".into()),
                ("title".into(), "hello".into()),
            ],
        });
        dom.append_child(div, p);
        let text = dom.create_node(NodeData::Text("Hello".into()));
        dom.append_child(p, text);

        //       <span class="text">World</span>
        let span = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("class".into(), "text".into())],
        });
        dom.append_child(div, span);

        //       <a href="https://example.com" lang="en-US">Link</a>
        let a = dom.create_node(NodeData::Element {
            name: "a".into(),
            attrs: vec![
                ("href".into(), "https://example.com".into()),
                ("lang".into(), "en-US".into()),
            ],
        });
        dom.append_child(div, a);

        (dom, p)
    }

    #[test]
    fn test_matches_basic() {
        let (dom, p) = setup_dom();

        assert!(matches(&parse_selector_list("p").unwrap(), &dom, p));
        assert!(matches(&parse_selector_list("P").unwrap(), &dom, p));
        assert!(matches(&parse_selector_list("*").unwrap(), &dom, p));
        assert!(matches(&parse_selector_list(".text").unwrap(), &dom, p));
        assert!(matches(&parse_selector_list("p.text").unwrap(), &dom, p));
        assert!(!matches(&parse_selector_list("div").unwrap(), &dom, p));
    }

    #[test]
    fn test_matches_id_class() {
        let (dom, p) = setup_dom();
        let div = dom.parent(p).unwrap();

        assert!(matches(
            &parse_selector_list("#container").unwrap(),
            &dom,
            div
        ));
        assert!(matches(&parse_selector_list(".foo").unwrap(), &dom, div));
        assert!(matches(&parse_selector_list(".bar").unwrap(), &dom, div));
        assert!(matches(
            &parse_selector_list(".foo.bar").unwrap(),
            &dom,
            div
        ));
        assert!(matches(
            &parse_selector_list("div#container.foo.bar").unwrap(),
            &dom,
            div
        ));
    }

    #[test]
    fn test_matches_attribute() {
        let (dom, p) = setup_dom();
        let div = dom.parent(p).unwrap();
        let a = *dom.children(div).last().unwrap();

        assert!(matches(&parse_selector_list("[title]").unwrap(), &dom, p));
        assert!(matches(
            &parse_selector_list("[title=\"hello\"]").unwrap(),
            &dom,
            p
        ));
        assert!(!matches(
            &parse_selector_list("[title=\"world\"]").unwrap(),
            &dom,
            p
        ));

        assert!(matches(
            &parse_selector_list("[href^=\"https\"]").unwrap(),
            &dom,
            a
        ));
        assert!(matches(
            &parse_selector_list("[href$=\"com\"]").unwrap(),
            &dom,
            a
        ));
        assert!(matches(
            &parse_selector_list("[href*=\"example\"]").unwrap(),
            &dom,
            a
        ));

        assert!(matches(
            &parse_selector_list("[lang|=\"en\"]").unwrap(),
            &dom,
            a
        ));
        assert!(!matches(
            &parse_selector_list("[lang|=\"jp\"]").unwrap(),
            &dom,
            a
        ));

        let body = dom.parent(div).unwrap();
        assert!(matches(
            &parse_selector_list("[class~=\"main\"]").unwrap(),
            &dom,
            body
        ));
    }

    #[test]
    fn test_matches_combinators() {
        let (dom, p) = setup_dom();
        let div = dom.parent(p).unwrap();
        let span = dom.children(div)[1];
        let a = dom.children(div)[2];

        // Descendant
        assert!(matches(&parse_selector_list("div p").unwrap(), &dom, p));
        assert!(matches(&parse_selector_list("body p").unwrap(), &dom, p));
        assert!(matches(&parse_selector_list("html p").unwrap(), &dom, p));

        // Child
        assert!(matches(&parse_selector_list("div > p").unwrap(), &dom, p));
        assert!(!matches(&parse_selector_list("body > p").unwrap(), &dom, p));

        // Next Sibling
        assert!(matches(
            &parse_selector_list("p + span").unwrap(),
            &dom,
            span
        ));
        assert!(!matches(&parse_selector_list("p + a").unwrap(), &dom, a));

        // Subsequent Sibling
        assert!(matches(
            &parse_selector_list("p ~ span").unwrap(),
            &dom,
            span
        ));
        assert!(matches(&parse_selector_list("p ~ a").unwrap(), &dom, a));
        assert!(!matches(&parse_selector_list("span ~ p").unwrap(), &dom, p));
    }

    #[test]
    fn test_matches_pseudo() {
        let (dom, p) = setup_dom();

        assert!(matches(&parse_selector_list("p:hover").unwrap(), &dom, p));
        assert!(matches(&parse_selector_list("p::before").unwrap(), &dom, p));

        // functional pseudo-classes are not yet supported by parser, so we construct them manually
        let compound = CompoundSelector {
            components: vec![
                Component::Type("p".to_string()),
                Component::PseudoClass("nth-child(1)".to_string()),
            ],
        };
        let complex = ComplexSelector {
            parts: vec![(Combinator::Descendant, compound)],
        };
        let list = SelectorList(vec![complex]);
        assert!(!matches(&list, &dom, p));
    }

    #[test]
    fn test_matches_functional_pseudo() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let parent = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, parent);

        let mut children = Vec::new();
        for i in 1..=5 {
            let child = dom.create_node(NodeData::Element {
                name: "p".into(),
                attrs: vec![("id".into(), format!("p{}", i))],
            });
            dom.append_child(parent, child);
            children.push(child);
        }

        // :first-child
        assert!(matches(
            &parse_selector_list(":first-child").unwrap(),
            &dom,
            children[0]
        ));
        assert!(!matches(
            &parse_selector_list(":first-child").unwrap(),
            &dom,
            children[1]
        ));

        // :last-child
        assert!(matches(
            &parse_selector_list(":last-child").unwrap(),
            &dom,
            children[4]
        ));
        assert!(!matches(
            &parse_selector_list(":last-child").unwrap(),
            &dom,
            children[3]
        ));

        // :nth-child(odd) (1, 3, 5)
        let odd = parse_selector_list(":nth-child(odd)").unwrap();
        assert!(matches(&odd, &dom, children[0]));
        assert!(!matches(&odd, &dom, children[1]));
        assert!(matches(&odd, &dom, children[2]));
        assert!(!matches(&odd, &dom, children[3]));
        assert!(matches(&odd, &dom, children[4]));

        // :nth-child(even) (2, 4)
        let even = parse_selector_list(":nth-child(even)").unwrap();
        assert!(!matches(&even, &dom, children[0]));
        assert!(matches(&even, &dom, children[1]));
        assert!(!matches(&even, &dom, children[2]));
        assert!(matches(&even, &dom, children[3]));
        assert!(!matches(&even, &dom, children[4]));

        // :nth-child(3)
        assert!(matches(
            &parse_selector_list(":nth-child(3)").unwrap(),
            &dom,
            children[2]
        ));

        // :nth-child(2n+1) same as odd
        assert!(matches(
            &parse_selector_list(":nth-child(2n+1)").unwrap(),
            &dom,
            children[2]
        ));

        // :not(#p1)
        let not_p1 = parse_selector_list(":not(#p1)").unwrap();
        assert!(!matches(&not_p1, &dom, children[0]));
        assert!(matches(&not_p1, &dom, children[1]));
    }

    #[test]
    fn test_non_element_nodes() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let text = dom.create_node(NodeData::Text("hi".into()));
        dom.append_child(doc, text);

        assert!(!matches(&parse_selector_list("*").unwrap(), &dom, text));
        assert!(!matches(&parse_selector_list("*").unwrap(), &dom, doc));
    }
}
