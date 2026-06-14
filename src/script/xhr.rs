use boa_engine::{Context, JsResult, Source};

/// Registers the global XMLHttpRequest constructor class.
pub fn register_xhr(context: &mut Context) -> JsResult<()> {
    let code = r#"
        (function() {
            const BaseClass = typeof EventTarget !== "undefined" ? EventTarget : class {};

            class XMLHttpRequest extends BaseClass {
                constructor() {
                    super();
                    this.readyState = 0; // UNSENT
                    this.status = 0;
                    this.statusText = "";
                    this.responseText = "";
                    this.response = "";
                    this.responseType = "";
                    this.timeout = 0;
                    this.withCredentials = false;
                    this.onreadystatechange = null;

                    this._method = "";
                    this._url = "";
                    this._headers = {};
                    this._overrideMime = null;
                }

                open(method, url, async, user, password) {
                    this._method = String(method);
                    this._url = String(url);
                    // TODO(spec): Validate method and url, throw error on invalid/unsupported schemes.
                    this._changeReadyState(1); // OPENED
                }

                send(body) {
                    // stub
                }

                setRequestHeader(name, value) {
                    // TODO(spec): Throw if readyState is not OPENED or if send() flag is set.
                    const lowerName = String(name).toLowerCase();
                    const valStr = String(value);
                    if (this._headers[lowerName] !== undefined) {
                        this._headers[lowerName] += ", " + valStr;
                    } else {
                        this._headers[lowerName] = valStr;
                    }
                }

                getResponseHeader(name) {
                    // Returns null while stubbed.
                    return null;
                }

                getAllResponseHeaders() {
                    // Returns "" while stubbed.
                    return "";
                }

                abort() {
                    this._changeReadyState(0); // UNSENT
                    this.status = 0;
                    this.statusText = "";
                    this._headers = {};
                }

                overrideMimeType(mime) {
                    this._overrideMime = String(mime);
                }

                _changeReadyState(newState) {
                    if (this.readyState !== newState) {
                        this.readyState = newState;
                        if (typeof this.onreadystatechange === "function") {
                            try {
                                this.onreadystatechange.call(this);
                            } catch (e) {
                                // Suppress or handle error
                            }
                        }
                        // If EventTarget is inherited, standard also dispatches "readystatechange" event
                        if (typeof this.dispatchEvent === "function" && typeof Event !== "undefined") {
                            try {
                                this.dispatchEvent(new Event("readystatechange"));
                            } catch (e) {
                                // Suppress or handle error
                            }
                        }
                    }
                }
            }

            // Set up readyState constants on constructor and instances
            const constants = {
                UNSENT: 0,
                OPENED: 1,
                HEADERS_RECEIVED: 2,
                LOADING: 3,
                DONE: 4
            };

            for (const [key, val] of Object.entries(constants)) {
                Object.defineProperty(XMLHttpRequest, key, {
                    value: val,
                    writable: false,
                    enumerable: true,
                    configurable: false
                });
                Object.defineProperty(XMLHttpRequest.prototype, key, {
                    value: val,
                    writable: false,
                    enumerable: true,
                    configurable: false
                });
            }

            globalThis.XMLHttpRequest = XMLHttpRequest;
        })();
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

    #[test]
    fn test_xhr_surface_compliance() {
        let mut context = Context::default();
        register_xhr(&mut context).expect("Failed to register XMLHttpRequest");

        let script = r#"
            const xhr = new XMLHttpRequest();
            if (xhr.readyState !== 0) throw new Error("readyState should be 0");
            if (xhr.status !== 0) throw new Error("status should be 0");
            if (xhr.statusText !== "") throw new Error("statusText should be empty");
            if (xhr.responseText !== "") throw new Error("responseText should be empty");
            if (xhr.response !== "") throw new Error("response should be empty");
            if (xhr.responseType !== "") throw new Error("responseType should be empty");
            if (xhr.timeout !== 0) throw new Error("timeout should be 0");
            if (xhr.withCredentials !== false) throw new Error("withCredentials should be false");

            // Test static constants on constructor
            if (XMLHttpRequest.UNSENT !== 0) throw new Error("UNSENT should be 0");
            if (XMLHttpRequest.OPENED !== 1) throw new Error("OPENED should be 1");
            if (XMLHttpRequest.HEADERS_RECEIVED !== 2) throw new Error("HEADERS_RECEIVED should be 2");
            if (XMLHttpRequest.LOADING !== 3) throw new Error("LOADING should be 3");
            if (XMLHttpRequest.DONE !== 4) throw new Error("DONE should be 4");

            // Test instance constants
            if (xhr.UNSENT !== 0) throw new Error("instance UNSENT should be 0");
            if (xhr.OPENED !== 1) throw new Error("instance OPENED should be 1");
            if (xhr.HEADERS_RECEIVED !== 2) throw new Error("instance HEADERS_RECEIVED should be 2");
            if (xhr.LOADING !== 3) throw new Error("instance LOADING should be 3");
            if (xhr.DONE !== 4) throw new Error("instance DONE should be 4");

            // Test change event / onreadystatechange callback
            let statesChanged = [];
            xhr.onreadystatechange = function() {
                statesChanged.push(this.readyState);
            };

            xhr.open("GET", "https://example.com");
            if (xhr.readyState !== 1) throw new Error("readyState should be 1 after open()");
            if (statesChanged.length !== 1 || statesChanged[0] !== 1) {
                throw new Error("onreadystatechange not called correctly: " + JSON.stringify(statesChanged));
            }

            // Test setRequestHeader
            xhr.setRequestHeader("Content-Type", "application/json");
            xhr.setRequestHeader("content-type", "charset=utf-8");
            if (xhr._headers["content-type"] !== "application/json, charset=utf-8") {
                throw new Error("content-type not stored case-insensitively or not appended: " + xhr._headers["content-type"]);
            }

            // Test response headers
            if (xhr.getResponseHeader("Content-Type") !== null) throw new Error("getResponseHeader should be null");
            if (xhr.getAllResponseHeaders() !== "") throw new Error("getAllResponseHeaders should be empty");

            // Test overrideMimeType
            xhr.overrideMimeType("text/html");
            if (xhr._overrideMime !== "text/html") throw new Error("overrideMimeType not stored");

            // Test abort
            xhr.abort();
            if (xhr.readyState !== 0) throw new Error("readyState should be 0 after abort()");
            if (statesChanged[statesChanged.length - 1] !== 0) throw new Error("onreadystatechange not called for abort()");
        "#;

        let res = context.eval(Source::from_bytes(script.as_bytes()));
        if let Err(e) = &res {
            panic!("XHR surface compliance script failed: {:?}", e);
        }
        assert!(res.is_ok());
    }
}
