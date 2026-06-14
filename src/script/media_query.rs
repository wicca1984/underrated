//! Implementation of the `window.matchMedia()` Web API.
//!
//! Spec: <https://drafts.csswg.org/cssom-view/#dom-window-matchmedia>

use boa_engine::object::ObjectInitializer;
use boa_engine::property::Attribute;
use boa_engine::{Context, JsError, JsString, JsValue, NativeFunction};

/// Native implementation of the `noop_listener` for `MediaQueryList`.
///
/// Accepts a callback or options and does nothing.
fn noop_listener(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> Result<JsValue, JsError> {
    Ok(JsValue::undefined())
}

/// Native implementation of `window.matchMedia(query)`.
///
/// Spec: <https://drafts.csswg.org/cssom-view/#dom-window-matchmedia>
pub fn match_media(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    // 1. Get query string, default to empty string if not provided.
    let query_str = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        String::new()
    };

    // 2. Evaluate query against current viewport width (default 1280 matching screen.width).
    // TODO(spec): Event firing on viewport change is out of scope. Use 1280 matching screen width.
    let matches = crate::css::media::media_matches(&query_str, 1280.0);

    // 3. Create MediaQueryList-like object.
    let mql = ObjectInitializer::new(context)
        .property(JsString::from("matches"), matches, Attribute::all())
        .property(
            JsString::from("media"),
            JsString::from(query_str),
            Attribute::all(),
        )
        .property(
            JsString::from("onchange"),
            JsValue::null(),
            Attribute::all(),
        )
        .function(
            NativeFunction::from_fn_ptr(noop_listener),
            JsString::from("addListener"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(noop_listener),
            JsString::from("removeListener"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(noop_listener),
            JsString::from("addEventListener"),
            2,
        )
        .function(
            NativeFunction::from_fn_ptr(noop_listener),
            JsString::from("removeEventListener"),
            2,
        )
        .build();

    Ok(JsValue::from(mql))
}

#[cfg(test)]
mod tests {
    use crate::script::{BoaHost, ScriptHost};

    #[test]
    fn test_media_query_onchange() {
        let mut host = BoaHost::new();

        // 1. Verify onchange defaults to null
        host.eval("if (matchMedia('(min-width: 1000px)').onchange !== null) throw 'onchange should default to null';").unwrap();

        // 2. Verify we can assign a function and read it back
        let assign_test = r#"{
            const mql = matchMedia('(min-width: 1000px)');
            const cb = () => {};
            mql.onchange = cb;
            if (mql.onchange !== cb) throw 'onchange setter/getter identity mismatch';
            if (typeof mql.onchange !== 'function') throw 'onchange should be a function';
        }"#;
        host.eval(assign_test).unwrap();

        // 3. Verify assigning null clears it
        let clear_test = r#"{
            const mql = matchMedia('(min-width: 1000px)');
            mql.onchange = () => {};
            mql.onchange = null;
            if (mql.onchange !== null) throw 'onchange should be cleared to null';
        }"#;
        host.eval(clear_test).unwrap();
    }
}
