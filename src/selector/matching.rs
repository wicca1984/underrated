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
            if let Some(prev) = get_previous_element_sibling(dom, node) {
                matches_complex_at_part(parts, dom, prev)
            } else {
                false
            }
        }
        Combinator::SubsequentSibling => {
            let mut current = get_previous_element_sibling(dom, node);
            while let Some(sibling) = current {
                if matches_complex_at_part(parts, dom, sibling) {
                    return true;
                }
                current = get_previous_element_sibling(dom, sibling);
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
        Component::Attribute {
            name,
            op,
            value,
            modifier,
        } => {
            let attr_val = attrs.iter().find(|(n, _)| n == name).map(|(_, v)| v);
            match (attr_val, op, value) {
                (Some(_), None, _) => true, // Presence only
                (Some(v), Some(op), Some(val)) => {
                    let case_insensitive = *modifier == Some('i');
                    if case_insensitive {
                        match op {
                            AttrOp::Exact => eq_ignore_ascii_case(v, val),
                            AttrOp::Includes => v
                                .split(ascii::is_html_whitespace)
                                .any(|c| eq_ignore_ascii_case(c, val)),
                            AttrOp::DashMatch => {
                                eq_ignore_ascii_case(v, val)
                                    || (starts_with_ignore_ascii_case(v, val)
                                        && v.as_bytes().get(val.len()) == Some(&b'-'))
                            }
                            AttrOp::Prefix => starts_with_ignore_ascii_case(v, val),
                            AttrOp::Suffix => ends_with_ignore_ascii_case(v, val),
                            AttrOp::Substring => contains_ignore_ascii_case(v, val),
                        }
                    } else {
                        match op {
                            AttrOp::Exact => v == val,
                            AttrOp::Includes => {
                                v.split(ascii::is_html_whitespace).any(|c| c == val)
                            }
                            AttrOp::DashMatch => {
                                v == val
                                    || (v.starts_with(val)
                                        && v.as_bytes().get(val.len()) == Some(&b'-'))
                            }
                            AttrOp::Prefix => v.starts_with(val),
                            AttrOp::Suffix => v.ends_with(val),
                            AttrOp::Substring => v.contains(val),
                        }
                    }
                }
                _ => false,
            }
        }
        Component::PseudoClass(name) => {
            if name.starts_with("nth-of-type(") && name.ends_with(')') {
                let content = &name["nth-of-type(".len()..name.len() - 1];
                let mut parts = content.split(',');
                let parsed = parts.next().zip(parts.next()).and_then(|(a_str, b_str)| {
                    a_str.parse::<i32>().ok().zip(b_str.parse::<i32>().ok())
                });
                if let Some((a, b)) = parsed {
                    if let Some(parent) = dom.parent(node) {
                        let current_tag_name = match dom.data(node) {
                            Some(NodeData::Element { name, .. }) => name,
                            _ => return false,
                        };
                        let children = dom.children(parent);
                        let mut element_index = 0;
                        for &child in children {
                            if child == node {
                                let i = element_index + 1; // 1-indexed
                                if a == 0 {
                                    return i == b;
                                }
                                let diff = i - b;
                                if a > 0 {
                                    return diff >= 0 && diff % a == 0;
                                } else {
                                    return diff <= 0 && diff % a == 0;
                                }
                            }
                            match dom.data(child) {
                                Some(NodeData::Element { name, .. })
                                    if ascii::eq_ignore_ascii_case(name, current_tag_name) =>
                                {
                                    element_index += 1;
                                }
                                _ => {}
                            }
                        }
                    }
                    return false;
                }
                false
            } else if name.starts_with("nth-child(") && name.ends_with(')') {
                let content = &name["nth-child(".len()..name.len() - 1];
                let mut parts = content.split(',');
                let parsed = parts.next().zip(parts.next()).and_then(|(a_str, b_str)| {
                    a_str.parse::<i32>().ok().zip(b_str.parse::<i32>().ok())
                });
                if let Some((a, b)) = parsed {
                    return nth_child(dom, node, a, b);
                }
                false
            } else if name.starts_with("nth-last-child(") && name.ends_with(')') {
                let content = &name["nth-last-child(".len()..name.len() - 1];
                let mut parts = content.split(',');
                let parsed = parts.next().zip(parts.next()).and_then(|(a_str, b_str)| {
                    a_str.parse::<i32>().ok().zip(b_str.parse::<i32>().ok())
                });
                if let Some((a, b)) = parsed {
                    return nth_last_child(dom, node, a, b);
                }
                false
            } else if name.starts_with("nth-last-of-type(") && name.ends_with(')') {
                let content = &name["nth-last-of-type(".len()..name.len() - 1];
                let mut parts = content.split(',');
                let parsed = parts.next().zip(parts.next()).and_then(|(a_str, b_str)| {
                    a_str.parse::<i32>().ok().zip(b_str.parse::<i32>().ok())
                });
                if let Some((a, b)) = parsed {
                    if let Some(parent) = dom.parent(node) {
                        let current_tag_name = match dom.data(node) {
                            Some(NodeData::Element { name, .. }) => name,
                            _ => return false,
                        };
                        let children = dom.children(parent);
                        let mut element_index = 0;
                        for &child in children.iter().rev() {
                            if child == node {
                                let i = element_index + 1; // 1-indexed
                                if a == 0 {
                                    return i == b;
                                }
                                let diff = i - b;
                                if a > 0 {
                                    return diff >= 0 && diff % a == 0;
                                } else {
                                    return diff <= 0 && diff % a == 0;
                                }
                            }
                            match dom.data(child) {
                                Some(NodeData::Element { name, .. })
                                    if ascii::eq_ignore_ascii_case(name, current_tag_name) =>
                                {
                                    element_index += 1;
                                }
                                _ => {}
                            }
                        }
                    }
                    return false;
                }
                false
            } else {
                match name.to_ascii_lowercase().as_str() {
                    "hover" => get_node_state(node).hover,
                    "focus" => get_node_state(node).focus,
                    "active" => get_node_state(node).active,
                    "first-child" => is_first_child(dom, node),
                    "last-child" => is_last_child(dom, node),
                    "first-of-type" => is_first_of_type(dom, node),
                    "last-of-type" => is_last_of_type(dom, node),
                    "only-child" => is_only_child(dom, node),
                    "only-of-type" => is_only_of_type(dom, node),
                    "empty" => is_empty(dom, node),
                    "root" => is_root(dom, node),
                    "link" => is_link(dom, node),
                    "any-link" => is_link(dom, node),
                    "checked" => is_checked(dom, node),
                    "disabled" => is_disabled(dom, node),
                    "enabled" => is_enabled(dom, node),
                    "required" => is_required(dom, node),
                    "optional" => is_optional(dom, node),
                    "read-only" => is_read_only(dom, node),
                    "read-write" => is_read_write(dom, node),
                    n if n.contains('(') => false,
                    _ => true, // Match other pseudo-classes by name for now as per SPEC.
                }
            }
        }
        Component::PseudoElement(_) => true, // Match any pseudo-element by name for now.
        Component::NthChild(a, b) => nth_child(dom, node, *a, *b),
        Component::Not(compound) => !matches_compound(compound, dom, node),
        Component::Is(list) | Component::Where(list) => matches(list, dom, node),
        Component::FirstChild => is_first_child(dom, node),
        Component::LastChild => is_last_child(dom, node),
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NodeState {
    pub hover: bool,
    pub focus: bool,
    pub active: bool,
}

use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static NODE_STATES: RefCell<HashMap<NodeId, NodeState>> = RefCell::new(HashMap::new());
}

/// Sets the pseudo-class state for a given node.
pub fn set_node_state(node: NodeId, state: NodeState) {
    NODE_STATES.with(|states| {
        states.borrow_mut().insert(node, state);
    });
}

/// Gets the pseudo-class state for a given node.
pub fn get_node_state(node: NodeId) -> NodeState {
    NODE_STATES.with(|states| states.borrow().get(&node).copied().unwrap_or_default())
}

/// Clears all node states.
pub fn clear_node_states() {
    NODE_STATES.with(|states| {
        states.borrow_mut().clear();
    });
}

fn is_first_child(dom: &Dom, node: NodeId) -> bool {
    match dom.data(node) {
        Some(NodeData::Element { .. }) => {}
        _ => return false,
    }
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

fn is_last_child(dom: &Dom, node: NodeId) -> bool {
    match dom.data(node) {
        Some(NodeData::Element { .. }) => {}
        _ => return false,
    }
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

fn nth_child(dom: &Dom, node: NodeId, a: i32, b: i32) -> bool {
    match dom.data(node) {
        Some(NodeData::Element { .. }) => {}
        _ => return false,
    }
    if let Some(parent) = dom.parent(node) {
        let children = dom.children(parent);
        let mut element_index = 0;
        for &child in children {
            if child == node {
                let i = element_index + 1; // 1-indexed
                if a == 0 {
                    return i == b;
                }
                let diff = i - b;
                if a > 0 {
                    return diff >= 0 && diff % a == 0;
                } else {
                    return diff <= 0 && diff % a == 0;
                }
            }
            if matches!(dom.data(child), Some(NodeData::Element { .. })) {
                element_index += 1;
            }
        }
    }
    false
}

fn nth_last_child(dom: &Dom, node: NodeId, a: i32, b: i32) -> bool {
    match dom.data(node) {
        Some(NodeData::Element { .. }) => {}
        _ => return false,
    }
    if let Some(parent) = dom.parent(node) {
        let children = dom.children(parent);
        let mut element_index = 0;
        for &child in children.iter().rev() {
            if child == node {
                let i = element_index + 1; // 1-indexed
                if a == 0 {
                    return i == b;
                }
                let diff = i - b;
                if a > 0 {
                    return diff >= 0 && diff % a == 0;
                } else {
                    return diff <= 0 && diff % a == 0;
                }
            }
            if matches!(dom.data(child), Some(NodeData::Element { .. })) {
                element_index += 1;
            }
        }
    }
    false
}

fn is_first_of_type(dom: &Dom, node: NodeId) -> bool {
    let current_tag_name = match dom.data(node) {
        Some(NodeData::Element { name, .. }) => name,
        _ => return false,
    };
    if let Some(parent) = dom.parent(node) {
        let children = dom.children(parent);
        for &child in children {
            if child == node {
                return true;
            }
            match dom.data(child) {
                Some(NodeData::Element { name, .. })
                    if ascii::eq_ignore_ascii_case(name, current_tag_name) =>
                {
                    return false;
                }
                _ => {}
            }
        }
    }
    false
}

fn is_last_of_type(dom: &Dom, node: NodeId) -> bool {
    let current_tag_name = match dom.data(node) {
        Some(NodeData::Element { name, .. }) => name,
        _ => return false,
    };
    if let Some(parent) = dom.parent(node) {
        let children = dom.children(parent);
        for &child in children.iter().rev() {
            if child == node {
                return true;
            }
            match dom.data(child) {
                Some(NodeData::Element { name, .. })
                    if ascii::eq_ignore_ascii_case(name, current_tag_name) =>
                {
                    return false;
                }
                _ => {}
            }
        }
    }
    false
}

fn is_only_child(dom: &Dom, node: NodeId) -> bool {
    match dom.data(node) {
        Some(NodeData::Element { .. }) => {}
        _ => return false,
    }
    if let Some(parent) = dom.parent(node) {
        let children = dom.children(parent);
        for &child in children {
            if child != node && matches!(dom.data(child), Some(NodeData::Element { .. })) {
                return false;
            }
        }
        true
    } else {
        false
    }
}

fn is_only_of_type(dom: &Dom, node: NodeId) -> bool {
    let current_tag_name = match dom.data(node) {
        Some(NodeData::Element { name, .. }) => name,
        _ => return false,
    };
    if let Some(parent) = dom.parent(node) {
        let children = dom.children(parent);
        for &child in children {
            if child != node {
                match dom.data(child) {
                    Some(NodeData::Element { name, .. })
                        if ascii::eq_ignore_ascii_case(name, current_tag_name) =>
                    {
                        return false;
                    }
                    _ => {}
                }
            }
        }
        true
    } else {
        false
    }
}

fn is_empty(dom: &Dom, node: NodeId) -> bool {
    // Matches an element that has no children at all, OR whose children are all nothing-but-whitespace text nodes.
    // Comment and doctype nodes are ignored per typical CSS selector matching specs.
    match dom.data(node) {
        Some(NodeData::Element { .. }) => {}
        _ => return false,
    }

    let children = dom.children(node);
    for &child in children {
        match dom.data(child) {
            Some(NodeData::Element { .. }) => {
                // Element child means NOT empty
                return false;
            }
            Some(NodeData::Text(s)) if !s.chars().all(ascii::is_html_whitespace) => {
                // Non-whitespace text means NOT empty
                return false;
            }
            _ => {}
        }
    }
    // TODO(spec): check modern vs legacy empty behavior regarding comment nodes or CDATA if applicable, though currently comments are ignored.
    true
}

fn is_root(dom: &Dom, node: NodeId) -> bool {
    // Matches the element that is the root of the document.
    match dom.data(node) {
        Some(NodeData::Element { .. }) => {}
        _ => return false,
    }

    let doc = dom.document();
    let children = dom.children(doc);
    for &child in children {
        if matches!(dom.data(child), Some(NodeData::Element { .. })) {
            return child == node;
        }
    }
    false
}

fn is_link(dom: &Dom, node: NodeId) -> bool {
    match dom.data(node) {
        Some(NodeData::Element { name, attrs }) => {
            let is_target_tag = ascii::eq_ignore_ascii_case(name, "a")
                || ascii::eq_ignore_ascii_case(name, "area")
                || ascii::eq_ignore_ascii_case(name, "link");
            is_target_tag
                && attrs
                    .iter()
                    .any(|(k, _)| ascii::eq_ignore_ascii_case(k, "href"))
        }
        _ => false,
    }
}

fn is_checked(dom: &Dom, node: NodeId) -> bool {
    match dom.data(node) {
        Some(NodeData::Element { name, attrs }) => {
            let is_applicable = ascii::eq_ignore_ascii_case(name, "input")
                || ascii::eq_ignore_ascii_case(name, "option");
            is_applicable
                && attrs
                    .iter()
                    .any(|(k, _)| ascii::eq_ignore_ascii_case(k, "checked"))
        }
        _ => false,
    }
}

fn is_form_associated(name: &str) -> bool {
    ascii::eq_ignore_ascii_case(name, "button")
        || ascii::eq_ignore_ascii_case(name, "input")
        || ascii::eq_ignore_ascii_case(name, "select")
        || ascii::eq_ignore_ascii_case(name, "textarea")
        || ascii::eq_ignore_ascii_case(name, "optgroup")
        || ascii::eq_ignore_ascii_case(name, "option")
        || ascii::eq_ignore_ascii_case(name, "fieldset")
}

fn is_disabled(dom: &Dom, node: NodeId) -> bool {
    match dom.data(node) {
        Some(NodeData::Element { name, attrs }) => {
            is_form_associated(name)
                && attrs
                    .iter()
                    .any(|(k, _)| ascii::eq_ignore_ascii_case(k, "disabled"))
        }
        _ => false,
    }
}

fn is_enabled(dom: &Dom, node: NodeId) -> bool {
    match dom.data(node) {
        Some(NodeData::Element { name, attrs }) => {
            is_form_associated(name)
                && !attrs
                    .iter()
                    .any(|(k, _)| ascii::eq_ignore_ascii_case(k, "disabled"))
        }
        _ => false,
    }
}

fn is_required(dom: &Dom, node: NodeId) -> bool {
    match dom.data(node) {
        Some(NodeData::Element { name, attrs }) => {
            let is_applicable = ascii::eq_ignore_ascii_case(name, "input")
                || ascii::eq_ignore_ascii_case(name, "select")
                || ascii::eq_ignore_ascii_case(name, "textarea");
            is_applicable
                && attrs
                    .iter()
                    .any(|(k, _)| ascii::eq_ignore_ascii_case(k, "required"))
        }
        _ => false,
    }
}

fn is_optional(dom: &Dom, node: NodeId) -> bool {
    match dom.data(node) {
        Some(NodeData::Element { name, attrs }) => {
            let is_applicable = ascii::eq_ignore_ascii_case(name, "input")
                || ascii::eq_ignore_ascii_case(name, "select")
                || ascii::eq_ignore_ascii_case(name, "textarea");
            is_applicable
                && !attrs
                    .iter()
                    .any(|(k, _)| ascii::eq_ignore_ascii_case(k, "required"))
        }
        _ => false,
    }
}

fn is_read_write(dom: &Dom, node: NodeId) -> bool {
    match dom.data(node) {
        Some(NodeData::Element { name, attrs }) => {
            let is_applicable = ascii::eq_ignore_ascii_case(name, "input")
                || ascii::eq_ignore_ascii_case(name, "textarea");
            // TODO(spec): Per HTML/CSS Selectors Level 4, some input types (like checkbox, radio, button, hidden)
            // are technically always read-only or not applicable. This implementation treats any mutable-by-default
            // input/textarea as read-write, leaving the complex per-type table out of scope.
            is_applicable
                && !attrs.iter().any(|(k, _)| {
                    ascii::eq_ignore_ascii_case(k, "readonly")
                        || ascii::eq_ignore_ascii_case(k, "disabled")
                })
        }
        _ => false,
    }
}

fn is_read_only(dom: &Dom, node: NodeId) -> bool {
    match dom.data(node) {
        Some(NodeData::Element { name, attrs }) => {
            let is_applicable = ascii::eq_ignore_ascii_case(name, "input")
                || ascii::eq_ignore_ascii_case(name, "textarea");
            // TODO(spec): General "any non-editable element is :read-only" rule and contenteditable elements
            // are out of scope for this scoped implementation.
            is_applicable
                && attrs.iter().any(|(k, _)| {
                    ascii::eq_ignore_ascii_case(k, "readonly")
                        || ascii::eq_ignore_ascii_case(k, "disabled")
                })
        }
        _ => false,
    }
}

fn get_previous_element_sibling(dom: &Dom, node: NodeId) -> Option<NodeId> {
    let parent = dom.parent(node)?;
    let children = dom.children(parent);
    let idx = children.iter().position(|&id| id == node)?;
    for i in (0..idx).rev() {
        let sibling = children[i];
        if matches!(dom.data(sibling), Some(NodeData::Element { .. })) {
            return Some(sibling);
        }
    }
    None
}

fn eq_ignore_ascii_case(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

fn starts_with_ignore_ascii_case(a: &str, b: &str) -> bool {
    if a.len() < b.len() {
        return false;
    }
    a[..b.len()].eq_ignore_ascii_case(b)
}

fn ends_with_ignore_ascii_case(a: &str, b: &str) -> bool {
    if a.len() < b.len() {
        return false;
    }
    a[a.len() - b.len()..].eq_ignore_ascii_case(b)
}

fn contains_ignore_ascii_case(a: &str, b: &str) -> bool {
    if b.is_empty() {
        return true;
    }
    if a.len() < b.len() {
        return false;
    }
    let b_lower = b.to_ascii_lowercase();
    a.to_ascii_lowercase().contains(&b_lower)
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

        set_node_state(
            p,
            NodeState {
                hover: true,
                focus: false,
                active: false,
            },
        );

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

    #[test]
    fn test_case_insensitive_attributes() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let element = dom.create_node(NodeData::Element {
            name: "input".into(),
            attrs: vec![
                ("type".into(), "text".into()),
                ("class".into(), "Foo Bar".into()),
                ("lang".into(), "en-US".into()),
            ],
        });
        dom.append_child(doc, element);

        // Case-sensitive (default) vs Case-insensitive (i modifier)
        assert!(matches(
            &parse_selector_list("[type=\"text\"]").unwrap(),
            &dom,
            element
        ));
        assert!(!matches(
            &parse_selector_list("[type=\"TEXT\"]").unwrap(),
            &dom,
            element
        ));
        assert!(matches(
            &parse_selector_list("[type=\"TEXT\" i]").unwrap(),
            &dom,
            element
        ));
        assert!(!matches(
            &parse_selector_list("[type=\"TEXT\" s]").unwrap(),
            &dom,
            element
        ));

        // prefix
        assert!(matches(
            &parse_selector_list("[type^=\"TE\" i]").unwrap(),
            &dom,
            element
        ));
        // suffix
        assert!(matches(
            &parse_selector_list("[type$=\"XT\" i]").unwrap(),
            &dom,
            element
        ));
        // substring
        assert!(matches(
            &parse_selector_list("[type*=\"EX\" i]").unwrap(),
            &dom,
            element
        ));
        // dashmatch
        assert!(matches(
            &parse_selector_list("[lang|=\"EN\" i]").unwrap(),
            &dom,
            element
        ));
        // includes
        assert!(matches(
            &parse_selector_list("[class~=\"foo\" i]").unwrap(),
            &dom,
            element
        ));
    }

    #[test]
    fn test_nth_of_type() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let parent = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, parent);

        // Children structure: span1, p1, span2, p2
        let span1 = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![],
        });
        let p1 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        let span2 = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![],
        });
        let p2 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });

        dom.append_child(parent, span1);
        dom.append_child(parent, p1);
        dom.append_child(parent, span2);
        dom.append_child(parent, p2);

        // nth-of-type(2)
        assert!(!matches(
            &parse_selector_list("p:nth-of-type(2)").unwrap(),
            &dom,
            p1
        ));
        assert!(matches(
            &parse_selector_list("p:nth-of-type(2)").unwrap(),
            &dom,
            p2
        ));
        assert!(!matches(
            &parse_selector_list("span:nth-of-type(2)").unwrap(),
            &dom,
            span1
        ));
        assert!(matches(
            &parse_selector_list("span:nth-of-type(2)").unwrap(),
            &dom,
            span2
        ));
    }

    #[test]
    fn test_nth_last_child() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let parent = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, parent);

        let p1 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        let p2 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        let p3 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });

        dom.append_child(parent, p1);
        dom.append_child(parent, p2);
        dom.append_child(parent, p3);

        // nth-last-child(1) matches the last child (p3)
        assert!(matches(
            &parse_selector_list("p:nth-last-child(1)").unwrap(),
            &dom,
            p3
        ));
        assert!(!matches(
            &parse_selector_list("p:nth-last-child(1)").unwrap(),
            &dom,
            p2
        ));

        // nth-last-child(2) matches the second to last child (p2)
        assert!(matches(
            &parse_selector_list("p:nth-last-child(2)").unwrap(),
            &dom,
            p2
        ));
    }

    #[test]
    fn test_nth_last_of_type() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let parent = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, parent);

        // Children structure: span1, p1, span2, p2, p3
        let span1 = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![],
        });
        let p1 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        let span2 = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![],
        });
        let p2 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        let p3 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });

        dom.append_child(parent, span1);
        dom.append_child(parent, p1);
        dom.append_child(parent, span2);
        dom.append_child(parent, p2);
        dom.append_child(parent, p3);

        // p:nth-last-of-type(1) -> last p (p3)
        assert!(matches(
            &parse_selector_list("p:nth-last-of-type(1)").unwrap(),
            &dom,
            p3
        ));
        assert!(!matches(
            &parse_selector_list("p:nth-last-of-type(1)").unwrap(),
            &dom,
            p2
        ));

        // p:nth-last-of-type(2) -> second to last p (p2)
        assert!(matches(
            &parse_selector_list("p:nth-last-of-type(2)").unwrap(),
            &dom,
            p2
        ));
        assert!(!matches(
            &parse_selector_list("p:nth-last-of-type(2)").unwrap(),
            &dom,
            p1
        ));

        // span:nth-last-of-type(1) -> last span (span2)
        assert!(matches(
            &parse_selector_list("span:nth-last-of-type(1)").unwrap(),
            &dom,
            span2
        ));
        assert!(!matches(
            &parse_selector_list("span:nth-last-of-type(1)").unwrap(),
            &dom,
            span1
        ));

        // an+b form: e.g. 2n+1 (odd elements from the end: p3 (1st from end), p1 (3rd from end))
        let odd_selector = parse_selector_list("p:nth-last-of-type(2n+1)").unwrap();
        assert!(matches(&odd_selector, &dom, p3)); // index 1 from end
        assert!(!matches(&odd_selector, &dom, p2)); // index 2 from end
        assert!(matches(&odd_selector, &dom, p1)); // index 3 from end
    }

    #[test]
    fn test_namespace_passthrough() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let rect = dom.create_node(NodeData::Element {
            name: "rect".into(),
            attrs: vec![],
        });
        dom.append_child(doc, rect);

        // svg|rect should match rect (ignoring svg prefix)
        assert!(matches(
            &parse_selector_list("svg|rect").unwrap(),
            &dom,
            rect
        ));
        // *|rect should match rect
        assert!(matches(&parse_selector_list("*|rect").unwrap(), &dom, rect));
        // |rect should match rect
        assert!(matches(&parse_selector_list("|rect").unwrap(), &dom, rect));
        // svg|* should match any element (like rect)
        assert!(matches(&parse_selector_list("svg|*").unwrap(), &dom, rect));
    }

    #[test]
    fn test_robustness_spec_53() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // Setup DOM:
        // <body>
        //   <div class="x">
        //     <p id="p1-1"></p>
        //     <p id="p1-2"></p>
        //   </div>
        //   <div>
        //     <p id="p2-1"></p>
        //     <!-- comment -->
        //     "   whitespace text node   "
        //     <p id="p2-2"></p>
        //     <p id="p2-3"></p>
        //   </div>
        // </body>
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let div1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "x".into())],
        });
        dom.append_child(body, div1);

        let p1_1 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![("id".into(), "p1-1".into())],
        });
        let p1_2 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![("id".into(), "p1-2".into())],
        });
        dom.append_child(div1, p1_1);
        dom.append_child(div1, p1_2);

        let div2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, div2);

        let p2_1 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![("id".into(), "p2-1".into())],
        });
        let comment = dom.create_node(NodeData::Comment("comment".into()));
        let text = dom.create_node(NodeData::Text("   \n   ".into()));
        let p2_2 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![("id".into(), "p2-2".into())],
        });
        let p2_3 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![("id".into(), "p2-3".into())],
        });
        dom.append_child(div2, p2_1);
        dom.append_child(div2, comment);
        dom.append_child(div2, text);
        dom.append_child(div2, p2_2);
        dom.append_child(div2, p2_3);

        // 1. Selector "div:not(.x) > p:nth-of-type(2)"
        let selector = parse_selector_list("div:not(.x) > p:nth-of-type(2)").unwrap();
        // Should match p2_2
        assert!(matches(&selector, &dom, p2_2));
        // Should NOT match p1_2 (parent has class .x)
        assert!(!matches(&selector, &dom, p1_2));
        // Should NOT match p2_1 (first of type) or p2_3 (third of type)
        assert!(!matches(&selector, &dom, p2_1));
        assert!(!matches(&selector, &dom, p2_3));

        // 2. NextSibling (+) ignoring comments & text nodes: "p + p"
        // In div2, the elements are p2_1, and p2_2.
        // Between them are a Comment and Text node, but they are adjacent element siblings!
        let next_sibling_sel = parse_selector_list("p + p").unwrap();
        assert!(matches(&next_sibling_sel, &dom, p2_2));
        assert!(matches(&next_sibling_sel, &dom, p2_3));
        assert!(!matches(&next_sibling_sel, &dom, p2_1));

        // 3. SubsequentSibling (~) ignoring comments & text nodes: "p ~ p"
        let subsequent_sibling_sel = parse_selector_list("p ~ p").unwrap();
        assert!(matches(&subsequent_sibling_sel, &dom, p2_2));
        assert!(matches(&subsequent_sibling_sel, &dom, p2_3));
        assert!(!matches(&subsequent_sibling_sel, &dom, p2_1));

        // 4. Robustness against nonexistent / stale NodeId
        let mut another_dom = Dom::new();
        let stale_node = another_dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        // Passing stale_node into `matches` against `dom` should safely return false without any panic
        assert!(!matches(&selector, &dom, stale_node));
    }

    #[test]
    fn test_state_pseudo_classes() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let el = dom.create_node(NodeData::Element {
            name: "button".into(),
            attrs: vec![],
        });
        dom.append_child(doc, el);

        // Initially all are false
        set_node_state(el, NodeState::default());
        assert!(!matches(
            &parse_selector_list("button:hover").unwrap(),
            &dom,
            el
        ));
        assert!(!matches(
            &parse_selector_list("button:focus").unwrap(),
            &dom,
            el
        ));
        assert!(!matches(
            &parse_selector_list("button:active").unwrap(),
            &dom,
            el
        ));

        // Set hover
        set_node_state(
            el,
            NodeState {
                hover: true,
                focus: false,
                active: false,
            },
        );
        assert!(matches(
            &parse_selector_list("button:hover").unwrap(),
            &dom,
            el
        ));
        assert!(!matches(
            &parse_selector_list("button:focus").unwrap(),
            &dom,
            el
        ));

        // Set focus
        set_node_state(
            el,
            NodeState {
                hover: false,
                focus: true,
                active: false,
            },
        );
        assert!(!matches(
            &parse_selector_list("button:hover").unwrap(),
            &dom,
            el
        ));
        assert!(matches(
            &parse_selector_list("button:focus").unwrap(),
            &dom,
            el
        ));

        // Set active
        set_node_state(
            el,
            NodeState {
                hover: false,
                focus: false,
                active: true,
            },
        );
        assert!(!matches(
            &parse_selector_list("button:hover").unwrap(),
            &dom,
            el
        ));
        assert!(matches(
            &parse_selector_list("button:active").unwrap(),
            &dom,
            el
        ));
    }

    #[test]
    fn test_form_state_pseudo_classes() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // <input checked>
        let input_checked = dom.create_node(NodeData::Element {
            name: "input".into(),
            attrs: vec![("checked".into(), "".into())],
        });
        dom.append_child(doc, input_checked);

        // <input>
        let input_unchecked = dom.create_node(NodeData::Element {
            name: "input".into(),
            attrs: vec![],
        });
        dom.append_child(doc, input_unchecked);

        // <option checked>
        let option_checked = dom.create_node(NodeData::Element {
            name: "option".into(),
            attrs: vec![("checked".into(), "".into())],
        });
        dom.append_child(doc, option_checked);

        // <option>
        let option_unchecked = dom.create_node(NodeData::Element {
            name: "option".into(),
            attrs: vec![],
        });
        dom.append_child(doc, option_unchecked);

        // <div checked> (div is not input/option-like, so should not match :checked)
        let div_checked = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("checked".into(), "".into())],
        });
        dom.append_child(doc, div_checked);

        // <button disabled>
        let button_disabled = dom.create_node(NodeData::Element {
            name: "button".into(),
            attrs: vec![("disabled".into(), "".into())],
        });
        dom.append_child(doc, button_disabled);

        // <button>
        let button_enabled = dom.create_node(NodeData::Element {
            name: "button".into(),
            attrs: vec![],
        });
        dom.append_child(doc, button_enabled);

        // <div disabled> (div is not form-associated, so should match neither :disabled nor :enabled)
        let div_disabled = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("disabled".into(), "".into())],
        });
        dom.append_child(doc, div_disabled);

        // <div>
        let div_normal = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div_normal);

        // Test :checked
        assert!(matches(
            &parse_selector_list(":checked").unwrap(),
            &dom,
            input_checked
        ));
        assert!(!matches(
            &parse_selector_list(":checked").unwrap(),
            &dom,
            input_unchecked
        ));
        assert!(matches(
            &parse_selector_list(":checked").unwrap(),
            &dom,
            option_checked
        ));
        assert!(!matches(
            &parse_selector_list(":checked").unwrap(),
            &dom,
            option_unchecked
        ));
        assert!(!matches(
            &parse_selector_list(":checked").unwrap(),
            &dom,
            div_checked
        ));

        // Test :disabled
        assert!(matches(
            &parse_selector_list(":disabled").unwrap(),
            &dom,
            button_disabled
        ));
        assert!(!matches(
            &parse_selector_list(":disabled").unwrap(),
            &dom,
            button_enabled
        ));
        assert!(!matches(
            &parse_selector_list(":disabled").unwrap(),
            &dom,
            div_disabled
        ));
        assert!(!matches(
            &parse_selector_list(":disabled").unwrap(),
            &dom,
            div_normal
        ));

        // Test :enabled
        assert!(!matches(
            &parse_selector_list(":enabled").unwrap(),
            &dom,
            button_disabled
        ));
        assert!(matches(
            &parse_selector_list(":enabled").unwrap(),
            &dom,
            button_enabled
        ));
        assert!(!matches(
            &parse_selector_list(":enabled").unwrap(),
            &dom,
            div_disabled
        ));
        assert!(!matches(
            &parse_selector_list(":enabled").unwrap(),
            &dom,
            div_normal
        ));

        // Test form-associated elements are matched properly with mixed case tags and mixed case attributes
        let input_mixed_checked = dom.create_node(NodeData::Element {
            name: "InPuT".into(),
            attrs: vec![("ChEcKeD".into(), "true".into())],
        });
        dom.append_child(doc, input_mixed_checked);

        let select_mixed_disabled = dom.create_node(NodeData::Element {
            name: "SeLeCt".into(),
            attrs: vec![("DiSaBlEd".into(), "true".into())],
        });
        dom.append_child(doc, select_mixed_disabled);

        assert!(matches(
            &parse_selector_list(":checked").unwrap(),
            &dom,
            input_mixed_checked
        ));
        assert!(matches(
            &parse_selector_list(":disabled").unwrap(),
            &dom,
            select_mixed_disabled
        ));
        assert!(!matches(
            &parse_selector_list(":enabled").unwrap(),
            &dom,
            select_mixed_disabled
        ));

        // Test :required and :optional
        // <input required>
        let input_required = dom.create_node(NodeData::Element {
            name: "input".into(),
            attrs: vec![("required".into(), "".into())],
        });
        dom.append_child(doc, input_required);

        // <input> (no required)
        let input_not_required = dom.create_node(NodeData::Element {
            name: "input".into(),
            attrs: vec![],
        });
        dom.append_child(doc, input_not_required);

        // <select required>
        let select_required = dom.create_node(NodeData::Element {
            name: "select".into(),
            attrs: vec![("required".into(), "".into())],
        });
        dom.append_child(doc, select_required);

        // <textarea>
        let textarea_optional = dom.create_node(NodeData::Element {
            name: "textarea".into(),
            attrs: vec![],
        });
        dom.append_child(doc, textarea_optional);

        // <div required>
        let div_required = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("required".into(), "".into())],
        });
        dom.append_child(doc, div_required);

        // Tests
        assert!(matches(
            &parse_selector_list("input:required").unwrap(),
            &dom,
            input_required
        ));
        assert!(!matches(
            &parse_selector_list("input:optional").unwrap(),
            &dom,
            input_required
        ));

        assert!(matches(
            &parse_selector_list("input:optional").unwrap(),
            &dom,
            input_not_required
        ));
        assert!(!matches(
            &parse_selector_list("input:required").unwrap(),
            &dom,
            input_not_required
        ));

        assert!(matches(
            &parse_selector_list(":required").unwrap(),
            &dom,
            select_required
        ));
        assert!(!matches(
            &parse_selector_list(":optional").unwrap(),
            &dom,
            select_required
        ));

        assert!(matches(
            &parse_selector_list(":optional").unwrap(),
            &dom,
            textarea_optional
        ));
        assert!(!matches(
            &parse_selector_list(":required").unwrap(),
            &dom,
            textarea_optional
        ));

        assert!(!matches(
            &parse_selector_list(":required").unwrap(),
            &dom,
            div_required
        ));
        assert!(!matches(
            &parse_selector_list(":optional").unwrap(),
            &dom,
            div_required
        ));

        // Mixed case and attribute spelling checks
        let input_mixed_required = dom.create_node(NodeData::Element {
            name: "InPuT".into(),
            attrs: vec![("ReQuIrEd".into(), "true".into())],
        });
        dom.append_child(doc, input_mixed_required);

        assert!(matches(
            &parse_selector_list(":required").unwrap(),
            &dom,
            input_mixed_required
        ));
        assert!(!matches(
            &parse_selector_list(":optional").unwrap(),
            &dom,
            input_mixed_required
        ));

        // Test :read-only and :read-write
        // <input>
        let input_rw = dom.create_node(NodeData::Element {
            name: "input".into(),
            attrs: vec![],
        });
        dom.append_child(doc, input_rw);

        // <input readonly>
        let input_ro_readonly = dom.create_node(NodeData::Element {
            name: "input".into(),
            attrs: vec![("readonly".into(), "".into())],
        });
        dom.append_child(doc, input_ro_readonly);

        // <input disabled>
        let input_ro_disabled = dom.create_node(NodeData::Element {
            name: "input".into(),
            attrs: vec![("disabled".into(), "".into())],
        });
        dom.append_child(doc, input_ro_disabled);

        // <textarea>
        let textarea_rw = dom.create_node(NodeData::Element {
            name: "textarea".into(),
            attrs: vec![],
        });
        dom.append_child(doc, textarea_rw);

        // <div>
        let div_ro_rw_none = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div_ro_rw_none);

        // <input READONLY>
        let input_mixed_readonly = dom.create_node(NodeData::Element {
            name: "InPuT".into(),
            attrs: vec![("READONLY".into(), "".into())],
        });
        dom.append_child(doc, input_mixed_readonly);

        // Assertions for :read-write
        assert!(matches(
            &parse_selector_list(":read-write").unwrap(),
            &dom,
            input_rw
        ));
        assert!(!matches(
            &parse_selector_list(":read-only").unwrap(),
            &dom,
            input_rw
        ));

        assert!(matches(
            &parse_selector_list(":read-only").unwrap(),
            &dom,
            input_ro_readonly
        ));
        assert!(!matches(
            &parse_selector_list(":read-write").unwrap(),
            &dom,
            input_ro_readonly
        ));

        assert!(matches(
            &parse_selector_list(":read-only").unwrap(),
            &dom,
            input_ro_disabled
        ));
        assert!(!matches(
            &parse_selector_list(":read-write").unwrap(),
            &dom,
            input_ro_disabled
        ));

        assert!(matches(
            &parse_selector_list(":read-write").unwrap(),
            &dom,
            textarea_rw
        ));
        assert!(!matches(
            &parse_selector_list(":read-only").unwrap(),
            &dom,
            textarea_rw
        ));

        assert!(!matches(
            &parse_selector_list(":read-only").unwrap(),
            &dom,
            div_ro_rw_none
        ));
        assert!(!matches(
            &parse_selector_list(":read-write").unwrap(),
            &dom,
            div_ro_rw_none
        ));

        assert!(matches(
            &parse_selector_list(":read-only").unwrap(),
            &dom,
            input_mixed_readonly
        ));
        assert!(!matches(
            &parse_selector_list(":read-write").unwrap(),
            &dom,
            input_mixed_readonly
        ));
    }

    #[test]
    fn test_structural_pseudo_classes() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let parent = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, parent);

        // 1. Only child / Only of type
        let p1 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(parent, p1);

        assert!(matches(
            &parse_selector_list("p:first-of-type").unwrap(),
            &dom,
            p1
        ));
        assert!(matches(
            &parse_selector_list("p:last-of-type").unwrap(),
            &dom,
            p1
        ));
        assert!(matches(
            &parse_selector_list("p:only-child").unwrap(),
            &dom,
            p1
        ));
        assert!(matches(
            &parse_selector_list("p:only-of-type").unwrap(),
            &dom,
            p1
        ));

        // 2. Add another p of the same type (now p1, p2)
        let p2 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(parent, p2);

        assert!(matches(
            &parse_selector_list("p:first-of-type").unwrap(),
            &dom,
            p1
        ));
        assert!(!matches(
            &parse_selector_list("p:first-of-type").unwrap(),
            &dom,
            p2
        ));

        assert!(!matches(
            &parse_selector_list("p:last-of-type").unwrap(),
            &dom,
            p1
        ));
        assert!(matches(
            &parse_selector_list("p:last-of-type").unwrap(),
            &dom,
            p2
        ));

        assert!(!matches(
            &parse_selector_list("p:only-child").unwrap(),
            &dom,
            p1
        ));
        assert!(!matches(
            &parse_selector_list("p:only-of-type").unwrap(),
            &dom,
            p1
        ));

        // 3. Add a span (now p1, p2, span1)
        let span1 = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![],
        });
        dom.append_child(parent, span1);

        assert!(matches(
            &parse_selector_list("span:first-of-type").unwrap(),
            &dom,
            span1
        ));
        assert!(matches(
            &parse_selector_list("span:last-of-type").unwrap(),
            &dom,
            span1
        ));
        assert!(matches(
            &parse_selector_list("span:only-of-type").unwrap(),
            &dom,
            span1
        ));
        assert!(!matches(
            &parse_selector_list("span:only-child").unwrap(),
            &dom,
            span1
        ));

        // 4. Test :empty
        // empty element (span1 currently has no children)
        assert!(matches(
            &parse_selector_list("span:empty").unwrap(),
            &dom,
            span1
        ));

        // add whitespace-only text node to span1
        let text_ws = dom.create_node(NodeData::Text("   \n\t ".into()));
        dom.append_child(span1, text_ws);
        // whitespace-only text nodes are ignored per typical empty behavior, so span1 is still empty!
        assert!(matches(
            &parse_selector_list("span:empty").unwrap(),
            &dom,
            span1
        ));

        // add a non-whitespace text node to span1
        let text_non_ws = dom.create_node(NodeData::Text("hello".into()));
        dom.append_child(span1, text_non_ws);
        // now span1 is NOT empty
        assert!(!matches(
            &parse_selector_list("span:empty").unwrap(),
            &dom,
            span1
        ));

        // Let's test with a child element
        // parent (div) has children p1, p2, span1, so it is NOT empty
        assert!(!matches(
            &parse_selector_list("div:empty").unwrap(),
            &dom,
            parent
        ));

        // 5. Test :root
        // parent is the first Element child of doc, so it is the document root element!
        assert!(matches(
            &parse_selector_list(":root").unwrap(),
            &dom,
            parent
        ));
        assert!(!matches(&parse_selector_list(":root").unwrap(), &dom, p1));
        assert!(!matches(&parse_selector_list("p:root").unwrap(), &dom, p1));
    }

    #[test]
    fn test_child_structural_pseudo_classes() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let parent = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, parent);

        // mixed children: <h1>, <p>, <p>, <span>
        let h1 = dom.create_node(NodeData::Element {
            name: "h1".into(),
            attrs: vec![],
        });
        let p1 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        let p2 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        let span1 = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![],
        });

        dom.append_child(parent, h1);
        dom.append_child(parent, p1);
        dom.append_child(parent, p2);
        dom.append_child(parent, span1);

        // 1. :first-child and :last-child select first/last element child regardless of tag
        assert!(matches(
            &parse_selector_list(":first-child").unwrap(),
            &dom,
            h1
        ));
        assert!(!matches(
            &parse_selector_list(":first-child").unwrap(),
            &dom,
            p1
        ));
        assert!(matches(
            &parse_selector_list(":last-child").unwrap(),
            &dom,
            span1
        ));
        assert!(!matches(
            &parse_selector_list(":last-child").unwrap(),
            &dom,
            p2
        ));

        // 2. p:nth-child(2) matches a p only when it is the 2nd element child overall (which is p1)
        // Note: element children are [h1, p1, p2, span1] (1-indexed: 1, 2, 3, 4)
        assert!(matches(
            &parse_selector_list("p:nth-child(2)").unwrap(),
            &dom,
            p1
        ));
        // p2 is the 3rd element child, so it does NOT match p:nth-child(2)
        assert!(!matches(
            &parse_selector_list("p:nth-child(2)").unwrap(),
            &dom,
            p2
        ));
        // h1 is the 1st element child, so it does NOT match :nth-child(2)
        assert!(!matches(
            &parse_selector_list(":nth-child(2)").unwrap(),
            &dom,
            h1
        ));

        // 3. :nth-child(odd) / :nth-child(2n) select the expected positions
        // odd indices: 1 (h1), 3 (p2)
        // even indices (2n): 2 (p1), 4 (span1)
        let odd_sel = parse_selector_list(":nth-child(odd)").unwrap();
        assert!(matches(&odd_sel, &dom, h1));
        assert!(!matches(&odd_sel, &dom, p1));
        assert!(matches(&odd_sel, &dom, p2));
        assert!(!matches(&odd_sel, &dom, span1));

        let even_sel = parse_selector_list(":nth-child(even)").unwrap();
        assert!(!matches(&even_sel, &dom, h1));
        assert!(matches(&even_sel, &dom, p1));
        assert!(!matches(&even_sel, &dom, p2));
        assert!(matches(&even_sel, &dom, span1));

        // 4. :nth-last-child(1) equals :last-child
        assert!(matches(
            &parse_selector_list(":nth-last-child(1)").unwrap(),
            &dom,
            span1
        ));
        assert!(!matches(
            &parse_selector_list(":nth-last-child(1)").unwrap(),
            &dom,
            p2
        ));
    }

    #[test]
    fn test_matches_is_where() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // Create a basic DOM structure for testing:
        // <div>
        //   <h1 class="title">Header 1</h1>
        //   <h2>Header 2</h2>
        //   <p class="title">Paragraph 1</p>
        //   <p>Paragraph 2</p>
        //   <a class="x">Link 1</a>
        //   <a class="z">Link 2</a>
        // </div>
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let h1 = dom.create_node(NodeData::Element {
            name: "h1".into(),
            attrs: vec![("class".into(), "title".into())],
        });
        dom.append_child(div, h1);

        let h2 = dom.create_node(NodeData::Element {
            name: "h2".into(),
            attrs: vec![],
        });
        dom.append_child(div, h2);

        let p1 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![("class".into(), "title".into())],
        });
        dom.append_child(div, p1);

        let p2 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(div, p2);

        let a1 = dom.create_node(NodeData::Element {
            name: "a".into(),
            attrs: vec![("class".into(), "x".into())],
        });
        dom.append_child(div, a1);

        let a2 = dom.create_node(NodeData::Element {
            name: "a".into(),
            attrs: vec![("class".into(), "z".into())],
        });
        dom.append_child(div, a2);

        // 1. :is(h1, h2, .title) matches h1, h2, p1 but not p2
        let sel_is = parse_selector_list(":is(h1, h2, .title)").unwrap();
        assert!(matches(&sel_is, &dom, h1));
        assert!(matches(&sel_is, &dom, h2));
        assert!(matches(&sel_is, &dom, p1));
        assert!(!matches(&sel_is, &dom, p2));

        // 2. :where(h1, h2, .title) matches identically
        let sel_where = parse_selector_list(":where(h1, h2, .title)").unwrap();
        assert!(matches(&sel_where, &dom, h1));
        assert!(matches(&sel_where, &dom, h2));
        assert!(matches(&sel_where, &dom, p1));
        assert!(!matches(&sel_where, &dom, p2));

        // 3. combined with compound: a:is(.x, .y) matches a1 but not a2 or div
        let sel_a_is = parse_selector_list("a:is(.x, .y)").unwrap();
        assert!(matches(&sel_a_is, &dom, a1));
        assert!(!matches(&sel_a_is, &dom, a2));
        assert!(!matches(&sel_a_is, &dom, div));

        // 4. combined with combinators: div :is(p, span) matches p1, p2 but not h1
        let sel_div_is = parse_selector_list("div :is(p, span)").unwrap();
        assert!(matches(&sel_div_is, &dom, p1));
        assert!(matches(&sel_div_is, &dom, p2));
        assert!(!matches(&sel_div_is, &dom, h1));

        // 5. :not(:is(h1, h2)) matches p1, p2, a1, a2 but not h1, h2
        let sel_not_is = parse_selector_list(":not(:is(h1, h2))").unwrap();
        assert!(matches(&sel_not_is, &dom, p1));
        assert!(matches(&sel_not_is, &dom, p2));
        assert!(matches(&sel_not_is, &dom, a1));
        assert!(matches(&sel_not_is, &dom, a2));
        assert!(!matches(&sel_not_is, &dom, h1));
        assert!(!matches(&sel_not_is, &dom, h2));
    }

    #[test]
    fn test_matches_link() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // <a href="x">t</a>
        let a_with_href = dom.create_node(NodeData::Element {
            name: "a".into(),
            attrs: vec![("href".into(), "x".into())],
        });
        dom.append_child(doc, a_with_href);

        // <area href="x">
        let area_with_href = dom.create_node(NodeData::Element {
            name: "area".into(),
            attrs: vec![("href".into(), "x".into())],
        });
        dom.append_child(doc, area_with_href);

        // <link href="x">
        let link_with_href = dom.create_node(NodeData::Element {
            name: "link".into(),
            attrs: vec![("href".into(), "x".into())],
        });
        dom.append_child(doc, link_with_href);

        // <a>t</a> (no href)
        let a_no_href = dom.create_node(NodeData::Element {
            name: "a".into(),
            attrs: vec![],
        });
        dom.append_child(doc, a_no_href);

        // <div href="x">t</div>
        let div_with_href = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("href".into(), "x".into())],
        });
        dom.append_child(doc, div_with_href);

        // Matches :link
        let sel_link = parse_selector_list(":link").unwrap();
        assert!(matches(&sel_link, &dom, a_with_href));
        assert!(matches(&sel_link, &dom, area_with_href));
        assert!(matches(&sel_link, &dom, link_with_href));
        assert!(!matches(&sel_link, &dom, a_no_href));
        assert!(!matches(&sel_link, &dom, div_with_href));

        // Matches :any-link
        let sel_any_link = parse_selector_list(":any-link").unwrap();
        assert!(matches(&sel_any_link, &dom, a_with_href));
        assert!(matches(&sel_any_link, &dom, area_with_href));
        assert!(matches(&sel_any_link, &dom, link_with_href));
        assert!(!matches(&sel_any_link, &dom, a_no_href));
        assert!(!matches(&sel_any_link, &dom, div_with_href));

        // Compound selector a:link
        let sel_a_link = parse_selector_list("a:link").unwrap();
        assert!(matches(&sel_a_link, &dom, a_with_href));
        assert!(!matches(&sel_a_link, &dom, a_no_href));
        assert!(!matches(&sel_a_link, &dom, area_with_href));
    }
}
