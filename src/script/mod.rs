//! Scripting module providing JavaScript execution via the Boa engine.
//!
//! This module implements the `ScriptHost` port, allowing the browser engine
//! to execute scripts. The current implementation uses the `boa_engine` crate.

use crate::dom::{Dom, NodeData};
use boa_engine::object::ObjectInitializer;
use boa_engine::property::Attribute;
use boa_engine::{Context, JsError, JsString, JsValue, NativeFunction, Source};

/// Errors that can occur during script execution.
#[derive(Debug, PartialEq)]
pub enum ScriptError {
    /// A syntax error in the script.
    Syntax(String),
    /// A runtime error during script execution.
    Runtime(String),
}

impl core::fmt::Display for ScriptError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Syntax(msg) => write!(f, "Syntax Error: {}", msg),
            Self::Runtime(msg) => write!(f, "Runtime Error: {}", msg),
        }
    }
}

impl std::error::Error for ScriptError {}

/// Trait for a host that can execute scripts.
pub trait ScriptHost {
    /// Evaluates the given script source.
    fn eval(&mut self, src: &str) -> Result<(), ScriptError>;
}

/// A `ScriptHost` implementation using the Boa JavaScript engine.
pub struct BoaHost {
    context: Context,
}

impl BoaHost {
    /// Creates a new `BoaHost` with an empty context.
    pub fn new() -> Self {
        let mut context = Context::default();

        // EXPERIMENTAL: Minimal DOM binding (document.title)
        // TODO(spec): Full DOM bindings are a large separate effort.
        Self::setup_experimental_dom(&mut context);

        Self { context }
    }

    fn setup_experimental_dom(context: &mut Context) {
        let get_element_by_id = NativeFunction::from_fn_ptr(|_this, args, context| {
            let id_val = if let Some(arg) = args.first() {
                arg.to_string(context)?.to_std_string().unwrap_or_default()
            } else {
                return Ok(JsValue::null());
            };

            let global = context.global_object();
            let document = global.get(JsString::from("document"), context)?;
            if let Some(document_obj) = document.as_object() {
                let elements_val = document_obj.get(JsString::from("__elements__"), context)?;
                if let Some(elements_obj) = elements_val.as_object() {
                    let elem = elements_obj.get(JsString::from(id_val), context)?;
                    if !elem.is_undefined() {
                        return Ok(elem);
                    }
                }
            }
            Ok(JsValue::null())
        });

        let document = ObjectInitializer::new(context)
            .property(
                JsString::from("title"),
                JsString::from("Underrated"),
                Attribute::all(),
            )
            .function(get_element_by_id, JsString::from("getElementById"), 1)
            .build();

        let _ = context.register_global_property(
            JsString::from("document"),
            document,
            Attribute::all(),
        );
    }

