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

const BASE64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(data: &[u8]) -> String {
    let mut result = String::with_capacity(data.len().div_ceil(3) * 4);
    let mut chunks = data.chunks_exact(3);
    for chunk in chunks.by_ref() {
        if chunk.len() < 3 {
            continue;
        }
        let b0 = chunk[0];
        let b1 = chunk[1];
        let b2 = chunk[2];
        let val = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);

        let c0 = BASE64_ALPHABET
            .get(((val >> 18) & 0x3F) as usize)
            .copied()
            .unwrap_or(b'A');
        let c1 = BASE64_ALPHABET
            .get(((val >> 12) & 0x3F) as usize)
            .copied()
            .unwrap_or(b'A');
        let c2 = BASE64_ALPHABET
            .get(((val >> 6) & 0x3F) as usize)
            .copied()
            .unwrap_or(b'A');
        let c3 = BASE64_ALPHABET
            .get((val & 0x3F) as usize)
            .copied()
            .unwrap_or(b'A');

        result.push(c0 as char);
        result.push(c1 as char);
        result.push(c2 as char);
        result.push(c3 as char);
    }

    let remainder = chunks.remainder();
    if remainder.len() == 1 {
        let b0 = remainder.first().copied().unwrap_or(0);
        let val = (b0 as u32) << 16;
        let c0 = BASE64_ALPHABET
            .get(((val >> 18) & 0x3F) as usize)
            .copied()
            .unwrap_or(b'A');
        let c1 = BASE64_ALPHABET
            .get(((val >> 12) & 0x3F) as usize)
            .copied()
            .unwrap_or(b'A');
        result.push(c0 as char);
        result.push(c1 as char);
        result.push('=');
        result.push('=');
    } else if remainder.len() == 2 {
        let b0 = remainder.first().copied().unwrap_or(0);
        let b1 = remainder.get(1).copied().unwrap_or(0);
        let val = ((b0 as u32) << 16) | ((b1 as u32) << 8);
        let c0 = BASE64_ALPHABET
            .get(((val >> 18) & 0x3F) as usize)
            .copied()
            .unwrap_or(b'A');
        let c1 = BASE64_ALPHABET
            .get(((val >> 12) & 0x3F) as usize)
            .copied()
            .unwrap_or(b'A');
        let c2 = BASE64_ALPHABET
            .get(((val >> 6) & 0x3F) as usize)
            .copied()
            .unwrap_or(b'A');
        result.push(c0 as char);
        result.push(c1 as char);
        result.push(c2 as char);
        result.push('=');
    }
    result
}

fn base64_char_to_value(c: char) -> Option<u8> {
    match c {
        'A'..='Z' => Some((c as u32 - 'A' as u32) as u8),
        'a'..='z' => Some((c as u32 - 'a' as u32 + 26) as u8),
        '0'..='9' => Some((c as u32 - '0' as u32 + 52) as u8),
        '+' => Some(62),
        '/' => Some(63),
        _ => None,
    }
}

fn btoa_fn(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let input_val = args.first().cloned().unwrap_or(JsValue::undefined());
    let input_str = input_val
        .to_string(context)?
        .to_std_string()
        .unwrap_or_default();

    let mut bytes = Vec::with_capacity(input_str.len());
    for c in input_str.chars() {
        let code = c as u32;
        if code > 0xFF {
            // TODO(spec): should be a DOMException InvalidCharacterError
            return Err(JsError::from(JsNativeError::typ().with_message(
                "InvalidCharacterError: String contains characters outside of Latin1 range (0..=255)"
            )));
        }
        bytes.push(code as u8);
    }

    let encoded = base64_encode(&bytes);
    Ok(JsValue::from(JsString::from(encoded)))
}

