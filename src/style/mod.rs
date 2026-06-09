use crate::css::parser::{Declaration, Rule, Stylesheet};
use crate::css::values::{CssValue, LengthUnit, parse_value};
use crate::dom::Dom;
use crate::infra::NodeId;
use crate::selector::{ComplexSelector, Component, matches_complex};
use std::collections::HashMap;

/// A map of property names to their computed values.
#[derive(Debug, Default, Clone)]
pub struct ComputedStyle {
    properties: HashMap<String, CssValue>,
}

impl ComputedStyle {
    /// Returns the computed value for a given property, if it exists.
    pub fn get(&self, property: &str) -> Option<&CssValue> {
        self.properties.get(property)
    }
}

/// Computes the specificity of a complex selector.
/// Returns a tuple of (inline, id, class, type) counts.
// spec: https://www.w3.org/TR/selectors-4/#specificity
pub fn specificity(sel: &ComplexSelector) -> (u32, u32, u32, u32) {
    let mut id = 0;
    let mut class = 0;
    let mut type_ = 0;

    for (_, compound) in &sel.parts {
        for component in &compound.components {
            match component {
                Component::Id(_) => id += 1,
                Component::Class(_)
                | Component::Attribute { .. }
                | Component::PseudoClass(_)
                | Component::NthChild(_, _)
                | Component::FirstChild
                | Component::LastChild => class += 1,
                Component::Type(_) | Component::PseudoElement(_) => type_ += 1,
                Component::Universal => {}
                Component::Not(compound) => {
                    for sub_comp in &compound.components {
                        match sub_comp {
                            Component::Id(_) => id += 1,
                            Component::Class(_)
                            | Component::Attribute { .. }
                            | Component::PseudoClass(_)
                            | Component::NthChild(_, _)
                            | Component::FirstChild
                            | Component::LastChild => class += 1,
                            Component::Type(_) | Component::PseudoElement(_) => type_ += 1,
                            Component::Universal => {}
                            // Nested :not() (e.g. `:not(:not(.x))`) can be parsed, so count
                            // it like a class rather than panicking on crafted input (I-6).
                            // TODO(spec): recurse for exact Selectors-4 specificity.
                            Component::Not(_) => class += 1,
                        }
                    }
                }
            }
        }
    }

    (0, id, class, type_)
}

/// Computes the styles for all nodes in the DOM based on the given stylesheet.
pub fn compute_styles(dom: &Dom, stylesheet: &Stylesheet) -> HashMap<NodeId, ComputedStyle> {
    compute_styles_with_viewport(dom, stylesheet, 1024.0)
}

/// Computes the styles for all nodes in the DOM based on the given stylesheet and viewport width.
pub fn compute_styles_with_viewport(
    dom: &Dom,
    stylesheet: &Stylesheet,
    viewport_width: f32,
) -> HashMap<NodeId, ComputedStyle> {
    let mut styles = HashMap::new();
    let root = dom.document();

    // Traverse the DOM in pre-order to resolve styles, allowing inheritance from parents.
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        let computed = compute_node_style(dom, node, stylesheet, &styles, viewport_width);
        styles.insert(node, computed);

        // Add children to stack in reverse order to maintain pre-order traversal.
        for &child in dom.children(node).iter().rev() {
            stack.push(child);
        }
    }

    styles
}

