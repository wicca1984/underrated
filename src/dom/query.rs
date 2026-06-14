use crate::dom::{Dom, NodeData};
use crate::infra::NodeId;
use crate::selector;

impl Dom {
    /// Returns true if `other` is an inclusive descendant of `node`
    /// (i.e. `other` is `node` itself, or a descendant of `node`).
    // spec: https://dom.spec.whatwg.org/#dom-node-contains
    pub fn contains(&self, node: NodeId, other: NodeId) -> bool {
        let mut curr = Some(other);
        while let Some(curr_node) = curr {
            if curr_node == node {
                return true;
            }
            curr = self.parent(curr_node);
        }
        false
    }

    /// Returns the first element in the document with the given `id`.
    // spec: https://dom.spec.whatwg.org/#dom-nonelementparentnode-getelementbyid
    pub fn get_element_by_id(&self, id: &str) -> Option<NodeId> {
        // Document order (pre-order) traversal.
        // We include the document root itself, although it won't match an ID attribute.
        std::iter::once(self.document())
            .chain(self.descendants_iter(self.document()))
            .find(|&node_id| {
                if let Some(NodeData::Element { attrs, .. }) = self.data(node_id) {
                    attrs.iter().any(|(n, v)| n == "id" && v == id)
                } else {
                    false
                }
            })
    }

    /// Returns the first descendant of the document root that matches the given `selector`.
    // spec: https://dom.spec.whatwg.org/#dom-parentnode-queryselector
    pub fn query_selector(&self, selector: &str) -> Option<NodeId> {
        self.query_selector_from(self.document(), selector)
    }

    /// Returns all descendants of the document root that match the given `selector` in document order.
    // spec: https://dom.spec.whatwg.org/#dom-parentnode-queryselectorall
    pub fn query_selector_all(&self, selector: &str) -> Vec<NodeId> {
        self.query_selector_all_from(self.document(), selector)
    }

    /// Returns the first descendant of the given `root` node that matches the given `selector`.
    pub fn query_selector_from(&self, root: NodeId, selector: &str) -> Option<NodeId> {
        let selector_list = match self.parse_scoped_selector(selector) {
            Ok(list) => list,
            Err(_) => return None,
        };

        self.descendants_iter(root)
            .find(|&node_id| matches_with_scope(&selector_list, self, node_id, root))
    }

    /// Returns all descendants of the given `root` node that match the given `selector` in document order.
    pub fn query_selector_all_from(&self, root: NodeId, selector: &str) -> Vec<NodeId> {
        let selector_list = match self.parse_scoped_selector(selector) {
            Ok(list) => list,
            Err(_) => return Vec::new(),
        };

        self.descendants_iter(root)
            .filter(|&node_id| matches_with_scope(&selector_list, self, node_id, root))
            .collect()
    }

    fn parse_scoped_selector(
        &self,
        selector: &str,
    ) -> Result<selector::SelectorList, selector::SelectorParseError> {
        let preprocessed = preprocess_relative_selector(selector);
        selector::parse_selector_list(&preprocessed)
    }

    /// Returns the first following sibling of the given `node` that is an element.
    // spec: https://dom.spec.whatwg.org/#dom-nondocumenttypechildnode-nextelementsibling
    pub fn next_element_sibling(&self, node: NodeId) -> Option<NodeId> {
        let parent = self.parent(node)?;
        let children = self.children(parent);
        let pos = children.iter().position(|&id| id == node)?;

        children
            .get(pos + 1..)?
            .iter()
            .copied()
            .find(|&sibling_id| matches!(self.data(sibling_id), Some(NodeData::Element { .. })))
    }

    /// Returns the nearest preceding sibling of the given `node` that is an element.
    // spec: https://dom.spec.whatwg.org/#dom-nondocumenttypechildnode-previouselementsibling
    pub fn previous_element_sibling(&self, node: NodeId) -> Option<NodeId> {
        let parent = self.parent(node)?;
        let children = self.children(parent);
        let pos = children.iter().position(|&id| id == node)?;

        children
            .get(..pos)?
            .iter()
            .copied()
            .rev()
            .find(|&sibling_id| matches!(self.data(sibling_id), Some(NodeData::Element { .. })))
    }

    /// Returns the first child of the given `node` that is an element.
    // spec: https://dom.spec.whatwg.org/#dom-parentnode-firstelementchild
    pub fn first_element_child(&self, node: NodeId) -> Option<NodeId> {
        self.children(node)
            .iter()
            .copied()
            .find(|&child_id| matches!(self.data(child_id), Some(NodeData::Element { .. })))
    }

    /// Returns the last child of the given `node` that is an element.
    // spec: https://dom.spec.whatwg.org/#dom-parentnode-lastelementchild
    pub fn last_element_child(&self, node: NodeId) -> Option<NodeId> {
        self.children(node)
            .iter()
            .copied()
            .rev()
            .find(|&child_id| matches!(self.data(child_id), Some(NodeData::Element { .. })))
    }

    /// Returns the number of child nodes of the given `node` that are elements.
    // spec: https://dom.spec.whatwg.org/#dom-parentnode-childelementcount
    pub fn child_element_count(&self, node: NodeId) -> usize {
        self.children(node)
            .iter()
            .filter(|&&child_id| matches!(self.data(child_id), Some(NodeData::Element { .. })))
            .count()
    }

    /// Returns a list of child nodes of the given `node` that are elements.
    // spec: https://dom.spec.whatwg.org/#dom-parentnode-children
    pub fn child_elements(&self, node: NodeId) -> Vec<NodeId> {
        self.children(node)
            .iter()
            .copied()
            .filter(|&child_id| matches!(self.data(child_id), Some(NodeData::Element { .. })))
            .collect()
    }

