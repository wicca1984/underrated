mod matching;

use crate::css::{CssToken, CssTokenizer};
pub use matching::{
    NodeState, clear_node_states, get_node_state, matches, matches_complex, set_node_state,
};

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
        modifier: Option<char>,
    },
    PseudoClass(String),
    PseudoElement(String),
    NthChild(i32, i32),
    NthLastChild(i32, i32),
    NthOfType(i32, i32),
    NthLastOfType(i32, i32),
    Not(Box<CompoundSelector>),
    Is(SelectorList),
    Where(SelectorList),
    FirstChild,
    LastChild,
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
    peeked: Vec<CssToken>,
}

impl<'a> SelectorParser<'a> {
    fn new(tokenizer: &'a mut CssTokenizer) -> Self {
        Self {
            tokenizer,
            peeked: Vec::new(),
        }
    }

    fn peek_nth(&mut self, n: usize) -> &CssToken {
        while self.peeked.len() <= n {
            self.peeked.push(self.tokenizer.next_token());
        }
        &self.peeked[n]
    }

    fn peek(&mut self) -> &CssToken {
        self.peek_nth(0)
    }

    fn consume(&mut self) -> CssToken {
        if !self.peeked.is_empty() {
            self.peeked.remove(0)
        } else {
            self.tokenizer.next_token()
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), CssToken::Whitespace) {
            self.consume();
        }
    }

    fn skip_to_next_forgiving_item(&mut self) -> Result<bool, SelectorParseError> {
        let mut depth = 0;
        loop {
            match self.peek() {
                CssToken::LeftParen
                | CssToken::Function(_)
                | CssToken::LeftBracket
                | CssToken::LeftBrace => {
                    depth += 1;
                    self.consume();
                }
                CssToken::RightParen | CssToken::RightBracket | CssToken::RightBrace => {
                    if depth == 0 {
                        return Ok(false);
                    }
                    depth -= 1;
                    self.consume();
                }
                CssToken::Comma => {
                    if depth == 0 {
                        self.consume();
                        return Ok(true);
                    }
                    self.consume();
                }
                CssToken::Eof => {
                    return Err(SelectorParseError::UnexpectedEof);
                }
                _ => {
                    self.consume();
                }
            }
        }
    }

    fn parse_forgiving_selector_list(&mut self) -> Result<SelectorList, SelectorParseError> {
        let mut selectors = Vec::new();
        loop {
            self.skip_whitespace();
            if matches!(self.peek(), CssToken::RightParen) {
                break;
            }

            match self.parse_complex_selector() {
                Ok(selector) => {
                    self.skip_whitespace();
                    match self.peek() {
                        CssToken::Comma => {
                            self.consume();
                            selectors.push(selector);
                        }
                        CssToken::RightParen => {
                            selectors.push(selector);
                        }
                        _ => {
                            if !self.skip_to_next_forgiving_item()? {
                                break;
                            }
                        }
                    }
                }
                Err(_) => {
                    if !self.skip_to_next_forgiving_item()? {
                        break;
                    }
                }
            }
        }
        Ok(SelectorList(selectors))
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
                            // TODO(spec): trailing invalid tokens after a valid selector (e.g.
                            // `div @media`) are silently dropped instead of raising a parse error.
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

        // Check for namespace prefix (e.g. ns|type, *|type, |type)
        let has_ns = matches!(self.peek_nth(0), CssToken::Ident(_))
            && matches!(self.peek_nth(1), CssToken::Delim('|'))
            || matches!(self.peek_nth(0), CssToken::Delim('*'))
                && matches!(self.peek_nth(1), CssToken::Delim('|'))
            || matches!(self.peek_nth(0), CssToken::Delim('|'));

        if has_ns {
            // Consume namespace prefix
            if let CssToken::Delim('|') = self.peek_nth(0) {
                self.consume(); // consume '|'
            } else {
                self.consume(); // consume ident or '*'
                self.consume(); // consume '|'
            }
        }

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
            _ => {
                if has_ns {
                    components.push(Component::Universal);
                }
            }
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

        // spec: Selectors L4 — an `i`/`s` modifier may follow even a presence-only
        // attribute selector (e.g. `[attr i]`), so do not gate this on `op`.
        let mut modifier = None;
        if let CssToken::Ident(s) = self.peek() {
            let s_lower = s.to_ascii_lowercase();
            if s_lower == "i" {
                modifier = Some('i');
                self.consume();
                self.skip_whitespace();
            } else if s_lower == "s" {
                modifier = Some('s');
                self.consume();
                self.skip_whitespace();
            } else {
                // TODO(spec): Unknown attribute modifier.
                self.consume();
                self.skip_whitespace();
            }
        }

        if !matches!(self.consume(), CssToken::RightBracket) {
            return Err(SelectorParseError::InvalidSelector);
        }

        Ok(Component::Attribute {
            name,
            op,
            value,
            modifier,
        })
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
            match self.consume() {
                CssToken::Ident(name) => match name.to_ascii_lowercase().as_str() {
                    "first-child" => Ok(Component::FirstChild),
                    "last-child" => Ok(Component::LastChild),
                    _ => Ok(Component::PseudoClass(name.to_ascii_lowercase())),
                },
                CssToken::Function(name) => match name.to_ascii_lowercase().as_str() {
                    "nth-child" => {
                        let (a, b) = self.parse_nth()?;
                        if !matches!(self.consume(), CssToken::RightParen) {
                            return Err(SelectorParseError::InvalidSelector);
                        }
                        Ok(Component::NthChild(a, b))
                    }
                    "nth-of-type" => {
                        let (a, b) = self.parse_nth()?;
                        if !matches!(self.consume(), CssToken::RightParen) {
                            return Err(SelectorParseError::InvalidSelector);
                        }
                        Ok(Component::NthOfType(a, b))
                    }
                    "nth-last-child" => {
                        let (a, b) = self.parse_nth()?;
                        if !matches!(self.consume(), CssToken::RightParen) {
                            return Err(SelectorParseError::InvalidSelector);
                        }
                        Ok(Component::NthLastChild(a, b))
                    }
                    "nth-last-of-type" => {
                        let (a, b) = self.parse_nth()?;
                        if !matches!(self.consume(), CssToken::RightParen) {
                            return Err(SelectorParseError::InvalidSelector);
                        }
                        Ok(Component::NthLastOfType(a, b))
                    }
                    "is" => {
                        let list = self.parse_forgiving_selector_list()?;
                        if !matches!(self.consume(), CssToken::RightParen) {
                            return Err(SelectorParseError::InvalidSelector);
                        }
                        Ok(Component::Is(list))
                    }
                    "where" => {
                        let list = self.parse_forgiving_selector_list()?;
                        if !matches!(self.consume(), CssToken::RightParen) {
                            return Err(SelectorParseError::InvalidSelector);
                        }
                        Ok(Component::Where(list))
                    }
                    "not" => {
                        let compound = self.parse_compound_selector()?;
                        if !matches!(self.consume(), CssToken::RightParen) {
                            return Err(SelectorParseError::InvalidSelector);
                        }
                        Ok(Component::Not(Box::new(compound)))
                    }
                    "lang" => {
                        let mut langs = Vec::new();
                        loop {
                            self.skip_whitespace();
                            match self.peek() {
                                CssToken::Ident(s) | CssToken::String(s) => {
                                    langs.push(s.clone());
                                    self.consume();
                                }
                                _ => return Err(SelectorParseError::InvalidSelector),
                            }
                            self.skip_whitespace();
                            match self.peek() {
                                CssToken::Comma => {
                                    self.consume();
                                }
                                CssToken::RightParen => {
                                    self.consume();
                                    break;
                                }
                                _ => return Err(SelectorParseError::InvalidSelector),
                            }
                        }
                        if langs.is_empty() {
                            return Err(SelectorParseError::InvalidSelector);
                        }
                        Ok(Component::PseudoClass(format!("lang({})", langs.join(","))))
                    }
                    _ => {
                        // TODO(spec): Other functional pseudo-classes.
                        // We need to consume until RightParen to stay in sync.
                        let mut depth = 1;
                        while depth > 0 {
                            match self.consume() {
                                CssToken::LeftParen | CssToken::Function(_) => depth += 1,
                                CssToken::RightParen => depth -= 1,
                                CssToken::Eof => return Err(SelectorParseError::UnexpectedEof),
                                _ => {}
                            }
                        }
                        Ok(Component::PseudoClass(name))
                    }
                },
                _ => Err(SelectorParseError::InvalidSelector),
            }
        }
    }

    fn consume_nth_b(&mut self, a: i32) -> Result<(i32, i32), SelectorParseError> {
        self.skip_whitespace();
        let b = match self.peek() {
            CssToken::Number(n) => {
                let val = *n as i32;
                self.consume();
                val
            }
            CssToken::Delim('+') | CssToken::Delim('-') => {
                let sign = if matches!(self.consume(), CssToken::Delim('+')) {
                    1
                } else {
                    -1
                };
                self.skip_whitespace();
                if let CssToken::Number(n) = self.consume() {
                    sign * n as i32
                } else {
                    return Err(SelectorParseError::InvalidSelector);
                }
            }
            _ => 0,
        };
        Ok((a, b))
    }

    fn parse_nth(&mut self) -> Result<(i32, i32), SelectorParseError> {
        self.skip_whitespace();
        match self.peek().clone() {
            CssToken::Ident(s) if s.eq_ignore_ascii_case("odd") => {
                self.consume();
                Ok((2, 1))
            }
            CssToken::Ident(s) if s.eq_ignore_ascii_case("even") => {
                self.consume();
                Ok((2, 0))
            }
            _ => {
                let mut sign = 1;
                if matches!(self.peek(), CssToken::Delim('+') | CssToken::Delim('-')) {
                    if matches!(self.consume(), CssToken::Delim('-')) {
                        sign = -1;
                    }
                    self.skip_whitespace();
                }

                match self.peek().clone() {
                    CssToken::Number(n) => {
                        self.consume();
                        Ok((0, sign * n as i32))
                    }
                    CssToken::Dimension { value, unit } if unit.eq_ignore_ascii_case("n") => {
                        self.consume();
                        self.consume_nth_b(sign * value as i32)
                    }
                    CssToken::Ident(s) => {
                        let s_lower = s.to_lowercase();
                        if s_lower == "n" {
                            self.consume();
                            self.consume_nth_b(sign)
                        } else if s_lower == "-n" && sign == 1 {
                            self.consume();
                            self.consume_nth_b(-1)
                        } else if s_lower == "n-" {
                            self.consume();
                            self.skip_whitespace();
                            if let CssToken::Number(n) = self.consume() {
                                Ok((sign, -(n as i32)))
                            } else {
                                Err(SelectorParseError::InvalidSelector)
                            }
                        } else if s_lower == "-n-" && sign == 1 {
                            self.consume();
                            self.skip_whitespace();
                            if let CssToken::Number(n) = self.consume() {
                                Ok((-1, -(n as i32)))
                            } else {
                                Err(SelectorParseError::InvalidSelector)
                            }
                        } else if let Some(rest) = s_lower.strip_prefix("n-") {
                            if let Ok(b) = rest.parse::<i32>() {
                                self.consume();
                                Ok((sign, -b))
                            } else {
                                Err(SelectorParseError::InvalidSelector)
                            }
                        } else if sign == 1 && s_lower.starts_with("-n-") {
                            let rest = &s_lower[3..];
                            if let Ok(b) = rest.parse::<i32>() {
                                self.consume();
                                Ok((-1, -b))
                            } else {
                                Err(SelectorParseError::InvalidSelector)
                            }
                        } else {
                            Err(SelectorParseError::InvalidSelector)
                        }
                    }
                    _ => Err(SelectorParseError::InvalidSelector),
                }
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
                value: None,
                modifier: None,
            }
        );
        assert_eq!(
            list.0[1].parts[0].1.components[0],
            Component::Attribute {
                name: "href".to_string(),
                op: Some(AttrOp::Exact),
                value: Some("#".to_string()),
                modifier: None,
            }
        );
    }

    #[test]
    fn test_parse_presence_attr_with_modifier() {
        // spec: Selectors L4 allows an `i` modifier on a presence-only selector.
        let list = parse_selector_list("[title i]").unwrap();
        assert_eq!(
            list.0[0].parts[0].1.components[0],
            Component::Attribute {
                name: "title".to_string(),
                op: None,
                value: None,
                modifier: Some('i'),
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
                    value: Some("x".to_string()),
                    modifier: None,
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
    fn test_parse_functional_pseudo() {
        // :nth-child(2n+1)
        let list = parse_selector_list(":nth-child(2n+1)").unwrap();
        assert_eq!(
            list.0[0].parts[0].1.components[0],
            Component::NthChild(2, 1)
        );

        // :nth-of-type(2n+1)
        let list = parse_selector_list(":nth-of-type(2n+1)").unwrap();
        assert_eq!(
            list.0[0].parts[0].1.components[0],
            Component::NthOfType(2, 1)
        );

        // :nth-last-child(even)
        let list = parse_selector_list(":nth-last-child(even)").unwrap();
        assert_eq!(
            list.0[0].parts[0].1.components[0],
            Component::NthLastChild(2, 0)
        );

        // :nth-last-of-type(odd)
        let list = parse_selector_list(":nth-last-of-type(odd)").unwrap();
        assert_eq!(
            list.0[0].parts[0].1.components[0],
            Component::NthLastOfType(2, 1)
        );

        // :nth-child(even)
        let list = parse_selector_list(":nth-child(even)").unwrap();
        assert_eq!(
            list.0[0].parts[0].1.components[0],
            Component::NthChild(2, 0)
        );

        // :nth-child(odd)
        let list = parse_selector_list(":nth-child(odd)").unwrap();
        assert_eq!(
            list.0[0].parts[0].1.components[0],
            Component::NthChild(2, 1)
        );

        // :nth-child(5)
        let list = parse_selector_list(":nth-child(5)").unwrap();
        assert_eq!(
            list.0[0].parts[0].1.components[0],
            Component::NthChild(0, 5)
        );

        // :nth-child(n)
        let list = parse_selector_list(":nth-child(n)").unwrap();
        assert_eq!(
            list.0[0].parts[0].1.components[0],
            Component::NthChild(1, 0)
        );

        // :nth-child(-n+3)
        let list = parse_selector_list(":nth-child(-n+3)").unwrap();
        assert_eq!(
            list.0[0].parts[0].1.components[0],
            Component::NthChild(-1, 3)
        );

        // :nth-child(n-1)
        let list = parse_selector_list(":nth-child(n-1)").unwrap();
        assert_eq!(
            list.0[0].parts[0].1.components[0],
            Component::NthChild(1, -1)
        );

        // :nth-child(2n - 1)
        let list = parse_selector_list(":nth-child(2n - 1)").unwrap();
        assert_eq!(
            list.0[0].parts[0].1.components[0],
            Component::NthChild(2, -1)
        );

        // :nth-child(-2n + 3)
        let list = parse_selector_list(":nth-child(-2n + 3)").unwrap();
        assert_eq!(
            list.0[0].parts[0].1.components[0],
            Component::NthChild(-2, 3)
        );

        // :nth-child(+n)
        let list = parse_selector_list(":nth-child(+n)").unwrap();
        assert_eq!(
            list.0[0].parts[0].1.components[0],
            Component::NthChild(1, 0)
        );

        // :nth-child(+5)
        let list = parse_selector_list(":nth-child(+5)").unwrap();
        assert_eq!(
            list.0[0].parts[0].1.components[0],
            Component::NthChild(0, 5)
        );

        // :nth-child(-5)
        let list = parse_selector_list(":nth-child(-5)").unwrap();
        assert_eq!(
            list.0[0].parts[0].1.components[0],
            Component::NthChild(0, -5)
        );

        // :not(div.foo)
        let list = parse_selector_list(":not(div.foo)").unwrap();
        if let Component::Not(c) = &list.0[0].parts[0].1.components[0] {
            assert_eq!(c.components.len(), 2);
            assert_eq!(c.components[0], Component::Type("div".to_string()));
            assert_eq!(c.components[1], Component::Class("foo".to_string()));
        } else {
            panic!("Expected Not component");
        }

        // :lang(...)
        let list = parse_selector_list(":lang(en)").unwrap();
        assert_eq!(
            list.0[0].parts[0].1.components[0],
            Component::PseudoClass("lang(en)".to_string())
        );
        let list = parse_selector_list(":lang(en-US, \"fr\")").unwrap();
        assert_eq!(
            list.0[0].parts[0].1.components[0],
            Component::PseudoClass("lang(en-US,fr)".to_string())
        );

        // :first-child, :last-child
        let list = parse_selector_list(":first-child").unwrap();
        assert_eq!(list.0[0].parts[0].1.components[0], Component::FirstChild);
        let list = parse_selector_list(":last-child").unwrap();
        assert_eq!(list.0[0].parts[0].1.components[0], Component::LastChild);
    }

    #[test]
    fn test_parse_is_where() {
        // Simple :is
        let list = parse_selector_list(":is(h1, h2, .title)").unwrap();
        if let Component::Is(sub_list) = &list.0[0].parts[0].1.components[0] {
            assert_eq!(sub_list.0.len(), 3);
            assert_eq!(
                sub_list.0[0].parts[0].1.components[0],
                Component::Type("h1".to_string())
            );
            assert_eq!(
                sub_list.0[1].parts[0].1.components[0],
                Component::Type("h2".to_string())
            );
            assert_eq!(
                sub_list.0[2].parts[0].1.components[0],
                Component::Class("title".to_string())
            );
        } else {
            panic!("Expected Is component");
        }

        // Simple :where
        let list = parse_selector_list(":where(h1, h2, .title)").unwrap();
        if let Component::Where(sub_list) = &list.0[0].parts[0].1.components[0] {
            assert_eq!(sub_list.0.len(), 3);
            assert_eq!(
                sub_list.0[0].parts[0].1.components[0],
                Component::Type("h1".to_string())
            );
            assert_eq!(
                sub_list.0[1].parts[0].1.components[0],
                Component::Type("h2".to_string())
            );
            assert_eq!(
                sub_list.0[2].parts[0].1.components[0],
                Component::Class("title".to_string())
            );
        } else {
            panic!("Expected Where component");
        }

        // Nesting inside :not
        let list = parse_selector_list(":not(:is(div))").unwrap();
        if let Component::Not(c) = &list.0[0].parts[0].1.components[0] {
            if let Component::Is(sub_list) = &c.components[0] {
                assert_eq!(
                    sub_list.0[0].parts[0].1.components[0],
                    Component::Type("div".to_string())
                );
            } else {
                panic!("Expected Is component inside Not");
            }
        } else {
            panic!("Expected Not component");
        }

        // Forgiving selector list: invalid selectors ignored
        let list = parse_selector_list(":is(div, [attr=], p)").unwrap();
        if let Component::Is(sub_list) = &list.0[0].parts[0].1.components[0] {
            assert_eq!(sub_list.0.len(), 2);
            assert_eq!(
                sub_list.0[0].parts[0].1.components[0],
                Component::Type("div".to_string())
            );
            assert_eq!(
                sub_list.0[1].parts[0].1.components[0],
                Component::Type("p".to_string())
            );
        } else {
            panic!("Expected Is component");
        }

        // Forgiving selector list: empty selectors ignored
        let list = parse_selector_list(":is(a, , b)").unwrap();
        if let Component::Is(sub_list) = &list.0[0].parts[0].1.components[0] {
            assert_eq!(sub_list.0.len(), 2);
            assert_eq!(
                sub_list.0[0].parts[0].1.components[0],
                Component::Type("a".to_string())
            );
            assert_eq!(
                sub_list.0[1].parts[0].1.components[0],
                Component::Type("b".to_string())
            );
        } else {
            panic!("Expected Is component");
        }

        // Empty :is() and :where()
        let list1 = parse_selector_list(":is()").unwrap();
        if let Component::Is(sub_list) = &list1.0[0].parts[0].1.components[0] {
            assert!(sub_list.0.is_empty());
        } else {
            panic!("Expected Is component");
        }

        let list2 = parse_selector_list(":where()").unwrap();
        if let Component::Where(sub_list) = &list2.0[0].parts[0].1.components[0] {
            assert!(sub_list.0.is_empty());
        } else {
            panic!("Expected Where component");
        }
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