fn compute_node_style(
    dom: &Dom,
    node: NodeId,
    stylesheet: &Stylesheet,
    computed_styles: &HashMap<NodeId, ComputedStyle>,
    viewport_width: f32,
) -> ComputedStyle {
    let mut properties = HashMap::new();

    // 1. Collect all matching rules and their declarations.
    let mut matched_declarations = Vec::new();

    collect_matched_rules(
        dom,
        node,
        &stylesheet.rules,
        viewport_width,
        &mut matched_declarations,
    );

    // 2. Add declarations from inline style attribute.
    if let Some(crate::dom::NodeData::Element { attrs, .. }) = dom.data(node)
        && let Some((_, style_attr)) = attrs.iter().find(|(name, _)| name == "style")
    {
        let mut tokenizer = crate::css::CssTokenizer::new(style_attr);
        let declarations = parse_declarations(&mut tokenizer);
        for decl in declarations {
            matched_declarations.push(MatchedDeclaration {
                declaration: decl,
                specificity: (1, 0, 0, 0),
                source_order: 0, // Doesn't matter for inline style if specificity is highest
            });
        }
    }

    // 3. Cascade: sort by (!important, specificity, source order).
    // spec: https://www.w3.org/TR/css-cascade-4/#cascading
    matched_declarations.sort_by(|a, b| {
        // !important
        if a.declaration.important != b.declaration.important {
            return a.declaration.important.cmp(&b.declaration.important);
        }
        // specificity
        if a.specificity != b.specificity {
            return a.specificity.cmp(&b.specificity);
        }
        // source order
        a.source_order.cmp(&b.source_order)
    });

    // 4. Apply declarations.
    for matched in matched_declarations {
        if let Some(value) = parse_value(&matched.declaration.value) {
            match matched.declaration.name.as_str() {
                "margin" => {
                    // spec: https://www.w3.org/TR/css-box-3/#propdef-margin
                    let (top, right, bottom, left) = expand_1_to_4(&value);
                    properties.insert("margin-top".to_string(), top);
                    properties.insert("margin-right".to_string(), right);
                    properties.insert("margin-bottom".to_string(), bottom);
                    properties.insert("margin-left".to_string(), left);
                }
                "padding" => {
                    // spec: https://www.w3.org/TR/css-box-3/#propdef-padding
                    let (top, right, bottom, left) = expand_1_to_4(&value);
                    properties.insert("padding-top".to_string(), top);
                    properties.insert("padding-right".to_string(), right);
                    properties.insert("padding-bottom".to_string(), bottom);
                    properties.insert("padding-left".to_string(), left);
                }
                "border-width" => {
                    // spec: https://www.w3.org/TR/css-backgrounds-3/#border-width
                    let (top, right, bottom, left) = expand_1_to_4(&value);
                    properties.insert("border-top-width".to_string(), top);
                    properties.insert("border-right-width".to_string(), right);
                    properties.insert("border-bottom-width".to_string(), bottom);
                    properties.insert("border-left-width".to_string(), left);
                }
                "border" => {
                    // spec: https://www.w3.org/TR/css-backgrounds-3/#border-shorthands
                    // At least expand border-*-width longhands.
                    if let Some(width) = find_border_width(&value) {
                        properties.insert("border-top-width".to_string(), width.clone());
                        properties.insert("border-right-width".to_string(), width.clone());
                        properties.insert("border-bottom-width".to_string(), width.clone());
                        properties.insert("border-left-width".to_string(), width.clone());
                    } else {
                        let medium = CssValue::Keyword("medium".to_string());
                        properties.insert("border-top-width".to_string(), medium.clone());
                        properties.insert("border-right-width".to_string(), medium.clone());
                        properties.insert("border-bottom-width".to_string(), medium.clone());
                        properties.insert("border-left-width".to_string(), medium.clone());
                    }
                    // TODO(spec): border-style, border-color, etc.
                }
                // TODO(spec): other shorthand properties like background, font, transition, etc.
                name => {
                    properties.insert(name.to_string(), value);
                }
            }
        }
    }

    // 5. Inheritance.
    // spec: https://www.w3.org/TR/css-cascade-4/#inheritance
    if let Some(parent_style) = dom
        .parent(node)
        .and_then(|parent| computed_styles.get(&parent))
    {
        for (prop, val) in &parent_style.properties {
            if is_inherited_property(prop) && !properties.contains_key(prop) {
                properties.insert(prop.clone(), val.clone());
            }
        }
    }

    // --- 6. Resolution of text/font properties (S-43) ---
    let parent_style = dom
        .parent(node)
        .and_then(|parent| computed_styles.get(&parent));

    // A. Resolve font-size first, as line-height depends on it.
    let resolved_font_size = {
        let raw_fs = properties.get("font-size");
        match raw_fs {
            Some(CssValue::Keyword(s)) if s == "inherit" => {
                let parent_fs = parent_style.and_then(|s| s.get("font-size")).cloned();
                parent_fs.unwrap_or(CssValue::Length(16.0, LengthUnit::Px))
            }
            Some(CssValue::Keyword(s)) if s == "initial" => CssValue::Length(16.0, LengthUnit::Px),
            Some(CssValue::Keyword(s)) => {
                let px = match s.to_ascii_lowercase().as_str() {
                    "xx-small" => 8.0,
                    "x-small" => 10.0,
                    "small" => 13.0,
                    "medium" => 16.0,
                    "large" => 20.0,
                    "x-large" => 24.0,
                    "xx-large" => 32.0,
                    _ => 16.0,
                };
                CssValue::Length(px, LengthUnit::Px)
            }
            Some(CssValue::Length(val, unit)) => match unit {
                LengthUnit::Px => CssValue::Length(*val, LengthUnit::Px),
                LengthUnit::Pt => CssValue::Length(*val * 96.0 / 72.0, LengthUnit::Px),
                LengthUnit::Em | LengthUnit::Percent => {
                    let parent_px = parent_style
                        .and_then(|s| s.get("font-size"))
                        .and_then(|v| match v {
                            CssValue::Length(px, LengthUnit::Px) => Some(*px),
                            _ => None,
                        })
                        .unwrap_or(16.0);
                    let factor = if *unit == LengthUnit::Percent {
                        *val / 100.0
                    } else {
                        *val
                    };
                    CssValue::Length(factor * parent_px, LengthUnit::Px)
                }
                LengthUnit::Rem => {
                    let root_px = get_root_font_size(dom, computed_styles);
                    CssValue::Length(*val * root_px, LengthUnit::Px)
                }
            },
            Some(_) => CssValue::Length(16.0, LengthUnit::Px),
            None => CssValue::Length(16.0, LengthUnit::Px),
        }
    };
    let own_fs_val = match &resolved_font_size {
        CssValue::Length(px, LengthUnit::Px) => *px,
        _ => 16.0,
    };
    properties.insert("font-size".to_string(), resolved_font_size);

    // B. Resolve font-weight
    let resolved_font_weight = {
        let raw_fw = properties.get("font-weight");
        match raw_fw {
            Some(CssValue::Keyword(s)) if s == "inherit" => {
                let parent_fw = parent_style.and_then(|s| s.get("font-weight")).cloned();
                parent_fw.unwrap_or(CssValue::Keyword("normal".to_string()))
            }
            Some(CssValue::Keyword(s)) if s == "initial" => CssValue::Keyword("normal".to_string()),
            Some(val) => val.clone(),
            None => CssValue::Keyword("normal".to_string()),
        }
    };
    properties.insert("font-weight".to_string(), resolved_font_weight);

    // C. Resolve line-height
    let resolved_line_height = {
        let raw_lh = properties.get("line-height");
        match raw_lh {
            Some(CssValue::Keyword(s)) if s == "inherit" => {
                let parent_lh = parent_style.and_then(|s| s.get("line-height")).cloned();
                parent_lh.unwrap_or(CssValue::Keyword("normal".to_string()))
            }
            Some(CssValue::Keyword(s)) if s == "initial" => CssValue::Keyword("normal".to_string()),
            Some(CssValue::Length(val, unit)) => match unit {
                LengthUnit::Px => CssValue::Length(*val, LengthUnit::Px),
                LengthUnit::Pt => CssValue::Length(*val * 96.0 / 72.0, LengthUnit::Px),
                LengthUnit::Em | LengthUnit::Percent => {
                    let factor = if *unit == LengthUnit::Percent {
                        *val / 100.0
                    } else {
                        *val
                    };
                    CssValue::Length(factor * own_fs_val, LengthUnit::Px)
                }
                LengthUnit::Rem => {
                    let root_px = get_root_font_size(dom, computed_styles);
                    CssValue::Length(*val * root_px, LengthUnit::Px)
                }
            },
            Some(CssValue::Number(val)) => CssValue::Number(*val),
            Some(val) => val.clone(),
            None => CssValue::Keyword("normal".to_string()),
        }
    };
    properties.insert("line-height".to_string(), resolved_line_height);

    // D. Resolve text-align
    let resolved_text_align = {
        let raw_ta = properties.get("text-align");
        match raw_ta {
            Some(CssValue::Keyword(s)) if s == "inherit" => {
                let parent_ta = parent_style.and_then(|s| s.get("text-align")).cloned();
                parent_ta.unwrap_or(CssValue::Keyword("left".to_string()))
            }
            Some(CssValue::Keyword(s)) if s == "initial" => CssValue::Keyword("left".to_string()),
            Some(val) => val.clone(),
            None => CssValue::Keyword("left".to_string()),
        }
    };
    properties.insert("text-align".to_string(), resolved_text_align);

    ComputedStyle { properties }
}

