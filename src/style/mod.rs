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

    /// Sets/inserts a computed value for a given property.
    pub fn insert(&mut self, property: String, value: CssValue) {
        self.properties.insert(property, value);
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
                Component::Is(_) | Component::Where(_) => {
                    // TODO(spec): :is() has specificity of its most specific argument, and :where() has 0 specificity.
                    // For now, specificity differences are out of scope for this task (t0178).
                }
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
                            Component::Is(_) | Component::Where(_) => {
                                // TODO(spec): :is() has specificity of its most specific argument, and :where() has 0 specificity.
                                // For now, specificity differences are out of scope for this task (t0178).
                            }
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

    // 1.5. Collect presentational hints.
    let ua_rules_count = stylesheet
        .rules
        .iter()
        .position(|rule| {
            if let Rule::Qualified(qr) = rule {
                let s = serialize_component_values(&qr.prelude);
                s.replace(" ", "") == "head,style,script,meta,link,title"
            } else {
                false
            }
        })
        .map(|pos| pos + 1)
        .unwrap_or(0);

    for decl in &mut matched_declarations {
        if decl.source_order >= ua_rules_count {
            decl.source_order += 1;
        }
    }

    collect_presentational_hints(dom, node, ua_rules_count, &mut matched_declarations);

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
        let name = matched.declaration.name.as_str();
        if name == "font" {
            if let Some(expanded) = expand_font_shorthand(&matched.declaration.value) {
                for (longhand_name, longhand_val) in expanded {
                    properties.insert(longhand_name, longhand_val);
                }
            }
            continue;
        }

        if let Some(value) = parse_value(&matched.declaration.value) {
            match name {
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
                "outline" => {
                    // spec: https://drafts.csswg.org/css-ui/#outline-shorthand
                    let width = find_outline_width(&value)
                        .unwrap_or_else(|| CssValue::Keyword("medium".to_string()));
                    let style = find_outline_style(&value)
                        .unwrap_or_else(|| CssValue::Keyword("none".to_string()));

                    properties.insert("outline-width".to_string(), width);
                    properties.insert("outline-style".to_string(), style);

                    if let Some(color) = find_outline_color(&value) {
                        properties.insert("outline-color".to_string(), color);
                    }
                }
                "background" => {
                    // spec: https://drafts.csswg.org/css-backgrounds-3/#background
                    if let Some(color) = find_background_color(&value) {
                        properties.insert("background-color".to_string(), color);
                    }
                    // TODO(spec): other background longhands (image/position/repeat/size/etc.)
                }
                // TODO(spec): other shorthand properties like font, transition, etc.
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
                // spec: viewport units depend on the viewport, which style does not
                // know here; pass them through for layout to resolve.
                LengthUnit::Vw | LengthUnit::Vh => CssValue::Length(*val, unit.clone()),
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
                // spec: viewport units resolved later at layout time.
                LengthUnit::Vw | LengthUnit::Vh => CssValue::Length(*val, unit.clone()),
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

    // E. Resolve white-space
    let resolved_white_space = {
        let raw_ws = properties.get("white-space");
        match raw_ws {
            Some(CssValue::Keyword(s)) if s == "inherit" => {
                let parent_ws = parent_style.and_then(|s| s.get("white-space")).cloned();
                parent_ws.unwrap_or(CssValue::Keyword("normal".to_string()))
            }
            Some(CssValue::Keyword(s)) if s == "initial" => CssValue::Keyword("normal".to_string()),
            Some(val) => val.clone(),
            None => CssValue::Keyword("normal".to_string()),
        }
    };
    properties.insert("white-space".to_string(), resolved_white_space);

    // F. Resolve letter-spacing
    let resolved_letter_spacing = {
        let raw_ls = properties.get("letter-spacing");
        match raw_ls {
            Some(CssValue::Keyword(s)) if s == "inherit" => {
                let parent_ls = parent_style.and_then(|s| s.get("letter-spacing")).cloned();
                parent_ls.unwrap_or(CssValue::Keyword("normal".to_string()))
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
                LengthUnit::Vw | LengthUnit::Vh => CssValue::Length(*val, unit.clone()),
            },
            Some(val) => val.clone(),
            None => CssValue::Keyword("normal".to_string()),
        }
    };
    properties.insert("letter-spacing".to_string(), resolved_letter_spacing);

    // G. Resolve word-spacing
    let resolved_word_spacing = {
        let raw_ws = properties.get("word-spacing");
        match raw_ws {
            Some(CssValue::Keyword(s)) if s == "inherit" => {
                let parent_ws = parent_style.and_then(|s| s.get("word-spacing")).cloned();
                parent_ws.unwrap_or(CssValue::Keyword("normal".to_string()))
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
                LengthUnit::Vw | LengthUnit::Vh => CssValue::Length(*val, unit.clone()),
            },
            Some(val) => val.clone(),
            None => CssValue::Keyword("normal".to_string()),
        }
    };
    properties.insert("word-spacing".to_string(), resolved_word_spacing);

    // H. Resolve visibility
    let resolved_visibility = {
        let raw_vis = properties.get("visibility");
        match raw_vis {
            Some(CssValue::Keyword(s)) if s == "inherit" => {
                let parent_vis = parent_style.and_then(|s| s.get("visibility")).cloned();
                parent_vis.unwrap_or(CssValue::Keyword("visible".to_string()))
            }
            Some(CssValue::Keyword(s)) if s == "initial" => {
                CssValue::Keyword("visible".to_string())
            }
            Some(val) => val.clone(),
            None => CssValue::Keyword("visible".to_string()),
        }
    };
    properties.insert("visibility".to_string(), resolved_visibility);

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
    let query_str = serialize_component_values(prelude);
    crate::css::media::media_matches(&query_str, viewport_width)
}

fn is_presentational_hint_element(name: &str) -> bool {
    name.eq_ignore_ascii_case("img")
        || name.eq_ignore_ascii_case("table")
        || name.eq_ignore_ascii_case("td")
        || name.eq_ignore_ascii_case("th")
        || name.eq_ignore_ascii_case("col")
        || name.eq_ignore_ascii_case("colgroup")
        || name.eq_ignore_ascii_case("tr")
        || name.eq_ignore_ascii_case("body")
}

fn map_presentational_dimension(val: &str) -> Option<String> {
    let trimmed = val.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(stripped) = trimmed.strip_suffix('%') {
        let num_part = stripped.trim();
        if num_part.parse::<f32>().is_ok() {
            return Some(format!("{}%", num_part));
        }
    } else if trimmed.parse::<u32>().is_ok() {
        return Some(format!("{}px", trimmed));
    }

    // TODO(spec): HTML spec allows mapping trailing characters after digits in some contexts (e.g. "200abc" -> "200px").
    // For now, we only support exact integer or percentage matches for simplicity and safety.
    None
}

fn collect_presentational_hints(
    dom: &Dom,
    node: NodeId,
    ua_rules_count: usize,
    matched_declarations: &mut Vec<MatchedDeclaration>,
) {
    if let Some(crate::dom::NodeData::Element { name, attrs }) = dom.data(node)
        && is_presentational_hint_element(name)
    {
        if let Some((_, width_val)) = attrs
            .iter()
            .find(|(attr_name, _)| attr_name.eq_ignore_ascii_case("width"))
            && let Some(css_val_str) = map_presentational_dimension(width_val)
        {
            let components = crate::css::parser::parse_component_values(&css_val_str);
            matched_declarations.push(MatchedDeclaration {
                declaration: Declaration {
                    name: "width".to_string(),
                    value: components,
                    important: false,
                },
                specificity: (0, 0, 0, 0),
                source_order: ua_rules_count,
            });
        }
        if let Some((_, height_val)) = attrs
            .iter()
            .find(|(attr_name, _)| attr_name.eq_ignore_ascii_case("height"))
            && let Some(css_val_str) = map_presentational_dimension(height_val)
        {
            let components = crate::css::parser::parse_component_values(&css_val_str);
            matched_declarations.push(MatchedDeclaration {
                declaration: Declaration {
                    name: "height".to_string(),
                    value: components,
                    important: false,
                },
                specificity: (0, 0, 0, 0),
                source_order: ua_rules_count,
            });
        }

        if name.eq_ignore_ascii_case("img") {
            if let Some((_, hspace_val)) = attrs
                .iter()
                .find(|(attr_name, _)| attr_name.eq_ignore_ascii_case("hspace"))
            {
                let trimmed = hspace_val.trim();
                if let Ok(num) = trimmed.parse::<u32>() {
                    let css_val_str = format!("{}px", num);
                    let components = crate::css::parser::parse_component_values(&css_val_str);
                    for side in &["margin-left", "margin-right"] {
                        matched_declarations.push(MatchedDeclaration {
                            declaration: Declaration {
                                name: side.to_string(),
                                value: components.clone(),
                                important: false,
                            },
                            specificity: (0, 0, 0, 0),
                            source_order: ua_rules_count,
                        });
                    }
                }
            }

            if let Some((_, vspace_val)) = attrs
                .iter()
                .find(|(attr_name, _)| attr_name.eq_ignore_ascii_case("vspace"))
            {
                let trimmed = vspace_val.trim();
                if let Ok(num) = trimmed.parse::<u32>() {
                    let css_val_str = format!("{}px", num);
                    let components = crate::css::parser::parse_component_values(&css_val_str);
                    for side in &["margin-top", "margin-bottom"] {
                        matched_declarations.push(MatchedDeclaration {
                            declaration: Declaration {
                                name: side.to_string(),
                                value: components.clone(),
                                important: false,
                            },
                            specificity: (0, 0, 0, 0),
                            source_order: ua_rules_count,
                        });
                    }
                }
            }
        }

        // 1. bgcolor on <table>, <td>, <th>, <tr>, <body> -> CSS background-color
        if (name.eq_ignore_ascii_case("table")
            || name.eq_ignore_ascii_case("td")
            || name.eq_ignore_ascii_case("th")
            || name.eq_ignore_ascii_case("tr")
            || name.eq_ignore_ascii_case("body"))
            && let Some((_, bgcolor_val)) = attrs
                .iter()
                .find(|(attr_name, _)| attr_name.eq_ignore_ascii_case("bgcolor"))
        {
            let trimmed = bgcolor_val.trim();
            let is_hex = if let Some(hex_part) = trimmed.strip_prefix('#') {
                (hex_part.len() == 3 || hex_part.len() == 6)
                    && hex_part.chars().all(|c| c.is_ascii_hexdigit())
            } else {
                false
            };
            let is_named = !is_hex && crate::css::colors::named_color(trimmed).is_some();

            if is_hex || is_named {
                let components = crate::css::parser::parse_component_values(trimmed);
                matched_declarations.push(MatchedDeclaration {
                    declaration: Declaration {
                        name: "background-color".to_string(),
                        value: components,
                        important: false,
                    },
                    specificity: (0, 0, 0, 0),
                    source_order: ua_rules_count,
                });
            } else {
                // TODO(spec): HTML 'rules for parsing a legacy colour value' allows mapping bare legacy color values (e.g. "ccc") which are not valid CSS hex or named colors, but we do not guess for now.
            }
        }

        // 2. align on <td>, <th>, <tr>, <table>, <col>, <colgroup> -> CSS text-align
        if (name.eq_ignore_ascii_case("td")
            || name.eq_ignore_ascii_case("th")
            || name.eq_ignore_ascii_case("tr")
            || name.eq_ignore_ascii_case("table")
            || name.eq_ignore_ascii_case("col")
            || name.eq_ignore_ascii_case("colgroup"))
            && let Some((_, align_val)) = attrs
                .iter()
                .find(|(attr_name, _)| attr_name.eq_ignore_ascii_case("align"))
        {
            let trimmed = align_val.trim();
            if trimmed.eq_ignore_ascii_case("left")
                || trimmed.eq_ignore_ascii_case("right")
                || trimmed.eq_ignore_ascii_case("center")
            {
                let css_val_str = trimmed.to_ascii_lowercase();
                let components = crate::css::parser::parse_component_values(&css_val_str);
                matched_declarations.push(MatchedDeclaration {
                    declaration: Declaration {
                        name: "text-align".to_string(),
                        value: components,
                        important: false,
                    },
                    specificity: (0, 0, 0, 0),
                    source_order: ua_rules_count,
                });
            }
        }

        // 3. valign on <td>, <th>, <tr> -> CSS vertical-align
        if (name.eq_ignore_ascii_case("td")
            || name.eq_ignore_ascii_case("th")
            || name.eq_ignore_ascii_case("tr"))
            && let Some((_, valign_val)) = attrs
                .iter()
                .find(|(attr_name, _)| attr_name.eq_ignore_ascii_case("valign"))
        {
            let trimmed = valign_val.trim();
            if trimmed.eq_ignore_ascii_case("top")
                || trimmed.eq_ignore_ascii_case("middle")
                || trimmed.eq_ignore_ascii_case("bottom")
            {
                let css_val_str = trimmed.to_ascii_lowercase();
                let components = crate::css::parser::parse_component_values(&css_val_str);
                matched_declarations.push(MatchedDeclaration {
                    declaration: Declaration {
                        name: "vertical-align".to_string(),
                        value: components,
                        important: false,
                    },
                    specificity: (0, 0, 0, 0),
                    source_order: ua_rules_count,
                });
            }
        }
    }
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

fn expand_font_shorthand(
    values: &[crate::css::parser::ComponentValue],
) -> Option<Vec<(String, CssValue)>> {
    use crate::css::CssToken;
    use crate::css::parser::ComponentValue;

    let mut non_ws = Vec::new();
    let mut orig_indices = Vec::new();
    for (i, cv) in values.iter().enumerate() {
        if !matches!(cv, ComponentValue::Token(CssToken::Whitespace)) {
            non_ws.push(cv);
            orig_indices.push(i);
        }
    }

    if non_ws.is_empty() {
        return None;
    }

    // Handle global keywords
    match non_ws.as_slice() {
        [ComponentValue::Token(CssToken::Ident(s))] if s == "inherit" || s == "initial" => {
            let val = CssValue::Keyword(s.clone());
            return Some(vec![
                ("font-size".to_string(), val.clone()),
                ("font-family".to_string(), val.clone()),
                ("font-weight".to_string(), val.clone()),
                ("font-style".to_string(), val.clone()),
                ("line-height".to_string(), val),
            ]);
        }
        _ => {}
    }

    // Helper to check if a component is font-size
    fn is_font_size_token(cv: &ComponentValue) -> bool {
        match cv {
            ComponentValue::Token(CssToken::Dimension { .. }) => true,
            ComponentValue::Token(CssToken::Percentage(_)) => true,
            ComponentValue::Token(CssToken::Ident(s)) => {
                matches!(
                    s.to_ascii_lowercase().as_str(),
                    "xx-small"
                        | "x-small"
                        | "small"
                        | "medium"
                        | "large"
                        | "x-large"
                        | "xx-large"
                        | "smaller"
                        | "larger"
                )
            }
            _ => false,
        }
    }

    // Find font-size index
    let mut fs_idx = None;
    for (i, cv) in non_ws.iter().enumerate() {
        if is_font_size_token(cv) {
            fs_idx = Some(i);
            break;
        }
    }

    let idx = fs_idx?;

    // Parse font-size
    let fs_val = parse_value(std::slice::from_ref(non_ws[idx]))?;

    // Default prefix properties
    let mut style_val = CssValue::Keyword("normal".to_string());
    let mut weight_val = CssValue::Keyword("normal".to_string());

    // Parse prefix properties (0..idx)
    for &prefix_item in &non_ws[0..idx] {
        if let ComponentValue::Token(CssToken::Ident(s)) = prefix_item {
            let lower = s.to_ascii_lowercase();
            if matches!(lower.as_str(), "italic" | "oblique") {
                style_val = parse_value(std::slice::from_ref(prefix_item))?;
            } else if matches!(lower.as_str(), "bold" | "bolder" | "lighter") {
                weight_val = parse_value(std::slice::from_ref(prefix_item))?;
            } else if lower == "normal" {
                // normal can reset style/weight
                style_val = CssValue::Keyword("normal".to_string());
                weight_val = CssValue::Keyword("normal".to_string());
            }
        } else if let ComponentValue::Token(CssToken::Number(_)) = prefix_item {
            // numeric font-weight
            weight_val = parse_value(std::slice::from_ref(prefix_item))?;
        }
    }

    // Parse line-height if present
    let mut lh_val = None;
    let family_start;
    if idx + 1 < non_ws.len()
        && matches!(non_ws[idx + 1], ComponentValue::Token(CssToken::Delim('/')))
    {
        if idx + 2 < non_ws.len() {
            lh_val = parse_value(std::slice::from_ref(non_ws[idx + 2]));
        }
        family_start = idx + 3;
    } else {
        family_start = idx + 1;
    }

    // Parse font-family (all tokens starting from family_start)
    if family_start >= non_ws.len() {
        return None; // font-family is required!
    }

    let orig_family_start = orig_indices[family_start];
    let family_slice = &values[orig_family_start..];
    let family_str = serialize_component_values(family_slice).trim().to_string();
    if family_str.is_empty() {
        return None;
    }
    let family_val = CssValue::Keyword(family_str);

    let mut result = vec![
        ("font-size".to_string(), fs_val),
        ("font-style".to_string(), style_val),
        ("font-weight".to_string(), weight_val),
        ("font-family".to_string(), family_val),
    ];
    if let Some(lh) = lh_val {
        result.push(("line-height".to_string(), lh));
    }

    Some(result)
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

fn find_outline_width(value: &CssValue) -> Option<CssValue> {
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

fn find_outline_style(value: &CssValue) -> Option<CssValue> {
    match value {
        CssValue::Multiple(values) => {
            for v in values {
                if is_outline_style_value(v) {
                    return Some(v.clone());
                }
            }
            None
        }
        v => {
            if is_outline_style_value(v) {
                Some(v.clone())
            } else {
                None
            }
        }
    }
}

fn find_outline_color(value: &CssValue) -> Option<CssValue> {
    match value {
        CssValue::Multiple(values) => {
            for v in values {
                if is_outline_color_value(v) {
                    return Some(v.clone());
                }
            }
            None
        }
        v => {
            if is_outline_color_value(v) {
                Some(v.clone())
            } else {
                None
            }
        }
    }
}

fn find_background_color(value: &CssValue) -> Option<CssValue> {
    match value {
        CssValue::Multiple(values) => {
            for v in values {
                if let Some(color) = to_background_color(v) {
                    return Some(color);
                }
            }
            None
        }
        v => to_background_color(v),
    }
}

fn to_background_color(value: &CssValue) -> Option<CssValue> {
    if is_outline_color_value(value) {
        if let CssValue::Keyword(s) = value
            && s.eq_ignore_ascii_case("invert")
        {
            return None;
        }
        Some(value.clone())
    } else if let CssValue::Keyword(s) = value {
        crate::css::colors::named_color(s).map(CssValue::Color)
    } else {
        None
    }
}

fn is_outline_style_value(value: &CssValue) -> bool {
    match value {
        CssValue::Keyword(s) => {
            matches!(
                s.to_ascii_lowercase().as_str(),
                "none"
                    | "hidden"
                    | "dotted"
                    | "dashed"
                    | "solid"
                    | "double"
                    | "groove"
                    | "ridge"
                    | "inset"
                    | "outset"
            )
        }
        _ => false,
    }
}

fn is_outline_color_value(value: &CssValue) -> bool {
    match value {
        CssValue::Color(_) => true,
        CssValue::Keyword(s) => s.eq_ignore_ascii_case("invert"),
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
            | "word-break"
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
    fn test_outline_properties() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        // 1. Test outline shorthand with width, style, and color
        let stylesheet_outline = parse_stylesheet("div { outline: 2px solid red; }");
        let styles_outline = compute_styles(&dom, &stylesheet_outline);
        let style_outline = styles_outline.get(&div).unwrap();

        assert_eq!(
            style_outline.get("outline-width"),
            Some(&CssValue::Length(2.0, crate::css::values::LengthUnit::Px))
        );
        assert_eq!(
            style_outline.get("outline-style"),
            Some(&CssValue::Keyword("solid".to_string()))
        );
        assert_eq!(
            style_outline.get("outline-color"),
            Some(&CssValue::Color(crate::css::values::Color::Rgba(
                255, 0, 0, 255
            )))
        );

        // 2. Test longhand overrides shorthand
        let stylesheet_override =
            parse_stylesheet("div { outline: 2px solid red; outline-width: 3px; }");
        let styles_override = compute_styles(&dom, &stylesheet_override);
        let style_override = styles_override.get(&div).unwrap();

        assert_eq!(
            style_override.get("outline-width"),
            Some(&CssValue::Length(3.0, crate::css::values::LengthUnit::Px))
        );
        assert_eq!(
            style_override.get("outline-style"),
            Some(&CssValue::Keyword("solid".to_string()))
        );

        // 3. Test longhand-only style and width
        let stylesheet_longhand =
            parse_stylesheet("div { outline-style: solid; outline-width: 3px; }");
        let styles_longhand = compute_styles(&dom, &stylesheet_longhand);
        let style_longhand = styles_longhand.get(&div).unwrap();

        assert_eq!(
            style_longhand.get("outline-width"),
            Some(&CssValue::Length(3.0, crate::css::values::LengthUnit::Px))
        );
        assert_eq!(
            style_longhand.get("outline-style"),
            Some(&CssValue::Keyword("solid".to_string()))
        );
        assert_eq!(style_longhand.get("outline-color"), None);
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

    #[test]
    fn test_media_queries_integration_acceptance() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(doc, p);

        // spec: @media rule matches screen and width thresholds
        let stylesheet = parse_stylesheet(
            "
            p { color: blue; }
            @media (max-width: 600px) {
                p { color: red; }
            }
            @media screen and (min-width: 800px) {
                p { color: green; }
            }
        ",
        );

        // At 500px, max-width: 600px matches, so p should be red
        let styles_500 = compute_styles_with_viewport(&dom, &stylesheet, 500.0);
        let p_style_500 = styles_500.get(&p).unwrap();
        assert_eq!(
            p_style_500.get("color"),
            Some(&CssValue::Color(crate::css::values::Color::Rgba(
                255, 0, 0, 255
            )))
        );

        // At 700px, neither @media matches, so p should be blue (fallback)
        let styles_700 = compute_styles_with_viewport(&dom, &stylesheet, 700.0);
        let p_style_700 = styles_700.get(&p).unwrap();
        assert_eq!(
            p_style_700.get("color"),
            Some(&CssValue::Color(crate::css::values::Color::Rgba(
                0, 0, 255, 255
            )))
        );

        // At 900px, screen and (min-width: 800px) matches, so p should be green
        let styles_900 = compute_styles_with_viewport(&dom, &stylesheet, 900.0);
        let p_style_900 = styles_900.get(&p).unwrap();
        assert_eq!(
            p_style_900.get("color"),
            Some(&CssValue::Color(crate::css::values::Color::Rgba(
                0, 128, 0, 255
            )))
        );
    }

    #[test]
    fn test_s69_white_space_and_font_shorthand() {
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

        // 1. Default initial values check
        let stylesheet_empty = parse_stylesheet("");
        let styles_empty = compute_styles(&dom, &stylesheet_empty);
        let div_style = styles_empty.get(&div).unwrap();
        assert_eq!(
            div_style.get("white-space"),
            Some(&CssValue::Keyword("normal".to_string()))
        );
        assert_eq!(
            div_style.get("letter-spacing"),
            Some(&CssValue::Keyword("normal".to_string()))
        );
        assert_eq!(
            div_style.get("word-spacing"),
            Some(&CssValue::Keyword("normal".to_string()))
        );
        assert_eq!(
            div_style.get("visibility"),
            Some(&CssValue::Keyword("visible".to_string()))
        );

        // 2. Inheritance check
        let stylesheet_inherited =
            parse_stylesheet("div { white-space: pre; visibility: hidden; }");
        let styles_inherited = compute_styles(&dom, &stylesheet_inherited);
        let p_style = styles_inherited.get(&p).unwrap();
        assert_eq!(
            p_style.get("white-space"),
            Some(&CssValue::Keyword("pre".to_string()))
        );
        assert_eq!(
            p_style.get("visibility"),
            Some(&CssValue::Keyword("hidden".to_string()))
        );

        // 3. Font shorthand expansion
        let stylesheet_font = parse_stylesheet(
            "
            div {
                font: italic bold 16px \"Helvetica Neue\", sans-serif;
            }
            p {
                font: 12px/20px serif;
            }
        ",
        );
        let styles_font = compute_styles(&dom, &stylesheet_font);
        let div_style = styles_font.get(&div).unwrap();
        assert_eq!(
            div_style.get("font-size"),
            Some(&CssValue::Length(16.0, LengthUnit::Px))
        );
        assert_eq!(
            div_style.get("font-style"),
            Some(&CssValue::Keyword("italic".to_string()))
        );
        assert_eq!(
            div_style.get("font-weight"),
            Some(&CssValue::Keyword("bold".to_string()))
        );
        assert_eq!(
            div_style.get("font-family"),
            Some(&CssValue::Keyword(
                "\"Helvetica Neue\", sans-serif".to_string()
            ))
        );

        let p_style = styles_font.get(&p).unwrap();
        assert_eq!(
            p_style.get("font-size"),
            Some(&CssValue::Length(12.0, LengthUnit::Px))
        );
        assert_eq!(
            p_style.get("font-style"),
            Some(&CssValue::Keyword("normal".to_string()))
        );
        assert_eq!(
            p_style.get("font-weight"),
            Some(&CssValue::Keyword("normal".to_string()))
        );
        assert_eq!(
            p_style.get("line-height"),
            Some(&CssValue::Length(20.0, LengthUnit::Px))
        );
        assert_eq!(
            p_style.get("font-family"),
            Some(&CssValue::Keyword("serif".to_string()))
        );

        // 4. Letter-spacing and word-spacing relative (em) resolution
        let stylesheet_spacing = parse_stylesheet(
            "
            div {
                font-size: 20px;
                letter-spacing: 0.1em;
                word-spacing: 0.2em;
            }
        ",
        );
        let styles_spacing = compute_styles(&dom, &stylesheet_spacing);
        let div_style = styles_spacing.get(&div).unwrap();
        // 0.1em of 20px = 2px
        assert_eq!(
            div_style.get("letter-spacing"),
            Some(&CssValue::Length(2.0, LengthUnit::Px))
        );
        // 0.2em of 20px = 4px
        assert_eq!(
            div_style.get("word-spacing"),
            Some(&CssValue::Length(4.0, LengthUnit::Px))
        );

        // 5. Keyword initial and inherit
        let stylesheet_kw = parse_stylesheet(
            "
            div {
                white-space: nowrap;
                letter-spacing: 2px;
                word-spacing: 4px;
                visibility: hidden;
            }
            p {
                white-space: initial;
                letter-spacing: inherit;
                word-spacing: initial;
                visibility: inherit;
            }
        ",
        );
        let styles_kw = compute_styles(&dom, &stylesheet_kw);
        let p_style_kw = styles_kw.get(&p).unwrap();
        assert_eq!(
            p_style_kw.get("white-space"),
            Some(&CssValue::Keyword("normal".to_string()))
        );
        assert_eq!(
            p_style_kw.get("letter-spacing"),
            Some(&CssValue::Length(2.0, LengthUnit::Px))
        );
        assert_eq!(
            p_style_kw.get("word-spacing"),
            Some(&CssValue::Keyword("normal".to_string()))
        );
        assert_eq!(
            p_style_kw.get("visibility"),
            Some(&CssValue::Keyword("hidden".to_string()))
        );
    }

    #[test]
    fn test_presentational_hints_mapping_px() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let img = dom.create_node(NodeData::Element {
            name: "img".into(),
            attrs: vec![
                ("width".into(), "200".into()),
                ("height".into(), "150".into()),
            ],
        });
        dom.append_child(doc, img);

        let stylesheet = parse_stylesheet("");
        let styles = compute_styles(&dom, &stylesheet);
        let img_style = styles.get(&img).unwrap();

        assert_eq!(
            img_style.get("width"),
            Some(&CssValue::Length(200.0, LengthUnit::Px))
        );
        assert_eq!(
            img_style.get("height"),
            Some(&CssValue::Length(150.0, LengthUnit::Px))
        );
    }

    #[test]
    fn test_presentational_hints_mapping_percent() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let table = dom.create_node(NodeData::Element {
            name: "table".into(),
            attrs: vec![
                ("width".into(), "50%".into()),
                ("height".into(), "100%".into()),
            ],
        });
        dom.append_child(doc, table);

        let stylesheet = parse_stylesheet("");
        let styles = compute_styles(&dom, &stylesheet);
        let table_style = styles.get(&table).unwrap();

        assert_eq!(
            table_style.get("width"),
            Some(&CssValue::Length(50.0, LengthUnit::Percent))
        );
        assert_eq!(
            table_style.get("height"),
            Some(&CssValue::Length(100.0, LengthUnit::Percent))
        );
    }

    #[test]
    fn test_presentational_hints_author_overrides() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let img = dom.create_node(NodeData::Element {
            name: "img".into(),
            attrs: vec![("width".into(), "200".into())],
        });
        dom.append_child(doc, img);

        // Author CSS rule `img { width: 10px; }` should win over presentational hint.
        let stylesheet = parse_stylesheet("img { width: 10px; }");
        let styles = compute_styles(&dom, &stylesheet);
        let img_style = styles.get(&img).unwrap();

        assert_eq!(
            img_style.get("width"),
            Some(&CssValue::Length(10.0, LengthUnit::Px))
        );
    }

    #[test]
    fn test_presentational_hints_inline_overrides() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let img = dom.create_node(NodeData::Element {
            name: "img".into(),
            attrs: vec![
                ("width".into(), "200".into()),
                ("style".into(), "width: 5px;".into()),
            ],
        });
        dom.append_child(doc, img);

        let stylesheet = parse_stylesheet("");
        let styles = compute_styles(&dom, &stylesheet);
        let img_style = styles.get(&img).unwrap();

        assert_eq!(
            img_style.get("width"),
            Some(&CssValue::Length(5.0, LengthUnit::Px))
        );
    }

    #[test]
    fn test_presentational_hints_bgcolor_valid() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let td = dom.create_node(NodeData::Element {
            name: "td".into(),
            attrs: vec![("bgcolor".into(), "#ff0000".into())],
        });
        dom.append_child(doc, td);

        let table = dom.create_node(NodeData::Element {
            name: "table".into(),
            attrs: vec![("bgcolor".into(), "blue".into())],
        });
        dom.append_child(doc, table);

        let stylesheet = parse_stylesheet("");
        let styles = compute_styles(&dom, &stylesheet);
        let td_style = styles.get(&td).unwrap();
        let table_style = styles.get(&table).unwrap();

        assert_eq!(
            td_style.get("background-color"),
            Some(&CssValue::Color(crate::css::values::Color::Rgba(
                255, 0, 0, 255
            )))
        );
        assert_eq!(
            table_style.get("background-color"),
            Some(&CssValue::Color(crate::css::values::Color::Rgba(
                0, 0, 255, 255
            )))
        );
    }

    #[test]
    fn test_presentational_hints_bgcolor_invalid() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let td = dom.create_node(NodeData::Element {
            name: "td".into(),
            attrs: vec![("bgcolor".into(), "ccc".into())], // invalid: no '#' or not a named color
        });
        dom.append_child(doc, td);

        let stylesheet = parse_stylesheet("");
        let styles = compute_styles(&dom, &stylesheet);
        let td_style = styles.get(&td).unwrap();

        assert_eq!(td_style.get("background-color"), None);
    }

    #[test]
    fn test_presentational_hints_align() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let td = dom.create_node(NodeData::Element {
            name: "td".into(),
            attrs: vec![("align".into(), "center".into())],
        });
        dom.append_child(doc, td);

        let tr = dom.create_node(NodeData::Element {
            name: "tr".into(),
            attrs: vec![("align".into(), "right".into())],
        });
        dom.append_child(doc, tr);

        let table = dom.create_node(NodeData::Element {
            name: "table".into(),
            attrs: vec![("align".into(), "left".into())],
        });
        dom.append_child(doc, table);

        // invalid align value should be ignored
        let col = dom.create_node(NodeData::Element {
            name: "col".into(),
            attrs: vec![("align".into(), "justify".into())],
        });
        dom.append_child(doc, col);

        let stylesheet = parse_stylesheet("");
        let styles = compute_styles(&dom, &stylesheet);

        assert_eq!(
            styles.get(&td).unwrap().get("text-align"),
            Some(&CssValue::Keyword("center".to_string()))
        );
        assert_eq!(
            styles.get(&tr).unwrap().get("text-align"),
            Some(&CssValue::Keyword("right".to_string()))
        );
        assert_eq!(
            styles.get(&table).unwrap().get("text-align"),
            Some(&CssValue::Keyword("left".to_string()))
        );
        assert_eq!(
            styles.get(&col).unwrap().get("text-align"),
            Some(&CssValue::Keyword("left".to_string()))
        );
    }

    #[test]
    fn test_presentational_hints_valign() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let td = dom.create_node(NodeData::Element {
            name: "td".into(),
            attrs: vec![("valign".into(), "top".into())],
        });
        dom.append_child(doc, td);

        let th = dom.create_node(NodeData::Element {
            name: "th".into(),
            attrs: vec![("valign".into(), "middle".into())],
        });
        dom.append_child(doc, th);

        let tr = dom.create_node(NodeData::Element {
            name: "tr".into(),
            attrs: vec![("valign".into(), "bottom".into())],
        });
        dom.append_child(doc, tr);

        // invalid valign value should be ignored
        let invalid_td = dom.create_node(NodeData::Element {
            name: "td".into(),
            attrs: vec![("valign".into(), "baseline".into())],
        });
        dom.append_child(doc, invalid_td);

        let stylesheet = parse_stylesheet("");
        let styles = compute_styles(&dom, &stylesheet);

        assert_eq!(
            styles.get(&td).unwrap().get("vertical-align"),
            Some(&CssValue::Keyword("top".to_string()))
        );
        assert_eq!(
            styles.get(&th).unwrap().get("vertical-align"),
            Some(&CssValue::Keyword("middle".to_string()))
        );
        assert_eq!(
            styles.get(&tr).unwrap().get("vertical-align"),
            Some(&CssValue::Keyword("bottom".to_string()))
        );
        assert_eq!(styles.get(&invalid_td).unwrap().get("vertical-align"), None);
    }

    #[test]
    fn test_presentational_hints_bgcolor_overrides() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let td = dom.create_node(NodeData::Element {
            name: "td".into(),
            attrs: vec![("bgcolor".into(), "red".into())],
        });
        dom.append_child(doc, td);

        // Author style should override presentational hint
        let stylesheet = parse_stylesheet("td { background-color: blue; }");
        let styles = compute_styles(&dom, &stylesheet);
        let td_style = styles.get(&td).unwrap();

        assert_eq!(
            td_style.get("background-color"),
            Some(&CssValue::Color(crate::css::values::Color::Rgba(
                0, 0, 255, 255
            )))
        );
    }

    #[test]
    fn test_presentational_hints_hspace_vspace() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let img = dom.create_node(NodeData::Element {
            name: "img".into(),
            attrs: vec![
                ("hspace".into(), "10".into()),
                ("vspace".into(), "5".into()),
            ],
        });
        dom.append_child(doc, img);

        let img_invalid = dom.create_node(NodeData::Element {
            name: "img".into(),
            attrs: vec![
                ("hspace".into(), "abc".into()),
                ("vspace".into(), "-5".into()),
            ],
        });
        dom.append_child(doc, img_invalid);

        let stylesheet = parse_stylesheet("");
        let styles = compute_styles(&dom, &stylesheet);

        let img_style = styles.get(&img).unwrap();
        assert_eq!(
            img_style.get("margin-left"),
            Some(&CssValue::Length(10.0, LengthUnit::Px))
        );
        assert_eq!(
            img_style.get("margin-right"),
            Some(&CssValue::Length(10.0, LengthUnit::Px))
        );
        assert_eq!(
            img_style.get("margin-top"),
            Some(&CssValue::Length(5.0, LengthUnit::Px))
        );
        assert_eq!(
            img_style.get("margin-bottom"),
            Some(&CssValue::Length(5.0, LengthUnit::Px))
        );

        let img_invalid_style = styles.get(&img_invalid).unwrap();
        assert_eq!(img_invalid_style.get("margin-left"), None);
        assert_eq!(img_invalid_style.get("margin-right"), None);
        assert_eq!(img_invalid_style.get("margin-top"), None);
        assert_eq!(img_invalid_style.get("margin-bottom"), None);
    }

    #[test]
    fn test_background_shorthand_sets_background_color() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        // Standard background shorthand expansion
        let stylesheet1 = parse_stylesheet("div { background: blue; }");
        let styles1 = compute_styles(&dom, &stylesheet1);
        let style1 = styles1.get(&div).unwrap();

        // Standard background-color
        let stylesheet_ref = parse_stylesheet("div { background-color: blue; }");
        let styles_ref = compute_styles(&dom, &stylesheet_ref);
        let style_ref = styles_ref.get(&div).unwrap();

        assert_eq!(
            style1.get("background-color"),
            style_ref.get("background-color")
        );

        // Shorthand with extra tokens (e.g., none keyword)
        let stylesheet2 = parse_stylesheet("div { background: blue none; }");
        let styles2 = compute_styles(&dom, &stylesheet2);
        let style2 = styles2.get(&div).unwrap();

        assert_eq!(
            style2.get("background-color"),
            style_ref.get("background-color")
        );
    }
}
