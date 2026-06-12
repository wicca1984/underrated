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
#[allow(dead_code)]
enum InsertionMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    InHeadNoscript,
    AfterHead,
    InBody,
    Text,
    InTable,
    InTableText,
    InCaption,
    InColumnGroup,
    InTableBody,
    InRow,
    InCell,
    InSelect,
    InSelectInTable,
    InTemplate,
    AfterBody,
    InFrameset,
    AfterFrameset,
    AfterAfterBody,
    AfterAfterFrameset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FormattingElement {
    Node(NodeId),
    Marker,
}

/// The tree builder state machine.
// spec: https://html.spec.whatwg.org/multipage/parsing.html#tree-construction
pub struct TreeBuilder {
    pub dom: Dom,
    tokenizer: Tokenizer,
    insertion_mode: InsertionMode,
    stack_of_open_elements: Vec<NodeId>,
    list_of_active_formatting_elements: Vec<FormattingElement>,
    head_element_pointer: Option<NodeId>,
    template_insertion_modes: Vec<InsertionMode>,
    foster_parenting: bool,
}

impl TreeBuilder {
    fn new(input: InputStream) -> Self {
        Self {
            dom: Dom::new(),
            tokenizer: Tokenizer::new(input),
            insertion_mode: InsertionMode::Initial,
            stack_of_open_elements: Vec::new(),
            list_of_active_formatting_elements: Vec::new(),
            head_element_pointer: None,
            template_insertion_modes: Vec::new(),
            foster_parenting: false,
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
            InsertionMode::InHeadNoscript => self.handle_in_head_noscript(token),
            InsertionMode::AfterHead => self.handle_after_head(token),
            InsertionMode::InBody => self.handle_in_body(token),
            InsertionMode::Text => self.handle_text(token),
            InsertionMode::InTable => self.handle_in_table(token),
            InsertionMode::InTableText => self.handle_in_table(token), // TODO(spec)
            InsertionMode::InCaption => self.handle_in_caption(token),
            InsertionMode::InColumnGroup => self.handle_in_column_group(token),
            InsertionMode::InTableBody => self.handle_in_table_body(token),
            InsertionMode::InRow => self.handle_in_row(token),
            InsertionMode::InCell => self.handle_in_cell(token),
            InsertionMode::InSelect => self.handle_in_select(token),
            InsertionMode::InSelectInTable => self.handle_in_select_in_table(token),
            InsertionMode::InTemplate => self.handle_in_template(token),
            InsertionMode::AfterBody => self.handle_after_body(token),
            InsertionMode::InFrameset => self.handle_in_body(token), // TODO(spec)
            InsertionMode::AfterFrameset => self.handle_in_body(token), // TODO(spec)
            InsertionMode::AfterAfterBody => self.handle_after_after_body(token),
            InsertionMode::AfterAfterFrameset => self.handle_after_after_body(token), // TODO(spec)
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
            Token::StartTag { name, attrs, .. }
                if name == "base"
                    || name == "basefont"
                    || name == "bgsound"
                    || name == "link"
                    || name == "meta" =>
            {
                self.create_and_insert_element(name, attrs);
            }
            Token::StartTag { name, attrs, .. } if name == "title" => {
                let node = self.create_and_insert_element(name.clone(), attrs);
                self.stack_of_open_elements.push(node);
                self.tokenizer.set_initial_state("RCDATA state");
                self.tokenizer.set_last_start_tag(&name);
                self.insertion_mode = InsertionMode::Text;
            }
            Token::StartTag { name, attrs, .. } if name == "style" => {
                let node = self.create_and_insert_element(name.clone(), attrs);
                self.stack_of_open_elements.push(node);
                self.tokenizer.set_initial_state("RAWTEXT state");
                self.tokenizer.set_last_start_tag(&name);
                self.insertion_mode = InsertionMode::Text;
            }
            Token::StartTag { name, attrs, .. } if name == "noscript" => {
                let node = self.create_and_insert_element(name, attrs);
                self.stack_of_open_elements.push(node);
                self.insertion_mode = InsertionMode::InHeadNoscript;
            }
            Token::StartTag { name, attrs, .. } if name == "script" => {
                let node = self.create_and_insert_element(name.clone(), attrs);
                self.stack_of_open_elements.push(node);
                self.tokenizer.set_initial_state("Script data state");
                self.tokenizer.set_last_start_tag(&name);
                self.insertion_mode = InsertionMode::Text;
            }
            Token::StartTag { name, attrs, .. } if name == "template" => {
                let node = self.create_and_insert_element(name, attrs);
                self.stack_of_open_elements.push(node);
                self.list_of_active_formatting_elements
                    .push(FormattingElement::Marker);
                // TODO(spec): frameset-ok = false
                self.insertion_mode = InsertionMode::InTemplate;
                self.template_insertion_modes
                    .push(InsertionMode::InTemplate);
            }
            Token::EndTag { ref name, .. } if name == "template" => {
                if !self.stack_of_open_elements.iter().any(|&id| {
                    matches!(self.dom.data(id), Some(NodeData::Element { name, .. }) if name == "template")
                }) {
                    // Parse error.
                    return;
                }
                self.pop_until("template");
                self.clear_active_formatting_elements_to_marker();
                self.template_insertion_modes.pop();
                self.reset_insertion_mode_appropriately();
            }
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

    // spec: §13.2.6.4.5 The "in head noscript" insertion mode
    fn handle_in_head_noscript(&mut self, token: Token) {
        match token {
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
            Token::EndTag { ref name, .. } if name == "noscript" => {
                self.stack_of_open_elements.pop();
                self.insertion_mode = InsertionMode::InHead;
            }
            Token::Character(c) if is_html_whitespace(c) => {
                self.handle_in_head(Token::Character(c));
            }
            Token::Comment(data) => {
                self.handle_in_head(Token::Comment(data));
            }
            Token::StartTag {
                name,
                attrs,
                self_closing,
            } if name == "basefont"
                || name == "bgsound"
                || name == "link"
                || name == "meta"
                || name == "noframes"
                || name == "style" =>
            {
                self.handle_in_head(Token::StartTag {
                    name,
                    attrs,
                    self_closing,
                });
            }
            Token::EndTag {
                name,
                attrs,
                self_closing,
            } if name == "br" => {
                self.handle_in_head_noscript_anything_else(Token::EndTag {
                    name,
                    attrs,
                    self_closing,
                });
            }
            Token::StartTag { ref name, .. } if name == "head" || name == "noscript" => {
                // Parse error. Ignore the token.
            }
            Token::EndTag { .. } => {
                // Parse error. Ignore the token.
            }
            _ => {
                self.handle_in_head_noscript_anything_else(token);
            }
        }
    }

    fn handle_in_head_noscript_anything_else(&mut self, token: Token) {
        // Parse error.
        self.stack_of_open_elements.pop();
        self.insertion_mode = InsertionMode::InHead;
        self.process_token(token);
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
            Token::StartTag { name, attrs, .. } if name == "template" => {
                self.handle_in_head(Token::StartTag {
                    name,
                    attrs,
                    self_closing: false,
                });
            }
            Token::StartTag {
                name,
                attrs,
                self_closing,
            } if name == "base"
                || name == "basefont"
                || name == "bgsound"
                || name == "link"
                || name == "meta"
                || name == "noframes"
                || name == "script"
                || name == "style"
                || name == "title" =>
            {
                if let Some(head_id) = self.head_element_pointer {
                    self.stack_of_open_elements.push(head_id);
                    let old_mode = self.insertion_mode;
                    self.insertion_mode = InsertionMode::InHead;
                    self.process_token(Token::StartTag {
                        name,
                        attrs,
                        self_closing,
                    });
                    self.stack_of_open_elements.retain(|&id| id != head_id);
                    self.insertion_mode = old_mode;
                }
            }
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

    // spec: §13.2.6.4.8 The "text" insertion mode
    fn handle_text(&mut self, token: Token) {
        match token {
            Token::Character(c) => {
                self.insert_character(c);
            }
            Token::Eof => {
                // Parse error.
                self.stack_of_open_elements.pop();
                self.reset_insertion_mode_appropriately();
                self.process_token(token);
            }
            Token::EndTag { .. } => {
                self.stack_of_open_elements.pop();
                self.reset_insertion_mode_appropriately();
            }
            _ => {}
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
                "html" | "head" | "body" => {
                    // Parse error. Ignore the token.
                }
                "p" => {
                    self.close_p_element_if_in_button_scope();
                    let node = self.create_and_insert_element(name, attrs);
                    self.stack_of_open_elements.push(node);
                }
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    self.close_p_element_if_in_button_scope();
                    let node = self.create_and_insert_element(name, attrs);
                    self.stack_of_open_elements.push(node);
                }
                "address" | "article" | "aside" | "blockquote" | "details" | "dialog" | "dir"
                | "div" | "dl" | "fieldset" | "figcaption" | "figure" | "footer" | "header"
                | "hgroup" | "main" | "menu" | "nav" | "ol" | "pre" | "listing" | "section"
                | "summary" | "ul" | "form" => {
                    self.close_p_element_if_in_button_scope();
                    let node = self.create_and_insert_element(name, attrs);
                    self.stack_of_open_elements.push(node);
                }
                "li" => {
                    // spec: https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inbody
                    for &id in self.stack_of_open_elements.iter().rev() {
                        if let Some(NodeData::Element { name, .. }) = self.dom.data(id) {
                            if name == "li" {
                                self.pop_until("li");
                                break;
                            }
                            if self.is_special_element(name)
                                && name != "address"
                                && name != "div"
                                && name != "p"
                            {
                                break;
                            }
                        }
                    }
                    self.close_p_element_if_in_button_scope();
                    let node = self.create_and_insert_element(name, attrs);
                    self.stack_of_open_elements.push(node);
                }
                "dd" | "dt" => {
                    // spec: https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inbody
                    for &id in self.stack_of_open_elements.iter().rev() {
                        if let Some(NodeData::Element { name, .. }) = self.dom.data(id) {
                            if name == "dd" || name == "dt" {
                                let name_cloned = name.clone();
                                self.pop_until(&name_cloned);
                                break;
                            }
                            if self.is_special_element(name)
                                && name != "address"
                                && name != "div"
                                && name != "p"
                            {
                                break;
                            }
                        }
                    }
                    self.close_p_element_if_in_button_scope();
                    let node = self.create_and_insert_element(name, attrs);
                    self.stack_of_open_elements.push(node);
                }
                "option" => {
                    // spec: https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inbody
                    if self.stack_of_open_elements.last().is_some_and(|&top_id| {
                        matches!(self.dom.data(top_id), Some(NodeData::Element { name, .. }) if name == "option")
                    }) {
                        self.pop_until("option");
                    }
                    let node = self.create_and_insert_element(name, attrs);
                    self.stack_of_open_elements.push(node);
                }
                "optgroup" => {
                    // spec: https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inbody
                    if self.stack_of_open_elements.last().is_some_and(|&top_id| {
                        matches!(self.dom.data(top_id), Some(NodeData::Element { name, .. }) if name == "option")
                    }) {
                        self.pop_until("option");
                    }
                    if self.stack_of_open_elements.last().is_some_and(|&top_id| {
                        matches!(self.dom.data(top_id), Some(NodeData::Element { name, .. }) if name == "optgroup")
                    }) {
                        self.pop_until("optgroup");
                    }
                    let node = self.create_and_insert_element(name, attrs);
                    self.stack_of_open_elements.push(node);
                }
                "select" => {
                    self.reconstruct_active_formatting_elements();
                    let node = self.create_and_insert_element(name, attrs);
                    self.stack_of_open_elements.push(node);
                    // TODO(spec): set frameset-ok to not ok
                    if matches!(
                        self.insertion_mode,
                        InsertionMode::InTable
                            | InsertionMode::InCaption
                            | InsertionMode::InTableBody
                            | InsertionMode::InRow
                            | InsertionMode::InCell
                    ) {
                        self.insertion_mode = InsertionMode::InSelectInTable;
                    } else {
                        self.insertion_mode = InsertionMode::InSelect;
                    }
                }
                "a" => {
                    // Adoption Agency Algorithm (subset)
                    if let Some(formatting_node_id) = self.get_formatting_element_id("a") {
                        self.run_adoption_agency_algorithm("a");
                        // After AAA, the element might have been removed from formatting list
                        self.list_of_active_formatting_elements
                            .retain(|&e| match e {
                                FormattingElement::Node(id) => id != formatting_node_id,
                                _ => true,
                            });
                        self.stack_of_open_elements
                            .retain(|&id| id != formatting_node_id);
                    }
                    self.reconstruct_active_formatting_elements();
                    let node = self.create_and_insert_element(name, attrs);
                    self.push_formatting_element(node);
                    self.stack_of_open_elements.push(node);
                }
                "b" | "big" | "code" | "em" | "font" | "i" | "nobr" | "s" | "small" | "strike"
                | "strong" | "tt" | "u" => {
                    self.reconstruct_active_formatting_elements();
                    let node = self.create_and_insert_element(name, attrs);
                    self.push_formatting_element(node);
                    self.stack_of_open_elements.push(node);
                }
                "table" => {
                    // TODO(spec): quirks mode
                    self.close_p_element_if_in_button_scope();
                    let node = self.create_and_insert_element(name, attrs);
                    self.stack_of_open_elements.push(node);
                    self.insertion_mode = InsertionMode::InTable;
                }
                "area" | "br" | "embed" | "img" | "keygen" | "wbr" => {
                    self.reconstruct_active_formatting_elements();
                    self.create_and_insert_element(name, attrs);
                    // void elements, don't push to stack
                }
                "input" => {
                    self.reconstruct_active_formatting_elements();
                    self.create_and_insert_element(name, attrs);
                    // TODO(spec): frameset-ok = false
                }
                "param" | "source" | "track" => {
                    self.create_and_insert_element(name, attrs);
                }
                "hr" => {
                    self.close_p_element_if_in_button_scope();
                    self.create_and_insert_element(name, attrs);
                }
                "template" => {
                    self.handle_in_head(Token::StartTag {
                        name,
                        attrs,
                        self_closing: false,
                    });
                }
                _ => {
                    self.reconstruct_active_formatting_elements();
                    let node = self.create_and_insert_element(name.clone(), attrs);
                    if !self.is_void_element(&name) {
                        self.stack_of_open_elements.push(node);
                    }
                }
            },
            Token::EndTag { name, .. } => {
                if name == "body" {
                    if self.is_in_scope("body") {
                        self.insertion_mode = InsertionMode::AfterBody;
                    }
                } else if name == "html" {
                    if self.is_in_scope("body") {
                        self.insertion_mode = InsertionMode::AfterBody;
                        self.process_token(Token::EndTag {
                            name: "html".to_string(),
                            attrs: Vec::new(),
                            self_closing: false,
                        });
                    }
                } else if self.is_formatting_element(&name) {
                    self.run_adoption_agency_algorithm(&name);
                } else if self.is_special_element(&name) {
                    if self.is_in_scope(&name) {
                        self.pop_until(&name);
                    }
                } else {
                    // TODO(spec): proper end tag handling
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

    // spec: §13.2.6.4.13 The "in column group" insertion mode
    fn handle_in_column_group(&mut self, token: Token) {
        match token {
            Token::Character(c) if is_html_whitespace(c) => {
                self.insert_character(c);
            }
            Token::Comment(data) => {
                self.insert_comment(data);
            }
            Token::Doctype { .. } => {
                // Parse error. Ignore.
            }
            Token::StartTag {
                name,
                attrs,
                self_closing,
            } => match name.as_str() {
                "html" => {
                    self.handle_in_body(Token::StartTag {
                        name,
                        attrs,
                        self_closing,
                    });
                }
                "col" => {
                    let node = self.create_and_insert_element(name, attrs);
                    self.stack_of_open_elements.push(node);
                    self.stack_of_open_elements.pop();
                }
                "template" => {
                    self.handle_in_head(Token::StartTag {
                        name,
                        attrs,
                        self_closing,
                    });
                }
                _ => {
                    self.handle_in_column_group_anything_else(Token::StartTag {
                        name,
                        attrs,
                        self_closing,
                    });
                }
            },
            Token::EndTag {
                name,
                attrs,
                self_closing,
            } => match name.as_str() {
                "colgroup" => {
                    let is_colgroup = if let Some(&top_id) = self.stack_of_open_elements.last() {
                        matches!(self.dom.data(top_id), Some(NodeData::Element { name, .. }) if name == "colgroup")
                    } else {
                        false
                    };
                    if !is_colgroup {
                        // Parse error. Ignore.
                    } else {
                        self.stack_of_open_elements.pop();
                        self.insertion_mode = InsertionMode::InTable;
                    }
                }
                "col" => {
                    // An end tag "col": parse error; ignore.
                }
                "template" => {
                    self.handle_in_head(Token::EndTag {
                        name,
                        attrs,
                        self_closing,
                    });
                }
                _ => {
                    self.handle_in_column_group_anything_else(Token::EndTag {
                        name,
                        attrs,
                        self_closing,
                    });
                }
            },
            Token::Eof => {
                self.handle_in_body(Token::Eof);
            }
            other_token => {
                self.handle_in_column_group_anything_else(other_token);
            }
        }
    }

    fn handle_in_column_group_anything_else(&mut self, token: Token) {
        let is_colgroup = if let Some(&top_id) = self.stack_of_open_elements.last() {
            matches!(self.dom.data(top_id), Some(NodeData::Element { name, .. }) if name == "colgroup")
        } else {
            false
        };
        if !is_colgroup {
            // Parse error. Ignore.
        } else {
            self.stack_of_open_elements.pop();
            self.insertion_mode = InsertionMode::InTable;
            self.process_token(token);
        }
    }

    // spec: §13.2.6.4.12 The "in table" insertion mode
    fn handle_in_table(&mut self, token: Token) {
        match token {
            Token::StartTag {
                ref name,
                ref attrs,
                ..
            } => match name.as_str() {
                "caption" => {
                    self.clear_stack_back_to_table_context();
                    self.list_of_active_formatting_elements
                        .push(FormattingElement::Marker);
                    let node = self.create_and_insert_element(name.clone(), attrs.clone());
                    self.stack_of_open_elements.push(node);
                    self.insertion_mode = InsertionMode::InCaption;
                }
                "colgroup" => {
                    self.clear_stack_back_to_table_context();
                    let node = self.create_and_insert_element(name.clone(), attrs.clone());
                    self.stack_of_open_elements.push(node);
                    self.insertion_mode = InsertionMode::InColumnGroup;
                }
                "col" => {
                    self.handle_in_table(Token::StartTag {
                        name: "colgroup".to_string(),
                        attrs: Vec::new(),
                        self_closing: false,
                    });
                    self.process_token(token);
                }
                "tbody" | "tfoot" | "thead" => {
                    self.clear_stack_back_to_table_context();
                    let node = self.create_and_insert_element(name.clone(), attrs.clone());
                    self.stack_of_open_elements.push(node);
                    self.insertion_mode = InsertionMode::InTableBody;
                }
                "td" | "th" | "tr" => {
                    self.handle_in_table(Token::StartTag {
                        name: "tbody".to_string(),
                        attrs: Vec::new(),
                        self_closing: false,
                    });
                    self.process_token(token);
                }
                "table" => {
                    if self.is_in_table_scope("table") {
                        self.pop_until("table");
                        self.reset_insertion_mode_appropriately();
                        self.process_token(token);
                    }
                }
                "template" => {
                    self.handle_in_head(token);
                }
                _ => {
                    self.foster_parenting = true;
                    self.handle_in_body(token);
                    self.foster_parenting = false;
                }
            },
            Token::EndTag { ref name, .. } => match name.as_str() {
                "table" => {
                    if self.is_in_table_scope("table") {
                        self.pop_until("table");
                        self.reset_insertion_mode_appropriately();
                    }
                }
                "body" | "caption" | "col" | "colgroup" | "html" | "tbody" | "td" | "tfoot"
                | "th" | "thead" | "tr" => {
                    // Parse error.
                }
                "template" => {
                    self.handle_in_head(token);
                }
                _ => {
                    self.foster_parenting = true;
                    self.handle_in_body(token);
                    self.foster_parenting = false;
                }
            },
            Token::Character(_) => {
                self.foster_parenting = true;
                self.handle_in_body(token);
                self.foster_parenting = false;
            }
            Token::Eof => {
                self.handle_in_body(token);
            }
            _ => {
                self.foster_parenting = true;
                self.handle_in_body(token);
                self.foster_parenting = false;
            }
        }
    }

    // spec: §13.2.6.4.14 The "in caption" insertion mode
    fn handle_in_caption(&mut self, token: Token) {
        match token {
            Token::EndTag { ref name, .. } if name == "caption" => {
                if self.is_in_table_scope("caption") {
                    self.pop_until("caption");
                    self.clear_active_formatting_elements_to_marker();
                    self.insertion_mode = InsertionMode::InTable;
                }
            }
            Token::StartTag { ref name, .. }
                if matches!(
                    name.as_str(),
                    "caption"
                        | "col"
                        | "colgroup"
                        | "tbody"
                        | "td"
                        | "tfoot"
                        | "th"
                        | "thead"
                        | "tr"
                ) =>
            {
                if self.is_in_table_scope("caption") {
                    self.pop_until("caption");
                    self.clear_active_formatting_elements_to_marker();
                    self.insertion_mode = InsertionMode::InTable;
                    self.process_token(token);
                }
            }
            Token::EndTag { ref name, .. } if name == "table" => {
                if self.is_in_table_scope("caption") {
                    self.pop_until("caption");
                    self.clear_active_formatting_elements_to_marker();
                    self.insertion_mode = InsertionMode::InTable;
                    self.process_token(token);
                }
            }
            _ => self.handle_in_body(token),
        }
    }

    // spec: §13.2.6.4.16 The "in table body" insertion mode
    fn handle_in_table_body(&mut self, token: Token) {
        match token {
            Token::StartTag {
                ref name,
                ref attrs,
                ..
            } if name == "tr" => {
                self.clear_stack_back_to_table_body_context();
                let node = self.create_and_insert_element(name.clone(), attrs.clone());
                self.stack_of_open_elements.push(node);
                self.insertion_mode = InsertionMode::InRow;
            }
            Token::StartTag { ref name, .. } if name == "th" || name == "td" => {
                self.handle_in_table_body(Token::StartTag {
                    name: "tr".to_string(),
                    attrs: Vec::new(),
                    self_closing: false,
                });
                self.process_token(token);
            }
            Token::EndTag { ref name, .. }
                if matches!(name.as_str(), "tbody" | "tfoot" | "thead") =>
            {
                if self.is_in_table_scope(name) {
                    self.clear_stack_back_to_table_body_context();
                    self.stack_of_open_elements.pop();
                    self.insertion_mode = InsertionMode::InTable;
                }
            }
            Token::StartTag { ref name, .. }
                if matches!(
                    name.as_str(),
                    "caption" | "col" | "colgroup" | "tbody" | "tfoot" | "thead"
                ) =>
            {
                if self.is_in_table_scope("tbody")
                    || self.is_in_table_scope("thead")
                    || self.is_in_table_scope("tfoot")
                {
                    self.clear_stack_back_to_table_body_context();
                    self.stack_of_open_elements.pop();
                    self.insertion_mode = InsertionMode::InTable;
                    self.process_token(token);
                }
            }
            Token::EndTag { ref name, .. } if name == "table" => {
                if self.is_in_table_scope("tbody")
                    || self.is_in_table_scope("thead")
                    || self.is_in_table_scope("tfoot")
                {
                    self.clear_stack_back_to_table_body_context();
                    self.stack_of_open_elements.pop();
                    self.insertion_mode = InsertionMode::InTable;
                    self.process_token(token);
                }
            }
            _ => self.handle_in_table(token),
        }
    }

    // spec: §13.2.6.4.17 The "in row" insertion mode
    fn handle_in_row(&mut self, token: Token) {
        match token {
            Token::StartTag {
                ref name,
                ref attrs,
                ..
            } if name == "th" || name == "td" => {
                self.clear_stack_back_to_table_row_context();
                let node = self.create_and_insert_element(name.clone(), attrs.clone());
                self.stack_of_open_elements.push(node);
                self.insertion_mode = InsertionMode::InCell;
                self.list_of_active_formatting_elements
                    .push(FormattingElement::Marker);
            }
            Token::EndTag { ref name, .. } if name == "tr" => {
                if self.is_in_table_scope("tr") {
                    self.clear_stack_back_to_table_row_context();
                    self.stack_of_open_elements.pop();
                    self.insertion_mode = InsertionMode::InTableBody;
                }
            }
            Token::StartTag { ref name, .. }
                if matches!(
                    name.as_str(),
                    "caption" | "col" | "colgroup" | "tbody" | "tfoot" | "thead" | "tr"
                ) =>
            {
                if self.is_in_table_scope("tr") {
                    self.clear_stack_back_to_table_row_context();
                    self.stack_of_open_elements.pop();
                    self.insertion_mode = InsertionMode::InTableBody;
                    self.process_token(token);
                }
            }
            Token::EndTag { ref name, .. } if name == "table" => {
                if self.is_in_table_scope("tr") {
                    self.clear_stack_back_to_table_row_context();
                    self.stack_of_open_elements.pop();
                    self.insertion_mode = InsertionMode::InTableBody;
                    self.process_token(token);
                }
            }
            _ => self.handle_in_table(token),
        }
    }

    // spec: §13.2.6.4.18 The "in cell" insertion mode
    fn handle_in_cell(&mut self, token: Token) {
        match token {
            Token::EndTag { ref name, .. } if name == "td" || name == "th" => {
                if self.is_in_table_scope(name) {
                    self.close_cell();
                    self.process_token(token);
                }
            }
            Token::StartTag { ref name, .. }
                if matches!(
                    name.as_str(),
                    "caption"
                        | "col"
                        | "colgroup"
                        | "tbody"
                        | "td"
                        | "tfoot"
                        | "th"
                        | "thead"
                        | "tr"
                ) =>
            {
                if self.is_in_table_scope("td") || self.is_in_table_scope("th") {
                    self.close_cell();
                    self.process_token(token);
                }
            }
            _ => self.handle_in_body(token),
        }
    }

    fn close_cell(&mut self) {
        if self.is_in_table_scope("td") {
            self.pop_until("td");
        } else {
            self.pop_until("th");
        }
        self.clear_active_formatting_elements_to_marker();
        self.insertion_mode = InsertionMode::InRow;
    }

    // spec: §13.2.6.4.19 The "in template" insertion mode
    fn handle_in_template(&mut self, token: Token) {
        match token {
            Token::StartTag { ref name, .. } if name == "template" => {
                self.handle_in_head(token);
            }
            Token::EndTag { ref name, .. } if name == "template" => {
                self.handle_in_head(token);
            }
            Token::Eof => {
                if !self.stack_of_open_elements.iter().any(|&id| {
                    matches!(self.dom.data(id), Some(NodeData::Element { name, .. }) if name == "template")
                }) {
                    return;
                }
                self.pop_until("template");
                self.clear_active_formatting_elements_to_marker();
                self.template_insertion_modes.pop();
                self.reset_insertion_mode_appropriately();
                self.process_token(token);
            }
            _ => {
                // Simplified template handling: push a new mode if needed
                // For now, just forward to in body or whatever was appropriate
                self.handle_in_body(token);
            }
        }
    }

    // spec: §13.2.6.4.20 The "in select" insertion mode
    fn handle_in_select(&mut self, token: Token) {
        match token {
            Token::Character(c) => {
                if c == '\0' {
                    // Parse error. Ignore.
                } else {
                    self.insert_character(c);
                }
            }
            Token::Comment(data) => {
                self.insert_comment(data);
            }
            Token::Doctype { .. } => {
                // Parse error. Ignore.
            }
            Token::StartTag {
                name,
                attrs,
                self_closing,
            } => match name.as_str() {
                "html" => {
                    self.handle_in_body(Token::StartTag {
                        name,
                        attrs,
                        self_closing,
                    });
                }
                "option" => {
                    if self.stack_of_open_elements.last().is_some_and(|&top_id| {
                        matches!(self.dom.data(top_id), Some(NodeData::Element { name, .. }) if name == "option")
                    }) {
                        self.stack_of_open_elements.pop();
                    }
                    let node = self.create_and_insert_element(name, attrs);
                    self.stack_of_open_elements.push(node);
                }
                "optgroup" => {
                    if self.stack_of_open_elements.last().is_some_and(|&top_id| {
                        matches!(self.dom.data(top_id), Some(NodeData::Element { name, .. }) if name == "option")
                    }) {
                        self.stack_of_open_elements.pop();
                    }
                    if self.stack_of_open_elements.last().is_some_and(|&top_id| {
                        matches!(self.dom.data(top_id), Some(NodeData::Element { name, .. }) if name == "optgroup")
                    }) {
                        self.stack_of_open_elements.pop();
                    }
                    let node = self.create_and_insert_element(name, attrs);
                    self.stack_of_open_elements.push(node);
                }
                "select" if self.is_in_select_scope("select") => {
                    self.pop_until("select");
                    self.reset_insertion_mode_appropriately();
                }
                _ => {
                    // Parse error. Ignore.
                }
            },
            Token::EndTag { name, .. } => match name.as_str() {
                "option" => {
                    if self.stack_of_open_elements.last().is_some_and(|&top_id| {
                        matches!(self.dom.data(top_id), Some(NodeData::Element { name, .. }) if name == "option")
                    }) {
                        self.stack_of_open_elements.pop();
                    }
                }
                "optgroup" => {
                    let len = self.stack_of_open_elements.len();
                    if len >= 2 {
                        let is_top_option = matches!(self.dom.data(self.stack_of_open_elements[len - 1]), Some(NodeData::Element { name, .. }) if name == "option");
                        let is_prev_optgroup = matches!(self.dom.data(self.stack_of_open_elements[len - 2]), Some(NodeData::Element { name, .. }) if name == "optgroup");
                        if is_top_option && is_prev_optgroup {
                            self.stack_of_open_elements.pop();
                        }
                    }
                    if self.stack_of_open_elements.last().is_some_and(|&top_id| {
                        matches!(self.dom.data(top_id), Some(NodeData::Element { name, .. }) if name == "optgroup")
                    }) {
                        self.stack_of_open_elements.pop();
                    }
                }
                "select" if self.is_in_select_scope("select") => {
                    self.pop_until("select");
                    self.reset_insertion_mode_appropriately();
                }
                _ => {
                    // Parse error. Ignore.
                }
            },
            Token::Eof => {
                let is_html = if let Some(&top_id) = self.stack_of_open_elements.last() {
                    matches!(self.dom.data(top_id), Some(NodeData::Element { name, .. }) if name == "html")
                } else {
                    false
                };
                if !is_html {
                    // Parse error.
                }
                // Stop parsing.
            }
        }
    }

    // spec: §13.2.6.4.21 The "in select in table" insertion mode
    fn handle_in_select_in_table(&mut self, token: Token) {
        match token {
            Token::StartTag { ref name, .. }
                if matches!(
                    name.as_str(),
                    "caption" | "table" | "tbody" | "tfoot" | "thead" | "tr" | "td" | "th"
                ) =>
            {
                // Parse error.
                if self.is_in_select_scope("select") {
                    self.pop_until("select");
                    self.reset_insertion_mode_appropriately();
                    self.process_token(token);
                }
            }
            Token::EndTag { ref name, .. }
                if matches!(
                    name.as_str(),
                    "caption" | "table" | "tbody" | "tfoot" | "thead" | "tr" | "td" | "th"
                ) =>
            {
                // Parse error.
                if self.is_in_table_scope(name) && self.is_in_select_scope("select") {
                    self.pop_until("select");
                    self.reset_insertion_mode_appropriately();
                    self.process_token(token);
                }
            }
            _ => self.handle_in_select(token),
        }
    }

    fn is_special_element(&self, name: &str) -> bool {
        matches!(
            name,
            "address"
                | "applet"
                | "area"
                | "article"
                | "aside"
                | "base"
                | "basefont"
                | "bgsound"
                | "blockquote"
                | "body"
                | "br"
                | "button"
                | "caption"
                | "center"
                | "col"
                | "colgroup"
                | "dd"
                | "details"
                | "dir"
                | "div"
                | "dl"
                | "dt"
                | "embed"
                | "fieldset"
                | "figcaption"
                | "figure"
                | "footer"
                | "form"
                | "frame"
                | "frameset"
                | "h1"
                | "h2"
                | "h3"
                | "h4"
                | "h5"
                | "h6"
                | "head"
                | "header"
                | "hgroup"
                | "hr"
                | "html"
                | "iframe"
                | "img"
                | "input"
                | "keygen"
                | "li"
                | "link"
                | "listing"
                | "main"
                | "marquee"
                | "menu"
                | "meta"
                | "nav"
                | "noembed"
                | "noframes"
                | "noscript"
                | "object"
                | "ol"
                | "p"
                | "param"
                | "plaintext"
                | "pre"
                | "script"
                | "section"
                | "select"
                | "source"
                | "style"
                | "summary"
                | "table"
                | "tbody"
                | "td"
                | "template"
                | "textarea"
                | "tfoot"
                | "th"
                | "thead"
                | "title"
                | "tr"
                | "track"
                | "ul"
                | "wbr"
                | "xmp"
        )
    }

    fn close_p_element_if_in_button_scope(&mut self) {
        // spec: https://html.spec.whatwg.org/multipage/parsing.html#close-a-p-element
        if self.is_in_button_scope("p") {
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
        let parent = self.get_appropriate_place_for_inserting_node();
        self.dom.append_child(parent, node);
        node
    }

    fn run_adoption_agency_algorithm(&mut self, name: &str) {
        // TODO(spec): full implementation of AAA.
        // For now, this is a very minimal subset to pass simple tests.
        if self.is_in_scope(name) {
            self.pop_until(name);
        }
    }

    fn get_formatting_element_id(&self, name: &str) -> Option<NodeId> {
        for &item in self.list_of_active_formatting_elements.iter().rev() {
            match item {
                FormattingElement::Marker => break,
                FormattingElement::Node(id) => {
                    if matches!(self.dom.data(id), Some(NodeData::Element { name: n, .. }) if n == name)
                    {
                        return Some(id);
                    }
                }
            }
        }
        None
    }

    fn is_void_element(&self, name: &str) -> bool {
        matches!(
            name,
            "area"
                | "base"
                | "br"
                | "col"
                | "embed"
                | "hr"
                | "img"
                | "input"
                | "keygen"
                | "link"
                | "meta"
                | "param"
                | "source"
                | "track"
                | "wbr"
        )
    }

    // Helper: insert a character
    // spec: §13.2.6.1 Creating and inserting nodes - insert a character step
    // If the node's last child is already a Text node, append the character to that
    // existing text node's data; otherwise create a new Text node.
    fn insert_character(&mut self, c: char) {
        let parent = self.get_appropriate_place_for_inserting_node();
        if let Some(&last_child) = self.dom.children(parent).last()
            && let Some(NodeData::Text(s)) = self.dom.data(last_child)
        {
            let mut s = s.clone();
            s.push(c);
            self.dom.set_text(last_child, &s);
            return;
        }
        let node = self.dom.create_node(NodeData::Text(c.to_string()));
        self.dom.append_child(parent, node);
    }

    fn get_appropriate_place_for_inserting_node(&self) -> NodeId {
        let target = self
            .stack_of_open_elements
            .last()
            .copied()
            .unwrap_or(self.dom.document());

        if self.foster_parenting
            && matches!(self.dom.data(target), Some(NodeData::Element { name, .. }) if name == "table" || name == "tbody" || name == "tfoot" || name == "thead" || name == "tr")
        {
            // TODO(spec): proper foster parenting search
            // For now, look for the last table element in the stack
            for &node_id in self.stack_of_open_elements.iter().rev() {
                if matches!(self.dom.data(node_id), Some(NodeData::Element { name, .. }) if name == "table")
                    && let Some(parent) = self.dom.parent(node_id)
                {
                    return parent;
                }
            }
        }

        target
    }

    fn is_in_scope(&self, target_name: &str) -> bool {
        self.is_in_specific_scope(
            target_name,
            &[
                "applet", "caption", "html", "table", "td", "th", "marquee", "object", "template",
            ],
        )
    }

    #[allow(dead_code)]
    fn is_in_button_scope(&self, target_name: &str) -> bool {
        self.is_in_specific_scope(
            target_name,
            &[
                "applet", "caption", "html", "table", "td", "th", "marquee", "object", "template",
                "button",
            ],
        )
    }

    fn is_in_table_scope(&self, target_name: &str) -> bool {
        self.is_in_specific_scope(target_name, &["html", "table", "template"])
    }

    fn is_in_select_scope(&self, target_name: &str) -> bool {
        for &node_id in self.stack_of_open_elements.iter().rev() {
            if let Some(NodeData::Element { name, .. }) = self.dom.data(node_id) {
                if name == target_name {
                    return true;
                }
                if name != "optgroup" && name != "option" {
                    return false;
                }
            }
        }
        false
    }

    fn is_in_specific_scope(&self, target_name: &str, list: &[&str]) -> bool {
        for &node_id in self.stack_of_open_elements.iter().rev() {
            if let Some(NodeData::Element { name, .. }) = self.dom.data(node_id) {
                if name == target_name {
                    return true;
                }
                if list.contains(&name.as_str()) {
                    return false;
                }
            }
        }
        false
    }

    fn push_formatting_element(&mut self, node: NodeId) {
        // TODO(spec): "Noah's Ark" clause
        self.list_of_active_formatting_elements
            .push(FormattingElement::Node(node));
    }

    fn reconstruct_active_formatting_elements(&mut self) {
        if self.list_of_active_formatting_elements.is_empty() {
            return;
        }

        let last_idx = self.list_of_active_formatting_elements.len() - 1;
        if !matches!(
            self.list_of_active_formatting_elements[last_idx],
            FormattingElement::Marker
        ) {
            // TODO(spec): full reconstruction algorithm
        }
    }

    fn clear_active_formatting_elements_to_marker(&mut self) {
        while let Some(item) = self.list_of_active_formatting_elements.pop() {
            if matches!(item, FormattingElement::Marker) {
                break;
            }
        }
    }

    fn is_formatting_element(&self, name: &str) -> bool {
        matches!(
            name,
            "a" | "b"
                | "big"
                | "code"
                | "em"
                | "font"
                | "i"
                | "nobr"
                | "s"
                | "small"
                | "strike"
                | "strong"
                | "tt"
                | "u"
        )
    }

    fn reset_insertion_mode_appropriately(&mut self) {
        // The spec guarantees a non-empty stack here, but guard so an empty stack
        // cannot underflow `len() - 1` (usize wrap → panic) on crafted input (I-6).
        if self.stack_of_open_elements.is_empty() {
            return;
        }
        let mut last = false;
        let mut node_idx = self.stack_of_open_elements.len() - 1;

        loop {
            let node_id = self.stack_of_open_elements[node_idx];
            if node_idx == 0 {
                last = true;
                // TODO(spec): fragment case
            }

            if let Some(NodeData::Element { name, .. }) = self.dom.data(node_id) {
                match name.as_str() {
                    "select" => {
                        if last {
                            self.insertion_mode = InsertionMode::InSelect;
                        } else {
                            let mut ancestor_idx = node_idx;
                            let mut found_table = false;
                            while ancestor_idx > 0 {
                                ancestor_idx -= 1;
                                let ancestor_id = self.stack_of_open_elements[ancestor_idx];
                                if let Some(NodeData::Element { name: a_name, .. }) =
                                    self.dom.data(ancestor_id)
                                {
                                    if a_name == "template" {
                                        break;
                                    }
                                    if a_name == "table" {
                                        found_table = true;
                                        break;
                                    }
                                }
                            }
                            if found_table {
                                self.insertion_mode = InsertionMode::InSelectInTable;
                            } else {
                                self.insertion_mode = InsertionMode::InSelect;
                            }
                        }
                    }
                    "template" => {
                        self.insertion_mode = *self
                            .template_insertion_modes
                            .last()
                            .unwrap_or(&InsertionMode::InTemplate);
                    }
                    "td" | "th" if !last => {
                        self.insertion_mode = InsertionMode::InCell;
                    }
                    "tr" => {
                        self.insertion_mode = InsertionMode::InRow;
                    }
                    "tbody" | "thead" | "tfoot" => {
                        self.insertion_mode = InsertionMode::InTableBody;
                    }
                    "caption" => {
                        self.insertion_mode = InsertionMode::InCaption;
                    }
                    "colgroup" => {
                        self.insertion_mode = InsertionMode::InColumnGroup;
                    }
                    "table" => {
                        self.insertion_mode = InsertionMode::InTable;
                    }
                    "head" if !last => {
                        self.insertion_mode = InsertionMode::InHead;
                    }
                    "body" => {
                        self.insertion_mode = InsertionMode::InBody;
                    }
                    "frameset" => {
                        self.insertion_mode = InsertionMode::InFrameset;
                    }
                    "html" => {
                        // TODO(spec): head element pointer
                        self.insertion_mode = InsertionMode::BeforeHead;
                    }
                    _ if last => {
                        self.insertion_mode = InsertionMode::InBody;
                    }
                    _ => {
                        node_idx -= 1;
                        continue;
                    }
                }
            }
            break;
        }
    }

    fn clear_stack_back_to_table_context(&mut self) {
        while let Some(&id) = self.stack_of_open_elements.last() {
            if matches!(self.dom.data(id), Some(NodeData::Element { name, .. }) if name == "table" || name == "template" || name == "html")
            {
                break;
            }
            self.stack_of_open_elements.pop();
        }
    }

    fn clear_stack_back_to_table_body_context(&mut self) {
        while let Some(&id) = self.stack_of_open_elements.last() {
            if matches!(self.dom.data(id), Some(NodeData::Element { name, .. }) if name == "tbody" || name == "tfoot" || name == "thead" || name == "template" || name == "html")
            {
                break;
            }
            self.stack_of_open_elements.pop();
        }
    }

    fn clear_stack_back_to_table_row_context(&mut self) {
        while let Some(&id) = self.stack_of_open_elements.last() {
            if matches!(self.dom.data(id), Some(NodeData::Element { name, .. }) if name == "tr" || name == "template" || name == "html")
            {
                break;
            }
            self.stack_of_open_elements.pop();
        }
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

    #[test]
    fn test_simple_table() {
        let html = "<table><tr><td>cell</td></tr></table>";
        let dom = parse_document(InputStream::from_utf8(html.as_bytes()));
        assert_eq!(
            dom.serialize(dom.document()),
            "<html><head></head><body><table><tbody><tr><td>cell</td></tr></tbody></table></body></html>"
        );
    }

    #[test]
    fn test_table_with_implicit_tbody() {
        let html = "<table><tr><td>1</td></tr><tr><td>2</td></tr></table>";
        let dom = parse_document(InputStream::from_utf8(html.as_bytes()));
        assert_eq!(
            dom.serialize(dom.document()),
            "<html><head></head><body><table><tbody><tr><td>1</td></tr><tr><td>2</td></tr></tbody></table></body></html>"
        );
    }

    #[test]
    fn test_table_foster_parenting() {
        let html = "<table>text<tr><td>cell</td></tr></table>";
        let dom = parse_document(InputStream::from_utf8(html.as_bytes()));
        // TODO(spec): proper foster parenting inserts BEFORE the table.
        // Currently we only have append_child, so it ends up AFTER the table's preceding siblings.
        // Since table is the first child here, it ends up after the table.
        assert_eq!(
            dom.serialize(dom.document()),
            "<html><head></head><body><table><tbody><tr><td>cell</td></tr></tbody></table>text</body></html>"
        );
    }

    #[test]
    fn test_formatting_elements_reconstruction() {
        let html = "<b>bold<i>italic</b>still italic</i>";
        let dom = parse_document(InputStream::from_utf8(html.as_bytes()));
        // AAA subset just pops elements, so it might not be perfect yet
        assert_eq!(
            dom.serialize(dom.document()),
            "<html><head></head><body><b>bold<i>italic</i></b>still italic</body></html>"
        );
    }

    #[test]
    fn test_html5lib_table_1() {
        // From tables01.dat: <table><th>
        let html = "<table><th>";
        let dom = parse_document(InputStream::from_utf8(html.as_bytes()));
        assert_eq!(
            dom.serialize(dom.document()),
            "<html><head></head><body><table><tbody><tr><th></th></tr></tbody></table></body></html>"
        );
    }

    #[test]
    fn test_html5lib_table_2() {
        // From tables01.dat: <table><td>
        let html = "<table><td>";
        let dom = parse_document(InputStream::from_utf8(html.as_bytes()));
        assert_eq!(
            dom.serialize(dom.document()),
            "<html><head></head><body><table><tbody><tr><td></td></tr></tbody></table></body></html>"
        );
    }

    #[test]
    fn test_template_basic() {
        let html = "<div><template><p>hidden</p></template></div>";
        let dom = parse_document(InputStream::from_utf8(html.as_bytes()));
        assert_eq!(
            dom.serialize(dom.document()),
            "<html><head></head><body><div><template><p>hidden</p></template></div></body></html>"
        );
    }

    #[test]
    fn test_robustness_table_implicit() {
        // "<table><tr><td>x" yields tbody>tr>td>x
        let html = "<table><tr><td>x";
        let dom = parse_document(InputStream::from_utf8(html.as_bytes()));
        assert_eq!(
            dom.serialize(dom.document()),
            "<html><head></head><body><table><tbody><tr><td>x</td></tr></tbody></table></body></html>"
        );
    }

    #[test]
    fn test_robustness_p_auto_close() {
        // "<p>a<p>b" yields two sibling <p> (already verified but let's have a dedicated test)
        let html = "<p>a<p>b";
        let dom = parse_document(InputStream::from_utf8(html.as_bytes()));
        assert_eq!(
            dom.serialize(dom.document()),
            "<html><head></head><body><p>a</p><p>b</p></body></html>"
        );

        // "<p>a<div>b" should auto-close p, yielding sibling p and div
        let html2 = "<p>a<div>b";
        let dom2 = parse_document(InputStream::from_utf8(html2.as_bytes()));
        assert_eq!(
            dom2.serialize(dom2.document()),
            "<html><head></head><body><p>a</p><div>b</div></body></html>"
        );
    }

    #[test]
    fn test_robustness_li_nesting() {
        // "<ul><li>item 1<li>item 2</ul>" resolves li nesting as sibling li elements
        let html = "<ul><li>item 1<li>item 2</ul>";
        let dom = parse_document(InputStream::from_utf8(html.as_bytes()));
        assert_eq!(
            dom.serialize(dom.document()),
            "<html><head></head><body><ul><li>item 1</li><li>item 2</li></ul></body></html>"
        );
    }

    #[test]
    fn test_robustness_option_nesting() {
        // "<select><option>one<option>two</select>" resolves option nesting as sibling option elements
        let html = "<select><option>one<option>two</select>";
        let dom = parse_document(InputStream::from_utf8(html.as_bytes()));
        assert_eq!(
            dom.serialize(dom.document()),
            "<html><head></head><body><select><option>one</option><option>two</option></select></body></html>"
        );
    }

    #[test]
    fn test_robustness_unknown_tags() {
        // unknown tags become ordinary elements and can be nested
        let html = "<my-custom-tag><other-tag>content</other-tag></my-custom-tag>";
        let dom = parse_document(InputStream::from_utf8(html.as_bytes()));
        assert_eq!(
            dom.serialize(dom.document()),
            "<html><head></head><body><my-custom-tag><other-tag>content</other-tag></my-custom-tag></body></html>"
        );
    }

    #[test]
    fn test_character_coalescing() {
        // parsing <p>Hello world</p> yields exactly ONE text node "Hello world"
        let html = "<p>Hello world</p>";
        let dom = parse_document(InputStream::from_utf8(html.as_bytes()));
        let p_node = dom.query_selector("p").expect("p element exists");
        let p_children = dom.children(p_node);
        assert_eq!(p_children.len(), 1, "p should have exactly 1 child");
        let first_child = p_children[0];
        assert_eq!(
            dom.data(first_child),
            Some(&NodeData::Text("Hello world".to_string())),
            "first child should be a single Text node with 'Hello world'"
        );

        // <p>a<b>x</b>b</p> yields text "a", element b with text "x", then text "b"
        let html2 = "<p>a<b>x</b>b</p>";
        let dom2 = parse_document(InputStream::from_utf8(html2.as_bytes()));
        let p_node2 = dom2.query_selector("p").expect("p element exists");
        let p_children2 = dom2.children(p_node2);
        assert_eq!(p_children2.len(), 3, "p should have exactly 3 children");
        assert_eq!(
            dom2.data(p_children2[0]),
            Some(&NodeData::Text("a".to_string())),
            "first child of p should be text 'a'"
        );
        let b_node = p_children2[1];
        assert!(
            matches!(dom2.data(b_node), Some(NodeData::Element { name, .. }) if name == "b"),
            "second child of p should be element 'b'"
        );
        let b_children = dom2.children(b_node);
        assert_eq!(b_children.len(), 1, "b should have exactly 1 child");
        assert_eq!(
            dom2.data(b_children[0]),
            Some(&NodeData::Text("x".to_string())),
            "b's child should be text 'x'"
        );
        assert_eq!(
            dom2.data(p_children2[2]),
            Some(&NodeData::Text("b".to_string())),
            "third child of p should be text 'b'"
        );
    }

    #[test]
    fn test_head_body_insertion_modes() {
        let html = "<html><head><title>T</title><meta><style>s</style></head><body><div><p>x</p></div></body></html>";
        let dom = parse_document(InputStream::from_utf8(html.as_bytes()));

        let head_node = dom.query_selector("head").expect("<head> exists");
        let head_children = dom.children(head_node);
        let head_child_names: Vec<String> = head_children
            .iter()
            .map(|&id| match dom.data(id) {
                Some(NodeData::Element { name, .. }) => name.clone(),
                _ => "non-element".to_string(),
            })
            .collect();
        assert_eq!(head_child_names, vec!["title", "meta", "style"]);

        let body_nodes = dom.query_selector_all("body");
        assert_eq!(
            body_nodes.len(),
            1,
            "There should be EXACTLY ONE body node in the DOM"
        );

        let body_node = body_nodes[0];
        let body_children = dom.children(body_node);
        let body_child_names: Vec<String> = body_children
            .iter()
            .map(|&id| match dom.data(id) {
                Some(NodeData::Element { name, .. }) => name.clone(),
                _ => "non-element".to_string(),
            })
            .collect();
        assert_eq!(body_child_names, vec!["div"]);
    }

    #[test]
    fn test_in_column_group_explicit() {
        let html = "<table><colgroup><col><col></colgroup><tr><td>x</table>";
        let dom = parse_document(InputStream::from_utf8(html.as_bytes()));
        assert_eq!(
            dom.serialize(dom.document()),
            "<html><head></head><body><table><colgroup><col><col></colgroup><tbody><tr><td>x</td></tr></tbody></table></body></html>"
        );
    }

    #[test]
    fn test_in_column_group_implicit_bare_col() {
        let html = "<table><col></table>";
        let dom = parse_document(InputStream::from_utf8(html.as_bytes()));
        assert_eq!(
            dom.serialize(dom.document()),
            "<html><head></head><body><table><colgroup><col></colgroup></table></body></html>"
        );
    }

    #[test]
    fn test_in_column_group_stray_text() {
        let html = "<table><colgroup>text<col></colgroup>";
        let dom = parse_document(InputStream::from_utf8(html.as_bytes()));
        let serialized = dom.serialize(dom.document());
        assert!(!serialized.is_empty());
    }

    #[test]
    fn test_in_select_nested_options() {
        let html = "<select><option>a</option><option>b</option></select>";
        let dom = parse_document(InputStream::from_utf8(html.as_bytes()));
        assert_eq!(
            dom.serialize(dom.document()),
            "<html><head></head><body><select><option>a</option><option>b</option></select></body></html>"
        );
    }

    #[test]
    fn test_in_select_unclosed_options() {
        let html = "<select><option>a<option>b</select>";
        let dom = parse_document(InputStream::from_utf8(html.as_bytes()));
        assert_eq!(
            dom.serialize(dom.document()),
            "<html><head></head><body><select><option>a</option><option>b</option></select></body></html>"
        );
    }

    #[test]
    fn test_in_select_optgroup() {
        let html = "<select><optgroup><option>a</option></optgroup></select>";
        let dom = parse_document(InputStream::from_utf8(html.as_bytes()));
        assert_eq!(
            dom.serialize(dom.document()),
            "<html><head></head><body><select><optgroup><option>a</option></optgroup></select></body></html>"
        );
    }

    #[test]
    fn test_noscript_in_head_with_link() {
        let html = "<html><head><noscript><link></noscript></head><body></body></html>";
        let dom = parse_document(InputStream::from_utf8(html.as_bytes()));
        assert_eq!(
            dom.serialize(dom.document()),
            "<html><head><noscript><link></noscript></head><body></body></html>"
        );
    }

    #[test]
    fn test_noscript_in_head_p_closes_noscript() {
        let html = "<html><head><noscript><p>x</p></noscript></head><body></body></html>";
        let dom = parse_document(InputStream::from_utf8(html.as_bytes()));
        assert_eq!(
            dom.serialize(dom.document()),
            "<html><head><noscript></noscript></head><body><p>x</p></body></html>"
        );
    }

    #[test]
    fn test_noscript_in_head_empty() {
        let html = "<html><head><noscript></noscript></head><body></body></html>";
        let dom = parse_document(InputStream::from_utf8(html.as_bytes()));
        assert_eq!(
            dom.serialize(dom.document()),
            "<html><head><noscript></noscript></head><body></body></html>"
        );
    }
}
