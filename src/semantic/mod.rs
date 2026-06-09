use crate::dom::{Dom, NodeData};

/// A semantic node representing a simplified structure of a DOM node.
///
/// This is a pure derivation from the DOM, used for high-level analysis or
/// serialization to formats like Markdown.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SemanticNode {
    /// A heading with a level (1-6) and plain text content.
    Heading { level: u8, text: String },
    /// A simple paragraph of plain text.
    Paragraph(String),
    /// A hyperlink with text and a URL.
    Link { text: String, href: String },
    /// A list (ordered or unordered) containing other semantic nodes.
    List(Vec<SemanticNode>),
    /// A single item in a list, containing plain text.
    ListItem(String),
    /// A piece of plain text.
    Text(String),
    /// A generic section grouping related semantic nodes.
    Section(Vec<SemanticNode>),

    // --- Wave L / S-63 Form elements ---
    /// A form container.
    Form {
        action: String,
        method: String,
        children: Vec<SemanticNode>,
    },
    /// An input element (textbox, checkbox, radio, textarea, etc.)
    Input {
        label: String,
        input_type: String,
        value: String,
        checked: bool,
    },
    /// A button element.
    Button { label: String, button_type: String },
    /// A select element.
    Select {
        label: String,
        selected: Option<String>,
        options: Vec<String>,
    },
}

/// A simplified, semantic view of a DOM document.
pub struct SemanticView {
    /// The top-level semantic nodes of the document.
    pub roots: Vec<SemanticNode>,
}

/// Builds a `SemanticView` from the given `Dom`.
///
/// This traverses the DOM tree and maps relevant elements to `SemanticNode`s.
/// Elements that are not explicitly recognized are recursed into, and their
/// children are processed.
pub fn build_semantic_view(dom: &Dom) -> SemanticView {
    let mut roots = Vec::new();
    let doc = dom.document();

    // Traverse children of the document root.
    for &child in dom.children(doc) {
        roots.extend(build_nodes(dom, child));
    }

    SemanticView { roots }
}

