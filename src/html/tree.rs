use crate::ascii::is_html_whitespace;
use crate::dom::{Dom, NodeData};
use crate::encoding::InputStream;
use crate::html::{Token, Tokenizer};
use crate::infra::NodeId;

/// Parses an HTML document from the given input stream.
// spec: https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-loop
pub fn parse_document(input: InputStream) -> Dom {
    let mut builder = TreeBuilder::new(input);
    builder.run();
    builder.dom
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InsertionMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    AfterHead,
    InBody,
    AfterBody,
    AfterAfterBody,
}

/// The tree builder state machine.
// spec: https://html.spec.whatwg.org/multipage/parsing.html#tree-construction
pub struct TreeBuilder {
    pub dom: Dom,
    tokenizer: Tokenizer,
    insertion_mode: InsertionMode,
    stack_of_open_elements: Vec<NodeId>,
    #[allow(dead_code)]
    head_element_pointer: Option<NodeId>,
}

impl TreeBuilder {
    fn new(input: InputStream) -> Self {
        Self {
            dom: Dom::new(),
            tokenizer: Tokenizer::new(input),
            insertion_mode: InsertionMode::Initial,
            stack_of_open_elements: Vec::new(),
            head_element_pointer: None,
        }
    }

    fn run(&mut self) {
        loop {
            let token = self.tokenizer.next_token();
            let is_eof = matches!(token, Token::Eof);
            self.process_token(token);
            if is_eof {
                break;
            }
        }
    }

    fn process_token(&mut self, token: Token) {
        match self.insertion_mode {
            InsertionMode::Initial => self.handle_initial(token),
            InsertionMode::BeforeHtml => self.handle_before_html(token),
            InsertionMode::BeforeHead => self.handle_before_head(token),
            InsertionMode::InHead => self.handle_in_head(token),
            InsertionMode::AfterHead => self.handle_after_head(token),
            InsertionMode::InBody => self.handle_in_body(token),
            InsertionMode::AfterBody => self.handle_after_body(token),
            InsertionMode::AfterAfterBody => self.handle_after_after_body(token),
        }
    }

    // spec: §13.2.6.4.1 The "initial" insertion mode
    fn handle_initial(&mut self, token: Token) {
        match token {
            Token::Character(c) if is_html_whitespace(c) => {
                // Ignore the token.
            }
            Token::Comment(data) => {
                let node = self.dom.create_node(NodeData::Comment(data));
                self.dom.append_child(self.dom.document(), node);
            }
            Token::Doctype {
                name,
                public_id,
                system_id,
                force_quirks: _,
            } => {
                // TODO(spec): handle quirks mode
                let node = self.dom.create_node(NodeData::Doctype {
                    name: name.unwrap_or_default(),
                    public_id: public_id.unwrap_or_default(),
                    system_id: system_id.unwrap_or_default(),
                });
                self.dom.append_child(self.dom.document(), node);
                self.insertion_mode = InsertionMode::BeforeHtml;
            }
            _ => {
                self.insertion_mode = InsertionMode::BeforeHtml;
                self.process_token(token);
            }
        }
    }

    // spec: §13.2.6.4.2 The "before html" insertion mode
    fn handle_before_html(&mut self, token: Token) {
        match token {
            Token::Doctype { .. } => {
                // Parse error. Ignore the token.
            }
            Token::Comment(data) => {
                let node = self.dom.create_node(NodeData::Comment(data));
                self.dom.append_child(self.dom.document(), node);
            }
            Token::Character(c) if is_html_whitespace(c) => {
                // Ignore the token.
            }
            Token::StartTag { name, attrs, .. } if name == "html" => {
                let node = self.create_and_insert_element(name, attrs);
                self.stack_of_open_elements.push(node);
                self.insertion_mode = InsertionMode::BeforeHead;
            }
            Token::EndTag { ref name, .. }
                if name != "head" && name != "body" && name != "html" && name != "br" =>
            {
                // Parse error. Ignore the token.
            }
            _ => {
                let node = self.create_and_insert_element("html".to_string(), Vec::new());
                self.stack_of_open_elements.push(node);
                self.insertion_mode = InsertionMode::BeforeHead;
                self.process_token(token);
            }
        }
    }

