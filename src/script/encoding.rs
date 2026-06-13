//! Implementation of the WHATWG Encoding API `TextEncoder` and `TextDecoder` for Boa.
//!
//! Spec: <https://encoding.spec.whatwg.org/#interface-textencoder>
//! Spec: <https://encoding.spec.whatwg.org/#interface-textdecoder>

use boa_engine::class::{Class, ClassBuilder};
use boa_engine::object::FunctionObjectBuilder;
use boa_engine::property::Attribute;
use boa_engine::{
    Context, JsData, JsError, JsNativeError, JsResult, JsString, JsValue, NativeFunction,
};
use boa_gc::{Finalize, Trace};

/// Implementation of the WHATWG `TextEncoder` interface.
#[derive(Debug, Trace, Finalize, JsData)]
pub struct TextEncoder {}

impl Class for TextEncoder {
    const NAME: &'static str = "TextEncoder";
    const LENGTH: usize = 0;

    fn data_constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<Self> {
        Ok(TextEncoder {})
    }

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        let realm = class.context().realm().clone();

        let get_encoding_fn = FunctionObjectBuilder::new(
            &realm,
            NativeFunction::from_fn_ptr(text_encoder_get_encoding),
        )
        .name("get encoding")
        .build();

        class
            .accessor(
                JsString::from("encoding"),
                Some(get_encoding_fn),
                None,
                Attribute::all(),
            )
            .method(
                JsString::from("encode"),
                1,
                NativeFunction::from_fn_ptr(text_encoder_encode),
            );

        Ok(())
    }
}

fn text_encoder_get_encoding(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let _encoder = obj.downcast_ref::<TextEncoder>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-TextEncoder object"))
    })?;
    Ok(JsValue::from(JsString::from("utf-8")))
}

fn text_encoder_encode(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let _encoder = obj.downcast_ref::<TextEncoder>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-TextEncoder object"))
    })?;

    let input_val = args.first().cloned().unwrap_or(JsValue::undefined());
    let input_str = if input_val.is_undefined() {
        String::new()
    } else {
        input_val
            .to_string(context)?
            .to_std_string()
            .unwrap_or_default()
    };

    let bytes = input_str.into_bytes();

    // Dynamically look up and construct standard JS Uint8Array
    let uint8_array_constructor = context
        .global_object()
        .get(JsString::from("Uint8Array"), context)?;
    let uint8_array_obj = uint8_array_constructor.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Uint8Array constructor not found"))
    })?;

    // Create standard Array to pass to the Uint8Array constructor
    let array_constructor = context
        .global_object()
        .get(JsString::from("Array"), context)?;
    let array_obj = array_constructor.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Array constructor not found"))
    })?;
    let js_array = array_obj.construct(&[], None, context)?;
    let push_fn = js_array.get(JsString::from("push"), context)?;
    if let Some(push_obj) = push_fn.as_object() {
        for byte in bytes {
            push_obj.call(
                &JsValue::from(js_array.clone()),
                &[JsValue::from(byte)],
                context,
            )?;
        }
    }

    let uint8_array_instance =
        uint8_array_obj.construct(&[JsValue::from(js_array)], None, context)?;
    Ok(JsValue::from(uint8_array_instance))
}

/// Implementation of the WHATWG `TextDecoder` interface.
#[derive(Debug, Trace, Finalize, JsData)]
pub struct TextDecoder {
    pub(crate) encoding: String,
}

impl Class for TextDecoder {
    const NAME: &'static str = "TextDecoder";
    const LENGTH: usize = 0;

    fn data_constructor(
        _new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<Self> {
        let label = if let Some(val) = args.first() {
            val.to_string(context)?
                .to_std_string()
                .unwrap_or_else(|_| "utf-8".to_string())
        } else {
            "utf-8".to_string()
        };

        // Only "utf-8" needs to work; for any non-utf-8 label leave a TODO and still treat it as utf-8.
        if label != "utf-8" && label != "utf8" {
            // TODO(spec): only utf-8 decoding is supported
        }

        Ok(TextDecoder {
            encoding: "utf-8".to_string(),
        })
    }

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        let realm = class.context().realm().clone();

        let get_encoding_fn = FunctionObjectBuilder::new(
            &realm,
            NativeFunction::from_fn_ptr(text_decoder_get_encoding),
        )
        .name("get encoding")
        .build();

        class
            .accessor(
                JsString::from("encoding"),
                Some(get_encoding_fn),
                None,
                Attribute::all(),
            )
            .method(
                JsString::from("decode"),
                1,
                NativeFunction::from_fn_ptr(text_decoder_decode),
            );

        Ok(())
    }
}