    /// Returns true if the element matches the given `selector`.
    // spec: https://dom.spec.whatwg.org/#dom-element-matches
    pub fn matches(&self, node: NodeId, selector: &str) -> bool {
        let selector_list = match self.parse_scoped_selector(selector) {
            Ok(list) => list,
            Err(_) => return false,
        };
        matches_with_scope(&selector_list, self, node, node)
    }

    /// Returns the closest ancestor of the given `node` (including `node` itself)
    /// that matches the given `selector`.
    // spec: https://dom.spec.whatwg.org/#dom-element-closest
    pub fn closest(&self, node: NodeId, selector: &str) -> Option<NodeId> {
        let selector_list = match self.parse_scoped_selector(selector) {
            Ok(list) => list,
            Err(_) => return None,
        };
        let mut curr = Some(node);
        while let Some(curr_node) = curr {
            if matches_with_scope(&selector_list, self, curr_node, node) {
                return Some(curr_node);
            }
            curr = self.parent(curr_node);
        }
        None
    }

    /// Returns descendants of the document root that have the given HTML local `tag_name`.
    /// If `tag_name` is `*`, returns all descendant element nodes.
    // spec: https://dom.spec.whatwg.org/#dom-document-getelementsbytagname
    pub fn get_elements_by_tag_name(&self, tag_name: &str) -> Vec<NodeId> {
        self.get_elements_by_tag_name_from(self.document(), tag_name)
    }

    /// Returns descendants of the given `root` node that have the given HTML local `tag_name`.
    /// If `tag_name` is `*`, returns all descendant element nodes.
    // spec: https://dom.spec.whatwg.org/#dom-element-getelementsbytagname
    pub fn get_elements_by_tag_name_from(&self, root: NodeId, tag_name: &str) -> Vec<NodeId> {
        self.descendants_iter(root)
            .filter(|&node_id| {
                if let Some(NodeData::Element { name, .. }) = self.data(node_id) {
                    if tag_name == "*" {
                        true
                    } else {
                        name.eq_ignore_ascii_case(tag_name)
                    }
                } else {
                    false
                }
            })
            .collect()
    }

    /// Returns descendants of the document root that have all the given space-separated class names.
    // spec: https://dom.spec.whatwg.org/#dom-document-getelementsbyclassname
    pub fn get_elements_by_class_name(&self, class_name: &str) -> Vec<NodeId> {
        self.get_elements_by_class_name_from(self.document(), class_name)
    }

    /// Returns descendants of the given `root` node that have all the given space-separated class names.
    // spec: https://dom.spec.whatwg.org/#dom-element-getelementsbyclassname
    pub fn get_elements_by_class_name_from(&self, root: NodeId, class_name: &str) -> Vec<NodeId> {
        let targets: Vec<&str> = class_name.split_ascii_whitespace().collect();
        if targets.is_empty() {
            return Vec::new();
        }

        self.descendants_iter(root)
            .filter(|&node_id| {
                if matches!(self.data(node_id), Some(NodeData::Element { .. })) {
                    targets
                        .iter()
                        .all(|&target| self.has_class(node_id, target))
                } else {
                    false
                }
            })
            .collect()
    }

    /// Returns descendants of the document root that have a `name` attribute equal to the given value.
    // spec: https://dom.spec.whatwg.org/#dom-document-getelementsbyname
    pub fn get_elements_by_name(&self, name: &str) -> Vec<NodeId> {
        self.get_elements_by_name_from(self.document(), name)
    }

    /// Returns descendants of the given `root` node that have a `name` attribute equal to the given value.
    pub fn get_elements_by_name_from(&self, root: NodeId, name: &str) -> Vec<NodeId> {
        self.descendants_iter(root)
            .filter(|&node_id| {
                if let Some(NodeData::Element { attrs, .. }) = self.data(node_id) {
                    attrs.iter().any(|(n, v)| n == "name" && v == name)
                } else {
                    false
                }
            })
            .collect()
    }

    /// Returns the first child of the given `node`, if any.
    // spec: https://dom.spec.whatwg.org/#dom-node-firstchild
    pub fn first_child(&self, node: NodeId) -> Option<NodeId> {
        self.children(node).first().copied()
    }

    /// Returns the last child of the given `node`, if any.
    // spec: https://dom.spec.whatwg.org/#dom-node-lastchild
    pub fn last_child(&self, node: NodeId) -> Option<NodeId> {
        self.children(node).last().copied()
    }

    /// Returns the previous sibling of the given `node`, if any.
    // spec: https://dom.spec.whatwg.org/#dom-node-previoussibling
    pub fn previous_sibling(&self, node: NodeId) -> Option<NodeId> {
        let parent = self.parent(node)?;
        let children = self.children(parent);
        let pos = children.iter().position(|&id| id == node)?;
        if pos > 0 {
            children.get(pos - 1).copied()
        } else {
            None
        }
    }

    /// Returns the next sibling of the given `node`, if any.
    // spec: https://dom.spec.whatwg.org/#dom-node-nextsibling
    pub fn next_sibling(&self, node: NodeId) -> Option<NodeId> {
        let parent = self.parent(node)?;
        let children = self.children(parent);
        let pos = children.iter().position(|&id| id == node)?;
        children.get(pos + 1).copied()
    }

    /// Returns true if the given `node` has any child nodes.
    // spec: https://dom.spec.whatwg.org/#dom-node-haschildnodes
    pub fn has_child_nodes(&self, node: NodeId) -> bool {
        !self.children(node).is_empty()
    }

    /// Returns the root of the given `node` (its furthest ancestor, or itself if it has no ancestor).
    // spec: https://dom.spec.whatwg.org/#dom-node-getrootnode
    pub fn get_root_node(&self, node: NodeId) -> NodeId {
        let mut curr = node;
        while let Some(parent) = self.parent(curr) {
            curr = parent;
        }
        curr
    }
}