fn atob_fn(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let input_val = args.first().cloned().unwrap_or(JsValue::undefined());
    let input_str = input_val
        .to_string(context)?
        .to_std_string()
        .unwrap_or_default();

    let mut cleaned = String::with_capacity(input_str.len());
    for c in input_str.chars() {
        if c != ' ' && c != '\t' && c != '\n' && c != '\r' && c != '\x0c' {
            cleaned.push(c);
        }
    }

    let chars: Vec<char> = cleaned.chars().collect();
    if chars.is_empty() {
        return Ok(JsValue::from(JsString::from("")));
    }

    if !chars.len().is_multiple_of(4) {
        // TODO(spec): should be a DOMException InvalidCharacterError
        return Err(JsError::from(JsNativeError::typ().with_message(
            "InvalidCharacterError: The string to be decoded is not correctly encoded (length is not a multiple of 4)"
        )));
    }

    let num_chunks = chars.len() / 4;
    let mut decoded_bytes = Vec::with_capacity(num_chunks * 3);

    for i in 0..num_chunks {
        let is_last = i == num_chunks - 1;
        let idx = i * 4;
        let c0 = *chars.get(idx).ok_or_else(|| {
            JsError::from(JsNativeError::typ().with_message("Index out of bounds"))
        })?;
        let c1 = *chars.get(idx + 1).ok_or_else(|| {
            JsError::from(JsNativeError::typ().with_message("Index out of bounds"))
        })?;
        let c2 = *chars.get(idx + 2).ok_or_else(|| {
            JsError::from(JsNativeError::typ().with_message("Index out of bounds"))
        })?;
        let c3 = *chars.get(idx + 3).ok_or_else(|| {
            JsError::from(JsNativeError::typ().with_message("Index out of bounds"))
        })?;

        if !is_last {
            if c0 == '=' || c1 == '=' || c2 == '=' || c3 == '=' {
                // TODO(spec): should be a DOMException InvalidCharacterError
                return Err(JsError::from(JsNativeError::typ().with_message(
                    "InvalidCharacterError: Padding character in invalid position",
                )));
            }
            let v0 = base64_char_to_value(c0).ok_or_else(|| {
                JsError::from(
                    JsNativeError::typ()
                        .with_message("InvalidCharacterError: Invalid base64 character"),
                )
            })? as u32;
            let v1 = base64_char_to_value(c1).ok_or_else(|| {
                JsError::from(
                    JsNativeError::typ()
                        .with_message("InvalidCharacterError: Invalid base64 character"),
                )
            })? as u32;
            let v2 = base64_char_to_value(c2).ok_or_else(|| {
                JsError::from(
                    JsNativeError::typ()
                        .with_message("InvalidCharacterError: Invalid base64 character"),
                )
            })? as u32;
            let v3 = base64_char_to_value(c3).ok_or_else(|| {
                JsError::from(
                    JsNativeError::typ()
                        .with_message("InvalidCharacterError: Invalid base64 character"),
                )
            })? as u32;
            let val = (v0 << 18) | (v1 << 12) | (v2 << 6) | v3;
            decoded_bytes.push(((val >> 16) & 0xFF) as u8);
            decoded_bytes.push(((val >> 8) & 0xFF) as u8);
            decoded_bytes.push((val & 0xFF) as u8);
        } else {
            let is_c0_pad = c0 == '=';
            let is_c1_pad = c1 == '=';
            let is_c2_pad = c2 == '=';
            let is_c3_pad = c3 == '=';

            if is_c0_pad || is_c1_pad {
                // TODO(spec): should be a DOMException InvalidCharacterError
                return Err(JsError::from(JsNativeError::typ().with_message(
                    "InvalidCharacterError: Padding character in invalid position",
                )));
            }

            if is_c2_pad && !is_c3_pad {
                // TODO(spec): should be a DOMException InvalidCharacterError
                return Err(JsError::from(JsNativeError::typ().with_message(
                    "InvalidCharacterError: Padding character in invalid position",
                )));
            }

            if !is_c2_pad && !is_c3_pad {
                let v0 = base64_char_to_value(c0).ok_or_else(|| {
                    JsError::from(
                        JsNativeError::typ()
                            .with_message("InvalidCharacterError: Invalid base64 character"),
                    )
                })? as u32;
                let v1 = base64_char_to_value(c1).ok_or_else(|| {
                    JsError::from(
                        JsNativeError::typ()
                            .with_message("InvalidCharacterError: Invalid base64 character"),
                    )
                })? as u32;
                let v2 = base64_char_to_value(c2).ok_or_else(|| {
                    JsError::from(
                        JsNativeError::typ()
                            .with_message("InvalidCharacterError: Invalid base64 character"),
                    )
                })? as u32;
                let v3 = base64_char_to_value(c3).ok_or_else(|| {
                    JsError::from(
                        JsNativeError::typ()
                            .with_message("InvalidCharacterError: Invalid base64 character"),
                    )
                })? as u32;
                let val = (v0 << 18) | (v1 << 12) | (v2 << 6) | v3;
                decoded_bytes.push(((val >> 16) & 0xFF) as u8);
                decoded_bytes.push(((val >> 8) & 0xFF) as u8);
                decoded_bytes.push((val & 0xFF) as u8);
            } else if !is_c2_pad && is_c3_pad {
                let v0 = base64_char_to_value(c0).ok_or_else(|| {
                    JsError::from(
                        JsNativeError::typ()
                            .with_message("InvalidCharacterError: Invalid base64 character"),
                    )
                })? as u32;
                let v1 = base64_char_to_value(c1).ok_or_else(|| {
                    JsError::from(
                        JsNativeError::typ()
                            .with_message("InvalidCharacterError: Invalid base64 character"),
                    )
                })? as u32;
                let v2 = base64_char_to_value(c2).ok_or_else(|| {
                    JsError::from(
                        JsNativeError::typ()
                            .with_message("InvalidCharacterError: Invalid base64 character"),
                    )
                })? as u32;
                let val = (v0 << 18) | (v1 << 12) | (v2 << 6);
                decoded_bytes.push(((val >> 16) & 0xFF) as u8);
                decoded_bytes.push(((val >> 8) & 0xFF) as u8);
            } else if is_c2_pad && is_c3_pad {
                let v0 = base64_char_to_value(c0).ok_or_else(|| {
                    JsError::from(
                        JsNativeError::typ()
                            .with_message("InvalidCharacterError: Invalid base64 character"),
                    )
                })? as u32;
                let v1 = base64_char_to_value(c1).ok_or_else(|| {
                    JsError::from(
                        JsNativeError::typ()
                            .with_message("InvalidCharacterError: Invalid base64 character"),
                    )
                })? as u32;
                let val = (v0 << 18) | (v1 << 12);
                decoded_bytes.push(((val >> 16) & 0xFF) as u8);
            }
        }
    }

    let mut output = String::with_capacity(decoded_bytes.len());
    for &b in &decoded_bytes {
        if let Some(c) = char::from_u32(b as u32) {
            output.push(c);
        }
    }

    Ok(JsValue::from(JsString::from(output)))
}