    /// Evaluates the given script with the provided DOM context.
    ///
    /// Exposes a read-only `document` object to the script enabling `document.getElementById`.
    pub fn eval_with_dom(&mut self, src: &str, dom: &Dom) -> Result<String, ScriptError> {
        // 1. Gather all element nodes in `dom` with an `id`.
        let mut elements_with_id = Vec::new();
        let root = dom.document();
        let mut nodes_to_check = vec![root];
        while let Some(node_id) = nodes_to_check.pop() {
            if let Some(NodeData::Element { attrs, .. }) = dom.data(node_id) {
                let id_attr = attrs.iter().find(|(n, _)| n == "id");
                if let Some((_, id_val)) = id_attr {
                    elements_with_id.push((id_val.clone(), dom.text_content(node_id)));
                }
            }
            nodes_to_check.extend(dom.children(node_id).iter().rev().copied());
        }

        // 2. Build the element JS objects.
        let mut element_objs = Vec::new();
        for (id_val, text_content_val) in elements_with_id {
            let element_obj = ObjectInitializer::new(&mut self.context)
                .property(
                    JsString::from("textContent"),
                    JsString::from(text_content_val),
                    Attribute::all(),
                )
                .property(
                    JsString::from("id"),
                    JsString::from(id_val.clone()),
                    Attribute::all(),
                )
                .build();
            element_objs.push((JsString::from(id_val), element_obj));
        }

        // Build the `__elements__` registry JS object.
        let mut registry_builder = ObjectInitializer::new(&mut self.context);
        for (id_js, element_obj) in element_objs {
            registry_builder.property(id_js, element_obj, Attribute::all());
        }
        let registry_obj = registry_builder.build();

        // 3. Find the global `document` object.
        let global = self.context.global_object();
        let document_val = global
            .get(JsString::from("document"), &mut self.context)
            .map_err(|e| ScriptError::Runtime(e.to_string()))?;
        let document_obj = document_val
            .as_object()
            .ok_or_else(|| ScriptError::Runtime("global document is not an object".to_string()))?;

        // 4. Attach `__elements__` to `document`.
        document_obj
            .set(
                JsString::from("__elements__"),
                JsValue::from(registry_obj),
                false,
                &mut self.context,
            )
            .map_err(|e| ScriptError::Runtime(e.to_string()))?;

        // 5. Evaluate the source code.
        let source = Source::from_bytes(src.as_bytes());
        let res_val = self.context.eval(source).map_err(map_boa_error)?;

        // 6. Coerce the JS result to String.
        let res_str = res_val
            .to_string(&mut self.context)
            .map_err(|e| ScriptError::Runtime(e.to_string()))?
            .to_std_string()
            .map_err(|e| ScriptError::Runtime(e.to_string()))?;

        Ok(res_str)
    }
}

impl Default for BoaHost {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptHost for BoaHost {
    fn eval(&mut self, src: &str) -> Result<(), ScriptError> {
        let source = Source::from_bytes(src.as_bytes());
        match self.context.eval(source) {
            Ok(_) => Ok(()),
            Err(err) => Err(map_boa_error(err)),
        }
    }
}

fn map_boa_error(err: JsError) -> ScriptError {
    let msg = err.to_string();
    if msg.contains("SyntaxError") {
        ScriptError::Syntax(msg)
    } else {
        ScriptError::Runtime(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boa_eval_basic() {
        let mut host = BoaHost::new();
        assert!(host.eval("1 + 1").is_ok());
    }

    #[test]
    fn test_boa_eval_syntax_error() {
        let mut host = BoaHost::new();
        // Invalid syntax: missing closing parenthesis
        let result = host.eval("console.log(");
        assert!(result.is_err());
        assert!(matches!(result, Err(ScriptError::Syntax(_))));
    }

    #[test]
    fn test_experimental_dom_binding() {
        let mut host = BoaHost::new();
        // Check if document.title is accessible.
        // We can't easily get the return value from eval(()) so we might need a way to check state.
        // But we can check if it doesn't throw.
        assert!(
            host.eval("if (document.title !== 'Underrated') throw 'Wrong title';")
                .is_ok()
        );
        assert!(host.eval("document.title = 'New Title';").is_ok());
        assert!(
            host.eval("if (document.title !== 'New Title') throw 'Title not updated';")
                .is_ok()
        );
    }

    #[test]
    fn test_eval_with_dom_basic() {
        let mut dom = Dom::new();
        let document = dom.document();

        let element_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "greeting".to_string())],
        });
        let text_id = dom.create_node(NodeData::Text("Hello".to_string()));
        dom.append_child(element_id, text_id);
        dom.append_child(document, element_id);

        let mut host = BoaHost::new();
        let res = host.eval_with_dom("document.getElementById('greeting').textContent", &dom);
        assert_eq!(res, Ok("Hello".to_string()));
    }

    #[test]
    fn test_eval_with_dom_missing_id() {
        let dom = Dom::new();
        let mut host = BoaHost::new();
        let res = host.eval_with_dom("document.getElementById('nonexistent')", &dom);
        assert_eq!(res, Ok("null".to_string()));
    }
}
