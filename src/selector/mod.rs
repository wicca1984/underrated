use crate::css::{CssToken, CssTokenizer};

#[derive(Debug, PartialEq, Clone)]
pub enum Component {
    Type(String),
    Universal,
    Id(String),
    Class(String),
    Attribute {
        name: String,
        op: Option<AttrOp>,
        value: Option<String>,
    },
    PseudoClass(String),
    PseudoElement(String),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AttrOp {
    Exact,     // =
    Includes,  // ~=
    DashMatch, // |=
    Prefix,    // ^=
    Suffix,    // $=
    Substring, // *=
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Combinator {
    Descendant,
    Child,
    NextSibling,
    SubsequentSibling,
}

#[derive(Debug, PartialEq, Clone)]
pub struct CompoundSelector {
    pub components: Vec<Component>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ComplexSelector {
    pub parts: Vec<(Combinator, CompoundSelector)>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SelectorList(pub Vec<ComplexSelector>);

#[derive(Debug, PartialEq, Clone)]
pub enum SelectorParseError {
    UnexpectedToken(CssToken),
    UnexpectedEof,
    InvalidSelector,
}

pub fn parse_selector_list(input: &str) -> Result<SelectorList, SelectorParseError> {
    if input.trim().is_empty() {
        return Err(SelectorParseError::InvalidSelector);
    }
    let mut tokenizer = CssTokenizer::new(input);
    let mut parser = SelectorParser::new(&mut tokenizer);
    parser.parse_selector_list()
}

struct SelectorParser<'a> {
    tokenizer: &'a mut CssTokenizer,
    peeked: Option<CssToken>,
}

impl<'a> SelectorParser<'a> {
    fn new(tokenizer: &'a mut CssTokenizer) -> Self {
        Self {
            tokenizer,
            peeked: None,
        }
    }

    fn peek(&mut self) -> &CssToken {
        self.peeked
            .get_or_insert_with(|| self.tokenizer.next_token())
    }

    fn consume(&mut self) -> CssToken {
        self.peeked
            .take()
            .unwrap_or_else(|| self.tokenizer.next_token())
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), CssToken::Whitespace) {
            self.consume();
        }
    }

    fn parse_selector_list(&mut self) -> Result<SelectorList, SelectorParseError> {
        let mut selectors = Vec::new();
        loop {
            self.skip_whitespace();
            selectors.push(self.parse_complex_selector()?);
            self.skip_whitespace();
            match self.peek() {
                CssToken::Comma => {
                    self.consume();
                }
                CssToken::Eof => break,
                _ => return Err(SelectorParseError::UnexpectedToken(self.consume())),
            }
        }
        Ok(SelectorList(selectors))
    }

    fn parse_complex_selector(&mut self) -> Result<ComplexSelector, SelectorParseError> {
        let mut parts = Vec::new();
        // First part always has Descendant combinator (implicit)
        let first_compound = self.parse_compound_selector()?;
        parts.push((Combinator::Descendant, first_compound));

        loop {
            let mut has_whitespace = false;
            while matches!(self.peek(), CssToken::Whitespace) {
                self.consume();
                has_whitespace = true;
            }

            match self.peek() {
                CssToken::Comma | CssToken::Eof => break,
                CssToken::Delim('>') | CssToken::Delim('+') | CssToken::Delim('~') => {
                    let comb = match self.consume() {
                        CssToken::Delim('>') => Combinator::Child,
                        CssToken::Delim('+') => Combinator::NextSibling,
                        CssToken::Delim('~') => Combinator::SubsequentSibling,
                        _ => unreachable!(),
                    };
                    self.skip_whitespace();
                    let compound = self.parse_compound_selector()?;
                    parts.push((comb, compound));
                }
                _ => {
                    if has_whitespace {
                        // Descendant combinator
                        if let Ok(compound) = self.parse_compound_selector() {
                            parts.push((Combinator::Descendant, compound));
                        } else {
                            // If it's not a compound selector, we might have just finished or it's invalid.
                            // But since we had whitespace and it's not Comma/Eof/ExplicitComb,
                            // it should be another compound selector.
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }
        Ok(ComplexSelector { parts })
    }

    fn parse_compound_selector(&mut self) -> Result<CompoundSelector, SelectorParseError> {
        let mut components = Vec::new();

        // TODO(spec): namespaces like ns|type
        // Type or Universal selector (optional, must be first)
        match self.peek() {
            CssToken::Ident(name) => {
                let name = name.clone();
                self.consume();
                components.push(Component::Type(name));
            }
            CssToken::Delim('*') => {
                self.consume();
                components.push(Component::Universal);
            }
            _ => {}
        }

        loop {
            match self.peek() {
                CssToken::Hash(name) => {
                    let name = name.clone();
                    self.consume();
                    components.push(Component::Id(name));
                }
                CssToken::Delim('.') => {
                    self.consume();
                    if let CssToken::Ident(name) = self.consume() {
                        components.push(Component::Class(name));
                    } else {
                        return Err(SelectorParseError::InvalidSelector);
                    }
                }
                CssToken::LeftBracket => {
                    components.push(self.parse_attribute_selector()?);
                }
                CssToken::Colon => {
                    components.push(self.parse_pseudo_selector()?);
                }
                _ => break,
            }
        }

        if components.is_empty() {
            Err(SelectorParseError::InvalidSelector)
        } else {
            Ok(CompoundSelector { components })
        }
    }

    fn parse_attribute_selector(&mut self) -> Result<Component, SelectorParseError> {
        if !matches!(self.consume(), CssToken::LeftBracket) {
            return Err(SelectorParseError::InvalidSelector);
        }
        self.skip_whitespace();
        let name = if let CssToken::Ident(name) = self.consume() {
            name
        } else {
            return Err(SelectorParseError::InvalidSelector);
        };
        self.skip_whitespace();

        let op = match self.peek() {
            CssToken::Delim('=') => {
                self.consume();
                Some(AttrOp::Exact)
            }
            CssToken::Delim('~') => {
                self.consume();
                if matches!(self.consume(), CssToken::Delim('=')) {
                    Some(AttrOp::Includes)
                } else {
                    return Err(SelectorParseError::InvalidSelector);
                }
            }
            CssToken::Delim('|') => {
                self.consume();
                if matches!(self.consume(), CssToken::Delim('=')) {
                    Some(AttrOp::DashMatch)
                } else {
                    return Err(SelectorParseError::InvalidSelector);
                }
            }
            CssToken::Delim('^') => {
                self.consume();
                if matches!(self.consume(), CssToken::Delim('=')) {
                    Some(AttrOp::Prefix)
                } else {
                    return Err(SelectorParseError::InvalidSelector);
                }
            }
            CssToken::Delim('$') => {
                self.consume();
                if matches!(self.consume(), CssToken::Delim('=')) {
                    Some(AttrOp::Suffix)
                } else {
                    return Err(SelectorParseError::InvalidSelector);
                }
            }
            CssToken::Delim('*') => {
                self.consume();
                if matches!(self.consume(), CssToken::Delim('=')) {
                    Some(AttrOp::Substring)
                } else {
                    return Err(SelectorParseError::InvalidSelector);
                }
            }
            _ => None,
        };

        let value = if op.is_some() {
            self.skip_whitespace();
            let val = match self.consume() {
                CssToken::Ident(s) | CssToken::String(s) => s,
                _ => return Err(SelectorParseError::InvalidSelector),
            };
            self.skip_whitespace();
            Some(val)
        } else {
            None
        };

        if !matches!(self.consume(), CssToken::RightBracket) {
            return Err(SelectorParseError::InvalidSelector);
        }

        Ok(Component::Attribute { name, op, value })
    }

    fn parse_pseudo_selector(&mut self) -> Result<Component, SelectorParseError> {
        if !matches!(self.consume(), CssToken::Colon) {
            return Err(SelectorParseError::InvalidSelector);
        }
        if matches!(self.peek(), CssToken::Colon) {
            self.consume(); // second colon
            if let CssToken::Ident(name) = self.consume() {
                Ok(Component::PseudoElement(name))
            } else {
                Err(SelectorParseError::InvalidSelector)
            }
        } else {
            if let CssToken::Ident(name) = self.consume() {
                // TODO(spec): functional pseudo-classes like :nth-child(...), :not(...)
                Ok(Component::PseudoClass(name))
            } else {
                Err(SelectorParseError::InvalidSelector)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_type() {
        let list = parse_selector_list("div").unwrap();
        assert_eq!(
            list,
            SelectorList(vec![ComplexSelector {
                parts: vec![(
                    Combinator::Descendant,
                    CompoundSelector {
                        components: vec![Component::Type("div".to_string())]
                    }
                )]
            }])
        );
    }

    #[test]
    fn test_parse_universal() {
        let list = parse_selector_list("*").unwrap();
        assert_eq!(
            list,
            SelectorList(vec![ComplexSelector {
                parts: vec![(
                    Combinator::Descendant,
                    CompoundSelector {
                        components: vec![Component::Universal]
                    }
                )]
            }])
        );
    }

    #[test]
    fn test_parse_id_class() {
        let list = parse_selector_list("#foo.bar").unwrap();
        assert_eq!(
            list,
            SelectorList(vec![ComplexSelector {
                parts: vec![(
                    Combinator::Descendant,
                    CompoundSelector {
                        components: vec![
                            Component::Id("foo".to_string()),
                            Component::Class("bar".to_string())
                        ]
                    }
                )]
            }])
        );
    }

    #[test]
    fn test_parse_attribute() {
        let list = parse_selector_list("[title],[href=\"#\"]").unwrap();
        assert_eq!(list.0.len(), 2);
        assert_eq!(
            list.0[0].parts[0].1.components[0],
            Component::Attribute {
                name: "title".to_string(),
                op: None,
                value: None
            }
        );
        assert_eq!(
            list.0[1].parts[0].1.components[0],
            Component::Attribute {
                name: "href".to_string(),
                op: Some(AttrOp::Exact),
                value: Some("#".to_string())
            }
        );
    }

    #[test]
    fn test_parse_combinators() {
        let list = parse_selector_list("div > p + span ~ a").unwrap();
        let parts = &list.0[0].parts;
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0].0, Combinator::Descendant);
        assert_eq!(parts[1].0, Combinator::Child);
        assert_eq!(parts[2].0, Combinator::NextSibling);
        assert_eq!(parts[3].0, Combinator::SubsequentSibling);
    }

    #[test]
    fn test_parse_pseudo() {
        let list = parse_selector_list("a:hover::before").unwrap();
        let components = &list.0[0].parts[0].1.components;
        assert_eq!(components[0], Component::Type("a".to_string()));
        assert_eq!(components[1], Component::PseudoClass("hover".to_string()));
        assert_eq!(
            components[2],
            Component::PseudoElement("before".to_string())
        );
    }

    #[test]
    fn test_parse_complex_mixed() {
        // div.a#b[href^="x"] > span
        let input = "div.a#b[href^=\"x\"] > span";
        let list = parse_selector_list(input).unwrap();
        assert_eq!(list.0.len(), 1);
        let parts = &list.0[0].parts;
        assert_eq!(parts.len(), 2);

        assert_eq!(parts[0].0, Combinator::Descendant);
        assert_eq!(
            parts[0].1.components,
            vec![
                Component::Type("div".to_string()),
                Component::Class("a".to_string()),
                Component::Id("b".to_string()),
                Component::Attribute {
                    name: "href".to_string(),
                    op: Some(AttrOp::Prefix),
                    value: Some("x".to_string())
                }
            ]
        );

        assert_eq!(parts[1].0, Combinator::Child);
        assert_eq!(
            parts[1].1.components,
            vec![Component::Type("span".to_string())]
        );
    }

    #[test]
    fn test_invalid_selectors() {
        assert!(parse_selector_list("").is_err());
        assert!(parse_selector_list(",").is_err());
        assert!(parse_selector_list("div >").is_err());
        assert!(parse_selector_list("div >> span").is_err());
        assert!(parse_selector_list("[attr=]").is_err());
    }
}