    // spec: §13.2.6.4.3 The "before head" insertion mode
    fn handle_before_head(&mut self, token: Token) {
        match token {
            Token::Character(c) if is_html_whitespace(c) => {
                // Ignore the token.
            }
            Token::Comment(data) => {
                self.insert_comment(data);
            }
            Token::Doctype { .. } => {
                // Parse error. Ignore the token.
            }
            Token::StartTag {
                ref name,
                ref attrs,
                ..
            } if name == "html" => {
                // Handle in "in body"
                let name = name.clone();
                let attrs = attrs.clone();
                self.handle_in_body(Token::StartTag {
                    name,
                    attrs,
                    self_closing: false,
                });
            }
            Token::StartTag { name, attrs, .. } if name == "head" => {
                let node = self.create_and_insert_element(name, attrs);
                self.head_element_pointer = Some(node);
                self.stack_of_open_elements.push(node);
                self.insertion_mode = InsertionMode::InHead;
            }
            Token::EndTag { ref name, .. }
                if name != "head" && name != "body" && name != "html" && name != "br" =>
            {
                // Parse error. Ignore the token.
            }
            _ => {
                let node = self.create_and_insert_element("head".to_string(), Vec::new());
                self.head_element_pointer = Some(node);
                self.stack_of_open_elements.push(node);
                self.insertion_mode = InsertionMode::InHead;
                self.process_token(token);
            }
        }
    }

    // spec: §13.2.6.4.4 The "in head" insertion mode
    fn handle_in_head(&mut self, token: Token) {
        match token {
            Token::Character(c) if is_html_whitespace(c) => {
                self.insert_character(c);
            }
            Token::Comment(data) => {
                self.insert_comment(data);
            }
            Token::Doctype { .. } => {
                // Parse error. Ignore the token.
            }
            Token::StartTag {
                ref name,
                ref attrs,
                ..
            } if name == "html" => {
                let name = name.clone();
                let attrs = attrs.clone();
                self.handle_in_body(Token::StartTag {
                    name,
                    attrs,
                    self_closing: false,
                });
            }
            // TODO(spec): base, basefont, bgsound, link, meta, title, noscript, style, script
            Token::EndTag { ref name, .. } if name == "head" => {
                self.stack_of_open_elements.pop();
                self.insertion_mode = InsertionMode::AfterHead;
            }
            Token::EndTag { ref name, .. } if name != "body" && name != "html" && name != "br" => {
                // Parse error. Ignore the token.
            }
            _ => {
                self.stack_of_open_elements.pop();
                self.insertion_mode = InsertionMode::AfterHead;
                self.process_token(token);
            }
        }
    }

    // spec: §13.2.6.4.6 The "after head" insertion mode
    fn handle_after_head(&mut self, token: Token) {
        match token {
            Token::Character(c) if is_html_whitespace(c) => {
                self.insert_character(c);
            }
            Token::Comment(data) => {
                self.insert_comment(data);
            }
            Token::Doctype { .. } => {
                // Parse error. Ignore the token.
            }
            Token::StartTag {
                ref name,
                ref attrs,
                ..
            } if name == "html" => {
                let name = name.clone();
                let attrs = attrs.clone();
                self.handle_in_body(Token::StartTag {
                    name,
                    attrs,
                    self_closing: false,
                });
            }
            Token::StartTag { name, attrs, .. } if name == "body" => {
                let node = self.create_and_insert_element(name, attrs);
                self.stack_of_open_elements.push(node);
                // TODO(spec): frameset-ok flag
                self.insertion_mode = InsertionMode::InBody;
            }
            // TODO(spec): frameset, base, basefont, bgsound, link, meta, noframes, script, style, template, title
            Token::EndTag { ref name, .. } if name != "body" && name != "html" && name != "br" => {
                // Parse error. Ignore the token.
            }
            _ => {
                let node = self.create_and_insert_element("body".to_string(), Vec::new());
                self.stack_of_open_elements.push(node);
                self.insertion_mode = InsertionMode::InBody;
                self.process_token(token);
            }
        }
    }