fn split_selector_list(selector: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut parens_depth = 0;
    let mut brackets_depth = 0;
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escaped = false;

    for c in selector.chars() {
        if escaped {
            current.push(c);
            escaped = false;
            continue;
        }

        match c {
            '\\' => {
                current.push(c);
                escaped = true;
            }
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
                current.push(c);
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
                current.push(c);
            }
            _ if in_single_quote || in_double_quote => {
                current.push(c);
            }
            '(' => {
                parens_depth += 1;
                current.push(c);
            }
            ')' => {
                if parens_depth > 0 {
                    parens_depth -= 1;
                }
                current.push(c);
            }
            '[' => {
                brackets_depth += 1;
                current.push(c);
            }
            ']' => {
                if brackets_depth > 0 {
                    brackets_depth -= 1;
                }
                current.push(c);
            }
            ',' if parens_depth == 0 && brackets_depth == 0 => {
                parts.push(current);
                current = String::new();
            }
            _ => {
                current.push(c);
            }
        }
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}

fn preprocess_relative_selector(selector: &str) -> String {
    let parts = split_selector_list(selector);
    let processed_parts: Vec<String> = parts
        .into_iter()
        .map(|part| {
            let trimmed = part.trim();
            if trimmed.starts_with('>') || trimmed.starts_with('+') || trimmed.starts_with('~') {
                format!(":scope {}", trimmed)
            } else {
                trimmed.to_string()
            }
        })
        .collect();
    processed_parts.join(", ")
}

fn matches_with_scope(
    list: &selector::SelectorList,
    dom: &Dom,
    node: NodeId,
    scope: NodeId,
) -> bool {
    list.0
        .iter()
        .any(|sel| matches_complex_with_scope(sel, dom, node, scope))
}

fn matches_complex_with_scope(
    sel: &selector::ComplexSelector,
    dom: &Dom,
    node: NodeId,
    scope: NodeId,
) -> bool {
    if sel.parts.is_empty() {
        return false;
    }

    let last_part_idx = sel.parts.len() - 1;
    let (_, compound) = &sel.parts[last_part_idx];

    if !matches_compound_with_scope(compound, dom, node, scope) {
        return false;
    }

    if last_part_idx == 0 {
        return true;
    }

    matches_rest_with_scope(
        &sel.parts[..last_part_idx],
        dom,
        node,
        sel.parts[last_part_idx].0,
        scope,
    )
}

fn matches_rest_with_scope(
    parts: &[(selector::Combinator, selector::CompoundSelector)],
    dom: &Dom,
    node: NodeId,
    comb: selector::Combinator,
    scope: NodeId,
) -> bool {
    match comb {
        selector::Combinator::Descendant => {
            let mut current = dom.parent(node);
            while let Some(ancestor) = current {
                if matches_complex_at_part_with_scope(parts, dom, ancestor, scope) {
                    return true;
                }
                current = dom.parent(ancestor);
            }
            false
        }
        selector::Combinator::Child => {
            if let Some(parent) = dom.parent(node) {
                matches_complex_at_part_with_scope(parts, dom, parent, scope)
            } else {
                false
            }
        }
        selector::Combinator::NextSibling => {
            if let Some(prev) = dom.previous_element_sibling(node) {
                matches_complex_at_part_with_scope(parts, dom, prev, scope)
            } else {
                false
            }
        }
        selector::Combinator::SubsequentSibling => {
            let mut current = dom.previous_element_sibling(node);
            while let Some(sibling) = current {
                if matches_complex_at_part_with_scope(parts, dom, sibling, scope) {
                    return true;
                }
                current = dom.previous_element_sibling(sibling);
            }
            false
        }
    }
}

fn matches_complex_at_part_with_scope(
    parts: &[(selector::Combinator, selector::CompoundSelector)],
    dom: &Dom,
    node: NodeId,
    scope: NodeId,
) -> bool {
    if parts.is_empty() {
        return false;
    }

    let last_idx = parts.len() - 1;
    let (_, compound) = &parts[last_idx];

    if !matches_compound_with_scope(compound, dom, node, scope) {
        return false;
    }

    if last_idx == 0 {
        return true;
    }

    matches_rest_with_scope(&parts[..last_idx], dom, node, parts[last_idx].0, scope)
}

fn matches_compound_with_scope(
    compound: &selector::CompoundSelector,
    dom: &Dom,
    node: NodeId,
    scope: NodeId,
) -> bool {
    if compound.components.is_empty() {
        return false;
    }
    compound
        .components
        .iter()
        .all(|comp| matches_component_with_scope(comp, dom, node, scope))
}

fn matches_component_with_scope(
    comp: &selector::Component,
    dom: &Dom,
    node: NodeId,
    scope: NodeId,
) -> bool {
    match comp {
        selector::Component::PseudoClass(s) if s == "scope" => node == scope,
        selector::Component::Not(sub) => !matches_compound_with_scope(sub, dom, node, scope),
        selector::Component::Is(list) => list
            .0
            .iter()
            .any(|sel| matches_complex_with_scope(sel, dom, node, scope)),
        selector::Component::Where(list) => list
            .0
            .iter()
            .any(|sel| matches_complex_with_scope(sel, dom, node, scope)),
        selector::Component::Has(list) => matches_has_with_scope(list, dom, node, scope),
        selector::Component::PseudoElement(_) => false, // Pseudo-elements do not match DOM element nodes under querySelector or matches()
        _ => {
            // For all other components, match using standard selector::matches_complex
            let temp_compound = selector::CompoundSelector {
                components: vec![comp.clone()],
            };
            let temp_sel = selector::ComplexSelector {
                parts: vec![(selector::Combinator::Descendant, temp_compound)],
            };
            selector::matches_complex(&temp_sel, dom, node)
        }
    }
}