fn get_root_font_size(dom: &Dom, computed_styles: &HashMap<NodeId, ComputedStyle>) -> f32 {
    let document_node = dom.document();
    let root_element = dom
        .children(document_node)
        .iter()
        .find(|&&child| matches!(dom.data(child), Some(crate::dom::NodeData::Element { .. })));
    if let Some(&root_id) = root_element
        && let Some(root_style) = computed_styles.get(&root_id)
        && let Some(CssValue::Length(px, LengthUnit::Px)) = root_style.get("font-size")
    {
        return *px;
    }
    16.0
}

fn collect_matched_rules(
    dom: &Dom,
    node: NodeId,
    rules: &[Rule],
    viewport_width: f32,
    matched_declarations: &mut Vec<MatchedDeclaration>,
) {
    for (rule_index, rule) in rules.iter().enumerate() {
        match rule {
            Rule::Qualified(qualified_rule) => {
                let selector_str = serialize_component_values(&qualified_rule.prelude);
                if let Ok(selector_list) = crate::selector::parse_selector_list(&selector_str) {
                    for sel in &selector_list.0 {
                        if matches_complex(sel, dom, node) {
                            let spec = specificity(sel);
                            for decl in &qualified_rule.declarations {
                                matched_declarations.push(MatchedDeclaration {
                                    declaration: decl.clone(),
                                    specificity: spec,
                                    source_order: rule_index,
                                });
                            }
                        }
                    }
                }
            }
            Rule::At(at_rule)
                if at_rule.name == "media"
                    && evaluate_media_query(&at_rule.prelude, viewport_width) =>
            {
                if let Some(block) = &at_rule.block {
                    let inner_css = serialize_component_values(block);
                    let inner_stylesheet = crate::css::parser::parse_stylesheet(&inner_css);
                    collect_matched_rules(
                        dom,
                        node,
                        &inner_stylesheet.rules,
                        viewport_width,
                        matched_declarations,
                    );
                }
            }
            _ => {}
        }
    }
}

