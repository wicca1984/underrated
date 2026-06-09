//! Scripting module providing JavaScript execution via the Boa engine.
//!
//! This module implements the `ScriptHost` port, allowing the browser engine
//! to execute scripts. The current implementation uses the `boa_engine` crate.

use boa_engine::{Context, JsError, Source};

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
        use boa_engine::JsString;
        use boa_engine::object::ObjectInitializer;
        use boa_engine::property::Attribute;

        let document = ObjectInitializer::new(context)
            .property(
                JsString::from("title"),
                JsString::from("Underrated"),
                Attribute::all(),
            )
            .build();

        let _ = context.register_global_property(
            JsString::from("document"),
            document,
            Attribute::all(),
        );
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
}
