use boa_engine::{Context, JsResult, Source};

/// Registers the global XMLHttpRequest constructor class.
pub fn register_xhr(context: &mut Context) -> JsResult<()> {
    let code = r#"
        class XMLHttpRequest {
            constructor() {
                this._method = "";
                this._url = "";
            }
            open(method, url) {
                this._method = String(method);
                this._url = String(url);
            }
            send() {
                // stub
            }
        }
        globalThis.XMLHttpRequest = XMLHttpRequest;
    "#;
    let source = Source::from_bytes(code.as_bytes());
    let _ = context.eval(source)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::Source;

    #[test]
    fn test_xhr_stub() {
        let mut context = Context::default();
        register_xhr(&mut context).expect("Failed to register XMLHttpRequest");

        let script = r#"
            const xhr = new XMLHttpRequest();
            xhr.open("GET", "https://example.com");
            xhr.send();
        "#;

        let res = context.eval(Source::from_bytes(script.as_bytes()));
        assert!(res.is_ok(), "XHR stub script failed: {:?}", res);
    }
}