fn has_sibling_combinator(parts: &[(selector::Combinator, selector::CompoundSelector)]) -> bool {
    parts.iter().any(|(comb, _)| {
        matches!(
            comb,
            selector::Combinator::NextSibling | selector::Combinator::SubsequentSibling
        )
    })
}

fn matches_has_with_scope(
    list: &selector::SelectorList,
    dom: &Dom,
    node: NodeId,
    scope: NodeId,
) -> bool {
    list.0.iter().any(|sel| {
        if sel.parts.is_empty() {
            return false;
        }

        let first_comb = sel.parts[0].0;
        let first_compound = &sel.parts[0].1;

        // Get starting nodes based on first_comb
        let starting_nodes: Vec<NodeId> = match first_comb {
            selector::Combinator::Child => dom.children(node).to_vec(),
            selector::Combinator::NextSibling => {
                dom.next_element_sibling(node).into_iter().collect()
            }
            selector::Combinator::SubsequentSibling => {
                let mut siblings = Vec::new();
                let mut current = dom.next_element_sibling(node);
                while let Some(sibling) = current {
                    siblings.push(sibling);
                    current = dom.next_element_sibling(sibling);
                }
                siblings
            }
            _ => {
                // Descendant
                dom.descendants_iter(node).collect()
            }
        };

        // Filter starting nodes that match first_compound
        let matched_starts: Vec<NodeId> = starting_nodes
            .into_iter()
            .filter(|&start| matches_compound_with_scope(first_compound, dom, start, scope))
            .collect();

        if matched_starts.is_empty() {
            return false;
        }

        if sel.parts.len() == 1 {
            // If len is 1, we just needed to find at least one starting node that matches first_compound
            return true;
        }

        let has_sibling_comb = has_sibling_combinator(&sel.parts[1..]);

        matched_starts.into_iter().any(|start_node| {
            if has_sibling_comb {
                let root = dom.get_root_node(start_node);
                dom.descendants_iter(root)
                    .any(|desc| matches_complex_relative(sel, dom, desc, start_node, scope))
            } else {
                dom.descendants_iter(start_node)
                    .any(|desc| matches_complex_relative(sel, dom, desc, start_node, scope))
            }
        })
    })
}

fn matches_complex_relative(
    sel: &selector::ComplexSelector,
    dom: &Dom,
    node: NodeId,
    start_node: NodeId,
    scope: NodeId,
) -> bool {
    if sel.parts.is_empty() {
        return false;
    }

    let last_part_idx = sel.parts.len() - 1;
    let (_, compound) = &sel.parts[last_part_idx];

    if !matches_compound_with_scope(compound, dom, node, scope) {
        return false;
    }

    if last_part_idx == 0 {
        return node == start_node;
    }

    matches_rest_relative(
        &sel.parts[..last_part_idx],
        dom,
        node,
        sel.parts[last_part_idx].0,
        start_node,
        scope,
    )
}

fn matches_rest_relative(
    parts: &[(selector::Combinator, selector::CompoundSelector)],
    dom: &Dom,
    node: NodeId,
    comb: selector::Combinator,
    start_node: NodeId,
    scope: NodeId,
) -> bool {
    match comb {
        selector::Combinator::Descendant => {
            let mut current = dom.parent(node);
            while let Some(ancestor) = current {
                if matches_complex_relative_at_part(parts, dom, ancestor, start_node, scope) {
                    return true;
                }
                current = dom.parent(ancestor);
            }
            false
        }
        selector::Combinator::Child => {
            if let Some(parent) = dom.parent(node) {
                matches_complex_relative_at_part(parts, dom, parent, start_node, scope)
            } else {
                false
            }
        }
        selector::Combinator::NextSibling => {
            if let Some(prev) = dom.previous_element_sibling(node) {
                matches_complex_relative_at_part(parts, dom, prev, start_node, scope)
            } else {
                false
            }
        }
        selector::Combinator::SubsequentSibling => {
            let mut current = dom.previous_element_sibling(node);
            while let Some(sibling) = current {
                if matches_complex_relative_at_part(parts, dom, sibling, start_node, scope) {
                    return true;
                }
                current = dom.previous_element_sibling(sibling);
            }
            false
        }
    }
}

fn matches_complex_relative_at_part(
    parts: &[(selector::Combinator, selector::CompoundSelector)],
    dom: &Dom,
    node: NodeId,
    start_node: NodeId,
    scope: NodeId,
) -> bool {
    if parts.is_empty() {
        return false;
    }

    let last_idx = parts.len() - 1;
    let (_, compound) = &parts[last_idx];

    if !matches_compound_with_scope(compound, dom, node, scope) {
        return false;
    }

    if last_idx == 0 {
        return node == start_node;
    }

    matches_rest_relative(
        &parts[..last_idx],
        dom,
        node,
        parts[last_idx].0,
        start_node,
        scope,
    )
}