fn text_decoder_get_encoding(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let decoder = obj.downcast_ref::<TextDecoder>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-TextDecoder object"))
    })?;
    Ok(JsValue::from(JsString::from(decoder.encoding.clone())))
}

fn text_decoder_decode(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let _decoder = obj.downcast_ref::<TextDecoder>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-TextDecoder object"))
    })?;

    let input_val = args.first().cloned().unwrap_or(JsValue::undefined());
    if input_val.is_undefined() || input_val.is_null() {
        return Ok(JsValue::from(JsString::from("")));
    }

    let mut bytes = Vec::new();
    if let Some(obj) = input_val.as_object() {
        let length_val = obj.get(JsString::from("length"), context)?;
        let length = length_val.as_number().map(|n| n as usize).unwrap_or(0);
        bytes.reserve(length);
        for i in 0..length {
            let val = obj.get(i, context)?;
            let byte = val.as_number().map(|n| n as u8).unwrap_or(0);
            bytes.push(byte);
        }
    }

    let decoded = String::from_utf8_lossy(&bytes);
    Ok(JsValue::from(JsString::from(decoded.into_owned())))
}

/// Registers the WHATWG Encoding API global classes `TextEncoder` and `TextDecoder`.
pub fn register_encoding_builtins(context: &mut Context) -> JsResult<()> {
    context.register_global_class::<TextEncoder>()?;
    context.register_global_class::<TextDecoder>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::Source;

    #[test]
    fn test_text_encoder_encode_t0505() {
        // Encode ASCII "A"
        {
            let mut context = Context::default();
            register_encoding_builtins(&mut context).unwrap();
            let source = Source::from_bytes(
                r#"
                const encoder = new TextEncoder();
                const arr = encoder.encode("A");
                arr.length === 1 && arr[0] === 65
            "#,
            );
            let res = context.eval(source).unwrap();
            assert_eq!(res.as_boolean(), Some(true));
        }

        // Encode Euro sign "€" which is [226, 130, 172] in UTF-8
        {
            let mut context = Context::default();
            register_encoding_builtins(&mut context).unwrap();
            let source = Source::from_bytes(
                r#"
                const encoder = new TextEncoder();
                const arr = encoder.encode("€");
                arr.length === 3 && arr[0] === 226 && arr[1] === 130 && arr[2] === 172
            "#,
            );
            let res = context.eval(source).unwrap();
            assert_eq!(res.as_boolean(), Some(true));
        }
    }

    #[test]
    fn test_text_decoder_decode_t0505() {
        // Decode Uint8Array [72, 105] ("Hi")
        {
            let mut context = Context::default();
            register_encoding_builtins(&mut context).unwrap();
            let source = Source::from_bytes(
                r#"
                const decoder = new TextDecoder();
                const arr = new Uint8Array([72, 105]);
                decoder.decode(arr) === "Hi"
            "#,
            );
            let res = context.eval(source).unwrap();
            assert_eq!(res.as_boolean(), Some(true));
        }

        // Round-trip text encoder / text decoder
        {
            let mut context = Context::default();
            register_encoding_builtins(&mut context).unwrap();
            let source = Source::from_bytes(
                r#"
                const encoder = new TextEncoder();
                const decoder = new TextDecoder();
                decoder.decode(encoder.encode("héllo")) === "héllo"
            "#,
            );
            let res = context.eval(source).unwrap();
            assert_eq!(res.as_boolean(), Some(true));
        }
    }

    #[test]
    fn test_text_encoder_encoding_prop_t0505() {
        let mut context = Context::default();
        register_encoding_builtins(&mut context).unwrap();

        let source = Source::from_bytes(
            r#"
            const encoder = new TextEncoder();
            const decoder = new TextDecoder();
            encoder.encoding === "utf-8" && decoder.encoding === "utf-8"
        "#,
        );
        let res = context.eval(source).unwrap();
        assert_eq!(res.as_boolean(), Some(true));
    }
}