    // spec: §13.2.6.4.7 The "in body" insertion mode
    fn handle_in_body(&mut self, token: Token) {
        match token {
            Token::Character(c) => {
                // TODO(spec): handle null character
                self.insert_character(c);
            }
            Token::Comment(data) => {
                self.insert_comment(data);
            }
            Token::Doctype { .. } => {
                // Parse error. Ignore the token.
            }
            Token::StartTag { name, attrs, .. } => match name.as_str() {
                "p" => {
                    self.close_p_element_if_in_button_scope();
                    let node = self.create_and_insert_element(name, attrs);
                    self.stack_of_open_elements.push(node);
                }
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    self.close_p_element_if_in_button_scope();
                    // TODO(spec): if current node is h1-h6, parse error and pop
                    let node = self.create_and_insert_element(name, attrs);
                    self.stack_of_open_elements.push(node);
                }
                _ => {
                    let is_void = matches!(
                        name.as_str(),
                        "area"
                            | "base"
                            | "br"
                            | "col"
                            | "embed"
                            | "hr"
                            | "img"
                            | "input"
                            | "link"
                            | "meta"
                            | "param"
                            | "source"
                            | "track"
                            | "wbr"
                    );
                    let node = self.create_and_insert_element(name, attrs);
                    if !is_void {
                        self.stack_of_open_elements.push(node);
                    }
                }
            },
            Token::EndTag { name, .. } => {
                if name == "body" {
                    // TODO(spec): check if there is a body element in scope
                    self.insertion_mode = InsertionMode::AfterBody;
                } else if name == "html" {
                    // TODO(spec): check if there is a body element in scope
                    self.insertion_mode = InsertionMode::AfterBody;
                    self.process_token(Token::EndTag {
                        name: "html".to_string(),
                        attrs: Vec::new(),
                        self_closing: false,
                    });
                } else {
                    self.pop_until(&name);
                }
            }
            Token::Eof => {
                // Stop parsing.
            }
        }
    }

    // spec: §13.2.6.4.9 The "after body" insertion mode
    fn handle_after_body(&mut self, token: Token) {
        match token {
            Token::Character(c) if is_html_whitespace(c) => {
                self.handle_in_body(Token::Character(c));
            }
            Token::Comment(data) => {
                // Append a Comment node to the first element in the stack of open elements (the html element).
                if let Some(&html_node) = self.stack_of_open_elements.first() {
                    let node = self.dom.create_node(NodeData::Comment(data));
                    self.dom.append_child(html_node, node);
                }
            }
            Token::Doctype { .. } => {
                // Parse error. Ignore the token.
            }
            Token::StartTag {
                name,
                attrs,
                self_closing,
            } if name == "html" => {
                self.handle_in_body(Token::StartTag {
                    name,
                    attrs,
                    self_closing,
                });
            }
            Token::EndTag { ref name, .. } if name == "html" => {
                // TODO(spec): handle fragment case
                self.insertion_mode = InsertionMode::AfterAfterBody;
            }
            Token::Eof => {
                // Stop parsing.
            }
            _ => {
                // Parse error.
                self.insertion_mode = InsertionMode::InBody;
                self.process_token(token);
            }
        }
    }

    // spec: §13.2.6.4.11 The "after after body" insertion mode
    fn handle_after_after_body(&mut self, token: Token) {
        match token {
            Token::Comment(data) => {
                let node = self.dom.create_node(NodeData::Comment(data));
                self.dom.append_child(self.dom.document(), node);
            }
            Token::Doctype { .. }
            | Token::Character(_)
            | Token::StartTag { .. }
            | Token::EndTag { .. } => {
                // Parse error.
                self.insertion_mode = InsertionMode::InBody;
                self.process_token(token);
            }
            Token::Eof => {
                // Stop parsing.
            }
        }
    }

    fn close_p_element_if_in_button_scope(&mut self) {
        // Simplified: check if 'p' is in stack
        let found = self.stack_of_open_elements.iter().rev().any(
            |&id| matches!(self.dom.data(id), Some(NodeData::Element { name, .. }) if name == "p"),
        );
        if found {
            self.pop_until("p");
        }
    }

    fn pop_until(&mut self, target_name: &str) {
        while let Some(&top_id) = self.stack_of_open_elements.last() {
            if matches!(self.dom.data(top_id), Some(NodeData::Element { name, .. }) if name == target_name)
            {
                self.stack_of_open_elements.pop();
                break;
            }
            self.stack_of_open_elements.pop();
        }
    }

    // Helper: insert a comment into current node
    fn insert_comment(&mut self, data: String) {
        let node = self.dom.create_node(NodeData::Comment(data));
        let parent = self
            .stack_of_open_elements
            .last()
            .copied()
            .unwrap_or(self.dom.document());
        self.dom.append_child(parent, node);
    }

    // Helper: create and insert an element
    fn create_and_insert_element(&mut self, name: String, attrs: Vec<(String, String)>) -> NodeId {
        let node = self.dom.create_node(NodeData::Element { name, attrs });
        let parent = self
            .stack_of_open_elements
            .last()
            .copied()
            .unwrap_or(self.dom.document());
        self.dom.append_child(parent, node);
        node
    }

    // Helper: insert a character
    fn insert_character(&mut self, c: char) {
        let parent = self
            .stack_of_open_elements
            .last()
            .copied()
            .unwrap_or(self.dom.document());
        let node = self.dom.create_node(NodeData::Text(c.to_string()));
        self.dom.append_child(parent, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tree() {
        let html = "<html><head></head><body><p>hi</p></body></html>";
        let dom = parse_document(InputStream::from_utf8(html.as_bytes()));
        assert_eq!(
            dom.serialize(dom.document()),
            "<html><head></head><body><p>hi</p></body></html>"
        );
    }

    #[test]
    fn test_implicit_tags() {
        let html = "hi";
        let dom = parse_document(InputStream::from_utf8(html.as_bytes()));
        assert_eq!(
            dom.serialize(dom.document()),
            "<html><head></head><body>hi</body></html>"
        );
    }

    #[test]
    fn test_p_closing() {
        let html = "<p>One<p>Two";
        let dom = parse_document(InputStream::from_utf8(html.as_bytes()));
        assert_eq!(
            dom.serialize(dom.document()),
            "<html><head></head><body><p>One</p><p>Two</p></body></html>"
        );
    }

    #[test]
    fn test_h1_h6() {
        let html = "<h1>Title</h1><p>Para";
        let dom = parse_document(InputStream::from_utf8(html.as_bytes()));
        assert_eq!(
            dom.serialize(dom.document()),
            "<html><head></head><body><h1>Title</h1><p>Para</p></body></html>"
        );
    }

    #[test]
    fn test_void_elements() {
        let html = "<p>Line 1<br>Line 2<img>After";
        let dom = parse_document(InputStream::from_utf8(html.as_bytes()));
        assert_eq!(
            dom.serialize(dom.document()),
            "<html><head></head><body><p>Line 1<br>Line 2<img>After</p></body></html>"
        );
    }

    #[test]
    fn test_nested_tags() {
        let html = "<div><span><i>italic</i> normal</span></div>";
        let dom = parse_document(InputStream::from_utf8(html.as_bytes()));
        assert_eq!(
            dom.serialize(dom.document()),
            "<html><head></head><body><div><span><i>italic</i> normal</span></div></body></html>"
        );
    }
}
