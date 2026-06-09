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
    /// An ordered list containing other semantic nodes.
    OrderedList(Vec<SemanticNode>),
    /// An image with alt text.
    Image { alt: String },
    /// A table with headers and rows.
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
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
            if let Some(NodeData::Element { name, .. }) = dom.data(node_id)
                && name == "ol"
            {
                return vec![SemanticNode::OrderedList(items)];
            }
            return vec![SemanticNode::List(items)];
        }
        Some("img") => {
            let alt = dom
                .get_attribute(node_id, "alt")
                .or_else(|| dom.get_attribute(node_id, "aria-label"))
                .unwrap_or_default()
                .to_string();
            return vec![SemanticNode::Image { alt }];
        }
        Some("table") => {
            return vec![build_table_node(dom, node_id)];
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
                "ul" => {
                    let items = build_children(dom, node_id);
                    vec![SemanticNode::List(items)]
                }
                "ol" => {
                    let items = build_children(dom, node_id);
                    vec![SemanticNode::OrderedList(items)]
                }
                // Image
                "img" => {
                    let alt = attrs
                        .iter()
                        .find(|(n, _)| n == "alt")
                        .map(|(_, v)| v.clone())
                        .or_else(|| {
                            attrs
                                .iter()
                                .find(|(n, _)| n == "aria-label")
                                .map(|(_, v)| v.clone())
                        })
                        .unwrap_or_default();
                    vec![SemanticNode::Image { alt }]
                }
                // Table
                "table" => {
                    vec![build_table_node(dom, node_id)]
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

/// Helper to find all `tr` elements inside a table node that are not inside a nested table.
fn find_table_rows(dom: &Dom, node_id: crate::infra::NodeId, rows: &mut Vec<crate::infra::NodeId>) {
    for &child in dom.children(node_id) {
        if let Some(NodeData::Element { name, .. }) = dom.data(child) {
            if name == "table" {
                continue;
            } else if name == "tr" {
                rows.push(child);
            } else {
                find_table_rows(dom, child, rows);
            }
        }
    }
}

/// Helper to find all cells (th, td) in a table row.
fn find_row_cells(dom: &Dom, node_id: crate::infra::NodeId, cells: &mut Vec<crate::infra::NodeId>) {
    for &child in dom.children(node_id) {
        if let Some(NodeData::Element { name, .. }) = dom.data(child) {
            if name == "td" || name == "th" {
                cells.push(child);
            } else {
                find_row_cells(dom, child, cells);
            }
        }
    }
}

/// Helper to build a table node.
fn build_table_node(dom: &Dom, node_id: crate::infra::NodeId) -> SemanticNode {
    let mut table_rows = Vec::new();
    find_table_rows(dom, node_id, &mut table_rows);

    let mut headers = Vec::new();
    let mut rows = Vec::new();

    for row_id in table_rows {
        let mut cell_nodes = Vec::new();
        find_row_cells(dom, row_id, &mut cell_nodes);

        let mut row_cells = Vec::new();
        let mut is_header_row = false;

        for cell_id in cell_nodes {
            if let Some(NodeData::Element { name, .. }) = dom.data(cell_id)
                && name == "th"
            {
                is_header_row = true;
            }
            let text = dom.text_content(cell_id).trim().to_string();
            row_cells.push(text);
        }

        if !row_cells.is_empty() {
            if is_header_row && headers.is_empty() {
                headers = row_cells;
            } else {
                rows.push(row_cells);
            }
        }
    }

    if headers.is_empty() && !rows.is_empty() {
        headers = rows.remove(0);
    }
    let num_cols = headers.len();
    if num_cols > 0 {
        for row in &mut rows {
            if row.len() < num_cols {
                row.resize(num_cols, String::new());
            } else {
                row.truncate(num_cols);
            }
        }
    }

    SemanticNode::Table { headers, rows }
}

/// Helper to check if a list of semantic nodes contains any Link.
fn has_links(nodes: &[SemanticNode]) -> bool {
    nodes.iter().any(|n| match n {
        SemanticNode::Link { .. } => true,
        SemanticNode::Section(children)
        | SemanticNode::List(children)
        | SemanticNode::OrderedList(children)
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
        SemanticNode::OrderedList(items) => {
            if !result.is_empty() && !result.ends_with("\n\n") {
                if !result.ends_with('\n') {
                    result.push('\n');
                }
                result.push('\n');
            }
            for (index, item) in items.iter().enumerate() {
                result.push_str(&format!("{}. ", index + 1));
                append_markdown(item, result, false);
                if !result.ends_with('\n') {
                    result.push('\n');
                }
            }
            result.push('\n');
        }
        SemanticNode::Image { alt } => {
            result.push_str("![");
            result.push_str(alt);
            result.push(']');
            if is_block {
                result.push_str("\n\n");
            }
        }
        SemanticNode::Table { headers, rows } => {
            if !result.is_empty() && !result.ends_with("\n\n") {
                if !result.ends_with('\n') {
                    result.push('\n');
                }
                result.push('\n');
            }
            if !headers.is_empty() {
                result.push('|');
                for header in headers {
                    result.push(' ');
                    result.push_str(&header.replace('|', "\\|"));
                    result.push_str(" |");
                }
                result.push('\n');

                result.push('|');
                for _ in headers {
                    result.push_str(" --- |");
                }
                result.push('\n');

                for row in rows {
                    result.push('|');
                    for cell in row {
                        result.push(' ');
                        result.push_str(&cell.replace('|', "\\|"));
                        result.push_str(" |");
                    }
                    result.push('\n');
                }
            }
            if is_block {
                result.push('\n');
            }
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
        // Table elements
        "table" => Some("table".to_string()),
        "tr" => Some("row".to_string()),
        "th" => Some("columnheader".to_string()),
        "td" => Some("cell".to_string()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::{Dom, NodeData};

    #[test]
    fn test_list_and_ordered_list() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // <ol>
        //   <li>First</li>
        //   <li>Second</li>
        // </ol>
        let ol = dom.create_node(NodeData::Element {
            name: "ol".into(),
            attrs: vec![],
        });
        let li1 = dom.create_node(NodeData::Element {
            name: "li".into(),
            attrs: vec![],
        });
        let t1 = dom.create_node(NodeData::Text("First".into()));
        dom.append_child(li1, t1);

        let li2 = dom.create_node(NodeData::Element {
            name: "li".into(),
            attrs: vec![],
        });
        let t2 = dom.create_node(NodeData::Text("Second".into()));
        dom.append_child(li2, t2);

        dom.append_child(ol, li1);
        dom.append_child(ol, li2);
        dom.append_child(doc, ol);

        let view = build_semantic_view(&dom);
        assert_eq!(view.roots.len(), 1);

        match &view.roots[0] {
            SemanticNode::OrderedList(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], SemanticNode::ListItem("First".into()));
                assert_eq!(items[1], SemanticNode::ListItem("Second".into()));
            }
            _ => panic!("Expected OrderedList, got {:?}", view.roots[0]),
        }

        let md = to_markdown(&view);
        assert_eq!(md, "1. First\n2. Second");
    }

    #[test]
    fn test_images() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // <img alt="A beautiful sunset">
        let img = dom.create_node(NodeData::Element {
            name: "img".into(),
            attrs: vec![("alt".into(), "A beautiful sunset".into())],
        });
        dom.append_child(doc, img);

        let view = build_semantic_view(&dom);
        assert_eq!(view.roots.len(), 1);
        assert_eq!(
            view.roots[0],
            SemanticNode::Image {
                alt: "A beautiful sunset".into()
            }
        );

        let md = to_markdown(&view);
        assert_eq!(md, "![A beautiful sunset]");
    }

    #[test]
    fn test_tables() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // <table>
        //   <tr>
        //     <th>Header 1</th>
        //     <th>Header 2</th>
        //   </tr>
        //   <tr>
        //     <td>Val 1</td>
        //     <td>Val 2</td>
        //   </tr>
        // </table>
        let table = dom.create_node(NodeData::Element {
            name: "table".into(),
            attrs: vec![],
        });

        let tr1 = dom.create_node(NodeData::Element {
            name: "tr".into(),
            attrs: vec![],
        });
        let th1 = dom.create_node(NodeData::Element {
            name: "th".into(),
            attrs: vec![],
        });
        let th1_text = dom.create_node(NodeData::Text("Header 1".into()));
        dom.append_child(th1, th1_text);
        let th2 = dom.create_node(NodeData::Element {
            name: "th".into(),
            attrs: vec![],
        });
        let th2_text = dom.create_node(NodeData::Text("Header 2".into()));
        dom.append_child(th2, th2_text);
        dom.append_child(tr1, th1);
        dom.append_child(tr1, th2);

        let tr2 = dom.create_node(NodeData::Element {
            name: "tr".into(),
            attrs: vec![],
        });
        let td1 = dom.create_node(NodeData::Element {
            name: "td".into(),
            attrs: vec![],
        });
        let td1_text = dom.create_node(NodeData::Text("Val 1".into()));
        dom.append_child(td1, td1_text);
        let td2 = dom.create_node(NodeData::Element {
            name: "td".into(),
            attrs: vec![],
        });
        let td2_text = dom.create_node(NodeData::Text("Val 2".into()));
        dom.append_child(td2, td2_text);
        dom.append_child(tr2, td1);
        dom.append_child(tr2, td2);

        dom.append_child(table, tr1);
        dom.append_child(table, tr2);
        dom.append_child(doc, table);

        let view = build_semantic_view(&dom);
        assert_eq!(view.roots.len(), 1);

        match &view.roots[0] {
            SemanticNode::Table { headers, rows } => {
                assert_eq!(
                    headers,
                    &vec!["Header 1".to_string(), "Header 2".to_string()]
                );
                assert_eq!(rows, &vec![vec!["Val 1".to_string(), "Val 2".to_string()]]);
            }
            _ => panic!("Expected Table, got {:?}", view.roots[0]),
        }

        let md = to_markdown(&view);
        let expected_md = "| Header 1 | Header 2 |\n| --- | --- |\n| Val 1 | Val 2 |";
        assert_eq!(md, expected_md);
    }
}