fn any_descendant_matches_with_scope(
    sel: &selector::ComplexSelector,
    dom: &Dom,
    node: NodeId,
    scope: NodeId,
) -> bool {
    let children = dom.children(node);
    for &child in children {
        if matches_complex_with_scope(sel, dom, child, scope) {
            return true;
        }
        if any_descendant_matches_with_scope(sel, dom, child, scope) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_dom() -> Dom {
        let mut dom = Dom::new();
        let doc = dom.document();

        // <html>
        let html = dom.create_node(NodeData::Element {
            name: "html".into(),
            attrs: vec![],
        });
        dom.append_child(doc, html);

        //   <body>
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(html, body);

        //     <div id="container" class="main box">
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![
                ("id".into(), "container".into()),
                ("class".into(), "main box".into()),
            ],
        });
        dom.append_child(body, div);

        //       <p class="text" id="p1">Hello</p>
        let p1 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![("id".into(), "p1".into()), ("class".into(), "text".into())],
        });
        dom.append_child(div, p1);

        //       <span class="text">World</span>
        let span = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("class".into(), "text".into())],
        });
        dom.append_child(div, span);

        dom
    }

    #[test]
    fn test_t0024_query_selector_types() {
        let dom = setup_test_dom();

        // 1. Type selector
        let html_node = dom.query_selector("html");
        assert!(html_node.is_some());
        if let Some(NodeData::Element { name, .. }) = dom.data(html_node.unwrap()) {
            assert_eq!(name, "html");
        } else {
            panic!("Expected html element");
        }

        // 2. ID selector
        let p1_node = dom.query_selector("#p1");
        assert!(p1_node.is_some());
        if let Some(NodeData::Element { name, .. }) = dom.data(p1_node.unwrap()) {
            assert_eq!(name, "p");
        } else {
            panic!("Expected p element");
        }

        // 3. Class selector
        let main_node = dom.query_selector(".main");
        assert!(main_node.is_some());
        if let Some(NodeData::Element { name, .. }) = dom.data(main_node.unwrap()) {
            assert_eq!(name, "div");
        } else {
            panic!("Expected div element");
        }

        // 4. Descendant selector
        let desc_p = dom.query_selector("div p");
        assert!(desc_p.is_some());
        assert_eq!(desc_p, p1_node);

        // 5. Child selector
        let child_span = dom.query_selector("div > span");
        assert!(child_span.is_some());
        if let Some(NodeData::Element { name, .. }) = dom.data(child_span.unwrap()) {
            assert_eq!(name, "span");
        } else {
            panic!("Expected span element");
        }

        // 6. Invalid selector returns None
        assert_eq!(dom.query_selector("div > > p"), None);
        assert_eq!(dom.query_selector(""), None);
    }

    #[test]
    fn test_t0024_query_selector_all_order() {
        let dom = setup_test_dom();

        // All .text elements (p then span in pre-order/document order)
        let matched = dom.query_selector_all(".text");
        assert_eq!(matched.len(), 2);

        if let Some(NodeData::Element { name, .. }) = dom.data(matched[0]) {
            assert_eq!(name, "p");
        } else {
            panic!("Expected first element to be p");
        }

        if let Some(NodeData::Element { name, .. }) = dom.data(matched[1]) {
            assert_eq!(name, "span");
        } else {
            panic!("Expected second element to be span");
        }

        // Invalid selector returns empty Vec
        assert!(dom.query_selector_all("div > > p").is_empty());
        assert!(dom.query_selector_all("").is_empty());
    }

    #[test]
    fn test_element_sibling_navigation() {
        let mut dom = Dom::new();
        let parent = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });

        let child_a = dom.create_node(NodeData::Element {
            name: "a".into(),
            attrs: vec![],
        });
        let child_text = dom.create_node(NodeData::Text("some text".into()));
        let child_b = dom.create_node(NodeData::Element {
            name: "b".into(),
            attrs: vec![],
        });
        let child_c = dom.create_node(NodeData::Element {
            name: "c".into(),
            attrs: vec![],
        });

        dom.append_child(parent, child_a);
        dom.append_child(parent, child_text);
        dom.append_child(parent, child_b);
        dom.append_child(parent, child_c);

        // Next element sibling
        assert_eq!(dom.next_element_sibling(child_a), Some(child_b));
        assert_eq!(dom.next_element_sibling(child_b), Some(child_c));
        assert_eq!(dom.next_element_sibling(child_c), None);

        // Previous element sibling
        assert_eq!(dom.previous_element_sibling(child_c), Some(child_b));
        assert_eq!(dom.previous_element_sibling(child_b), Some(child_a));
        assert_eq!(dom.previous_element_sibling(child_a), None);

        // Unattached/no parent node
        let unattached = dom.create_node(NodeData::Element {
            name: "img".into(),
            attrs: vec![],
        });
        assert_eq!(dom.next_element_sibling(unattached), None);
        assert_eq!(dom.previous_element_sibling(unattached), None);
    }

    #[test]
    fn test_node_contains_query() {
        let mut dom = setup_test_dom();
        let doc = dom.document();
        let html = dom.query_selector("html").unwrap();
        let body = dom.query_selector("body").unwrap();
        let div = dom.query_selector("#container").unwrap();
        let p1 = dom.query_selector("#p1").unwrap();
        let span = dom.query_selector("span").unwrap();

        // 1. A node contains itself (contains(n, n) == true).
        assert!(dom.contains(doc, doc));
        assert!(dom.contains(html, html));
        assert!(dom.contains(p1, p1));

        // 2. A parent contains its direct child.
        assert!(dom.contains(html, body));
        assert!(dom.contains(div, p1));
        assert!(dom.contains(div, span));

        // 3. An ancestor contains a deep (grand+) descendant.
        assert!(dom.contains(html, p1));
        assert!(dom.contains(doc, span));

        // 4. A node does NOT contain its own ancestor (contains(child, parent) == false).
        assert!(!dom.contains(body, html));
        assert!(!dom.contains(p1, div));
        assert!(!dom.contains(span, doc));

        // 5. Two sibling subtrees do not contain each other.
        assert!(!dom.contains(p1, span));
        assert!(!dom.contains(span, p1));

        // 6. The document root contains every node in the tree.
        assert!(dom.contains(doc, doc));
        assert!(dom.contains(doc, html));
        assert!(dom.contains(doc, body));
        assert!(dom.contains(doc, div));
        assert!(dom.contains(doc, p1));
        assert!(dom.contains(doc, span));

        // Extra: Unattached node in the same DOM
        let unattached = dom.create_node(NodeData::Element {
            name: "img".into(),
            attrs: vec![],
        });
        assert!(dom.contains(unattached, unattached));
        assert!(!dom.contains(doc, unattached));
        assert!(!dom.contains(unattached, doc));
    }

    #[test]
    fn test_expanded_query_and_traversal() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // Let's build:
        // <html>
        //   <body>
        //     <div id="main" class="container content" name="app-root">
        //       text node child
        //       <p class="text item" id="p1" name="paragraph">First paragraph</p>
        //       comment node child
        //       <span class="text label" id="s1">Some span</span>
        //       <div class="footer item" name="paragraph">Footer div</div>
        //     </div>
        //   </body>
        // </html>

        let html = dom.create_node(NodeData::Element {
            name: "html".into(),
            attrs: vec![],
        });
        dom.append_child(doc, html);

        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(html, body);

        let main_div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![
                ("id".into(), "main".into()),
                ("class".into(), "container content".into()),
                ("name".into(), "app-root".into()),
            ],
        });
        dom.append_child(body, main_div);

        let text_node = dom.create_node(NodeData::Text("   some text   ".into()));
        dom.append_child(main_div, text_node);

        let p1 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![
                ("id".into(), "p1".into()),
                ("class".into(), "text item".into()),
                ("name".into(), "paragraph".into()),
            ],
        });
        dom.append_child(main_div, p1);

        let comment_node = dom.create_node(NodeData::Comment("this is a comment".into()));
        dom.append_child(main_div, comment_node);

        let span1 = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![
                ("id".into(), "s1".into()),
                ("class".into(), "text label".into()),
            ],
        });
        dom.append_child(main_div, span1);

        let footer_div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![
                ("class".into(), "footer item".into()),
                ("name".into(), "paragraph".into()),
            ],
        });
        dom.append_child(main_div, footer_div);

        // --- 1. first_element_child, last_element_child, child_element_count, child_elements ---
        assert_eq!(dom.first_element_child(main_div), Some(p1));
        assert_eq!(dom.last_element_child(main_div), Some(footer_div));
        assert_eq!(dom.child_element_count(main_div), 3);
        assert_eq!(dom.child_elements(main_div), vec![p1, span1, footer_div]);

        // HTML/body check
        assert_eq!(dom.first_element_child(html), Some(body));
        assert_eq!(dom.last_element_child(html), Some(body));
        assert_eq!(dom.child_element_count(html), 1);
        assert_eq!(dom.child_elements(html), vec![body]);

        // No child element checks
        assert_eq!(dom.first_element_child(p1), None);
        assert_eq!(dom.last_element_child(p1), None);
        assert_eq!(dom.child_element_count(p1), 0);
        assert!(dom.child_elements(p1).is_empty());

        // --- 2. matches ---
        assert!(dom.matches(p1, "p"));
        assert!(dom.matches(p1, ".text"));
        assert!(dom.matches(p1, ".item"));
        assert!(dom.matches(p1, "p.text.item"));
        assert!(dom.matches(p1, "div p"));
        assert!(dom.matches(p1, "#p1"));
        assert!(!dom.matches(p1, "span"));
        assert!(!dom.matches(p1, "div"));
        assert!(!dom.matches(text_node, "p")); // Non-element node

        // --- 3. closest ---
        assert_eq!(dom.closest(p1, "p"), Some(p1));
        assert_eq!(dom.closest(p1, "div"), Some(main_div));
        assert_eq!(dom.closest(p1, "body"), Some(body));
        assert_eq!(dom.closest(p1, "html"), Some(html));
        assert_eq!(dom.closest(p1, "span"), None);
        assert_eq!(dom.closest(text_node, "div"), Some(main_div)); // Text node matches parent

        // --- 4. get_elements_by_tag_name ---
        // Document-wide tag matching
        let divs = dom.get_elements_by_tag_name("div");
        assert_eq!(divs, vec![main_div, footer_div]);

        let ps = dom.get_elements_by_tag_name("p");
        assert_eq!(ps, vec![p1]);

        let stars = dom.get_elements_by_tag_name("*");
        assert_eq!(stars, vec![html, body, main_div, p1, span1, footer_div]);

        // Case-insensitivity
        let upper_divs = dom.get_elements_by_tag_name("DIV");
        assert_eq!(upper_divs, vec![main_div, footer_div]);

        // Root-specific tag matching
        let sub_divs = dom.get_elements_by_tag_name_from(main_div, "div");
        assert_eq!(sub_divs, vec![footer_div]);

        let sub_stars = dom.get_elements_by_tag_name_from(main_div, "*");
        assert_eq!(sub_stars, vec![p1, span1, footer_div]);

        // --- 5. get_elements_by_class_name ---
        // Document-wide class matching
        let texts = dom.get_elements_by_class_name("text");
        assert_eq!(texts, vec![p1, span1]);

        let items = dom.get_elements_by_class_name("item");
        assert_eq!(items, vec![p1, footer_div]);

        // Multiple class matching (any order)
        let multiple1 = dom.get_elements_by_class_name("text item");
        assert_eq!(multiple1, vec![p1]);

        let multiple2 = dom.get_elements_by_class_name("item text");
        assert_eq!(multiple2, vec![p1]);

        // Empty class names or non-existent
        assert!(dom.get_elements_by_class_name("").is_empty());
        assert!(dom.get_elements_by_class_name("nonexistent").is_empty());

        // Root-specific class matching
        let sub_items = dom.get_elements_by_class_name_from(main_div, "item");
        assert_eq!(sub_items, vec![p1, footer_div]);

        // --- 6. get_elements_by_name ---
        let app_roots = dom.get_elements_by_name("app-root");
        assert_eq!(app_roots, vec![main_div]);

        let paragraphs = dom.get_elements_by_name("paragraph");
        assert_eq!(paragraphs, vec![p1, footer_div]);

        let sub_paragraphs = dom.get_elements_by_name_from(main_div, "paragraph");
        assert_eq!(sub_paragraphs, vec![p1, footer_div]);

        let nonexistent_name = dom.get_elements_by_name("nonexistent");
        assert!(nonexistent_name.is_empty());
    }

    #[test]
    fn test_node_traversal_gap_apis() {
        let mut dom = Dom::new();
        let doc = dom.document();

        let parent = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, parent);

        let child_text = dom.create_node(NodeData::Text("Hello".into()));
        let child_element = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![],
        });
        let child_comment = dom.create_node(NodeData::Comment("World".into()));

        dom.append_child(parent, child_text);
        dom.append_child(parent, child_element);
        dom.append_child(parent, child_comment);

        // 1. first_child & last_child
        assert_eq!(dom.first_child(parent), Some(child_text));
        assert_eq!(dom.last_child(parent), Some(child_comment));
        assert_eq!(dom.first_child(child_text), None);
        assert_eq!(dom.last_child(child_text), None);

        // 2. has_child_nodes
        assert!(dom.has_child_nodes(parent));
        assert!(dom.has_child_nodes(doc));
        assert!(!dom.has_child_nodes(child_text));
        assert!(!dom.has_child_nodes(child_element));

        // 3. previous_sibling & next_sibling
        assert_eq!(dom.previous_sibling(child_text), None);
        assert_eq!(dom.next_sibling(child_text), Some(child_element));

        assert_eq!(dom.previous_sibling(child_element), Some(child_text));
        assert_eq!(dom.next_sibling(child_element), Some(child_comment));

        assert_eq!(dom.previous_sibling(child_comment), Some(child_element));
        assert_eq!(dom.next_sibling(child_comment), None);

        // 4. get_root_node
        assert_eq!(dom.get_root_node(doc), doc);
        assert_eq!(dom.get_root_node(parent), doc);
        assert_eq!(dom.get_root_node(child_text), doc);
        assert_eq!(dom.get_root_node(child_comment), doc);

        // Detached nodes
        let detached_parent = dom.create_node(NodeData::Element {
            name: "ul".into(),
            attrs: vec![],
        });
        let detached_child = dom.create_node(NodeData::Element {
            name: "li".into(),
            attrs: vec![],
        });
        dom.append_child(detached_parent, detached_child);

        assert_eq!(dom.get_root_node(detached_parent), detached_parent);
        assert_eq!(dom.get_root_node(detached_child), detached_parent);
    }

    #[test]
    fn test_scoped_relative_selector_completeness() {
        let mut dom = Dom::new();
        let doc = dom.document();

        let parent_div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "parent".into())],
        });
        dom.append_child(doc, parent_div);

        let child_span = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("id".into(), "child".into())],
        });
        dom.append_child(parent_div, child_span);

        let sibling_p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![("id".into(), "sibling".into())],
        });
        dom.append_child(parent_div, sibling_p);

        // 1. query_selector_from with :scope and relative selectors
        assert_eq!(
            dom.query_selector_from(parent_div, ":scope > span"),
            Some(child_span)
        );
        assert_eq!(
            dom.query_selector_from(parent_div, "> span"),
            Some(child_span)
        );
        assert_eq!(
            dom.query_selector_from(parent_div, "span + p"),
            Some(sibling_p)
        );
        assert_eq!(
            dom.query_selector_from(parent_div, "> span + p"),
            Some(sibling_p)
        );

        // 2. recursive/functional :scope matching
        assert_eq!(
            dom.query_selector_from(parent_div, ":not(:scope)"),
            Some(child_span)
        );

        // 3. matches and closest with :scope
        assert!(dom.matches(parent_div, ":scope"));
        assert!(dom.matches(parent_div, "div:scope"));
        assert!(!dom.matches(parent_div, "span:scope"));
        assert_eq!(dom.closest(child_span, ":scope"), Some(child_span));

        // Multi-part lists
        let matched_list = dom.query_selector_all_from(parent_div, "> span, > p");
        assert_eq!(matched_list, vec![child_span, sibling_p]);
    }

    #[test]
    fn test_t0847_extended_completeness() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // Let's build a rich tree to test our features:
        // <div class="main" data-role="container" id="host">
        //   <span class="prefix-apple-suffix" data-type="apple-fruit">Apple</span>
        //   <span class="prefix-banana-suffix" data-type="banana-fruit">Banana</span>
        //   <div class="nested-box">
        //     <p class="inner-text" data-title="hello-world">Paragraph</p>
        //   </div>
        // </div>
        let host = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![
                ("class".into(), "main".into()),
                ("data-role".into(), "container".into()),
                ("id".into(), "host".into()),
            ],
        });
        dom.append_child(doc, host);

        let apple = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![
                ("class".into(), "prefix-apple-suffix".into()),
                ("data-type".into(), "apple-fruit".into()),
            ],
        });
        dom.append_child(host, apple);

        let banana = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![
                ("class".into(), "prefix-banana-suffix".into()),
                ("data-type".into(), "banana-fruit".into()),
            ],
        });
        dom.append_child(host, banana);

        let nested_box = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "nested-box".into())],
        });
        dom.append_child(host, nested_box);

        let p_text = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![
                ("class".into(), "inner-text".into()),
                ("data-title".into(), "hello-world".into()),
            ],
        });
        dom.append_child(nested_box, p_text);

        // --- 1. Attribute operators ^= $= *= ---
        // Prefix operator ^=
        assert_eq!(dom.query_selector("[data-type^=\"apple\"]"), Some(apple));
        assert_eq!(
            dom.query_selector("[class^=\"prefix-banana\"]"),
            Some(banana)
        );

        // Suffix operator $=
        assert_eq!(dom.query_selector("[data-type$=\"fruit\"]"), Some(apple)); // Find first matching
        let fruit_list = dom.query_selector_all("[data-type$=\"fruit\"]");
        assert_eq!(fruit_list, vec![apple, banana]);

        // Substring operator *=
        assert_eq!(dom.query_selector("[data-title*=\"lo-wo\"]"), Some(p_text));
        assert_eq!(dom.query_selector("[class*=\"banana\"]"), Some(banana));

        // --- 2. :not ---
        // Span elements that do NOT have data-type starting with "apple"
        let not_apple = dom.query_selector_all("span:not([data-type^=\"apple\"])");
        assert_eq!(not_apple, vec![banana]);

        // Elements under host that are NOT div
        let not_div = dom.query_selector_all_from(host, ":not(div)");
        assert_eq!(not_div, vec![apple, banana, p_text]);

        // --- 3. descendant/child combinators ---
        assert_eq!(dom.query_selector("div > span"), Some(apple));
        assert_eq!(dom.query_selector_all("div span"), vec![apple, banana]);
        assert_eq!(dom.query_selector_from(host, "> span"), Some(apple));
        assert_eq!(dom.query_selector_from(host, "div > p"), Some(p_text));

        // --- 4. :has with scope/relative selectors ---
        // Matches host because it has a child with class prefix-banana-suffix
        assert_eq!(
            dom.query_selector("div:has(> .prefix-banana-suffix)"),
            Some(host)
        );

        // Matches nested-box because it has a descendant p
        assert_eq!(dom.query_selector("div:has(p)"), Some(host));
        let has_divs = dom.query_selector_all("div:has(p)");
        assert_eq!(has_divs, vec![host, nested_box]);

        // matches_has sibling relative matching
        // apple has next sibling banana
        assert_eq!(
            dom.query_selector("span:has(+ [data-type^=\"banana\"])"),
            Some(apple)
        );
        // banana has previous sibling apple (banana is subsequent to apple)
        assert_eq!(dom.query_selector("span:has(~ span)"), Some(apple));
    }

    #[test]
    fn test_t0867_query_selector_completeness() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // Let's build a tree:
        // <div id="host">
        //   <div id="first-child" class="child-div">
        //     <span class="deep-span">Hello</span>
        //   </div>
        //   <p id="sibling-p">
        //     <span class="nested-p-span">World</span>
        //   </p>
        // </div>
        let host = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "host".into())],
        });
        dom.append_child(doc, host);

        let first_child = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![
                ("id".into(), "first-child".into()),
                ("class".into(), "child-div".into()),
            ],
        });
        dom.append_child(host, first_child);

        let deep_span = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("class".into(), "deep-span".into())],
        });
        dom.append_child(first_child, deep_span);

        let sibling_p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![("id".into(), "sibling-p".into())],
        });
        dom.append_child(host, sibling_p);

        let nested_p_span = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("class".into(), "nested-p-span".into())],
        });
        dom.append_child(sibling_p, nested_p_span);

        // 1. Pseudo-elements match nothing (return None)
        assert_eq!(dom.query_selector("div::after"), None);
        assert_eq!(dom.query_selector("span::before"), None);
        assert!(!dom.matches(host, "div::after"));

        // 2. Multi-stage relative selectors in :has() with direct child + descendant
        // matches because host has a direct child div that has descendant span
        assert_eq!(dom.query_selector("div:has(> div span)"), Some(host));
        assert!(dom.matches(host, "div:has(> div span)"));

        // Does NOT match host because first_child does NOT have a direct child div with descendant span
        assert!(!dom.matches(first_child, "div:has(> div span)"));

        // 3. Sibling relative selector in :has() with subsequent sibling + descendant
        // first_child has subsequent sibling p, which has descendant span
        assert_eq!(dom.query_selector("div:has(~ p span)"), Some(first_child));
        assert!(dom.matches(first_child, "div:has(~ p span)"));

        // 4. NextSibling relative selector in :has() with next sibling + descendant
        assert_eq!(dom.query_selector("div:has(+ p span)"), Some(first_child));
        assert!(dom.matches(first_child, "div:has(+ p span)"));
    }

    #[test]
    fn test_t0898_closest_and_split_selector_list_gaps() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // Build a tree:
        // <div id="grandparent">
        //   <div id="parent">
        //     <span id="child" class="target"></span>
        //   </div>
        // </div>
        let grandparent = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "grandparent".into())],
        });
        dom.append_child(doc, grandparent);

        let parent = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "parent".into())],
        });
        dom.append_child(grandparent, parent);

        let child = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![
                ("id".into(), "child".into()),
                ("class".into(), "target".into()),
            ],
        });
        dom.append_child(parent, child);

        // 1. Test closest with relative selector in :has() referring to :scope (which is original child node)
        // Ancestor "parent" should match "div:has(> :scope)" where :scope is "child"
        assert_eq!(dom.closest(child, "div:has(> :scope)"), Some(parent));

        // Ancestor "grandparent" should match "div:has(> div > :scope)"
        assert_eq!(
            dom.closest(child, "div:has(> div > :scope)"),
            Some(grandparent)
        );

        // 2. Test split_selector_list robust preprocessing with commas inside attribute strings
        // e.g., '[class="a,b"]' should not be split on the comma
        let comma_attr_selector = "[data-val=\"val1,val2\"]";
        let parts = split_selector_list(comma_attr_selector);
        assert_eq!(parts, vec![comma_attr_selector.to_string()]);

        // Add matching element to DOM with comma in attribute
        let element_with_comma_attr = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("data-val".into(), "val1,val2".into())],
        });
        dom.append_child(child, element_with_comma_attr);

        // Make sure it parses and matches correctly!
        assert_eq!(
            dom.query_selector_from(child, "[data-val=\"val1,val2\"]"),
            Some(element_with_comma_attr)
        );
    }
}