/// Registers the WHATWG Encoding API global classes `TextEncoder` and `TextDecoder`.
pub fn register_encoding_builtins(context: &mut Context) -> JsResult<()> {
    context.register_global_class::<TextEncoder>()?;
    context.register_global_class::<TextDecoder>()?;
    Ok(())
}

/// Registers the WHATWG HTML base64 utility functions `btoa` and `atob` globally.
pub fn register_base64_builtins(context: &mut Context) -> JsResult<()> {
    context.register_global_builtin_callable(
        JsString::from("btoa"),
        1,
        NativeFunction::from_fn_ptr(btoa_fn),
    )?;
    context.register_global_builtin_callable(
        JsString::from("atob"),
        1,
        NativeFunction::from_fn_ptr(atob_fn),
    )?;
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

    #[test]
    fn test_base64_builtins_t0512() {
        let mut context = Context::default();
        register_base64_builtins(&mut context).unwrap();

        // btoa("hello") == "aGVsbG8="
        let res = context
            .eval(Source::from_bytes(r#"btoa("hello") === "aGVsbG8=""#))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // btoa("") == ""
        let res = context
            .eval(Source::from_bytes(r#"btoa("") === """#))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // atob("aGVsbG8=") == "hello"
        let res = context
            .eval(Source::from_bytes(r#"atob("aGVsbG8=") === "hello""#))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // round-trip: atob(btoa("Man is distinguished")) == "Man is distinguished"
        let res = context
            .eval(Source::from_bytes(
                r#"atob(btoa("Man is distinguished")) === "Man is distinguished""#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // btoa with char > 0xFF (e.g. "\u{100}") throws
        let res = context
            .eval(Source::from_bytes(
                r#"
            {
                let threw = false;
                try {
                    btoa("\u{100}");
                } catch (e) {
                    threw = true;
                }
                threw
            }
        "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // atob("!!!!") throws
        let res = context
            .eval(Source::from_bytes(
                r#"
            {
                let threw = false;
                try {
                    atob("!!!!");
                } catch (e) {
                    threw = true;
                }
                threw
            }
        "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // atob strips whitespace and parses correctly
        let res = context
            .eval(Source::from_bytes(
                r#"atob(" aGVs bG8 = \r\n") === "hello""#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));
    }
}