fn evaluate_media_query(
    prelude: &[crate::css::parser::ComponentValue],
    viewport_width: f32,
) -> bool {
    use crate::css::CssToken;
    use crate::css::parser::ComponentValue;

    // Very basic evaluation: look for (max-width: Npx) or (min-width: Npx)
    for val in prelude {
        if let ComponentValue::SimpleBlock {
            associated: '(',
            value,
        } = val
        {
            let mut i = 0;
            while i < value.len() {
                if let ComponentValue::Token(CssToken::Ident(name)) = &value[i]
                    && (name == "max-width" || name == "min-width")
                    && i + 2 < value.len()
                    && let ComponentValue::Token(CssToken::Colon) = &value[i + 1]
                {
                    // Skip whitespace
                    let mut next_idx = i + 2;
                    while next_idx < value.len() {
                        if let ComponentValue::Token(CssToken::Whitespace) = &value[next_idx] {
                            next_idx += 1;
                        } else {
                            break;
                        }
                    }
                    if next_idx < value.len()
                        && let ComponentValue::Token(CssToken::Dimension { value: v, unit }) =
                            &value[next_idx]
                        && unit == "px"
                    {
                        if name == "max-width" {
                            return viewport_width <= *v as f32;
                        } else if name == "min-width" {
                            return viewport_width >= *v as f32;
                        }
                    }
                }
                i += 1;
            }
        }
    }

    // TODO(spec): Other media features
    true // Default to true if not recognized or no condition
}

struct PeekableTokenizer<'a> {
    tokenizer: &'a mut crate::css::CssTokenizer,
    peeked: Option<crate::css::CssToken>,
}

impl<'a> PeekableTokenizer<'a> {
    fn new(tokenizer: &'a mut crate::css::CssTokenizer) -> Self {
        Self {
            tokenizer,
            peeked: None,
        }
    }

    fn next_token(&mut self) -> crate::css::CssToken {
        if let Some(token) = self.peeked.take() {
            token
        } else {
            self.tokenizer.next_token()
        }
    }

    fn peek_token(&mut self) -> &crate::css::CssToken {
        self.peeked
            .get_or_insert_with(|| self.tokenizer.next_token())
    }
}

/// Parses a list of declarations from a tokenizer.
// spec: https://www.w3.org/TR/css-syntax-3/#consume-list-of-declarations
/// We implement a simplified version here since we can't edit src/css/parser.rs.
fn parse_declarations(tokenizer: &mut crate::css::CssTokenizer) -> Vec<Declaration> {
    let mut declarations = Vec::new();
    let mut pt = PeekableTokenizer::new(tokenizer);
    loop {
        let token = pt.next_token();
        match token {
            crate::css::CssToken::Whitespace | crate::css::CssToken::Semicolon => {}
            crate::css::CssToken::Eof => return declarations,
            crate::css::CssToken::Ident(name) => {
                // Consume until colon
                let mut tokens = vec![crate::css::CssToken::Ident(name)];
                loop {
                    let next = pt.peek_token();
                    if *next == crate::css::CssToken::Semicolon
                        || *next == crate::css::CssToken::Eof
                    {
                        break;
                    }
                    tokens.push(pt.next_token());
                }
                if let Some(decl) = parse_declaration_from_tokens(tokens) {
                    declarations.push(decl);
                }
            }
            _ => {
                // Skip until semicolon or EOF
                loop {
                    let next = pt.peek_token();
                    if *next == crate::css::CssToken::Semicolon
                        || *next == crate::css::CssToken::Eof
                    {
                        break;
                    }
                    pt.next_token();
                }
            }
        }
    }
}

fn parse_declaration_from_tokens(tokens: Vec<crate::css::CssToken>) -> Option<Declaration> {
    let mut it = tokens.into_iter();
    let name = if let Some(crate::css::CssToken::Ident(name)) = it.next() {
        name
    } else {
        return None;
    };

    let mut next = it.next();
    while let Some(crate::css::CssToken::Whitespace) = next {
        next = it.next();
    }

    if next != Some(crate::css::CssToken::Colon) {
        return None;
    }

    let mut tokens_for_value: Vec<crate::css::CssToken> = it.collect();
    let mut important = false;
    let mut non_whitespace_indices = Vec::new();
    for (i, t) in tokens_for_value.iter().enumerate() {
        if !matches!(t, crate::css::CssToken::Whitespace) {
            non_whitespace_indices.push(i);
        }
    }

    if non_whitespace_indices.len() >= 2 {
        let idx1 = non_whitespace_indices[non_whitespace_indices.len() - 2];
        let idx2 = non_whitespace_indices[non_whitespace_indices.len() - 1];
        match (&tokens_for_value[idx1], &tokens_for_value[idx2]) {
            (crate::css::CssToken::Delim('!'), crate::css::CssToken::Ident(ident))
                if ident.eq_ignore_ascii_case("important") =>
            {
                important = true;
                tokens_for_value.truncate(idx1);
            }
            _ => {}
        }
    }

    let value = tokens_to_component_values(tokens_for_value);

    Some(Declaration {
        name,
        value,
        important,
    })
}

