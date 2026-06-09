use crate::css::parser::{Declaration, Rule, Stylesheet};
use crate::css::values::{CssValue, parse_value};
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
/// Returns a tuple of (id, class, type) counts.
// spec: https://www.w3.org/TR/selectors-4/#specificity
pub fn specificity(sel: &ComplexSelector) -> (u32, u32, u32) {
    let mut id = 0;
    let mut class = 0;
    let mut type_ = 0;

    for (_, compound) in &sel.parts {
        for component in &compound.components {
            match component {
                Component::Id(_) => id += 1,
                Component::Class(_) | Component::Attribute { .. } | Component::PseudoClass(_) => {
                    class += 1
                }
                Component::Type(_) | Component::PseudoElement(_) => type_ += 1,
                Component::Universal => {}
            }
        }
    }

    (id, class, type_)
}

/// Computes the styles for all nodes in the DOM based on the given stylesheet.
pub fn compute_styles(dom: &Dom, stylesheet: &Stylesheet) -> HashMap<NodeId, ComputedStyle> {
    let mut styles = HashMap::new();
    let root = dom.document();

    // Traverse the DOM in pre-order to resolve styles, allowing inheritance from parents.
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        let computed = compute_node_style(dom, node, stylesheet, &styles);
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
) -> ComputedStyle {
    let mut properties = HashMap::new();

    // 1. Collect all matching rules and their declarations.
    let mut matched_declarations = Vec::new();

    for (rule_index, rule) in stylesheet.rules.iter().enumerate() {
        if let Rule::Qualified(qualified_rule) = rule {
            // Re-parse prelude as selector list.
            // In a real engine, this would be done once during stylesheet parsing.
            // SPEC S-10: "serialize/parse it back to a selector string via parse_selector_list, or match per complex selector".
            // Since Rule::Qualified prelude is Vec<ComponentValue>, and selector::parse_selector_list takes &str,
            // we should have a way to get selectors.
            // However, the task says: "the qualified rule prelude is Vec<ComponentValue> — serialize/parse it back to a selector string via parse_selector_list, or match per complex selector".

            // For now, let's assume we can match against the rules.
            // I need to find a way to get ComplexSelectors from QualifiedRule.
            // The task implies I might need to serialize ComponentValue back to string to use parse_selector_list.

            let selector_str = serialize_component_values(&qualified_rule.prelude);
            if let Ok(selector_list) = crate::selector::parse_selector_list(&selector_str) {
                for sel in &selector_list.0 {
                    if matches_complex(sel, dom, node) {
                        let spec = specificity(sel);
                        for decl in &qualified_rule.declarations {
                            matched_declarations.push(MatchedDeclaration {
                                declaration: decl,
                                specificity: spec,
                                source_order: rule_index,
                            });
                        }
                    }
                }
            }
        }
    }

    // 2. Cascade: sort by (!important, specificity, source order).
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

    // 3. Apply declarations.
    for matched in matched_declarations {
        if let Some(value) = parse_value(&matched.declaration.value) {
            properties.insert(matched.declaration.name.clone(), value);
        }
    }

    // 4. Inheritance.
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

    ComputedStyle { properties }
}

struct MatchedDeclaration<'a> {
    declaration: &'a Declaration,
    specificity: (u32, u32, u32),
    source_order: usize,
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
        assert_eq!(specificity(&sel), (0, 0, 1));

        let sel = crate::selector::parse_selector_list(".foo")
            .unwrap()
            .0
            .remove(0);
        assert_eq!(specificity(&sel), (0, 1, 0));

        let sel = crate::selector::parse_selector_list("#bar")
            .unwrap()
            .0
            .remove(0);
        assert_eq!(specificity(&sel), (1, 0, 0));

        let sel = crate::selector::parse_selector_list("div.foo#bar")
            .unwrap()
            .0
            .remove(0);
        assert_eq!(specificity(&sel), (1, 1, 1));

        let sel = crate::selector::parse_selector_list("div > span")
            .unwrap()
            .0
            .remove(0);
        assert_eq!(specificity(&sel), (0, 0, 2));
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
}
