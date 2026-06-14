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

    /// Returns the parent of the given `node` if it is an element.
    // spec: https://dom.spec.whatwg.org/#dom-node-parentelement
    pub fn parent_element(&self, node: NodeId) -> Option<NodeId> {
        let parent = self.parent(node)?;
        if matches!(self.data(parent), Some(NodeData::Element { .. })) {
            Some(parent)
        } else {
            None
        }
    }

    /// Returns the first element in the document with the given `id`.
    // spec: https://dom.spec.whatwg.org/#dom-nonelementparentnode-getelementbyid
    pub fn get_element_by_id(&self, id: &str) -> Option<NodeId> {
        if id.is_empty() {
            return None;
        }
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

        let has_outside = selector_list.0.iter().any(can_match_outside);

        if has_outside {
            let search_root = self.get_root_node(root);
            std::iter::once(search_root)
                .chain(self.descendants_iter(search_root))
                .find(|&node_id| {
                    if node_id == root {
                        return false;
                    }
                    if !matches!(self.data(node_id), Some(NodeData::Element { .. })) {
                        return false;
                    }
                    selector_list.0.iter().any(|sel| {
                        matches_complex_with_scope(sel, self, node_id, root)
                            && (self.contains(root, node_id) || can_match_outside(sel))
                    })
                })
        } else {
            self.descendants_iter(root)
                .find(|&node_id| matches_with_scope(&selector_list, self, node_id, root))
        }
    }

    /// Returns all descendants of the given `root` node that match the given `selector` in document order.
    pub fn query_selector_all_from(&self, root: NodeId, selector: &str) -> Vec<NodeId> {
        let selector_list = match self.parse_scoped_selector(selector) {
            Ok(list) => list,
            Err(_) => return Vec::new(),
        };

        let has_outside = selector_list.0.iter().any(can_match_outside);

        if has_outside {
            let search_root = self.get_root_node(root);
            std::iter::once(search_root)
                .chain(self.descendants_iter(search_root))
                .filter(|&node_id| {
                    if node_id == root {
                        return false;
                    }
                    if !matches!(self.data(node_id), Some(NodeData::Element { .. })) {
                        return false;
                    }
                    selector_list.0.iter().any(|sel| {
                        matches_complex_with_scope(sel, self, node_id, root)
                            && (self.contains(root, node_id) || can_match_outside(sel))
                    })
                })
                .collect()
        } else {
            self.descendants_iter(root)
                .filter(|&node_id| matches_with_scope(&selector_list, self, node_id, root))
                .collect()
        }
    }

    fn parse_scoped_selector(
        &self,
        selector: &str,
    ) -> Result<selector::SelectorList, selector::SelectorParseError> {
        let preprocessed_not = preprocess_not_selectors(selector);
        let preprocessed = preprocess_relative_selector(&preprocessed_not);
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

    /// Returns descendants of the document root that have the given namespace and local name.
    /// If `namespace` is `*`, matches any namespace. If `namespace` is `""`, matches elements with no namespace.
    /// If `local_name` is `*`, matches any local name.
    // spec: https://dom.spec.whatwg.org/#dom-document-getelementsbytagnamens
    pub fn get_elements_by_tag_name_ns(&self, namespace: &str, local_name: &str) -> Vec<NodeId> {
        self.get_elements_by_tag_name_ns_from(self.document(), namespace, local_name)
    }

    /// Returns descendants of the given `root` node that have the given namespace and local name.
    /// If `namespace` is `*`, matches any namespace. If `namespace` is `""`, matches elements with no namespace.
    /// If `local_name` is `*`, matches any local name.
    // spec: https://dom.spec.whatwg.org/#dom-element-getelementsbytagnamens
    pub fn get_elements_by_tag_name_ns_from(
        &self,
        root: NodeId,
        namespace: &str,
        local_name: &str,
    ) -> Vec<NodeId> {
        self.descendants_iter(root)
            .filter(|&node_id| {
                if let Some(NodeData::Element { name, .. }) = self.data(node_id) {
                    // In our simplified DOM, all elements are in the "http://www.w3.org/1999/xhtml" namespace.
                    let el_ns = "http://www.w3.org/1999/xhtml";
                    let ns_match = namespace == "*" || namespace == el_ns;
                    let local_match = local_name == "*" || name.eq_ignore_ascii_case(local_name);
                    ns_match && local_match
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

fn preprocess_not_selectors(selector: &str) -> String {
    let chars: Vec<char> = selector.chars().collect();
    let mut result = String::new();
    let mut i = 0;

    let mut parens_depth = 0;
    let mut brackets_depth = 0;
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escaped = false;

    // A stack of parenthesis depths at which we should output an extra ')' when closing.
    let mut extra_close_at_depth = Vec::new();

    while i < chars.len() {
        let c = chars[i];

        if escaped {
            result.push(c);
            escaped = false;
            i += 1;
            continue;
        }

        match c {
            '\\' => {
                result.push(c);
                escaped = true;
                i += 1;
            }
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
                result.push(c);
                i += 1;
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
                result.push(c);
                i += 1;
            }
            _ if in_single_quote || in_double_quote => {
                result.push(c);
                i += 1;
            }
            '[' => {
                brackets_depth += 1;
                result.push(c);
                i += 1;
            }
            ']' => {
                if brackets_depth > 0 {
                    brackets_depth -= 1;
                }
                result.push(c);
                i += 1;
            }
            ':' if brackets_depth == 0 && i + 4 < chars.len() => {
                // Check if it matches :not(
                let is_not = (chars[i + 1] == 'n' || chars[i + 1] == 'N')
                    && (chars[i + 2] == 'o' || chars[i + 2] == 'O')
                    && (chars[i + 3] == 't' || chars[i + 3] == 'T')
                    && chars[i + 4] == '(';

                if is_not {
                    // Rewrite :not( to :not(:is(
                    result.push_str(":not(:is(");
                    parens_depth += 2; // We opened both :not( and :is(
                    extra_close_at_depth.push(parens_depth); // Record that at this depth we need double closing
                    i += 5; // Skip past ":not("
                } else {
                    result.push(c);
                    i += 1;
                }
            }
            '(' => {
                parens_depth += 1;
                result.push(c);
                i += 1;
            }
            ')' => {
                if parens_depth > 0 {
                    if Some(&parens_depth) == extra_close_at_depth.last() {
                        extra_close_at_depth.pop();
                        result.push_str("))");
                        parens_depth -= 2;
                    } else {
                        result.push(c);
                        parens_depth -= 1;
                    }
                } else {
                    result.push(c);
                }
                i += 1;
            }
            _ => {
                result.push(c);
                i += 1;
            }
        }
    }
    result
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

fn can_match_outside(sel: &selector::ComplexSelector) -> bool {
    if sel.parts.len() < 2 {
        return false;
    }
    // Check if the first part is :scope
    let first_compound = &sel.parts[0].1;
    let has_scope = first_compound.components.iter().any(|comp| {
        if let selector::Component::PseudoClass(s) = comp {
            s.eq_ignore_ascii_case("scope")
        } else {
            false
        }
    });
    if !has_scope {
        return false;
    }
    // Check if the next combinator is a sibling combinator
    let next_comb = sel.parts[1].0;
    matches!(
        next_comb,
        selector::Combinator::NextSibling | selector::Combinator::SubsequentSibling
    )
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

fn is_default_html_case_insensitive_attribute(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "accept"
            | "accept-charset"
            | "align"
            | "alink"
            | "axis"
            | "bgcolor"
            | "charset"
            | "checked"
            | "clear"
            | "codetype"
            | "color"
            | "compact"
            | "declare"
            | "defer"
            | "dir"
            | "direction"
            | "disabled"
            | "enctype"
            | "face"
            | "frame"
            | "gopher"
            | "hreflang"
            | "http-equiv"
            | "lang"
            | "language"
            | "link"
            | "media"
            | "method"
            | "multiple"
            | "nohref"
            | "noresize"
            | "noshade"
            | "nowrap"
            | "readonly"
            | "rel"
            | "rev"
            | "rules"
            | "scope"
            | "scrolling"
            | "selected"
            | "shape"
            | "target"
            | "text"
            | "type"
            | "valign"
            | "valuetype"
            | "vlink"
    )
}

fn matches_an_plus_b(index: i32, a: i32, b: i32) -> bool {
    if a == 0 {
        index == b
    } else {
        let diff = index - b;
        if a > 0 {
            diff >= 0 && diff % a == 0
        } else {
            diff <= 0 && diff % a == 0
        }
    }
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
                return matches_an_plus_b(i, a, b);
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
                return matches_an_plus_b(i, a, b);
            }
            if matches!(dom.data(child), Some(NodeData::Element { .. })) {
                element_index += 1;
            }
        }
    }
    false
}

fn nth_of_type(dom: &Dom, node: NodeId, a: i32, b: i32) -> bool {
    let current_tag_name = match dom.data(node) {
        Some(NodeData::Element { name, .. }) => name,
        _ => return false,
    };
    if let Some(parent) = dom.parent(node) {
        let children = dom.children(parent);
        let mut element_index = 0;
        for &child in children {
            if child == node {
                let i = element_index + 1; // 1-indexed
                return matches_an_plus_b(i, a, b);
            }
            match dom.data(child) {
                Some(NodeData::Element { name, .. })
                    if name.eq_ignore_ascii_case(current_tag_name) =>
                {
                    element_index += 1;
                }
                _ => {}
            }
        }
    }
    false
}

fn nth_last_of_type(dom: &Dom, node: NodeId, a: i32, b: i32) -> bool {
    let current_tag_name = match dom.data(node) {
        Some(NodeData::Element { name, .. }) => name,
        _ => return false,
    };
    if let Some(parent) = dom.parent(node) {
        let children = dom.children(parent);
        let mut element_index = 0;
        for &child in children.iter().rev() {
            if child == node {
                let i = element_index + 1; // 1-indexed
                return matches_an_plus_b(i, a, b);
            }
            match dom.data(child) {
                Some(NodeData::Element { name, .. })
                    if name.eq_ignore_ascii_case(current_tag_name) =>
                {
                    element_index += 1;
                }
                _ => {}
            }
        }
    }
    false
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

fn matches_component_with_scope(
    comp: &selector::Component,
    dom: &Dom,
    node: NodeId,
    scope: NodeId,
) -> bool {
    // Non-element nodes never match any selector component, except for the :scope pseudo-class.
    // Spec: https://drafts.csswg.org/selectors-4/#match-against-element
    if !matches!(dom.data(node), Some(NodeData::Element { .. })) {
        if let selector::Component::PseudoClass(s) = comp
            && s.eq_ignore_ascii_case("scope")
        {
            return node == scope;
        }
        return false;
    }

    match comp {
        selector::Component::Type(name) => {
            if let Some(NodeData::Element { name: tag_name, .. }) = dom.data(node) {
                tag_name.eq_ignore_ascii_case(name)
            } else {
                false
            }
        }
        selector::Component::Universal => true,
        selector::Component::Id(id) => {
            if let Some(NodeData::Element { attrs, .. }) = dom.data(node) {
                attrs.iter().any(|(n, v)| n == "id" && v == id)
            } else {
                false
            }
        }
        selector::Component::Class(class) => {
            if let Some(NodeData::Element { attrs, .. }) = dom.data(node) {
                attrs.iter().any(|(n, v)| {
                    n == "class"
                        && v.split(crate::ascii::is_html_whitespace)
                            .any(|c| c == class)
                })
            } else {
                false
            }
        }
        selector::Component::PseudoClass(s) if s.eq_ignore_ascii_case("scope") => node == scope,
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
        selector::Component::NthChild(a, b) => nth_child(dom, node, *a, *b),
        selector::Component::NthLastChild(a, b) => nth_last_child(dom, node, *a, *b),
        selector::Component::NthOfType(a, b) => nth_of_type(dom, node, *a, *b),
        selector::Component::NthLastOfType(a, b) => nth_last_of_type(dom, node, *a, *b),
        selector::Component::FirstChild => is_first_child(dom, node),
        selector::Component::LastChild => is_last_child(dom, node),
        selector::Component::Attribute {
            name,
            op,
            value,
            modifier,
        } => {
            let attrs = match dom.data(node) {
                Some(NodeData::Element { attrs, .. }) => attrs,
                _ => return false,
            };
            let attr_val = attrs
                .iter()
                .find(|(n, _)| n.eq_ignore_ascii_case(name))
                .map(|(_, v)| v);

            match (attr_val, op, value) {
                (Some(_), None, _) => true, // Presence only
                (Some(v), Some(op), Some(val)) => {
                    // Edge case: if val is empty and operator is NOT Exact (=), it never matches!
                    if val.is_empty() && *op != selector::AttrOp::Exact {
                        return false;
                    }

                    let case_insensitive = match modifier {
                        Some('i') | Some('I') => true,
                        Some('s') | Some('S') => false,
                        _ => {
                            // Default HTML case-insensitive attributes
                            is_default_html_case_insensitive_attribute(name)
                        }
                    };

                    if case_insensitive {
                        match op {
                            selector::AttrOp::Exact => v.eq_ignore_ascii_case(val),
                            selector::AttrOp::Includes => v
                                .split(crate::ascii::is_html_whitespace)
                                .any(|c| c.eq_ignore_ascii_case(val)),
                            selector::AttrOp::DashMatch => {
                                v.eq_ignore_ascii_case(val)
                                    || (starts_with_ignore_ascii_case(v, val)
                                        && v.as_bytes().get(val.len()) == Some(&b'-'))
                            }
                            selector::AttrOp::Prefix => starts_with_ignore_ascii_case(v, val),
                            selector::AttrOp::Suffix => ends_with_ignore_ascii_case(v, val),
                            selector::AttrOp::Substring => contains_ignore_ascii_case(v, val),
                        }
                    } else {
                        match op {
                            selector::AttrOp::Exact => v == val,
                            selector::AttrOp::Includes => {
                                v.split(crate::ascii::is_html_whitespace).any(|c| c == val)
                            }
                            selector::AttrOp::DashMatch => {
                                v == val
                                    || (v.starts_with(val)
                                        && v.as_bytes().get(val.len()) == Some(&b'-'))
                            }
                            selector::AttrOp::Prefix => v.starts_with(val),
                            selector::AttrOp::Suffix => v.ends_with(val),
                            selector::AttrOp::Substring => v.contains(val),
                        }
                    }
                }
                _ => false,
            }
        }
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

        // 4. Scoped relative selectors on child elements that match siblings outside the scoped root
        assert_eq!(dom.query_selector_from(child_span, "+ p"), Some(sibling_p));
        assert_eq!(
            dom.query_selector_all_from(child_span, "~ p"),
            vec![sibling_p]
        );
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

    #[test]
    fn test_get_elements_by_tag_name_ns_gaps() {
        let mut dom = Dom::new();
        let doc = dom.document();

        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let span = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![],
        });
        dom.append_child(div, span);

        // 1. Matches "*" namespace and "*" local name
        let all = dom.get_elements_by_tag_name_ns("*", "*");
        assert_eq!(all, vec![div, span]);

        // 2. Matches "http://www.w3.org/1999/xhtml" namespace and specific tag name
        let divs = dom.get_elements_by_tag_name_ns("http://www.w3.org/1999/xhtml", "div");
        assert_eq!(divs, vec![div]);

        // 3. Matches "*" namespace and specific tag name
        let spans = dom.get_elements_by_tag_name_ns("*", "span");
        assert_eq!(spans, vec![span]);

        // 4. Non-matching namespace
        let empty = dom.get_elements_by_tag_name_ns("http://www.w3.org/2000/svg", "div");
        assert!(empty.is_empty());

        // 5. Query from root
        let sub_spans =
            dom.get_elements_by_tag_name_ns_from(div, "http://www.w3.org/1999/xhtml", "span");
        assert_eq!(sub_spans, vec![span]);
    }

    #[test]
    fn test_t0935_non_elements_selector_and_parent_element() {
        let mut dom = Dom::new();
        let doc = dom.document();

        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let text = dom.create_node(NodeData::Text("some text".into()));
        dom.append_child(div, text);

        let comment = dom.create_node(NodeData::Comment("some comment".into()));
        dom.append_child(div, comment);

        // --- Test parent_element ---
        assert_eq!(dom.parent_element(text), Some(div));
        assert_eq!(dom.parent_element(comment), Some(div));
        assert_eq!(dom.parent_element(div), None); // parent is doc, which is not an Element
        assert_eq!(dom.parent_element(doc), None);

        // --- Test selector matching on non-elements ---
        // A Text node or Comment node must NEVER match any selector like :not(div) or div
        assert!(!dom.matches(text, "div"));
        assert!(!dom.matches(text, ":not(div)"));
        assert!(!dom.matches(comment, ":not(div)"));

        // querySelector / querySelectorAll must never return non-elements even if the selector is :not(div)
        let results = dom.query_selector_all_from(div, ":not(div)");
        assert!(results.is_empty());
    }

    #[test]
    fn test_t0960_advanced_attribute_and_scope_matching() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // 1. Case-insensitive :scope matching
        let host = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "host".into())],
        });
        dom.append_child(doc, host);

        assert!(dom.matches(host, ":SCOPE"));
        assert!(dom.matches(host, ":Scope"));
        assert!(dom.matches(host, ":scope"));

        // 2. Case-insensitive attribute name matching
        let btn = dom.create_node(NodeData::Element {
            name: "button".into(),
            attrs: vec![
                ("class".into(), "btn-test".into()),
                ("type".into(), "SUBMIT".into()),
                ("lang".into(), "en-US".into()),
            ],
        });
        dom.append_child(host, btn);

        // [CLASS=...] matches even with uppercase attribute name
        assert_eq!(
            dom.query_selector_from(host, "[CLASS=\"btn-test\"]"),
            Some(btn)
        );
        assert_eq!(
            dom.query_selector_from(host, "[Class=\"btn-test\"]"),
            Some(btn)
        );

        // 3. Case-insensitive attribute value modifier ('i' and 'I')
        // exact match
        assert_eq!(
            dom.query_selector_from(host, "[class=\"BTN-TEST\" i]"),
            Some(btn)
        );
        assert_eq!(
            dom.query_selector_from(host, "[class=\"BTN-TEST\" I]"),
            Some(btn)
        );
        // prefix match
        assert_eq!(
            dom.query_selector_from(host, "[class^=\"BTN\" i]"),
            Some(btn)
        );
        // suffix match
        assert_eq!(
            dom.query_selector_from(host, "[class$=\"TEST\" i]"),
            Some(btn)
        );
        // substring match
        assert_eq!(
            dom.query_selector_from(host, "[class*=\"N-TE\" i]"),
            Some(btn)
        );

        // 4. Case-sensitive forcing with 's' and 'S'
        // type="SUBMIT" in btn is uppercase. Default "type" is case-insensitive, but 's' or 'S' forces it to be sensitive.
        assert_eq!(
            dom.query_selector_from(host, "[type=\"submit\"]"),
            Some(btn)
        ); // default is insensitive, matches "SUBMIT"
        assert_eq!(dom.query_selector_from(host, "[type=\"submit\" s]"), None); // forced sensitive, "submit" != "SUBMIT"
        assert_eq!(
            dom.query_selector_from(host, "[type=\"SUBMIT\" s]"),
            Some(btn)
        ); // forced sensitive, matches "SUBMIT"
        assert_eq!(dom.query_selector_from(host, "[type=\"submit\" S]"), None); // forced sensitive with 'S'

        // 5. Default HTML case-insensitive matching for standard attributes
        // type, lang, dir, etc.
        assert_eq!(dom.query_selector_from(host, "[lang=\"en-us\"]"), Some(btn)); // "en-US" matches "en-us"
        assert_eq!(dom.query_selector_from(host, "[lang|=\"en\"]"), Some(btn)); // dash-match
    }

    #[test]
    fn test_t0980_not_selector_with_complex_and_multiple_arguments() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // Build a DOM tree:
        // <div id="wrapper">
        //   <div id="div1">
        //     <p id="p1" class="foo">p1</p>
        //     <span id="s1">s1</span>
        //     <p id="p2">p2</p>
        //   </div>
        //   <div id="div2">
        //     <p id="p3" class="foo">p3</p>
        //   </div>
        // </div>
        let wrapper = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "wrapper".into())],
        });
        dom.append_child(doc, wrapper);

        let div1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "div1".into())],
        });
        dom.append_child(wrapper, div1);

        let p1 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![("id".into(), "p1".into()), ("class".into(), "foo".into())],
        });
        dom.append_child(div1, p1);

        let s1 = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("id".into(), "s1".into())],
        });
        dom.append_child(div1, s1);

        let p2 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![("id".into(), "p2".into())],
        });
        dom.append_child(div1, p2);

        let div2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "div2".into())],
        });
        dom.append_child(wrapper, div2);

        let p3 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![("id".into(), "p3".into()), ("class".into(), "foo".into())],
        });
        dom.append_child(div2, p3);

        // 1. :not(A, B) multiple argument (selector list) matching
        // In div1, children are p1, s1, p2.
        // Elements in div1 that are NOT p1 or s1 should be p2.
        // Let's test with selector list: "div > div > :not(#p1, span)"
        let matches = dom.query_selector_all_from(wrapper, "div > div > :not(#p1, span)");
        // Expected elements matching 'div > div > :not(#p1, span)':
        // - p1: matches #p1, so excluded.
        // - s1: matches span, so excluded.
        // - p2: matches neither, so included.
        // - p3: matches neither, so included.
        // Total matched: p2, p3.
        assert_eq!(matches, vec![p2, p3]);

        // 2. :not(A B) complex selector argument matching
        // Elements under wrapper that are NOT (div with id=div1 followed by any p)
        // div1 p matches p1 and p2, but not p3.
        // So :not(#div1 p) on all p elements should select p3 but not p1 or p2.
        let matches_p = dom.query_selector_all_from(wrapper, "p:not(#div1 p)");
        assert_eq!(matches_p, vec![p3]);

        // 3. :not(A + B) sibling combinator argument matching
        // p2 is preceded by s1. So "span + p" matches p2.
        // Therefore, p:not(span + p) should match p1, p3, but NOT p2.
        let matches_sib = dom.query_selector_all_from(wrapper, "p:not(span + p)");
        assert_eq!(matches_sib, vec![p1, p3]);

        // 4. Nested `:not` and case insensitivity
        // :nOt(:NoT(#p1)) should match p1.
        let matches_nested = dom.query_selector_all_from(wrapper, "p:nOt(:NoT(#p1))");
        assert_eq!(matches_nested, vec![p1]);
    }

    #[test]
    fn test_t1005_selectors_correctness_edge_cases() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // Structure:
        // <div id="test-root">
        //   <div id="div1" class="item" data-empty="">
        //     <span id="span1">S1</span>
        //     <p id="p1">P1</p>
        //     <span id="span2">S2</span>
        //     <p id="p2">P2</p>
        //     <span id="span3">S3</span>
        //     <div id="div2">D2</div>
        //   </div>
        // </div>
        let test_root = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "test-root".into())],
        });
        dom.append_child(doc, test_root);

        let div1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![
                ("id".into(), "div1".into()),
                ("class".into(), "item".into()),
                ("data-empty".into(), "".into()),
            ],
        });
        dom.append_child(test_root, div1);

        let span1 = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("id".into(), "span1".into())],
        });
        dom.append_child(div1, span1);

        let p1 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![("id".into(), "p1".into())],
        });
        dom.append_child(div1, p1);

        let span2 = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("id".into(), "span2".into())],
        });
        dom.append_child(div1, span2);

        let p2 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![("id".into(), "p2".into())],
        });
        dom.append_child(div1, p2);

        let span3 = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("id".into(), "span3".into())],
        });
        dom.append_child(div1, span3);

        let div2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "div2".into())],
        });
        dom.append_child(div1, div2);

        // --- 1. :nth-child / :nth-of-type formulas ---
        // odd/even nth-child
        assert_eq!(
            dom.query_selector_all_from(div1, "span:nth-child(odd)"),
            vec![span1, span2, span3]
        );
        assert_eq!(
            dom.query_selector_all_from(div1, "span:nth-child(even)"),
            vec![] // even children are p1(2), p2(4), div2(6)
        );

        // odd/even nth-of-type
        assert_eq!(
            dom.query_selector_all_from(div1, "span:nth-of-type(odd)"),
            vec![span1, span3]
        );
        assert_eq!(
            dom.query_selector_all_from(div1, "span:nth-of-type(even)"),
            vec![span2]
        );

        // arithmetic nth-child: 2n+1
        assert_eq!(
            dom.query_selector_all_from(div1, "span:nth-child(2n+1)"),
            vec![span1, span2, span3]
        );
        // arithmetic nth-child: -n+2
        assert_eq!(
            dom.query_selector_all_from(div1, "span:nth-child(-n+2)"),
            vec![span1] // children with index <= 2 are span1(1), p1(2). Only span1 is a span.
        );

        // nth-last-child / nth-last-of-type
        // last element in div1 is div2(6).
        assert_eq!(
            dom.query_selector_all_from(div1, "span:nth-last-child(2)"),
            vec![span3] // span3 is 5th child (2nd from end)
        );
        assert_eq!(
            dom.query_selector_all_from(div1, "span:nth-last-of-type(1)"),
            vec![span3]
        );

        // --- 2. Attribute operators with empty value ---
        // data-empty is exactly ""
        assert_eq!(dom.query_selector("[data-empty=\"\"]"), Some(div1));
        // operators with empty value should NOT match
        assert_eq!(dom.query_selector_all("[data-empty^=\"\"]"), vec![]);
        assert_eq!(dom.query_selector_all("[data-empty$=\"\"]"), vec![]);
        assert_eq!(dom.query_selector_all("[data-empty*=\"\"]"), vec![]);
        assert_eq!(dom.query_selector_all("[data-empty~=\"\"]"), vec![]);
        assert_eq!(dom.query_selector_all("[data-empty|=\"\"]"), vec![]);

        // --- 3. Combinator chains and :not()/:is()/:where() handling ---
        // div:not(:is(p, span)) > div
        // div1 is a div, not p or span. It contains div2.
        assert_eq!(
            dom.query_selector_all_from(test_root, "div:not(:is(p, span)) > div"),
            vec![div1, div2] // div1 is child of test-root (which is div), div2 is child of div1 (which is div)
        );

        // using :where (which functions identically to :is but has 0 specificity)
        assert_eq!(
            dom.query_selector_all_from(test_root, "div:not(:where(p, span)) > div"),
            vec![div1, div2]
        );
    }

    #[test]
    fn test_t1023_selectors_exhaustive_coverage() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // Let's build a rich tree for testing all Selectors L4 features:
        // <html>
        //   <head></head>
        //   <body>
        //     <div id="container">
        //       <a id="link1" href="https://example.com">Link 1</a>
        //       <a id="link2">Not a link (no href)</a>
        //
        //       <input id="input_enabled" type="text" />
        //       <input id="input_disabled" type="text" disabled="" />
        //
        //       <input id="input_checked" type="checkbox" checked="" />
        //       <input id="input_unchecked" type="checkbox" />
        //
        //       <input id="input_required" type="text" required="" />
        //       <input id="input_optional" type="text" />
        //
        //       <input id="input_rw" type="text" />
        //       <input id="input_ro" type="text" readonly="" />
        //
        //       <div id="empty_div"></div>
        //       <div id="whitespace_div">   </div>
        //       <div id="nonempty_div"><span></span></div>
        //
        //       <div id="sibling_parent">
        //         <div id="sib1" class="box"></div>
        //         <p id="sib2" class="text"></p>
        //         <span id="sib3" class="marker"></span>
        //       </div>
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

        let container = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "container".into())],
        });
        dom.append_child(body, container);

        let link1 = dom.create_node(NodeData::Element {
            name: "a".into(),
            attrs: vec![
                ("id".into(), "link1".into()),
                ("href".into(), "https://example.com".into()),
            ],
        });
        dom.append_child(container, link1);

        let link2 = dom.create_node(NodeData::Element {
            name: "a".into(),
            attrs: vec![("id".into(), "link2".into())],
        });
        dom.append_child(container, link2);

        let input_enabled = dom.create_node(NodeData::Element {
            name: "input".into(),
            attrs: vec![
                ("id".into(), "input_enabled".into()),
                ("type".into(), "text".into()),
            ],
        });
        dom.append_child(container, input_enabled);

        let input_disabled = dom.create_node(NodeData::Element {
            name: "input".into(),
            attrs: vec![
                ("id".into(), "input_disabled".into()),
                ("type".into(), "text".into()),
                ("disabled".into(), "".into()),
            ],
        });
        dom.append_child(container, input_disabled);

        let input_checked = dom.create_node(NodeData::Element {
            name: "input".into(),
            attrs: vec![
                ("id".into(), "input_checked".into()),
                ("type".into(), "checkbox".into()),
                ("checked".into(), "".into()),
            ],
        });
        dom.append_child(container, input_checked);

        let input_unchecked = dom.create_node(NodeData::Element {
            name: "input".into(),
            attrs: vec![
                ("id".into(), "input_unchecked".into()),
                ("type".into(), "checkbox".into()),
            ],
        });
        dom.append_child(container, input_unchecked);

        let input_required = dom.create_node(NodeData::Element {
            name: "input".into(),
            attrs: vec![
                ("id".into(), "input_required".into()),
                ("type".into(), "text".into()),
                ("required".into(), "".into()),
            ],
        });
        dom.append_child(container, input_required);

        let input_optional = dom.create_node(NodeData::Element {
            name: "input".into(),
            attrs: vec![
                ("id".into(), "input_optional".into()),
                ("type".into(), "text".into()),
            ],
        });
        dom.append_child(container, input_optional);

        let input_rw = dom.create_node(NodeData::Element {
            name: "input".into(),
            attrs: vec![
                ("id".into(), "input_rw".into()),
                ("type".into(), "text".into()),
            ],
        });
        dom.append_child(container, input_rw);

        let input_ro = dom.create_node(NodeData::Element {
            name: "input".into(),
            attrs: vec![
                ("id".into(), "input_ro".into()),
                ("type".into(), "text".into()),
                ("readonly".into(), "".into()),
            ],
        });
        dom.append_child(container, input_ro);

        let empty_div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "empty_div".into())],
        });
        dom.append_child(container, empty_div);

        let whitespace_div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "whitespace_div".into())],
        });
        dom.append_child(container, whitespace_div);
        let ws_text = dom.create_node(NodeData::Text("   ".into()));
        dom.append_child(whitespace_div, ws_text);

        let nonempty_div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "nonempty_div".into())],
        });
        dom.append_child(container, nonempty_div);
        let inner_span = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![],
        });
        dom.append_child(nonempty_div, inner_span);

        let sibling_parent = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "sibling_parent".into())],
        });
        dom.append_child(container, sibling_parent);

        let sib1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "sib1".into()), ("class".into(), "box".into())],
        });
        dom.append_child(sibling_parent, sib1);

        let sib2 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![
                ("id".into(), "sib2".into()),
                ("class".into(), "text".into()),
            ],
        });
        dom.append_child(sibling_parent, sib2);

        let sib3 = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![
                ("id".into(), "sib3".into()),
                ("class".into(), "marker".into()),
            ],
        });
        dom.append_child(sibling_parent, sib3);

        // --- 1. Sibling Combinator matching ---
        // div + p (sib1 is div, sib2 is p)
        assert_eq!(dom.query_selector("div + p"), Some(sib2));
        assert_eq!(dom.query_selector_all("div + p"), vec![sib2]);
        // div ~ span (sib1 is div, sib3 is span)
        assert_eq!(dom.query_selector("div ~ span"), Some(sib3));
        assert_eq!(dom.query_selector_all("div ~ span"), vec![sib3]);
        // p + span (sib2 is p, sib3 is span)
        assert_eq!(dom.query_selector("p + span"), Some(sib3));

        // --- 2. Pseudo-class: :link and :any-link ---
        assert_eq!(dom.query_selector("a:link"), Some(link1));
        assert_eq!(dom.query_selector("a:any-link"), Some(link1));
        assert_eq!(dom.query_selector_all("a:link"), vec![link1]);

        // --- 3. Pseudo-class: :disabled and :enabled ---
        assert_eq!(dom.query_selector("input:disabled"), Some(input_disabled));
        assert_eq!(
            dom.query_selector_all("input:enabled"),
            vec![
                input_enabled,
                input_checked,
                input_unchecked,
                input_required,
                input_optional,
                input_rw,
                input_ro
            ]
        );

        // --- 4. Pseudo-class: :checked ---
        assert_eq!(dom.query_selector("input:checked"), Some(input_checked));

        // --- 5. Pseudo-class: :required and :optional ---
        assert_eq!(dom.query_selector("input:required"), Some(input_required));
        assert_eq!(dom.query_selector("input:optional"), Some(input_enabled)); // first optional input is input_enabled

        // --- 6. Pseudo-class: :read-only and :read-write ---
        assert_eq!(dom.query_selector("input:read-only"), Some(input_disabled));
        assert_eq!(
            dom.query_selector_all("input:read-only"),
            vec![input_disabled, input_ro]
        );
        assert_eq!(dom.query_selector("input:read-write"), Some(input_enabled)); // input_enabled is read-write

        // --- 7. Pseudo-class: :empty ---
        // empty_div is empty, whitespace_div has only whitespace so also considered empty by is_empty definition!
        assert_eq!(dom.query_selector("div:empty"), Some(empty_div));
        let empty_divs = dom.query_selector_all("div:empty");
        assert!(empty_divs.contains(&empty_div));
        assert!(empty_divs.contains(&whitespace_div));
        assert!(!empty_divs.contains(&nonempty_div));

        // --- 8. Pseudo-class: :root ---
        assert_eq!(dom.query_selector(":root"), Some(html));
        assert!(dom.matches(html, ":root"));
        assert!(!dom.matches(body, ":root"));

        // --- 9. Pseudo-class: :scope ---
        // matches context root
        assert_eq!(dom.query_selector_from(container, ":scope"), None); // because :scope is root itself and is excluded from descendants_iter
        assert!(dom.matches(container, ":scope")); // but it matches itself via .matches()

        // --- 10. Complex combination: :not(:has(...)) ---
        // Get divs inside container that do not have a p as descendant
        let divs_without_p = dom.query_selector_all_from(container, "div:not(:has(p))");
        // empty_div, whitespace_div, nonempty_div, sib1 should be in this list (sibling_parent has a p sibling? No, p is child of sibling_parent, so sibling_parent has a p descendant, thus it is excluded).
        assert!(divs_without_p.contains(&empty_div));
        assert!(divs_without_p.contains(&whitespace_div));
        assert!(divs_without_p.contains(&nonempty_div));
        assert!(divs_without_p.contains(&sib1));
        assert!(!divs_without_p.contains(&sibling_parent));
        assert!(!divs_without_p.contains(&container));
    }

    #[test]
    fn test_t1071_compliance_improvements() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // 1. Test get_element_by_id with empty string
        let empty_id_element = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "".into())],
        });
        dom.append_child(doc, empty_id_element);

        // Even though an element with id="" exists, get_element_by_id("") must return None
        assert_eq!(dom.get_element_by_id(""), None);

        // 2. Test sibling/relative matches combined with :has and nested selectors
        let div_parent = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "parent".into())],
        });
        dom.append_child(doc, div_parent);

        let p1 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![("id".into(), "p1".into())],
        });
        dom.append_child(div_parent, p1);

        let p2 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![("id".into(), "p2".into())],
        });
        dom.append_child(div_parent, p2);

        // p1 matches :has(+ p) because p2 is its next sibling
        assert!(dom.matches(p1, "p:has(+ p)"));
        // p2 does not match :has(+ p)
        assert!(!dom.matches(p2, "p:has(+ p)"));

        // 3. Robust case-insensitivity in functional pseudo-classes and tag names
        assert_eq!(
            dom.query_selector("DIV:nOt(:hAs(P))"),
            Some(empty_id_element)
        );
    }
}