fn tokens_to_component_values(
    tokens: Vec<crate::css::CssToken>,
) -> Vec<crate::css::parser::ComponentValue> {
    use crate::css::CssToken;
    use crate::css::parser::ComponentValue;

    let mut values = Vec::new();
    let mut it = tokens.into_iter().peekable();
    while let Some(token) = it.next() {
        match token {
            CssToken::LeftBrace | CssToken::LeftBracket | CssToken::LeftParen => {
                // The outer arm guarantees a bracket token; the wildcard default
                // keeps this panic-free on any future change (I-6 — inline style is
                // untrusted input).
                let (associated, closing) = match token {
                    CssToken::LeftBrace => ('{', CssToken::RightBrace),
                    CssToken::LeftBracket => ('[', CssToken::RightBracket),
                    CssToken::LeftParen => ('(', CssToken::RightParen),
                    _ => ('{', CssToken::RightBrace),
                };
                let mut block_tokens = Vec::new();
                let mut depth = 1;
                for t in it.by_ref() {
                    if t == token {
                        depth += 1;
                    } else if t == closing {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    block_tokens.push(t);
                }
                values.push(ComponentValue::SimpleBlock {
                    associated,
                    value: tokens_to_component_values(block_tokens),
                });
            }
            CssToken::Function(name) => {
                let mut func_tokens = Vec::new();
                let mut depth = 1;
                for t in it.by_ref() {
                    if let CssToken::Function(_) = t {
                        depth += 1;
                    } else if t == CssToken::LeftParen {
                        depth += 1;
                    } else if t == CssToken::RightParen {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    func_tokens.push(t);
                }
                values.push(ComponentValue::Function {
                    name,
                    value: tokens_to_component_values(func_tokens),
                });
            }
            _ => {
                values.push(ComponentValue::Token(token));
            }
        }
    }
    values
}

struct MatchedDeclaration {
    declaration: Declaration,
    specificity: (u32, u32, u32, u32),
    source_order: usize,
}

fn expand_1_to_4(value: &CssValue) -> (CssValue, CssValue, CssValue, CssValue) {
    // spec: https://www.w3.org/TR/css-box-3/#propdef-margin
    match value {
        CssValue::Multiple(values) => match values.len() {
            0 => {
                let fallback = CssValue::Keyword("initial".to_string());
                (
                    fallback.clone(),
                    fallback.clone(),
                    fallback.clone(),
                    fallback,
                )
            }
            1 => {
                let v = &values[0];
                (v.clone(), v.clone(), v.clone(), v.clone())
            }
            2 => {
                let top_bottom = &values[0];
                let left_right = &values[1];
                (
                    top_bottom.clone(),
                    left_right.clone(),
                    top_bottom.clone(),
                    left_right.clone(),
                )
            }
            3 => {
                let top = &values[0];
                let left_right = &values[1];
                let bottom = &values[2];
                (
                    top.clone(),
                    left_right.clone(),
                    bottom.clone(),
                    left_right.clone(),
                )
            }
            _ => {
                let top = &values[0];
                let right = &values[1];
                let bottom = &values[2];
                let left = &values[3];
                (top.clone(), right.clone(), bottom.clone(), left.clone())
            }
        },
        v => (v.clone(), v.clone(), v.clone(), v.clone()),
    }
}

fn find_border_width(value: &CssValue) -> Option<CssValue> {
    match value {
        CssValue::Multiple(values) => {
            for v in values {
                if is_border_width_value(v) {
                    return Some(v.clone());
                }
            }
            None
        }
        v => {
            if is_border_width_value(v) {
                Some(v.clone())
            } else {
                None
            }
        }
    }
}

fn is_border_width_value(value: &CssValue) -> bool {
    match value {
        CssValue::Length(_, _) => true,
        CssValue::Number(_) => true,
        CssValue::Keyword(s) => {
            matches!(s.to_ascii_lowercase().as_str(), "thin" | "medium" | "thick")
        }
        _ => false,
    }
}

fn is_inherited_property(property: &str) -> bool {
    // spec: basic inherited properties
    matches!(
        property,
        "color"
            | "font-family"
            | "font-size"
            | "font-style"
            | "font-variant"
            | "font-weight"
            | "font"
            | "letter-spacing"
            | "line-height"
            | "list-style-image"
            | "list-style-position"
            | "list-style-type"
            | "list-style"
            | "text-align"
            | "text-indent"
            | "text-transform"
            | "visibility"
            | "white-space"
            | "word-spacing"
    )
}

fn serialize_component_values(values: &[crate::css::parser::ComponentValue]) -> String {
    use crate::css::CssToken;
    use crate::css::parser::ComponentValue;

    let mut s = String::new();
    for val in values {
        match val {
            ComponentValue::Token(t) => match t {
                CssToken::Ident(v) => s.push_str(v),
                CssToken::Function(v) => {
                    s.push_str(v);
                    s.push('(');
                }
                CssToken::AtKeyword(v) => {
                    s.push('@');
                    s.push_str(v);
                }
                CssToken::Hash(v) => {
                    s.push('#');
                    s.push_str(v);
                }
                CssToken::String(v) => {
                    s.push('"');
                    s.push_str(v);
                    s.push('"');
                }
                CssToken::Number(v) => s.push_str(&v.to_string()),
                CssToken::Percentage(v) => {
                    s.push_str(&v.to_string());
                    s.push('%');
                }
                CssToken::Dimension { value, unit } => {
                    s.push_str(&value.to_string());
                    s.push_str(unit);
                }
                CssToken::Delim(c) => s.push(*c),
                CssToken::Whitespace => s.push(' '),
                CssToken::Colon => s.push(':'),
                CssToken::Semicolon => s.push(';'),
                CssToken::Comma => s.push(','),
                CssToken::LeftBrace => s.push('{'),
                CssToken::RightBrace => s.push('}'),
                CssToken::LeftParen => s.push('('),
                CssToken::RightParen => s.push(')'),
                CssToken::LeftBracket => s.push('['),
                CssToken::RightBracket => s.push(']'),
                CssToken::Cdo => s.push_str("<!--"),
                CssToken::Cdc => s.push_str("-->"),
                CssToken::Url(v) => {
                    s.push_str("url(");
                    s.push_str(v);
                    s.push(')');
                }
                _ => {}
            },
            ComponentValue::Function { name, value } => {
                s.push_str(name);
                s.push('(');
                s.push_str(&serialize_component_values(value));
                s.push(')');
            }
            ComponentValue::SimpleBlock { associated, value } => {
                s.push(*associated);
                s.push_str(&serialize_component_values(value));
                match associated {
                    '{' => s.push('}'),
                    '[' => s.push(']'),
                    '(' => s.push(')'),
                    _ => {}
                }
            }
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::parser::parse_stylesheet;
    use crate::dom::NodeData;

    #[test]
    fn test_specificity() {
        let sel = crate::selector::parse_selector_list("div")
            .unwrap()
            .0
            .remove(0);
        assert_eq!(specificity(&sel), (0, 0, 0, 1));

        let sel = crate::selector::parse_selector_list(".foo")
            .unwrap()
            .0
            .remove(0);
        assert_eq!(specificity(&sel), (0, 0, 1, 0));

        let sel = crate::selector::parse_selector_list("#bar")
            .unwrap()
            .0
            .remove(0);
        assert_eq!(specificity(&sel), (0, 1, 0, 0));

        let sel = crate::selector::parse_selector_list("div.foo#bar")
            .unwrap()
            .0
            .remove(0);
        assert_eq!(specificity(&sel), (0, 1, 1, 1));

        let sel = crate::selector::parse_selector_list("div > span")
            .unwrap()
            .0
            .remove(0);
        assert_eq!(specificity(&sel), (0, 0, 0, 2));
    }

    #[test]
    fn test_cascade_basic() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "foo".into())],
        });
        dom.append_child(doc, div);

        let stylesheet = parse_stylesheet("div { color: red; } .foo { color: blue; }");
        let styles = compute_styles(&dom, &stylesheet);

        let div_style = styles.get(&div).unwrap();
        if let Some(CssValue::Color(c)) = div_style.get("color") {
            // blue wins because .foo has higher specificity than div
            assert_eq!(c, &crate::css::values::Color::Rgba(0, 0, 255, 255));
        } else {
            panic!("Expected color blue");
        }
    }

    #[test]
    fn test_cascade_important() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "foo".into())],
        });
        dom.append_child(doc, div);

        let stylesheet = parse_stylesheet("div { color: red !important; } .foo { color: blue; }");
        let styles = compute_styles(&dom, &stylesheet);

        let div_style = styles.get(&div).unwrap();
        if let Some(CssValue::Color(c)) = div_style.get("color") {
            // red wins because of !important
            assert_eq!(c, &crate::css::values::Color::Rgba(255, 0, 0, 255));
        } else {
            panic!("Expected color red");
        }
    }

    #[test]
    fn test_cascade_source_order() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let stylesheet = parse_stylesheet("div { color: red; } div { color: blue; }");
        let styles = compute_styles(&dom, &stylesheet);

        let div_style = styles.get(&div).unwrap();
        if let Some(CssValue::Color(c)) = div_style.get("color") {
            // blue wins because it comes later in source order
            assert_eq!(c, &crate::css::values::Color::Rgba(0, 0, 255, 255));
        } else {
            panic!("Expected color blue");
        }
    }

    #[test]
    fn test_inheritance() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        let p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);
        dom.append_child(div, p);

        let stylesheet = parse_stylesheet("div { color: red; }");
        let styles = compute_styles(&dom, &stylesheet);

        let p_style = styles.get(&p).unwrap();
        if let Some(CssValue::Color(c)) = p_style.get("color") {
            // red is inherited from div
            assert_eq!(c, &crate::css::values::Color::Rgba(255, 0, 0, 255));
        } else {
            panic!("Expected inherited color red");
        }

        // non-inherited property
        let stylesheet = parse_stylesheet("div { margin: 10px; }"); // assuming margin is parsed as keyword or something if not fully supported
        // Wait, margin is not in is_inherited_property.
        let styles = compute_styles(&dom, &stylesheet);
        let p_style = styles.get(&p).unwrap();
        assert!(p_style.get("margin").is_none());
    }

    #[test]
    fn test_box_model_shorthands_and_properties() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        // 1. Test 1-4 value forms of margin/padding/border-width
        let stylesheet = parse_stylesheet(
            "
            div {
                margin: 10px 20px;
                padding: 5px 10px 15px 20px;
                border-width: 2px;
                display: flex;
                box-sizing: border-box;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let style = styles.get(&div).unwrap();

        // Check margin top/bottom=10px, left/right=20px
        assert_eq!(
            style.get("margin-top"),
            Some(&CssValue::Length(10.0, crate::css::values::LengthUnit::Px))
        );
        assert_eq!(
            style.get("margin-right"),
            Some(&CssValue::Length(20.0, crate::css::values::LengthUnit::Px))
        );
        assert_eq!(
            style.get("margin-bottom"),
            Some(&CssValue::Length(10.0, crate::css::values::LengthUnit::Px))
        );
        assert_eq!(
            style.get("margin-left"),
            Some(&CssValue::Length(20.0, crate::css::values::LengthUnit::Px))
        );

        // Check 4-value padding expansion
        assert_eq!(
            style.get("padding-top"),
            Some(&CssValue::Length(5.0, crate::css::values::LengthUnit::Px))
        );
        assert_eq!(
            style.get("padding-right"),
            Some(&CssValue::Length(10.0, crate::css::values::LengthUnit::Px))
        );
        assert_eq!(
            style.get("padding-bottom"),
            Some(&CssValue::Length(15.0, crate::css::values::LengthUnit::Px))
        );
        assert_eq!(
            style.get("padding-left"),
            Some(&CssValue::Length(20.0, crate::css::values::LengthUnit::Px))
        );

        // Check border-width (1-value form)
        assert_eq!(
            style.get("border-top-width"),
            Some(&CssValue::Length(2.0, crate::css::values::LengthUnit::Px))
        );
        assert_eq!(
            style.get("border-right-width"),
            Some(&CssValue::Length(2.0, crate::css::values::LengthUnit::Px))
        );
        assert_eq!(
            style.get("border-bottom-width"),
            Some(&CssValue::Length(2.0, crate::css::values::LengthUnit::Px))
        );
        assert_eq!(
            style.get("border-left-width"),
            Some(&CssValue::Length(2.0, crate::css::values::LengthUnit::Px))
        );

        // Check display and box-sizing
        assert_eq!(
            style.get("display"),
            Some(&CssValue::Keyword("flex".to_string()))
        );
        assert_eq!(
            style.get("box-sizing"),
            Some(&CssValue::Keyword("border-box".to_string()))
        );

        // 2. Test border shorthand width parsing
        let stylesheet_border = parse_stylesheet("div { border: 3px solid red; }");
        let styles_border = compute_styles(&dom, &stylesheet_border);
        let style_border = styles_border.get(&div).unwrap();
        assert_eq!(
            style_border.get("border-top-width"),
            Some(&CssValue::Length(3.0, crate::css::values::LengthUnit::Px))
        );
        assert_eq!(
            style_border.get("border-right-width"),
            Some(&CssValue::Length(3.0, crate::css::values::LengthUnit::Px))
        );
        assert_eq!(
            style_border.get("border-bottom-width"),
            Some(&CssValue::Length(3.0, crate::css::values::LengthUnit::Px))
        );
        assert_eq!(
            style_border.get("border-left-width"),
            Some(&CssValue::Length(3.0, crate::css::values::LengthUnit::Px))
        );

        // 3. Test border shorthand with no width specified (defaults to medium)
        let stylesheet_border_no_w = parse_stylesheet("div { border: solid red; }");
        let styles_border_no_w = compute_styles(&dom, &stylesheet_border_no_w);
        let style_border_no_w = styles_border_no_w.get(&div).unwrap();
        assert_eq!(
            style_border_no_w.get("border-top-width"),
            Some(&CssValue::Keyword("medium".to_string()))
        );

        // 4. Test border shorthand with named width
        let stylesheet_border_thick = parse_stylesheet("div { border: thick double blue; }");
        let styles_border_thick = compute_styles(&dom, &stylesheet_border_thick);
        let style_border_thick = styles_border_thick.get(&div).unwrap();
        assert_eq!(
            style_border_thick.get("border-top-width"),
            Some(&CssValue::Keyword("thick".to_string()))
        );
    }

    #[test]
    fn test_text_font_computed_values_and_inheritance() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        let p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);
        dom.append_child(div, p);

        // 1. Initial/default values check when nothing is specified.
        let stylesheet_empty = parse_stylesheet("");
        let styles_empty = compute_styles(&dom, &stylesheet_empty);

        let div_style = styles_empty.get(&div).unwrap();
        assert_eq!(
            div_style.get("font-size"),
            Some(&CssValue::Length(16.0, LengthUnit::Px))
        );
        assert_eq!(
            div_style.get("font-weight"),
            Some(&CssValue::Keyword("normal".to_string()))
        );
        assert_eq!(
            div_style.get("line-height"),
            Some(&CssValue::Keyword("normal".to_string()))
        );
        assert_eq!(
            div_style.get("text-align"),
            Some(&CssValue::Keyword("left".to_string()))
        );

        // 2. Specified values and relative em/rem resolution, along with inheritance.
        let stylesheet = parse_stylesheet(
            "
            div {
                font-size: 16px;
                font-weight: bold;
                line-height: 2;
                text-align: center;
            }
            p {
                font-size: 2em;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let div_style = styles.get(&div).unwrap();
        let p_style = styles.get(&p).unwrap();

        // Div checks
        assert_eq!(
            div_style.get("font-size"),
            Some(&CssValue::Length(16.0, LengthUnit::Px))
        );
        assert_eq!(
            div_style.get("font-weight"),
            Some(&CssValue::Keyword("bold".to_string()))
        );
        assert_eq!(div_style.get("line-height"), Some(&CssValue::Number(2.0)));
        assert_eq!(
            div_style.get("text-align"),
            Some(&CssValue::Keyword("center".to_string()))
        );

        // P checks (em resolution: 2em * parent's 16px = 32px)
        assert_eq!(
            p_style.get("font-size"),
            Some(&CssValue::Length(32.0, LengthUnit::Px))
        );
        // Inherited font-weight (bold), line-height (number 2.0), text-align (center)
        assert_eq!(
            p_style.get("font-weight"),
            Some(&CssValue::Keyword("bold".to_string()))
        );
        assert_eq!(p_style.get("line-height"), Some(&CssValue::Number(2.0)));
        assert_eq!(
            p_style.get("text-align"),
            Some(&CssValue::Keyword("center".to_string()))
        );

        // 3. Child explicitly overrides inherited values.
        let stylesheet_override = parse_stylesheet(
            "
            div {
                font-size: 16px;
                font-weight: bold;
                line-height: 24px;
                text-align: center;
            }
            p {
                font-weight: normal;
                line-height: 1.5em;
                text-align: right;
            }
        ",
        );
        let styles_override = compute_styles(&dom, &stylesheet_override);
        let p_style_override = styles_override.get(&p).unwrap();

        // p has explicitly overridden: font-weight, line-height, text-align.
        // And p's line-height: 1.5em is relative to p's own font-size (which is inherited 16px, so 1.5 * 16px = 24px)
        assert_eq!(
            p_style_override.get("font-size"),
            Some(&CssValue::Length(16.0, LengthUnit::Px))
        );
        assert_eq!(
            p_style_override.get("font-weight"),
            Some(&CssValue::Keyword("normal".to_string()))
        );
        assert_eq!(
            p_style_override.get("line-height"),
            Some(&CssValue::Length(24.0, LengthUnit::Px))
        );
        assert_eq!(
            p_style_override.get("text-align"),
            Some(&CssValue::Keyword("right".to_string()))
        );

        // 4. Test root element font-size with rem resolution.
        let stylesheet_rem = parse_stylesheet(
            "
            div {
                font-size: 20px;
            }
            p {
                font-size: 1.5rem;
            }
        ",
        );
        // div is the first element child of doc (the root element).
        // Its font-size is 20px, so root font-size is 20px.
        // p's font-size: 1.5rem should resolve to 1.5 * 20px = 30px.
        let styles_rem = compute_styles(&dom, &stylesheet_rem);
        let p_style_rem = styles_rem.get(&p).unwrap();
        assert_eq!(
            p_style_rem.get("font-size"),
            Some(&CssValue::Length(30.0, LengthUnit::Px))
        );

        // 5. Test inherit / initial keywords.
        let stylesheet_keywords = parse_stylesheet(
            "
            div {
                font-size: 24px;
                font-weight: 500;
                text-align: right;
            }
            p {
                font-size: initial;
                font-weight: inherit;
                text-align: initial;
            }
        ",
        );
        let styles_keywords = compute_styles(&dom, &stylesheet_keywords);
        let p_style_keywords = styles_keywords.get(&p).unwrap();
        // p's font-size: initial -> 16px
        assert_eq!(
            p_style_keywords.get("font-size"),
            Some(&CssValue::Length(16.0, LengthUnit::Px))
        );
        // p's font-weight: inherit -> 500
        assert_eq!(
            p_style_keywords.get("font-weight"),
            Some(&CssValue::Number(500.0))
        );
        // p's text-align: initial -> left
        assert_eq!(
            p_style_keywords.get("text-align"),
            Some(&CssValue::Keyword("left".to_string()))
        );
    }
}