/// Helper to find the label text associated with a form control.
fn find_label(dom: &Dom, node_id: crate::infra::NodeId) -> String {
    // 1. Check aria-label attribute
    if let Some(label) = dom.get_attribute(node_id, "aria-label") {
        return label.to_string();
    }

    // 2. Check if there's an associated <label> element via `for` attribute
    if let Some(id_val) = dom.get_attribute(node_id, "id") {
        let id_trimmed = id_val.trim();
        if !id_trimmed.is_empty() {
            let doc = dom.document();
            for desc_id in dom.descendants(doc) {
                match dom.data(desc_id) {
                    Some(NodeData::Element { name, .. }) if name == "label" => {
                        if let Some(for_val) = dom.get_attribute(desc_id, "for")
                            && for_val.trim() == id_trimmed
                        {
                            return dom.text_content(desc_id).trim().to_string();
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // 3. Check if input is nested inside a <label> element
    let mut curr = dom.parent(node_id);
    while let Some(p) = curr {
        match dom.data(p) {
            Some(NodeData::Element { name, .. }) if name == "label" => {
                return dom.text_content(p).trim().to_string();
            }
            _ => {}
        }
        curr = dom.parent(p);
    }

    // 4. Check placeholder attribute
    if let Some(placeholder) = dom.get_attribute(node_id, "placeholder") {
        return placeholder.to_string();
    }

    // 5. Check name attribute
    if let Some(name_val) = dom.get_attribute(node_id, "name") {
        return name_val.to_string();
    }

    String::new()
}

/// Recursively builds a list of semantic nodes from a DOM node.
fn build_nodes(dom: &Dom, node_id: crate::infra::NodeId) -> Vec<SemanticNode> {
    // spec: Prune elements hidden via aria-hidden or display: none / visibility: hidden (S-54 / S-63)
    if is_hidden(dom, node_id) {
        return Vec::new();
    }

    // spec: presentation/none roles are pruned (skipped, children promoted)
    if let Some(r) = role(dom, node_id)
        && (r == "presentation" || r == "none")
    {
        return build_children(dom, node_id);
    }

    let Some(data) = dom.data(node_id) else {
        return Vec::new();
    };

    // Try role-based semantic matching first for consistency with AXTree (t0084 / S-63)
    let computed_role = role(dom, node_id);
    match computed_role.as_deref() {
        Some("heading") => {
            let level = if let Some(NodeData::Element { name, .. }) = dom.data(node_id) {
                if name.starts_with('h') && name.len() == 2 {
                    name[1..].parse().unwrap_or(1)
                } else if let Some(level_str) = dom.get_attribute(node_id, "aria-level") {
                    level_str.parse().unwrap_or(2)
                } else {
                    2
                }
            } else {
                2
            };
            return vec![SemanticNode::Heading {
                level,
                text: dom.text_content(node_id),
            }];
        }
        Some("link") => {
            let href = dom
                .get_attribute(node_id, "href")
                .unwrap_or_default()
                .to_string();
            return vec![SemanticNode::Link {
                text: dom.text_content(node_id),
                href,
            }];
        }
        Some("button") => {
            let label = accessible_name(dom, node_id);
            let button_type = dom
                .get_attribute(node_id, "type")
                .unwrap_or("submit")
                .to_ascii_lowercase();
            return vec![SemanticNode::Button { label, button_type }];
        }
        Some("checkbox") => {
            let label = find_label(dom, node_id);
            let value = dom
                .get_attribute(node_id, "value")
                .unwrap_or("on")
                .to_string();
            let checked = dom.get_attribute(node_id, "checked").is_some();
            return vec![SemanticNode::Input {
                label,
                input_type: "checkbox".to_string(),
                value,
                checked,
            }];
        }
        Some("radio") => {
            let label = find_label(dom, node_id);
            let value = dom
                .get_attribute(node_id, "value")
                .unwrap_or("on")
                .to_string();
            let checked = dom.get_attribute(node_id, "checked").is_some();
            return vec![SemanticNode::Input {
                label,
                input_type: "radio".to_string(),
                value,
                checked,
            }];
        }
        Some("textbox" | "searchbox" | "slider" | "spinbutton") => {
            let label = find_label(dom, node_id);
            let mut value = String::new();
            let mut input_type = "text".to_string();
            if let Some(NodeData::Element { name, .. }) = dom.data(node_id) {
                if name == "textarea" {
                    input_type = "textarea".to_string();
                    value = dom.text_content(node_id);
                } else {
                    if let Some(type_val) = dom.get_attribute(node_id, "type") {
                        input_type = type_val.to_ascii_lowercase();
                    }
                    if let Some(value_val) = dom.get_attribute(node_id, "value") {
                        value = value_val.to_string();
                    }
                }
            }
            return vec![SemanticNode::Input {
                label,
                input_type,
                value,
                checked: false,
            }];
        }
        Some("form") => {
            let action = dom
                .get_attribute(node_id, "action")
                .unwrap_or_default()
                .to_string();
            let method = dom
                .get_attribute(node_id, "method")
                .unwrap_or("get")
                .to_ascii_lowercase();
            let children = build_children(dom, node_id);
            return vec![SemanticNode::Form {
                action,
                method,
                children,
            }];
        }
        Some("list") => {
            let items = build_children(dom, node_id);
            return vec![SemanticNode::List(items)];
        }
        Some("listitem") => {
            let children_nodes = build_children(dom, node_id);
            if has_links(&children_nodes) {
                return vec![SemanticNode::Section(children_nodes)];
            } else {
                return vec![SemanticNode::ListItem(dom.text_content(node_id))];
            }
        }
        _ => {}
    }

    match data {
        NodeData::Element { name, attrs } => {
            match name.as_str() {
                // Headings
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    let level = name[1..].parse().unwrap_or(1);
                    vec![SemanticNode::Heading {
                        level,
                        text: dom.text_content(node_id),
                    }]
                }
                // Paragraphs
                "p" => {
                    let children_nodes = build_children(dom, node_id);
                    if has_links(&children_nodes) {
                        vec![SemanticNode::Section(children_nodes)]
                    } else {
                        vec![SemanticNode::Paragraph(dom.text_content(node_id))]
                    }
                }
                // Links
                "a" => {
                    let href = attrs
                        .iter()
                        .find(|(n, _)| n == "href")
                        .map(|(_, v)| v.clone())
                        .unwrap_or_default();
                    vec![SemanticNode::Link {
                        text: dom.text_content(node_id),
                        href,
                    }]
                }
                // Lists
                "ul" | "ol" => {
                    let items = build_children(dom, node_id);
                    vec![SemanticNode::List(items)]
                }
                // List Items
                "li" => {
                    let children_nodes = build_children(dom, node_id);
                    if has_links(&children_nodes) {
                        vec![SemanticNode::Section(children_nodes)]
                    } else {
                        vec![SemanticNode::ListItem(dom.text_content(node_id))]
                    }
                }
                // Form element (fallback)
                "form" => {
                    let action = attrs
                        .iter()
                        .find(|(n, _)| n == "action")
                        .map(|(_, v)| v.clone())
                        .unwrap_or_default();
                    let method = attrs
                        .iter()
                        .find(|(n, _)| n == "method")
                        .map(|(_, v)| v.to_ascii_lowercase())
                        .unwrap_or_else(|| "get".to_string());
                    let children = build_children(dom, node_id);
                    vec![SemanticNode::Form {
                        action,
                        method,
                        children,
                    }]
                }
                // Input elements (fallback)
                "input" => {
                    let label = find_label(dom, node_id);
                    let input_type = attrs
                        .iter()
                        .find(|(n, _)| n == "type")
                        .map(|(_, v)| v.to_ascii_lowercase())
                        .unwrap_or_else(|| "text".to_string());
                    let value = attrs
                        .iter()
                        .find(|(n, _)| n == "value")
                        .map(|(_, v)| v.clone())
                        .unwrap_or_default();
                    let checked = attrs.iter().any(|(k, _)| k == "checked");
                    vec![SemanticNode::Input {
                        label,
                        input_type,
                        value,
                        checked,
                    }]
                }
                // Textarea (fallback)
                "textarea" => {
                    let label = find_label(dom, node_id);
                    let value = dom.text_content(node_id);
                    vec![SemanticNode::Input {
                        label,
                        input_type: "textarea".to_string(),
                        value,
                        checked: false,
                    }]
                }
                // Select elements
                "select" => {
                    let label = find_label(dom, node_id);
                    let mut options = Vec::new();
                    let mut selected = None;
                    for &child in dom.children(node_id) {
                        match dom.data(child) {
                            Some(NodeData::Element {
                                name: child_name, ..
                            }) if child_name == "option" => {
                                let opt_text = dom.text_content(child);
                                options.push(opt_text.clone());
                                if dom.get_attribute(child, "selected").is_some() {
                                    selected = Some(opt_text);
                                }
                            }
                            _ => {}
                        }
                    }
                    if selected.is_none() && !options.is_empty() {
                        selected = Some(options[0].clone());
                    }
                    vec![SemanticNode::Select {
                        label,
                        selected,
                        options,
                    }]
                }
                // Sections and other block containers
                "div" | "section" | "article" | "main" | "nav" | "aside" | "header" | "footer" => {
                    vec![SemanticNode::Section(build_children(dom, node_id))]
                }
                // Inline text formatting (flatten to Text)
                "strong" | "b" | "em" | "i" | "span" => build_children(dom, node_id),
                // Unknown elements: recurse into children
                _ => build_children(dom, node_id),
            }
        }
        NodeData::Text(s) => {
            if s.trim().is_empty() {
                Vec::new()
            } else {
                vec![SemanticNode::Text(s.clone())]
            }
        }
        _ => Vec::new(),
    }
}

/// Helper to build semantic nodes for all children of a DOM node.
fn build_children(dom: &Dom, node_id: crate::infra::NodeId) -> Vec<SemanticNode> {
    dom.children(node_id)
        .iter()
        .flat_map(|&c| build_nodes(dom, c))
        .collect()
}

/// Helper to check if a list of semantic nodes contains any Link.
fn has_links(nodes: &[SemanticNode]) -> bool {
    nodes.iter().any(|n| match n {
        SemanticNode::Link { .. } => true,
        SemanticNode::Section(children)
        | SemanticNode::List(children)
        | SemanticNode::Form { children, .. } => has_links(children),
        _ => false,
    })
}

/// Serializes a `SemanticView` to a Markdown string.
pub fn to_markdown(view: &SemanticView) -> String {
    let mut result = String::new();
    for node in &view.roots {
        append_markdown(node, &mut result, true);
    }
    result.trim().to_string()
}

/// Recursively appends Markdown representation of a node to the result string.
fn append_markdown(node: &SemanticNode, result: &mut String, is_block: bool) {
    match node {
        SemanticNode::Heading { level, text } => {
            if !result.is_empty() && !result.ends_with("\n\n") {
                if !result.ends_with('\n') {
                    result.push('\n');
                }
                result.push('\n');
            }
            for _ in 0..*level {
                result.push('#');
            }
            result.push(' ');
            result.push_str(text);
            result.push_str("\n\n");
        }
        SemanticNode::Paragraph(text) => {
            if !result.is_empty() && !result.ends_with("\n\n") {
                if !result.ends_with('\n') {
                    result.push('\n');
                }
                result.push('\n');
            }
            result.push_str(text);
            result.push_str("\n\n");
        }
        SemanticNode::Link { text, href } => {
            result.push('[');
            result.push_str(text);
            result.push(']');
            result.push('(');
            result.push_str(href);
            result.push(')');
            if is_block {
                result.push_str("\n\n");
            }
        }
        SemanticNode::List(items) => {
            if !result.is_empty() && !result.ends_with("\n\n") {
                if !result.ends_with('\n') {
                    result.push('\n');
                }
                result.push('\n');
            }
            for item in items {
                result.push_str("- ");
                append_markdown(item, result, false);
                if !result.ends_with('\n') {
                    result.push('\n');
                }
            }
            result.push('\n');
        }
        SemanticNode::ListItem(text) => {
            result.push_str(text);
        }
        SemanticNode::Text(text) => {
            result.push_str(text);
        }
        SemanticNode::Section(children) => {
            let mut sub = String::new();
            for child in children {
                append_markdown(child, &mut sub, false);
            }
            if !sub.is_empty() {
                if is_block && !result.is_empty() && !result.ends_with("\n\n") {
                    if !result.ends_with('\n') {
                        result.push('\n');
                    }
                    result.push('\n');
                }
                result.push_str(&sub);
                if is_block {
                    result.push_str("\n\n");
                }
            }
        }
        SemanticNode::Form {
            action,
            method,
            children,
        } => {
            if is_block && !result.is_empty() && !result.ends_with("\n\n") {
                if !result.ends_with('\n') {
                    result.push('\n');
                }
                result.push('\n');
            }
            result.push_str(&format!("[Form: {} ({})]\n", action, method));
            let mut sub = String::new();
            for child in children {
                append_markdown(child, &mut sub, true);
            }
            result.push_str(sub.trim());
            result.push_str("\n\n");
        }
        SemanticNode::Input {
            label,
            input_type,
            value,
            checked,
        } => {
            if is_block && !result.is_empty() && !result.ends_with("\n\n") {
                if !result.ends_with('\n') {
                    result.push('\n');
                }
                result.push('\n');
            }
            if input_type == "checkbox" {
                let box_char = if *checked { 'x' } else { ' ' };
                if label.is_empty() {
                    result.push_str(&format!("[{}]", box_char));
                } else {
                    result.push_str(&format!("[{}] {}", box_char, label));
                }
            } else if input_type == "radio" {
                let radio_char = if *checked { 'x' } else { ' ' };
                if label.is_empty() {
                    result.push_str(&format!("({})", radio_char));
                } else {
                    result.push_str(&format!("({}) {}", radio_char, label));
                }
            } else {
                if label.is_empty() {
                    if value.is_empty() {
                        result.push_str("[Input]");
                    } else {
                        result.push_str(&format!("[Input: {}]", value));
                    }
                } else {
                    if value.is_empty() {
                        result.push_str(&format!("{}: [ ]", label));
                    } else {
                        result.push_str(&format!("{}: [{}]", label, value));
                    }
                }
            }
            if is_block {
                result.push_str("\n\n");
            }
        }
        SemanticNode::Button {
            label,
            button_type: _,
        } => {
            if is_block && !result.is_empty() && !result.ends_with("\n\n") {
                if !result.ends_with('\n') {
                    result.push('\n');
                }
                result.push('\n');
            }
            if label.is_empty() {
                result.push_str("[Button]");
            } else {
                result.push_str(&format!("[Button: {}]", label));
            }
            if is_block {
                result.push_str("\n\n");
            }
        }
        SemanticNode::Select {
            label,
            selected,
            options: _,
        } => {
            if is_block && !result.is_empty() && !result.ends_with("\n\n") {
                if !result.ends_with('\n') {
                    result.push('\n');
                }
                result.push('\n');
                result.push('\n');
            }
            if label.is_empty() {
                if let Some(sel) = selected {
                    result.push_str(&format!("[Select: {}]", sel));
                } else {
                    result.push_str("[Select]");
                }
            } else {
                if let Some(sel) = selected {
                    result.push_str(&format!("{}: [{}] v", label, sel));
                } else {
                    result.push_str(&format!("{}: [ ] v", label));
                }
            }
            if is_block {
                result.push_str("\n\n");
            }
        }
    }
}

/// Computes the implicit or explicit ARIA role of a DOM element node.
///
/// This implements SPEC S-42: maps HTML elements to their implicit ARIA roles
/// (e.g., `button` -> "button", `a[href]` -> "link", `h1`-`h6` -> "heading", `img` -> "img",
/// `input` -> by type, landmarks like `nav` -> "navigation"), and honors any explicit `role`
/// attribute (taking the first non-empty role token specified).
///
/// If the node is not an element, or the ID is invalid, returns `None`.
// TODO(spec): Support full HTML-ARIA mapping, conditional roles, and role hierarchy.
pub fn role(dom: &Dom, node: crate::infra::NodeId) -> Option<String> {
    let data = dom.data(node)?;

    let NodeData::Element { name, attrs } = data else {
        return None;
    };

    // 1. Honoring explicit role attribute (first token if multi-valued)
    if let Some((_, role_val)) = attrs.iter().find(|(k, _)| k == "role") {
        let trimmed = role_val.trim();
        if let Some(first_role) = trimmed.split_whitespace().next() {
            return Some(first_role.to_string());
        }
    }

    // 2. Implicit ARIA roles based on element tag name
    match name.as_str() {
        "button" => Some("button".to_string()),
        "a" => {
            // "a" has "link" role ONLY if it has "href"
            if attrs.iter().any(|(k, _)| k == "href") {
                Some("link".to_string())
            } else {
                None
            }
        }
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => Some("heading".to_string()),
        "img" => Some("img".to_string()),
        "input" => {
            // Determine input type, defaulting to "text" if missing
            let input_type = attrs
                .iter()
                .find(|(k, _)| k == "type")
                .map(|(_, v)| v.to_ascii_lowercase())
                .unwrap_or_else(|| "text".to_string());

            match input_type.as_str() {
                "button" | "image" | "submit" | "reset" => Some("button".to_string()),
                "checkbox" => Some("checkbox".to_string()),
                "radio" => Some("radio".to_string()),
                "search" => Some("searchbox".to_string()),
                "range" => Some("slider".to_string()),
                "number" => Some("spinbutton".to_string()),
                // text, email, tel, url, password, or unrecognized -> textbox
                _ => Some("textbox".to_string()),
            }
        }
        // Landmark elements
        "nav" => Some("navigation".to_string()),
        "main" => Some("main".to_string()),
        "aside" => Some("complementary".to_string()),
        "header" => Some("banner".to_string()),
        "footer" => Some("contentinfo".to_string()),
        "article" => Some("article".to_string()),
        "section" => Some("region".to_string()),
        "form" => Some("form".to_string()),
        // List elements
        "ul" | "ol" => Some("list".to_string()),
        "li" => Some("listitem".to_string()),
        _ => None,
    }
}

/// Computes the accessible name of a DOM node.
///
/// This implements SPEC S-42: computes the accessible name from the `aria-label`
/// attribute, the `alt` attribute (for elements like images), or falls back to
/// the node's text content.
///
/// If the node is invalid or cannot have an accessible name, returns an empty string.
// TODO(spec): Implement full AccName 1.1 computation algorithm, including aria-labelledby, aria-describedby, and layout visibility.
pub fn accessible_name(dom: &Dom, node: crate::infra::NodeId) -> String {
    let Some(data) = dom.data(node) else {
        return String::new();
    };

    match data {
        NodeData::Element { attrs, .. } => {
            // 1. aria-label attribute
            if let Some((_, label)) = attrs.iter().find(|(k, _)| k == "aria-label") {
                return label.clone();
            }

            // 2. alt attribute
            if let Some((_, alt)) = attrs.iter().find(|(k, _)| k == "alt") {
                return alt.clone();
            }

            // 3. Fall back to text content
            dom.text_content(node)
        }
        NodeData::Text(s) => s.clone(),
        _ => String::new(),
    }
}

/// An accessibility node in the lightweight accessibility tree (AXTree).
///
/// This implements SPEC S-54: represents a simplified DOM structure
/// combined with ARIA roles and accessible names.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AxNode {
    /// The ARIA role of the node, if any.
    pub role: Option<String>,
    /// The computed accessible name of the node.
    pub name: String,
    /// The children of this accessibility node.
    pub children: Vec<AxNode>,
}

/// Builds a lightweight accessibility tree (AXTree) from the given `Dom`.
///
/// This implements SPEC S-54: recursively builds an `AxNode` hierarchy,
/// reusing implicit and explicit roles and accessible name computation.
/// Nodes with a "presentation" or "none" role are pruned (the node itself is
/// excluded but its children are promoted), and elements hidden via
/// `aria-hidden="true"` or hidden inline styles (e.g. `display: none`)
/// are entirely pruned along with their subtrees.
pub fn ax_tree(dom: &Dom) -> AxNode {
    let doc = dom.document();
    let mut root_nodes = process_ax_node(dom, doc);

    // spec: S-54: Return a single root AxNode representing the document.
    // If process_ax_node returns multiple or zero nodes (e.g., if everything is hidden),
    // we fallback to an empty default AxNode.
    root_nodes.pop().unwrap_or_else(|| AxNode {
        role: None,
        name: String::new(),
        children: Vec::new(),
    })
}

/// Helper to recursively process a DOM node and return its corresponding `AxNode` list.
///
/// Returns a `Vec<AxNode>` to elegantly support promoting children of presentational/none nodes,
/// as well as skipping ignored or hidden subtrees entirely.
fn process_ax_node(dom: &Dom, node_id: crate::infra::NodeId) -> Vec<AxNode> {
    let Some(data) = dom.data(node_id) else {
        return Vec::new();
    };

    // spec: S-54: elements hidden via aria-hidden or display: none/visibility: hidden inline styles are pruned.
    if is_hidden(dom, node_id) {
        return Vec::new();
    }

    match data {
        NodeData::Text(s) => {
            // Trim empty/whitespace text nodes to avoid cluttering the tree.
            if s.trim().is_empty() {
                Vec::new()
            } else {
                vec![AxNode {
                    role: None,
                    name: s.clone(),
                    children: Vec::new(),
                }]
            }
        }
        NodeData::Element { .. } => {
            let r = role(dom, node_id);

            // spec: S-54: presentation/none roles are pruned (node is skipped, children promoted).
            if matches!(r.as_deref(), Some("presentation" | "none")) {
                let mut promoted_children = Vec::new();
                for &child in dom.children(node_id) {
                    promoted_children.extend(process_ax_node(dom, child));
                }
                return promoted_children;
            }

            // Normal element: compute role, accessible name, and recursively process children.
            let name = accessible_name(dom, node_id);
            let mut children = Vec::new();
            for &child in dom.children(node_id) {
                children.extend(process_ax_node(dom, child));
            }

            vec![AxNode {
                role: r,
                name,
                children,
            }]
        }
        NodeData::Document => {
            // Document root is processed, returning its children inside a container AxNode.
            let mut children = Vec::new();
            for &child in dom.children(node_id) {
                children.extend(process_ax_node(dom, child));
            }
            vec![AxNode {
                role: None,
                name: String::new(),
                children,
            }]
        }
        NodeData::Doctype { .. } | NodeData::Comment(_) => {
            // spec: S-54: Skip Doctype and Comment nodes from the accessibility tree.
            Vec::new()
        }
    }
}

/// Helper to determine if a node is hidden (e.g. via `aria-hidden="true"` or inline `display: none`).
// TODO(spec): Support full display and visibility coupling from style resolution / layout.
fn is_hidden(dom: &Dom, node_id: crate::infra::NodeId) -> bool {
    let Some(data) = dom.data(node_id) else {
        return false;
    };

    if let NodeData::Element { attrs, .. } = data {
        // spec: Check aria-hidden="true" (case-insensitive)
        if attrs
            .iter()
            .any(|(k, v)| k == "aria-hidden" && v.trim().eq_ignore_ascii_case("true"))
        {
            return true;
        }

        // spec: Check basic display: none or visibility: hidden inline styles.
        if let Some((_, style_val)) = attrs.iter().find(|(k, _)| k == "style") {
            let normalized = style_val.to_ascii_lowercase();
            if normalized.contains("display:none")
                || normalized.contains("display: none")
                || normalized.contains("visibility:hidden")
                || normalized.contains("visibility: hidden")
            {
                return true;
            }
        }
    }

    false
}
