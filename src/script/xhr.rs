use boa_engine::{Context, JsResult, Source};

/// Registers the global XMLHttpRequest constructor class.
pub fn register_xhr(context: &mut Context) -> JsResult<()> {
    let code = r#"
        (function() {
            const BaseClass = typeof EventTarget !== "undefined" ? EventTarget : class {};
            const EventClass = typeof Event !== "undefined" ? Event : class {};

            class ProgressEvent extends EventClass {
                constructor(type, eventInitDict = {}) {
                    super(type, eventInitDict);
                    this.type = type;
                    this._lengthComputable = !!eventInitDict.lengthComputable;
                    this._loaded = Number(eventInitDict.loaded) || 0;
                    this._total = Number(eventInitDict.total) || 0;
                }

                get lengthComputable() {
                    return this._lengthComputable;
                }

                get loaded() {
                    return this._loaded;
                }

                get total() {
                    return this._total;
                }
            }

            globalThis.ProgressEvent = ProgressEvent;

            class XMLHttpRequestUpload extends BaseClass {
                constructor() {
                    super();
                    this.onloadstart = null;
                    this.onprogress = null;
                    this.onabort = null;
                    this.onerror = null;
                    this.onload = null;
                    this.ontimeout = null;
                    this.onloadend = null;
                }

                dispatchEvent(event) {
                    if (event && typeof event.type === "string") {
                        const handlerName = "on" + event.type;
                        if (typeof this[handlerName] === "function") {
                            try {
                                this[handlerName].call(this, event);
                            } catch (e) {}
                        }
                    }
                    if (super.dispatchEvent) {
                        return super.dispatchEvent(event);
                    }
                    return true;
                }
            }

            class XMLHttpRequest extends BaseClass {
                constructor() {
                    super();
                    this._readyState = 0; // UNSENT
                    this._status = 0;
                    this._statusText = "";
                    this._responseText = "";
                    this._responseType = "";
                    this._headers = {};
                    this._overrideMime = null;
                    this._sendFlag = false;

                    this._timeout = 0;
                    this._withCredentials = false;
                    this._async = true;
                    this._upload = new XMLHttpRequestUpload();

                    this.onreadystatechange = null;
                    this.onloadstart = null;
                    this.onprogress = null;
                    this.onabort = null;
                    this.onerror = null;
                    this.onload = null;
                    this.ontimeout = null;
                    this.onloadend = null;

                    this._method = "";
                    this._url = "";
                }

                get readyState() {
                    return this._readyState;
                }

                get status() {
                    if (this._readyState === 0 || this._readyState === 1) {
                        return 0;
                    }
                    return this._status;
                }

                get statusText() {
                    if (this._readyState === 0 || this._readyState === 1) {
                        return "";
                    }
                    return this._statusText;
                }

                get responseType() {
                    return this._responseType;
                }

                set responseType(value) {
                    const valStr = String(value);
                    const allowedTypes = ["", "arraybuffer", "blob", "document", "json", "text"];
                    if (!allowedTypes.includes(valStr)) {
                        return;
                    }
                    if (this._async === false && this._readyState !== 0 && this._readyState !== 1) {
                        const err = new Error("InvalidAccessError");
                        err.name = "InvalidAccessError";
                        throw err;
                    }
                    if (this._readyState === 3 || this._readyState === 4) {
                        const err = new Error("InvalidStateError");
                        err.name = "InvalidStateError";
                        throw err;
                    }
                    this._responseType = valStr;
                    if (this._async === false && this._responseType !== "") {
                        const err = new Error("InvalidAccessError");
                        err.name = "InvalidAccessError";
                        throw err;
                    }
                }

                get responseText() {
                    if (this._responseType !== "" && this._responseType !== "text") {
                        const err = new Error("InvalidStateError");
                        err.name = "InvalidStateError";
                        throw err;
                    }
                    if (this._readyState !== 3 && this._readyState !== 4) {
                        return "";
                    }
                    return this._responseText;
                }

                get response() {
                    if (this._readyState !== 3 && this._readyState !== 4) {
                        if (this._responseType === "" || this._responseType === "text") {
                            return "";
                        }
                        return null;
                    }
                    if (this._responseType === "" || this._responseType === "text") {
                        return this._responseText;
                    }
                    if (this._readyState !== 4) {
                        return null;
                    }
                    if (this._responseType === "json") {
                        if (this._responseText === "") {
                            return null;
                        }
                        try {
                            return JSON.parse(this._responseText);
                        } catch (e) {
                            return null;
                        }
                    }
                    if (this._responseType === "document") {
                        return this.responseXML;
                    }
                    if (this._responseType === "arraybuffer") {
                        if (this._responseText === "") {
                            return new ArrayBuffer(0);
                        }
                        const buf = new ArrayBuffer(this._responseText.length);
                        const bufView = new Uint8Array(buf);
                        for (let i = 0; i < this._responseText.length; i++) {
                            bufView[i] = this._responseText.charCodeAt(i) & 0xff;
                        }
                        return buf;
                    }
                    if (this._responseType === "blob") {
                        if (typeof Blob !== "undefined") {
                            return new Blob([this._responseText]);
                        }
                        return null;
                    }
                    return null;
                }

                get responseXML() {
                    if (this._responseType !== "" && this._responseType !== "document") {
                        const err = new Error("InvalidStateError");
                        err.name = "InvalidStateError";
                        throw err;
                    }
                    if (this._readyState !== 4) {
                        return null;
                    }
                    if (!this._responseText) {
                        return null;
                    }
                    if (typeof DOMParser !== "undefined") {
                        try {
                            const parser = new DOMParser();
                            let mime = "text/xml";
                            if (this._overrideMime) {
                                mime = this._overrideMime;
                            } else {
                                const ct = this.getResponseHeader("content-type");
                                if (ct) {
                                    const match = ct.match(/^([^;\s]+)/);
                                    if (match) {
                                        mime = match[1];
                                    }
                                }
                            }
                            return parser.parseFromString(this._responseText, mime);
                        } catch (e) {
                            return null;
                        }
                    }
                    return null;
                }

                get responseURL() {
                    if (this._readyState === 0 || this._readyState === 1) {
                        return "";
                    }
                    if (!this._url) {
                        return "";
                    }
                    const hashIdx = this._url.indexOf('#');
                    if (hashIdx !== -1) {
                        return this._url.substring(0, hashIdx);
                    }
                    return this._url;
                }

                get timeout() {
                    return this._timeout;
                }

                set timeout(value) {
                    if (this._async === false) {
                        const err = new Error("InvalidAccessError");
                        err.name = "InvalidAccessError";
                        throw err;
                    }
                    const num = Number(value);
                    if (!isNaN(num) && num >= 0) {
                        this._timeout = Math.floor(num);
                    }
                }

                get withCredentials() {
                    return this._withCredentials;
                }

                set withCredentials(value) {
                    if (this._readyState !== 0 && this._readyState !== 1) {
                        const err = new Error("InvalidStateError");
                        err.name = "InvalidStateError";
                        throw err;
                    }
                    if (this._sendFlag) {
                        const err = new Error("InvalidStateError");
                        err.name = "InvalidStateError";
                        throw err;
                    }
                    if (this._async === false) {
                        const err = new Error("InvalidAccessError");
                        err.name = "InvalidAccessError";
                        throw err;
                    }
                    this._withCredentials = !!value;
                }

                get upload() {
                    return this._upload;
                }

                open(method, url, async, user, password) {
                    const methodStr = String(method);
                    // Validate method as a valid HTTP token
                    if (!/^[!#$%&'*+\-.^_`|~a-zA-Z0-9]+$/.test(methodStr)) {
                        const err = new Error("SyntaxError: Invalid method");
                        err.name = "SyntaxError";
                        throw err;
                    }

                    // Check for forbidden methods
                    const upperMethod = methodStr.toUpperCase();
                    if (upperMethod === "CONNECT" || upperMethod === "TRACE" || upperMethod === "TRACK") {
                        const err = new Error("SecurityError: Method is forbidden");
                        err.name = "SecurityError";
                        throw err;
                    }

                    this._method = methodStr;
                    this._url = String(url);
                    this._sendFlag = false;
                    this._status = 0;
                    this._statusText = "";
                    this._responseText = "";
                    this._headers = {};
                    this._async = async !== false;
                    this._withCredentials = false;
                    this._responseType = "";

                    this._changeReadyState(1); // OPENED
                }

                send(body) {
                    if (this._readyState !== 1) {
                        const err = new Error("InvalidStateError");
                        err.name = "InvalidStateError";
                        throw err;
                    }
                    if (this._sendFlag) {
                        const err = new Error("InvalidStateError");
                        err.name = "InvalidStateError";
                        throw err;
                    }

                    this._sendFlag = true;

                    let serializedBody = "";
                    if (body !== undefined && body !== null) {
                        if (typeof FormData !== "undefined" && body instanceof FormData) {
                            const formDataBoundary = "----WebKitFormBoundary" + Math.random().toString(36).substring(2, 10);
                            if (this._headers["content-type"] === undefined) {
                                this._headers["content-type"] = "multipart/form-data; boundary=" + formDataBoundary;
                            }
                            let parts = [];
                            body.forEach((val, key) => {
                                parts.push("--" + formDataBoundary + "\r\nContent-Disposition: form-data; name=\"" + key + "\"\r\n\r\n" + val + "\r\n");
                            });
                            parts.push("--" + formDataBoundary + "--\r\n");
                            serializedBody = parts.join("");
                        } else if (typeof URLSearchParams !== "undefined" && body instanceof URLSearchParams) {
                            if (this._headers["content-type"] === undefined) {
                                this._headers["content-type"] = "application/x-www-form-urlencoded;charset=UTF-8";
                            }
                            serializedBody = body.toString();
                        } else if (typeof ArrayBuffer !== "undefined" && body instanceof ArrayBuffer) {
                            const view = new Uint8Array(body);
                            let str = "";
                            for (let i = 0; i < view.length; i++) {
                                str += String.fromCharCode(view[i]);
                            }
                            serializedBody = str;
                        } else if (typeof ArrayBuffer !== "undefined" && ArrayBuffer.isView && ArrayBuffer.isView(body)) {
                            const view = new Uint8Array(body.buffer, body.byteOffset, body.byteLength);
                            let str = "";
                            for (let i = 0; i < view.length; i++) {
                                str += String.fromCharCode(view[i]);
                            }
                            serializedBody = str;
                        } else {
                            serializedBody = String(body);
                        }
                    } else {
                        serializedBody = "mock response";
                    }

                    const ProgressEventClass = typeof ProgressEvent !== "undefined" ? ProgressEvent : Event;

                    if (this._async !== false && typeof this.dispatchEvent === "function" && typeof Event !== "undefined") {
                        try {
                            this.dispatchEvent(new ProgressEventClass("loadstart", {
                                lengthComputable: false,
                                loaded: 0,
                                total: 0
                            }));
                            if (body !== undefined && body !== null && this._upload) {
                                this._upload.dispatchEvent(new ProgressEventClass("loadstart", {
                                    lengthComputable: true,
                                    loaded: 0,
                                    total: serializedBody.length
                                }));
                            }
                        } catch (e) {}
                    }

                    if (!this._sendFlag) return;

                    this._status = 200;
                    this._statusText = "OK";

                    this._changeReadyState(2); // HEADERS_RECEIVED
                    if (!this._sendFlag) return;

                    this._responseText = serializedBody;

                    this._changeReadyState(3); // LOADING
                    if (!this._sendFlag) return;

                    if (this._async !== false && typeof this.dispatchEvent === "function" && typeof Event !== "undefined") {
                        try {
                            const responseLen = this._responseText ? this._responseText.length : 0;
                            this.dispatchEvent(new ProgressEventClass("progress", {
                                lengthComputable: true,
                                loaded: responseLen,
                                total: responseLen
                            }));
                            if (body !== undefined && body !== null && this._upload) {
                                const bodyLen = serializedBody.length;
                                this._upload.dispatchEvent(new ProgressEventClass("progress", {
                                    lengthComputable: true,
                                    loaded: bodyLen,
                                    total: bodyLen
                                }));
                            }
                        } catch (e) {}
                    }

                    if (!this._sendFlag) return;

                    this._changeReadyState(4); // DONE
                    if (!this._sendFlag) return;
                    this._sendFlag = false;

                    if (this._async !== false && typeof this.dispatchEvent === "function" && typeof Event !== "undefined") {
                        try {
                            const responseLen = this._responseText ? this._responseText.length : 0;
                            const bodyLen = body !== undefined && body !== null ? serializedBody.length : 0;

                            if (body !== undefined && body !== null && this._upload) {
                                this._upload.dispatchEvent(new ProgressEventClass("load", {
                                    lengthComputable: true,
                                    loaded: bodyLen,
                                    total: bodyLen
                                }));
                                this._upload.dispatchEvent(new ProgressEventClass("loadend", {
                                    lengthComputable: true,
                                    loaded: bodyLen,
                                    total: bodyLen
                                }));
                            }
                            this.dispatchEvent(new ProgressEventClass("load", {
                                lengthComputable: true,
                                loaded: responseLen,
                                total: responseLen
                            }));
                            this.dispatchEvent(new ProgressEventClass("loadend", {
                                lengthComputable: true,
                                loaded: responseLen,
                                total: responseLen
                            }));
                        } catch (e) {}
                    }
                }

                setRequestHeader(name, value) {
                    if (this._readyState !== 1) {
                        const err = new Error("InvalidStateError");
                        err.name = "InvalidStateError";
                        throw err;
                    }
                    if (this._sendFlag) {
                        const err = new Error("InvalidStateError");
                        err.name = "InvalidStateError";
                        throw err;
                    }

                    const nameStr = String(name);
                    let valStr = String(value).trim();

                    // Validate header name as a valid HTTP token
                    if (!/^[!#$%&'*+\-.^_`|~a-zA-Z0-9]+$/.test(nameStr)) {
                        const err = new Error("SyntaxError: Invalid header name");
                        err.name = "SyntaxError";
                        throw err;
                    }

                    // Validate header value
                    if (!/^[\t\u0020-\u007E\u0080-\u00FF]*$/.test(valStr)) {
                        const err = new Error("SyntaxError: Invalid header value");
                        err.name = "SyntaxError";
                        throw err;
                    }

                    const lowerName = nameStr.toLowerCase();

                    // Check for forbidden headers
                    const forbiddenHeaders = [
                        "accept-charset", "accept-encoding", "access-control-request-headers",
                        "access-control-request-method", "connection", "content-length",
                        "cookie", "cookie2", "date", "dnt", "expect", "host", "keep-alive",
                        "origin", "referer", "te", "trailer", "transfer-encoding", "upgrade", "via"
                    ];
                    if (forbiddenHeaders.includes(lowerName) || lowerName.startsWith("sec-") || lowerName.startsWith("proxy-")) {
                        return; // Silently ignore
                    }

                    if (this._headers[lowerName] !== undefined) {
                        this._headers[lowerName] += ", " + valStr;
                    } else {
                        this._headers[lowerName] = valStr;
                    }
                }

                getResponseHeader(name) {
                    const lowerName = String(name).toLowerCase();
                    if (this._readyState === 0 || this._readyState === 1) {
                        return null;
                    }
                    if (lowerName === "set-cookie" || lowerName === "set-cookie2") {
                        return null;
                    }
                    return this._headers[lowerName] !== undefined ? this._headers[lowerName] : null;
                }

                getAllResponseHeaders() {
                    if (this._readyState === 0 || this._readyState === 1) {
                        return "";
                    }
                    let res = "";
                    const keys = Object.keys(this._headers).sort();
                    for (const key of keys) {
                        if (key === "set-cookie" || key === "set-cookie2") {
                            continue;
                        }
                        res += key + ": " + this._headers[key] + "\r\n";
                    }
                    return res;
                }

                abort() {
                    const state = this._readyState;
                    const wasSending = this._sendFlag;

                    // Clear response properties
                    this._status = 0;
                    this._statusText = "";
                    this._headers = {};
                    this._responseText = "";
                    this._sendFlag = false;

                    if (state === 0 || (state === 1 && !wasSending)) {
                        this._readyState = 0; // Set to UNSENT directly, no events
                        return;
                    }

                    if (state === 4) {
                        this._readyState = 0; // Set to UNSENT directly, no events
                        return;
                    }

                    // For states 1 (with wasSending), 2, 3: Active request error steps
                    // 1. Change readyState to 4 (DONE), and fire readystatechange
                    this._changeReadyState(4);

                    // 2. Fire progress events
                    if (this._async !== false && typeof this.dispatchEvent === "function" && typeof Event !== "undefined") {
                        try {
                            const ProgressEventClass = typeof ProgressEvent !== "undefined" ? ProgressEvent : Event;
                            if (wasSending && this._upload) {
                                this._upload.dispatchEvent(new ProgressEventClass("abort", {
                                    lengthComputable: false,
                                    loaded: 0,
                                    total: 0
                                }));
                                this._upload.dispatchEvent(new ProgressEventClass("loadend", {
                                    lengthComputable: false,
                                    loaded: 0,
                                    total: 0
                                }));
                            }
                            this.dispatchEvent(new ProgressEventClass("abort", {
                                lengthComputable: false,
                                loaded: 0,
                                total: 0
                            }));
                            this.dispatchEvent(new ProgressEventClass("loadend", {
                                lengthComputable: false,
                                loaded: 0,
                                total: 0
                            }));
                        } catch (e) {}
                    }

                    // 3. Finally set state to 0 (UNSENT) directly (no readystatechange fired)
                    this._readyState = 0;
                }

                _simulateTimeout() {
                    if (this._readyState === 0 || this._readyState === 4) {
                        return;
                    }
                    this._status = 0;
                    this._statusText = "";
                    this._headers = {};
                    this._responseText = "";
                    this._sendFlag = false;

                    this._changeReadyState(4); // DONE

                    const ProgressEventClass = typeof ProgressEvent !== "undefined" ? ProgressEvent : Event;
                    if (typeof this.dispatchEvent === "function" && typeof Event !== "undefined") {
                        try {
                            if (this._upload) {
                                this._upload.dispatchEvent(new ProgressEventClass("timeout", {
                                    lengthComputable: false,
                                    loaded: 0,
                                    total: 0
                                }));
                                this._upload.dispatchEvent(new ProgressEventClass("loadend", {
                                    lengthComputable: false,
                                    loaded: 0,
                                    total: 0
                                }));
                            }
                            this.dispatchEvent(new ProgressEventClass("timeout", {
                                lengthComputable: false,
                                loaded: 0,
                                total: 0
                            }));
                            this.dispatchEvent(new ProgressEventClass("loadend", {
                                lengthComputable: false,
                                loaded: 0,
                                total: 0
                            }));
                        } catch (e) {}
                    }
                }

                _simulateNetworkError() {
                    if (this._readyState === 0 || this._readyState === 4) {
                        return;
                    }
                    this._status = 0;
                    this._statusText = "";
                    this._headers = {};
                    this._responseText = "";
                    this._sendFlag = false;

                    this._changeReadyState(4); // DONE

                    const ProgressEventClass = typeof ProgressEvent !== "undefined" ? ProgressEvent : Event;
                    if (typeof this.dispatchEvent === "function" && typeof Event !== "undefined") {
                        try {
                            if (this._upload) {
                                this._upload.dispatchEvent(new ProgressEventClass("error", {
                                    lengthComputable: false,
                                    loaded: 0,
                                    total: 0
                                }));
                                this._upload.dispatchEvent(new ProgressEventClass("loadend", {
                                    lengthComputable: false,
                                    loaded: 0,
                                    total: 0
                                }));
                            }
                            this.dispatchEvent(new ProgressEventClass("error", {
                                lengthComputable: false,
                                loaded: 0,
                                total: 0
                            }));
                            this.dispatchEvent(new ProgressEventClass("loadend", {
                                lengthComputable: false,
                                loaded: 0,
                                total: 0
                            }));
                        } catch (e) {}
                    }
                }

                _parseResponseHeaders(rawHeadersString) {
                    this._headers = {};
                    if (!rawHeadersString) return;
                    const lines = rawHeadersString.split(/\r?\n/);
                    for (const line of lines) {
                        if (!line.trim()) continue;
                        const colonIdx = line.indexOf(":");
                        if (colonIdx === -1) continue;
                        const key = line.substring(0, colonIdx).trim().toLowerCase();
                        const val = line.substring(colonIdx + 1).trim();
                        if (this._headers[key] !== undefined) {
                            this._headers[key] += ", " + val;
                        } else {
                            this._headers[key] = val;
                        }
                    }
                }

                overrideMimeType(mime) {
                    if (this._readyState === 3 || this._readyState === 4) {
                        const err = new Error("InvalidStateError");
                        err.name = "InvalidStateError";
                        throw err;
                    }
                    this._overrideMime = String(mime);
                }

                _changeReadyState(newState) {
                    if (this._readyState !== newState) {
                        this._readyState = newState;
                        if (newState === 0) {
                            return;
                        }
                        if (this._async === false && newState !== 4) {
                            return;
                        }
                        if (typeof this.dispatchEvent === "function" && typeof Event !== "undefined") {
                            try {
                                this.dispatchEvent(new Event("readystatechange"));
                            } catch (e) {
                                // Suppress or handle error
                            }
                        } else if (typeof this.onreadystatechange === "function") {
                            try {
                                this.onreadystatechange.call(this);
                            } catch (e) {
                                // Suppress or handle error
                            }
                        }
                    }
                }

                dispatchEvent(event) {
                    if (event && typeof event.type === "string") {
                        const handlerName = "on" + event.type;
                        if (typeof this[handlerName] === "function") {
                            try {
                                this[handlerName].call(this, event);
                            } catch (e) {}
                        }
                    }
                    if (super.dispatchEvent) {
                        return super.dispatchEvent(event);
                    }
                    return true;
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

            globalThis.XMLHttpRequestUpload = XMLHttpRequestUpload;
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
            globalThis.test_error = null;
            try {
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
                if (statesChanged.includes(0)) throw new Error("onreadystatechange should not be called for abort() when send() was not called");
            } catch (e) {
                globalThis.test_error = "JS_FAIL: " + e.message + "\nStack:\n" + e.stack;
            }
        "#;

        let res = context.eval(Source::from_bytes(script.as_bytes()));
        assert!(res.is_ok(), "Evaluation itself failed: {:?}", res);

        let error_val = context
            .eval(Source::from_bytes("globalThis.test_error".as_bytes()))
            .expect("Failed to get globalThis.test_error");

        if !error_val.is_null() {
            let error_str = error_val.as_string().unwrap().to_std_string_escaped();
            panic!(
                "test_xhr_surface_compliance JS assert failed: {}",
                error_str
            );
        }
    }

    #[test]
    fn test_xhr_extended_features() {
        let mut context = Context::default();
        register_xhr(&mut context).expect("Failed to register XMLHttpRequest");

        let script = r#"
            globalThis.test_error = null;
            try {
                const xhr = new XMLHttpRequest();

                // 1. Check responseType validation and assignment
                if (xhr.responseType !== "") throw new Error("responseType default should be empty");
                xhr.responseType = "json";
                if (xhr.responseType !== "json") throw new Error("responseType should have updated to json");
                xhr.responseType = "invalid-type";
                if (xhr.responseType !== "json") throw new Error("invalid-type should have been ignored, keeping json");
                xhr.responseType = "text";
                if (xhr.responseType !== "text") throw new Error("responseType should have updated to text");

                // 2. Check setRequestHeader validation before open() [should throw InvalidStateError]
                try {
                    xhr.setRequestHeader("Content-Type", "application/json");
                    throw new Error("setRequestHeader should have thrown InvalidStateError before open()");
                } catch (e) {
                    if (e.name !== "InvalidStateError") throw e;
                }

                // 3. Open the request and test readyState transition
                xhr.open("POST", "https://example.com");
                if (xhr.readyState !== 1) throw new Error("readyState should be 1 (OPENED)");

                // 4. Test status and statusText read-only properties in OPENED state
                if (xhr.status !== 0) throw new Error("status should be 0 in OPENED state");
                if (xhr.statusText !== "") throw new Error("statusText should be empty in OPENED state");

                // 5. Test setRequestHeader validations
                // Invalid name token
                try {
                    xhr.setRequestHeader("Content Type", "application/json");
                    throw new Error("setRequestHeader should have thrown SyntaxError for space in name");
                } catch (e) {
                    if (e.name !== "SyntaxError") throw e;
                }
                // Invalid value characters (CRLF/control chars)
                try {
                    xhr.setRequestHeader("Content-Type", "application/json\r\nHost: example.com");
                    throw new Error("setRequestHeader should have thrown SyntaxError for CRLF in value");
                } catch (e) {
                    if (e.name !== "SyntaxError") throw e;
                }
                // Forbidden header names should be silently ignored (not throw, not stored)
                xhr.setRequestHeader("Cookie", "foo=bar");
                xhr.setRequestHeader("sec-ch-ua", "browser");
                if (xhr.getResponseHeader("Cookie") !== null) throw new Error("forbidden header Cookie was not ignored");
                if (xhr.getResponseHeader("sec-ch-ua") !== null) throw new Error("forbidden header sec-ch-ua was not ignored");

                // Valid header setting
                xhr.setRequestHeader("X-Custom", "value1");
                xhr.setRequestHeader("X-Custom", "value2");
                if (xhr._headers["x-custom"] !== "value1, value2") {
                    throw new Error("X-Custom value incorrect in _headers: " + xhr._headers["x-custom"]);
                }

                // 6. Test send() state transitions
                let statesSeen = [];
                xhr.onreadystatechange = function() {
                    statesSeen.push(this.readyState);
                };

                // Before sending, set responseType to json
                xhr.responseType = "json";
                
                xhr.send('{"success": true}');

                // It should have transitioned: OPENED(1) -> HEADERS_RECEIVED(2) -> LOADING(3) -> DONE(4)
                if (statesSeen.join(",") !== "2,3,4") {
                    throw new Error("send() readyState transition pattern incorrect: " + statesSeen.join(","));
                }

                // After DONE, status and statusText should be updated
                if (xhr.status !== 200) throw new Error("status should be 200 after DONE");
                if (xhr.statusText !== "OK") throw new Error("statusText should be OK after DONE");

                // After DONE, getResponseHeader should successfully read response headers from _headers
                if (xhr.getResponseHeader("X-Custom") !== "value1, value2") {
                    throw new Error("getResponseHeader incorrect after send: " + xhr.getResponseHeader("X-Custom"));
                }

                // Accessing responseText when responseType is "json" should throw InvalidStateError
                try {
                    const text = xhr.responseText;
                    throw new Error("responseText should throw InvalidStateError when responseType is json");
                } catch (e) {
                    if (e.name !== "InvalidStateError") throw e;
                }

                // response should correctly parse the response body as JSON
                const resp = xhr.response;
                if (resp === null || resp.success !== true) {
                    throw new Error("response did not correctly parse JSON: " + JSON.stringify(resp));
                }

                // Setting responseType after DONE/LOADING should throw InvalidStateError
                try {
                    xhr.responseType = "text";
                    throw new Error("Setting responseType in DONE state should throw InvalidStateError");
                } catch (e) {
                    if (e.name !== "InvalidStateError") throw e;
                }

                // Calling setRequestHeader after send() should throw InvalidStateError
                try {
                    xhr.setRequestHeader("X-Post-Send", "fail");
                    throw new Error("setRequestHeader should throw InvalidStateError after send()");
                } catch (e) {
                    if (e.name !== "InvalidStateError") throw e;
                }

                // Calling send() again should throw InvalidStateError
                try {
                    xhr.send();
                    throw new Error("Calling send() twice should throw InvalidStateError");
                } catch (e) {
                    if (e.name !== "InvalidStateError") throw e;
                }
            } catch (e) {
                globalThis.test_error = "JS_FAIL: " + e.message + "\nStack:\n" + e.stack;
            }
        "#;

        let res = context.eval(Source::from_bytes(script.as_bytes()));
        assert!(res.is_ok(), "Evaluation itself failed: {:?}", res);

        let error_val = context
            .eval(Source::from_bytes("globalThis.test_error".as_bytes()))
            .expect("Failed to get globalThis.test_error");

        if !error_val.is_null() {
            let error_str = error_val.as_string().unwrap().to_std_string_escaped();
            panic!("test_xhr_extended_features JS assert failed: {}", error_str);
        }
    }

    #[test]
    fn test_xhr_completeness() {
        let mut context = Context::default();
        register_xhr(&mut context).expect("Failed to register XMLHttpRequest");

        let script = r#"
            globalThis.test_error = null;
            try {
                // Mock Event if it doesn't exist
                if (typeof Event === "undefined") {
                    globalThis.Event = class Event {
                        constructor(type) {
                            this.type = type;
                        }
                    };
                }

                // Mock DOMParser
                class MockDocument {
                    constructor(text) {
                        this.text = text;
                    }
                }
                globalThis.DOMParser = class DOMParser {
                    parseFromString(text, mime) {
                        if (mime !== "text/xml" && mime !== "text/html") {
                            throw new Error("Invalid mime in parseFromString: " + mime);
                        }
                        return new MockDocument(text);
                    }
                };

                const xhr = new XMLHttpRequest();

                // 1. Verify XMLHttpRequestUpload class is registered on global
                if (typeof XMLHttpRequestUpload === "undefined") {
                    throw new Error("XMLHttpRequestUpload should be defined globally");
                }
                if (!(xhr.upload instanceof XMLHttpRequestUpload)) {
                    throw new Error("xhr.upload should be an instance of XMLHttpRequestUpload");
                }

                // 2. Verify responseURL behavior
                if (xhr.responseURL !== "") throw new Error("responseURL should be empty initially");
                xhr.open("GET", "https://example.com/api/v1");
                if (xhr.responseURL !== "") throw new Error("responseURL should be empty in OPENED state before response received");

                // 3. Verify event listeners & callbacks on both xhr and upload
                let xhrEvents = [];
                let uploadEvents = [];

                xhr.onloadstart = (e) => xhrEvents.push("on_" + e.type);
                xhr.onprogress = (e) => xhrEvents.push("on_" + e.type);
                xhr.onload = (e) => xhrEvents.push("on_" + e.type);
                xhr.onloadend = (e) => xhrEvents.push("on_" + e.type);

                xhr.upload.onloadstart = (e) => uploadEvents.push("on_" + e.type);
                xhr.upload.onprogress = (e) => uploadEvents.push("on_" + e.type);
                xhr.upload.onload = (e) => uploadEvents.push("on_" + e.type);
                xhr.upload.onloadend = (e) => uploadEvents.push("on_" + e.type);

                // Send with body to trigger upload events
                xhr.send("some upload payload");

                if (xhr.responseURL !== "https://example.com/api/v1") {
                    throw new Error("responseURL should be the requested URL after response received");
                }

                // Check xhr events triggered
                const expectedXhr = ["on_loadstart", "on_progress", "on_load", "on_loadend"];
                for (const ev of expectedXhr) {
                    if (!xhrEvents.includes(ev)) {
                        throw new Error("XHR missing expected event: " + ev + ", got: " + JSON.stringify(xhrEvents));
                    }
                }

                // Check upload events triggered
                const expectedUpload = ["on_loadstart", "on_progress", "on_load", "on_loadend"];
                for (const ev of expectedUpload) {
                    if (!uploadEvents.includes(ev)) {
                        throw new Error("Upload missing expected event: " + ev + ", got: " + JSON.stringify(uploadEvents));
                    }
                }

                // 4. Verify responseXML behavior
                // Since responseType is default "", responseXML should parse correctly
                const xmlDoc = xhr.responseXML;
                if (!xmlDoc || xmlDoc.text !== "some upload payload") {
                    throw new Error("responseXML returned incorrect document: " + JSON.stringify(xmlDoc));
                }

                // 5. Verify timeout and withCredentials synchronous throws
                const syncXhr = new XMLHttpRequest();
                syncXhr.open("GET", "https://example.com", false); // async = false

                try {
                    syncXhr.timeout = 1000;
                    throw new Error("syncXhr.timeout setter should throw InvalidAccessError");
                } catch (e) {
                    if (e.name !== "InvalidAccessError") throw e;
                }

                try {
                    syncXhr.withCredentials = true;
                    throw new Error("syncXhr.withCredentials setter should throw InvalidAccessError");
                } catch (e) {
                    if (e.name !== "InvalidAccessError") throw e;
                }

                // 6. Verify withCredentials state-based throws (cannot be set after send/DONE)
                const stateXhr = new XMLHttpRequest();
                stateXhr.open("GET", "https://example.com");
                stateXhr.responseType = "json"; // set before send
                stateXhr.send();

                // Since responseType is "json", responseXML must throw InvalidStateError
                try {
                    const x = stateXhr.responseXML;
                    throw new Error("responseXML should throw InvalidStateError when responseType is json");
                } catch (e) {
                    if (e.name !== "InvalidStateError") throw e;
                }

                try {
                    stateXhr.withCredentials = true;
                    throw new Error("stateXhr.withCredentials setter should throw InvalidStateError after send()");
                } catch (e) {
                    if (e.name !== "InvalidStateError") throw e;
                }

                // 7. Verify overrideMimeType in responseXML
                const mimeXhr = new XMLHttpRequest();
                mimeXhr.open("GET", "https://example.com");
                mimeXhr.overrideMimeType("text/html");
                mimeXhr.responseType = "document";
                mimeXhr.send();
                const mimeDoc = mimeXhr.responseXML;
                if (!mimeDoc) throw new Error("responseXML with overridden mime should work");

                // 8. Verify abort event on both XHR and upload
                let abortXhrEvents = [];
                let abortUploadEvents = [];
                const abortXhr = new XMLHttpRequest();
                abortXhr.onloadstart = (e) => abortXhrEvents.push(e.type);
                abortXhr.onabort = (e) => abortXhrEvents.push(e.type);
                abortXhr.onloadend = (e) => abortXhrEvents.push(e.type);

                abortXhr.upload.onabort = (e) => abortUploadEvents.push(e.type);
                abortXhr.upload.onloadend = (e) => abortUploadEvents.push(e.type);

                abortXhr.open("POST", "https://example.com");
                // send() sets sendFlag to true and dispatches loadstart
                // we abort immediately to simulate cancelation
                // Since our send is synchronous/mock, let's call send, but wait, send() runs synchronously and transitions to DONE(4)
                // To test abort when sendFlag is true, let's design standard event listeners or check that abort dispatches correctly.
                // If we open and then send, send() completes. What if we abort after open but before send?
                // In that case sendFlag is false, so upload events shouldn't fire, but xhr abort should.
                // Let's test both!
                const abortXhr2 = new XMLHttpRequest();
                let abortEvents2 = [];
                abortXhr2.onabort = () => abortEvents2.push("abort");
                abortXhr2.onloadend = () => abortEvents2.push("loadend");
                abortXhr2.open("GET", "https://example.com");
                abortXhr2.abort();
                if (abortEvents2.length !== 0) {
                    throw new Error("abort() before send should not dispatch any events, got: " + JSON.stringify(abortEvents2));
                }

            } catch (e) {
                globalThis.test_error = "JS_FAIL: " + e.message + "\nStack:\n" + e.stack;
            }
        "#;

        let res = context.eval(Source::from_bytes(script.as_bytes()));
        assert!(res.is_ok(), "Evaluation itself failed: {:?}", res);

        let error_val = context
            .eval(Source::from_bytes("globalThis.test_error".as_bytes()))
            .expect("Failed to get globalThis.test_error");

        if !error_val.is_null() {
            let error_str = error_val.as_string().unwrap().to_std_string_escaped();
            panic!("test_xhr_completeness JS assert failed: {}", error_str);
        }
    }

    #[test]
    fn test_xhr_additive_improvements() {
        let mut context = Context::default();
        register_xhr(&mut context).expect("Failed to register XMLHttpRequest");

        let script = r#"
            globalThis.test_error = null;
            try {
                // Mock Event if it doesn't exist
                if (typeof Event === "undefined") {
                    globalThis.Event = class Event {
                        constructor(type) {
                            this.type = type;
                        }
                    };
                }

                // 1. Verify ProgressEvent global constructor and features
                if (typeof ProgressEvent === "undefined") {
                    throw new Error("ProgressEvent should be defined globally");
                }

                const pe = new ProgressEvent("progress", {
                    lengthComputable: true,
                    loaded: 42,
                    total: 100
                });

                if (pe.type !== "progress") throw new Error("ProgressEvent type should be progress");
                if (pe.lengthComputable !== true) throw new Error("ProgressEvent lengthComputable should be true");
                if (pe.loaded !== 42) throw new Error("ProgressEvent loaded should be 42");
                if (pe.total !== 100) throw new Error("ProgressEvent total should be 100");

                const pe2 = new ProgressEvent("load");
                if (pe2.type !== "load") throw new Error("ProgressEvent type should be load");
                if (pe2.lengthComputable !== false) throw new Error("ProgressEvent default lengthComputable should be false");
                if (pe2.loaded !== 0) throw new Error("ProgressEvent default loaded should be 0");
                if (pe2.total !== 0) throw new Error("ProgressEvent default total should be 0");

                // 2. Verify open() HTTP method token validation
                const xhr = new XMLHttpRequest();
                try {
                    xhr.open("GET / HTTP/1.1", "https://example.com");
                    throw new Error("open() with spaces in method should throw SyntaxError");
                } catch (e) {
                    if (e.name !== "SyntaxError") throw e;
                }

                try {
                    xhr.open("M@THOD", "https://example.com");
                    throw new Error("open() with invalid characters in method should throw SyntaxError");
                } catch (e) {
                    if (e.name !== "SyntaxError") throw e;
                }

                // 3. Verify open() forbidden methods check (case-insensitive)
                try {
                    xhr.open("CONNECT", "https://example.com");
                    throw new Error("open() with CONNECT should throw SecurityError");
                } catch (e) {
                    if (e.name !== "SecurityError") throw e;
                }

                try {
                    xhr.open("trace", "https://example.com");
                    throw new Error("open() with TRACE (lowercase) should throw SecurityError");
                } catch (e) {
                    if (e.name !== "SecurityError") throw e;
                }

                try {
                    xhr.open("TRACK", "https://example.com");
                    throw new Error("open() with TRACK should throw SecurityError");
                } catch (e) {
                    if (e.name !== "SecurityError") throw e;
                }

                // 4. Verify overrideMimeType() readyState validation
                const xhr2 = new XMLHttpRequest();
                xhr2.open("POST", "https://example.com");
                // Calling in state 1 (OPENED) should succeed
                xhr2.overrideMimeType("text/html");

                // Send to transition to state 4 (DONE)
                xhr2.send("payload");

                try {
                    xhr2.overrideMimeType("application/json");
                    throw new Error("overrideMimeType() in DONE state should throw InvalidStateError");
                } catch (e) {
                    if (e.name !== "InvalidStateError") throw e;
                }

                // 5. Verify actual events dispatched are ProgressEvents
                let lastProgressEvent = null;
                const xhr3 = new XMLHttpRequest();
                xhr3.onprogress = (e) => {
                    lastProgressEvent = e;
                };

                xhr3.open("POST", "https://example.com");
                xhr3.send("my body");

                if (!lastProgressEvent) {
                    throw new Error("onprogress handler was not invoked");
                }

                if (!(lastProgressEvent instanceof ProgressEvent)) {
                    throw new Error("dispatched progress event should be an instance of ProgressEvent");
                }

                if (lastProgressEvent.lengthComputable !== true) {
                    throw new Error("progress event lengthComputable should be true");
                }

                if (lastProgressEvent.loaded !== 7) { // "my body".length is 7
                    throw new Error("progress event loaded should be 7, got: " + lastProgressEvent.loaded);
                }

                if (lastProgressEvent.total !== 7) {
                    throw new Error("progress event total should be 7, got: " + lastProgressEvent.total);
                }

            } catch (e) {
                globalThis.test_error = "JS_FAIL: " + e.message + "\nStack:\n" + e.stack;
            }
        "#;

        let res = context.eval(Source::from_bytes(script.as_bytes()));
        assert!(res.is_ok(), "Evaluation itself failed: {:?}", res);

        let error_val = context
            .eval(Source::from_bytes("globalThis.test_error".as_bytes()))
            .expect("Failed to get globalThis.test_error");

        if !error_val.is_null() {
            let error_str = error_val.as_string().unwrap().to_std_string_escaped();
            panic!(
                "test_xhr_additive_improvements JS assert failed: {}",
                error_str
            );
        }
    }

    #[test]
    fn test_xhr_new_gaps() {
        let mut context = Context::default();
        register_xhr(&mut context).expect("Failed to register XMLHttpRequest");

        // We also need to mock DOMParser and Blob if we test document or blob
        let setup_script = r#"
            // Mock Event if it doesn't exist
            if (typeof Event === "undefined") {
                globalThis.Event = class Event {
                    constructor(type) {
                        this.type = type;
                    }
                };
            }

            // Mock DOMParser
            class MockDocument {
                constructor(text) {
                    this.text = text;
                }
            }
            globalThis.DOMParser = class DOMParser {
                parseFromString(text, mime) {
                    return new MockDocument(text);
                }
            };
            
            // Mock Blob if undefined
            if (typeof Blob === "undefined") {
                globalThis.Blob = class Blob {
                    constructor(parts, options = {}) {
                        this.parts = parts;
                        this.type = options.type || "";
                    }
                };
            }
        "#;
        context
            .eval(Source::from_bytes(setup_script.as_bytes()))
            .unwrap();

        let script = r#"
            globalThis.test_error = null;
            try {
                // 1. Test responseType document, arraybuffer, blob
                const xhr1 = new XMLHttpRequest();
                xhr1.open("GET", "https://example.com/api#foo");
                
                // Set non-text response types
                xhr1.responseType = "arraybuffer";
                
                // Should return null for non-text before DONE (even when states 2/3 are reached, but let's test in OPENED)
                if (xhr1.response !== null) {
                    throw new Error("response should be null in OPENED state for arraybuffer");
                }
                
                xhr1.send("hello");
                
                // Now state is DONE. response should return the ArrayBuffer
                const buf = xhr1.response;
                if (!(buf instanceof ArrayBuffer)) {
                    throw new Error("expected ArrayBuffer for arraybuffer responseType");
                }
                if (buf.byteLength !== 5) {
                    throw new Error("ArrayBuffer byteLength should be 5, got: " + buf.byteLength);
                }
                const view = new Uint8Array(buf);
                if (view[0] !== 104 || view[4] !== 111) { // "h" and "o"
                    throw new Error("ArrayBuffer content is incorrect");
                }
                
                // responseURL fragment strip
                if (xhr1.responseURL !== "https://example.com/api") {
                    throw new Error("expected responseURL to strip fragment, got: " + xhr1.responseURL);
                }

                // 2. Test responseType blob
                const xhr2 = new XMLHttpRequest();
                xhr2.open("GET", "https://example.com");
                xhr2.responseType = "blob";
                xhr2.send("blobData");
                const blob = xhr2.response;
                if (!blob) {
                    throw new Error("expected Blob for blob responseType");
                }

                // 3. Test responseType document
                const xhr3 = new XMLHttpRequest();
                xhr3.open("GET", "https://example.com");
                xhr3.responseType = "document";
                xhr3.send("docData");
                const doc = xhr3.response;
                if (!doc || !(doc instanceof MockDocument) || doc.text !== "docData") {
                    throw new Error("expected MockDocument for document responseType");
                }

                // 4. Test getAllResponseHeaders sorted lexicographically
                const xhr4 = new XMLHttpRequest();
                xhr4.open("GET", "https://example.com");
                xhr4.setRequestHeader("Z-Header", "z-val");
                xhr4.setRequestHeader("A-Header", "a-val");
                xhr4.setRequestHeader("M-Header", "m-val");
                xhr4.send();
                
                const headersStr = xhr4.getAllResponseHeaders();
                const expected = "a-header: a-val\r\nm-header: m-val\r\nz-header: z-val\r\n";
                if (headersStr !== expected) {
                    throw new Error("getAllResponseHeaders not sorted correctly, got: " + JSON.stringify(headersStr));
                }

                // 5. Test _simulateTimeout() helper
                const xhr7 = new XMLHttpRequest();
                let timeoutFired = false;
                let loadendFired = false;
                xhr7.ontimeout = () => { timeoutFired = true; };
                xhr7.onloadend = () => { loadendFired = true; };
                xhr7.open("GET", "https://example.com");
                
                xhr7._simulateTimeout();
                if (!timeoutFired || !loadendFired) {
                    throw new Error("expected timeout and loadend to fire");
                }
                if (xhr7.readyState !== 4) {
                    throw new Error("readyState should be DONE (4) after timeout, got: " + xhr7.readyState);
                }

            } catch (e) {
                globalThis.test_error = "JS_FAIL: " + e.message + "\nStack:\n" + e.stack;
            }
        "#;

        let res = context.eval(Source::from_bytes(script.as_bytes()));
        assert!(res.is_ok(), "Evaluation itself failed: {:?}", res);

        let error_val = context
            .eval(Source::from_bytes("globalThis.test_error".as_bytes()))
            .expect("Failed to get globalThis.test_error");

        if !error_val.is_null() {
            let error_str = error_val.as_string().unwrap().to_std_string_escaped();
            panic!("test_xhr_new_gaps JS assert failed: {}", error_str);
        }
    }

    #[test]
    fn test_xhr_surgical_gaps() {
        let mut context = Context::default();
        register_xhr(&mut context).expect("Failed to register XMLHttpRequest");

        let script = r#"
            globalThis.test_error = null;
            try {
                // 1. Verify trimming in setRequestHeader
                const xhr = new XMLHttpRequest();
                xhr.open("GET", "https://example.com");
                xhr.setRequestHeader("X-Trimmed", "  my trimmed value  \r\n ");
                if (xhr._headers["x-trimmed"] !== "my trimmed value") {
                    throw new Error("expected header value to be trimmed, got: " + JSON.stringify(xhr._headers["x-trimmed"]));
                }

                // 2. Verify synchronous responseType restriction
                const syncXhr = new XMLHttpRequest();
                syncXhr.open("GET", "https://example.com", false);
                
                // setting responseType on sync XHR should throw InvalidAccessError
                try {
                    syncXhr.responseType = "json";
                    throw new Error("setting responseType on sync XHR did not throw");
                } catch (e) {
                    if (e.name !== "InvalidAccessError") throw e;
                }
                try {
                    syncXhr.responseType = "text";
                    throw new Error("setting responseType on sync XHR did not throw");
                } catch (e) {
                    if (e.name !== "InvalidAccessError") throw e;
                }
            } catch (e) {
                globalThis.test_error = "JS_FAIL: " + e.message + "\nStack:\n" + e.stack;
            }
        "#;

        let res = context.eval(Source::from_bytes(script.as_bytes()));
        assert!(res.is_ok(), "Evaluation itself failed: {:?}", res);

        let error_val = context
            .eval(Source::from_bytes("globalThis.test_error".as_bytes()))
            .expect("Failed to get globalThis.test_error");

        if !error_val.is_null() {
            let error_str = error_val.as_string().unwrap().to_std_string_escaped();
            panic!("test_xhr_surgical_gaps JS assert failed: {}", error_str);
        }
    }

    #[test]
    fn test_xhr_compliance_t0987() {
        let mut context = Context::default();
        register_xhr(&mut context).expect("Failed to register XMLHttpRequest");

        // Mock Event if it doesn't exist
        let setup_script = r#"
            if (typeof Event === "undefined") {
                globalThis.Event = class Event {
                    constructor(type) {
                        this.type = type;
                    }
                };
            }
        "#;
        context
            .eval(Source::from_bytes(setup_script.as_bytes()))
            .unwrap();

        let script = r#"
            globalThis.test_error = null;
            try {
                // 1. Verify abort() during active request (with sendFlag set)
                const xhr1 = new XMLHttpRequest();
                let eventsFired1 = [];
                let stateSequence1 = [];
                
                xhr1.onreadystatechange = () => {
                    stateSequence1.push(xhr1.readyState);
                };
                xhr1.onabort = () => { eventsFired1.push("abort"); };
                xhr1.onloadend = () => { eventsFired1.push("loadend"); };

                xhr1.open("POST", "https://example.com");
                // Reset/clear sequence for state after open
                stateSequence1 = [];

                // Call send() which will transition to 2, 3, 4 synchronously under our mock.
                // But wait! To simulate aborting an *active* request (where sendFlag is true),
                // we can intercept the readyState transitions!
                // Let's call abort() inside a readyState event handler when state is 2 or 3!
                xhr1.onreadystatechange = () => {
                    stateSequence1.push(xhr1.readyState);
                    if (xhr1.readyState === 2) {
                        xhr1.abort();
                    }
                };

                xhr1.send("my upload body");

                // Under WHATWG:
                // When abort is called in state 2:
                // 1) changeReadyState(4) is called, transition state 2 -> 4. So onreadystatechange fires with state 4.
                // 2) abort/loadend events are fired on upload & XHR.
                // 3) state becomes UNSENT (0) directly (no extra event).
                // So the stateSequence should end up with:
                // - [2] from send() HEADERS_RECEIVED
                // - [4] from abort() changeReadyState(4)
                // And total events fired on XHR should include "abort" and "loadend".
                if (stateSequence1.join(",") !== "2,4") {
                    throw new Error("abort in active state should transition 2 -> 4. Sequence was: " + stateSequence1.join(","));
                }
                if (xhr1.readyState !== 0) {
                    throw new Error("XHR readyState after abort should be 0 (UNSENT)");
                }
                if (!eventsFired1.includes("abort") || !eventsFired1.includes("loadend")) {
                    throw new Error("Active abort should fire abort and loadend events. Got: " + JSON.stringify(eventsFired1));
                }

                // 2. Verify abort() when state is DONE (4)
                const xhr2 = new XMLHttpRequest();
                let eventsFired2 = [];
                let stateSequence2 = [];
                
                xhr2.open("GET", "https://example.com");
                xhr2.send();
                
                // Now state is DONE (4). Let's attach abort handlers.
                xhr2.onreadystatechange = () => { stateSequence2.push(xhr2.readyState); };
                xhr2.onabort = () => { eventsFired2.push("abort"); };
                xhr2.onloadend = () => { eventsFired2.push("loadend"); };

                xhr2.abort();

                // On DONE(4), abort() should silently transition state to 0 and fire no events.
                if (xhr2.readyState !== 0) {
                    throw new Error("XHR readyState after aborting DONE should be 0");
                }
                if (stateSequence2.length !== 0) {
                    throw new Error("Aborting DONE should not fire readystatechange, got: " + JSON.stringify(stateSequence2));
                }
                if (eventsFired2.length !== 0) {
                    throw new Error("Aborting DONE should not fire any abort/loadend events, got: " + JSON.stringify(eventsFired2));
                }

                // 3. Verify getAllResponseHeaders() excludes Set-Cookie and Set-Cookie2 case-insensitively
                const xhr3 = new XMLHttpRequest();
                xhr3.open("GET", "https://example.com");
                xhr3.setRequestHeader("Set-Cookie", "mycookie=123");
                xhr3.setRequestHeader("set-cookie2", "mycookie2=456");
                xhr3.setRequestHeader("X-Custom", "allowed");
                xhr3.send();
                
                const headers = xhr3.getAllResponseHeaders();
                if (headers.includes("set-cookie") || headers.includes("Set-Cookie")) {
                    throw new Error("getAllResponseHeaders must exclude Set-Cookie, got: " + JSON.stringify(headers));
                }
                if (headers.includes("set-cookie2")) {
                    throw new Error("getAllResponseHeaders must exclude Set-Cookie2, got: " + JSON.stringify(headers));
                }
                if (!headers.includes("x-custom: allowed")) {
                    throw new Error("getAllResponseHeaders should include allowed headers, got: " + JSON.stringify(headers));
                }

                // 4. Verify accessors responseText and response return empty string or null before LOADING (3)
                const xhr4 = new XMLHttpRequest();
                xhr4.open("GET", "https://example.com");
                
                // State is OPENED (1)
                if (xhr4.responseText !== "") {
                    throw new Error("responseText should return empty string in OPENED state");
                }
                if (xhr4.response !== "") {
                    throw new Error("response should return empty string in OPENED state (for default text responseType)");
                }

                xhr4.responseType = "json";
                if (xhr4.response !== null) {
                    throw new Error("response should return null in OPENED state for non-text responseType");
                }

            } catch (e) {
                globalThis.test_error = "JS_FAIL: " + e.message + "\nStack:\n" + e.stack;
            }
        "#;

        let res = context.eval(Source::from_bytes(script.as_bytes()));
        assert!(res.is_ok(), "Evaluation itself failed: {:?}", res);

        let error_val = context
            .eval(Source::from_bytes("globalThis.test_error".as_bytes()))
            .expect("Failed to get globalThis.test_error");

        if !error_val.is_null() {
            let error_str = error_val.as_string().unwrap().to_std_string_escaped();
            panic!("test_xhr_compliance_t0987 JS assert failed: {}", error_str);
        }
    }

    #[test]
    fn test_xhr_ms4_extended_coverage() {
        let mut context = Context::default();
        register_xhr(&mut context).expect("Failed to register XMLHttpRequest");

        // Mock Event if it doesn't exist
        let setup_script = r#"
            if (typeof Event === "undefined") {
                globalThis.Event = class Event {
                    constructor(type) {
                        this.type = type;
                    }
                };
            }
        "#;
        context
            .eval(Source::from_bytes(setup_script.as_bytes()))
            .unwrap();

        let script = r#"
            globalThis.test_error = null;
            try {
                // 1. Verify withCredentials and responseType are reset on open()
                const xhr1 = new XMLHttpRequest();
                xhr1.open("GET", "https://example.com");
                xhr1.withCredentials = true;
                xhr1.responseType = "json";
                
                // second open should reset
                xhr1.open("GET", "https://example.com");
                if (xhr1.withCredentials !== false) {
                    throw new Error("withCredentials should be reset to false on open()");
                }
                if (xhr1.responseType !== "") {
                    throw new Error("responseType should be reset to empty string on open()");
                }

                // 2. Verify status and statusText are correct during HEADERS_RECEIVED transition
                const xhr2 = new XMLHttpRequest();
                let statusInState2 = null;
                let statusTextInState2 = null;
                xhr2.onreadystatechange = () => {
                    if (xhr2.readyState === 2) {
                        statusInState2 = xhr2.status;
                        statusTextInState2 = xhr2.statusText;
                    }
                };
                xhr2.open("GET", "https://example.com");
                xhr2.send();
                if (statusInState2 !== 200) {
                    throw new Error("status should be set to 200 during HEADERS_RECEIVED, got: " + statusInState2);
                }
                if (statusTextInState2 !== "OK") {
                    throw new Error("statusText should be 'OK' during HEADERS_RECEIVED, got: " + statusTextInState2);
                }

                // 3. Verify _parseResponseHeaders parses and getAllResponseHeaders works correctly
                const xhr3 = new XMLHttpRequest();
                xhr3.open("GET", "https://example.com");
                xhr3._parseResponseHeaders("Content-Type: text/plain\r\nCache-Control: public, max-age=3600\r\nSet-Cookie: dummy\r\n");
                xhr3.send();
                
                const headers = xhr3.getAllResponseHeaders();
                const expectedHeaders = "cache-control: public, max-age=3600\r\ncontent-type: text/plain\r\n";
                if (headers !== expectedHeaders) {
                    throw new Error("getAllResponseHeaders or _parseResponseHeaders mismatch: " + JSON.stringify(headers));
                }

                // 4. Verify _simulateNetworkError fires error/loadend events on XHR and upload
                const xhr4 = new XMLHttpRequest();
                let xhrErr = false;
                let xhrLoadend = false;
                let uploadErr = false;
                let uploadLoadend = false;
                
                xhr4.onerror = () => { xhrErr = true; };
                xhr4.onloadend = () => { xhrLoadend = true; };
                xhr4.upload.onerror = () => { uploadErr = true; };
                xhr4.upload.onloadend = () => { uploadLoadend = true; };
                
                xhr4.open("POST", "https://example.com");
                xhr4._simulateNetworkError();
                
                if (!xhrErr || !xhrLoadend) {
                    throw new Error("XHR onerror or onloadend event failed to trigger");
                }
                if (!uploadErr || !uploadLoadend) {
                    throw new Error("XHR upload onerror or onloadend event failed to trigger");
                }
                if (xhr4.readyState !== 4) {
                    throw new Error("XHR readyState should be 4 (DONE) after simulated network error");
                }

                // 5. Verify responseXML throws InvalidStateError when responseType is incompatible
                const xhr5 = new XMLHttpRequest();
                xhr5.open("GET", "https://example.com");
                xhr5.responseType = "arraybuffer";
                
                try {
                    const doc = xhr5.responseXML;
                    throw new Error("responseXML getter should have thrown InvalidStateError");
                } catch (e) {
                    if (e.name !== "InvalidStateError") throw e;
                }

            } catch (e) {
                globalThis.test_error = "JS_FAIL: " + e.message + "\nStack:\n" + e.stack;
            }
        "#;

        let res = context.eval(Source::from_bytes(script.as_bytes()));
        assert!(res.is_ok(), "Evaluation itself failed: {:?}", res);

        let error_val = context
            .eval(Source::from_bytes("globalThis.test_error".as_bytes()))
            .expect("Failed to get globalThis.test_error");

        if !error_val.is_null() {
            let error_str = error_val.as_string().unwrap().to_std_string_escaped();
            panic!(
                "test_xhr_ms4_extended_coverage JS assert failed: {}",
                error_str
            );
        }
    }

    #[test]
    fn test_xhr_ms4_improved_spec_conformity() {
        let mut context = Context::default();
        register_xhr(&mut context).expect("Failed to register XMLHttpRequest");

        // Mock Event if it doesn't exist
        let setup_script = r#"
            if (typeof Event === "undefined") {
                globalThis.Event = class Event {
                    constructor(type) {
                        this.type = type;
                    }
                };
            }
        "#;
        context
            .eval(Source::from_bytes(setup_script.as_bytes()))
            .unwrap();

        let script = r#"
            globalThis.test_error = null;
            try {
                // 1. Verify responseType setter on sync XHR
                const syncXhr1 = new XMLHttpRequest();
                syncXhr1.open("GET", "https://example.com", false); // sync = false
                
                // Should NOT throw when setting to ""
                syncXhr1.responseType = "";
                
                // Should throw InvalidAccessError when setting to non-empty
                try {
                    syncXhr1.responseType = "json";
                    throw new Error("setting responseType to json on sync XHR should throw InvalidAccessError");
                } catch (e) {
                    if (e.name !== "InvalidAccessError") throw e;
                }

                try {
                    syncXhr1.responseType = "text";
                    throw new Error("setting responseType to text on sync XHR should throw InvalidAccessError");
                } catch (e) {
                    if (e.name !== "InvalidAccessError") throw e;
                }

                // 2. Verify Set-Cookie and Set-Cookie2 case-insensitively return null in getResponseHeader
                const xhr1 = new XMLHttpRequest();
                xhr1.open("GET", "https://example.com");
                xhr1._parseResponseHeaders("Content-Type: text/plain\r\nSet-Cookie: foo=bar\r\nset-cookie2: abc=123\r\n");
                xhr1.send();

                if (xhr1.getResponseHeader("Set-Cookie") !== null) {
                    throw new Error("getResponseHeader('Set-Cookie') must return null");
                }
                if (xhr1.getResponseHeader("set-cookie") !== null) {
                    throw new Error("getResponseHeader('set-cookie') must return null");
                }
                if (xhr1.getResponseHeader("Set-Cookie2") !== null) {
                    throw new Error("getResponseHeader('Set-Cookie2') must return null");
                }
                if (xhr1.getResponseHeader("set-cookie2") !== null) {
                    throw new Error("getResponseHeader('set-cookie2') must return null");
                }
                if (xhr1.getResponseHeader("Content-Type") !== "text/plain") {
                    throw new Error("getResponseHeader('Content-Type') returned incorrect value: " + xhr1.getResponseHeader("Content-Type"));
                }

                // 3. Verify readyState and events for synchronous requests
                const syncXhr2 = new XMLHttpRequest();
                let syncEvents = [];
                let syncStates = [];

                syncXhr2.onreadystatechange = () => {
                    syncStates.push(syncXhr2.readyState);
                };
                syncXhr2.onloadstart = () => { syncEvents.push("loadstart"); };
                syncXhr2.onprogress = () => { syncEvents.push("progress"); };
                syncXhr2.onload = () => { syncEvents.push("load"); };
                syncXhr2.onloadend = () => { syncEvents.push("loadend"); };

                syncXhr2.open("GET", "https://example.com", false); // sync = false
                // Note: open() transitions from UNSENT(0) to OPENED(1) and fires onreadystatechange
                // Let's clear states and events tracking before sending
                syncStates = [];
                syncEvents = [];

                syncXhr2.send();

                // Under WHATWG:
                // For synchronous XHR, we should not fire any progress events,
                // and we should not fire readystatechange events except when transitioning to DONE (4).
                if (syncEvents.length > 0) {
                    throw new Error("Synchronous XHR must not dispatch progress events, but got: " + JSON.stringify(syncEvents));
                }
                if (syncStates.join(",") !== "4") {
                    throw new Error("Synchronous XHR readystatechange sequence should contain only DONE (4), but got: " + syncStates.join(","));
                }

            } catch (e) {
                globalThis.test_error = "JS_FAIL: " + e.message + "\nStack:\n" + e.stack;
            }
        "#;

        let res = context.eval(Source::from_bytes(script.as_bytes()));
        assert!(res.is_ok(), "Evaluation itself failed: {:?}", res);

        let error_val = context
            .eval(Source::from_bytes("globalThis.test_error".as_bytes()))
            .expect("Failed to get globalThis.test_error");

        if !error_val.is_null() {
            let error_str = error_val.as_string().unwrap().to_std_string_escaped();
            panic!(
                "test_xhr_ms4_improved_spec_conformity JS assert failed: {}",
                error_str
            );
        }
    }

    #[test]
    fn test_xhr_t1043_improvements() {
        use crate::script::{BoaHost, ScriptHost};
        let mut host = BoaHost::new();

        let script = r#"
            globalThis.test_error = null;
            try {
                // 1. Verify readystatechange passing the event argument
                const xhr1 = new XMLHttpRequest();
                let lastEvent = null;
                xhr1.onreadystatechange = (e) => {
                    lastEvent = e;
                };

                xhr1.open("GET", "https://example.com");
                if (!lastEvent) {
                    throw new Error("onreadystatechange was not called on open()");
                }
                if (lastEvent.type !== "readystatechange") {
                    throw new Error("Expected event type 'readystatechange', got: " + lastEvent.type);
                }

                // 2. Verify addEventListener("readystatechange")
                let listenerCalled = false;
                xhr1.addEventListener("readystatechange", (e) => {
                    listenerCalled = true;
                });

                // Reset lastEvent
                lastEvent = null;
                xhr1.send();

                if (!listenerCalled) {
                    throw new Error("Event listener added via addEventListener was not called");
                }
                if (!lastEvent) {
                    throw new Error("onreadystatechange was not called on send()");
                }

                // 3. Verify FormData serialization in send()
                const xhr2 = new XMLHttpRequest();
                xhr2.open("POST", "https://example.com");
                const fd = new FormData();
                fd.append("username", "testuser");
                fd.append("email", "test@example.com");

                xhr2.send(fd);

                const ct = xhr2.getResponseHeader("Content-Type");
                if (!ct || !ct.includes("multipart/form-data; boundary=")) {
                    throw new Error("Content-Type header for FormData should be multipart/form-data with a boundary, got: " + ct);
                }

                const bodyText = xhr2.responseText;
                if (!bodyText.includes("Content-Disposition: form-data; name=\"username\"") || !bodyText.includes("testuser")) {
                    throw new Error("FormData payload not serialized correctly: " + bodyText);
                }
                if (!bodyText.includes("Content-Disposition: form-data; name=\"email\"") || !bodyText.includes("test@example.com")) {
                    throw new Error("FormData payload not serialized correctly: " + bodyText);
                }

                // 4. Verify URLSearchParams serialization in send()
                const xhr3 = new XMLHttpRequest();
                xhr3.open("POST", "https://example.com");
                const params = new URLSearchParams();
                params.append("foo", "bar");
                params.append("abc", "123");

                xhr3.send(params);

                const ct3 = xhr3.getResponseHeader("Content-Type");
                if (ct3 !== "application/x-www-form-urlencoded;charset=UTF-8") {
                    throw new Error("Content-Type header for URLSearchParams should be application/x-www-form-urlencoded;charset=UTF-8, got: " + ct3);
                }
                if (xhr3.responseText !== "foo=bar&abc=123") {
                    throw new Error("URLSearchParams payload not serialized correctly: " + xhr3.responseText);
                }

                // 5. Verify ArrayBuffer serialization and roundtrip
                const xhr4 = new XMLHttpRequest();
                xhr4.open("POST", "https://example.com");
                xhr4.responseType = "arraybuffer";

                const arrayBuf = new ArrayBuffer(4);
                const view = new Uint8Array(arrayBuf);
                view[0] = 65; view[1] = 66; view[2] = 67; view[3] = 68; // "ABCD"

                xhr4.send(arrayBuf);

                const respBuf = xhr4.response;
                if (!(respBuf instanceof ArrayBuffer)) {
                    throw new Error("Expected response to be ArrayBuffer");
                }
                const respView = new Uint8Array(respBuf);
                if (respView[0] !== 65 || respView[1] !== 66 || respView[2] !== 67 || respView[3] !== 68) {
                    throw new Error("ArrayBuffer content not roundtripped correctly");
                }

                // 6. Verify TypedArray view serialization and roundtrip
                const xhr5 = new XMLHttpRequest();
                xhr5.open("POST", "https://example.com");
                xhr5.responseType = "arraybuffer";

                const typedArray = new Uint8Array([72, 69, 76, 76, 79]); // "HELLO"
                xhr5.send(typedArray);

                const respBuf5 = xhr5.response;
                if (!(respBuf5 instanceof ArrayBuffer)) {
                    throw new Error("Expected response to be ArrayBuffer");
                }
                const respView5 = new Uint8Array(respBuf5);
                if (respView5[0] !== 72 || respView5[1] !== 69 || respView5[2] !== 76 || respView5[3] !== 76 || respView5[4] !== 79) {
                    throw new Error("TypedArray content not roundtripped correctly");
                }

            } catch (e) {
                globalThis.test_error = "JS_FAIL: " + e.message + "\nStack:\n" + e.stack;
            }
        "#;

        host.eval(script).expect("Execution failed");

        let error_val = host
            .context
            .eval(boa_engine::Source::from_bytes(
                "globalThis.test_error".as_bytes(),
            ))
            .expect("Failed to get globalThis.test_error");

        if !error_val.is_null() {
            let error_str = error_val.as_string().unwrap().to_std_string_escaped();
            panic!(
                "test_xhr_t1043_improvements JS assert failed: {}",
                error_str
            );
        }
    }
}
