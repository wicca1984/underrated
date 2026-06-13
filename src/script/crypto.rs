//! Web Crypto JS object implementation.
//!
//! This module defines the `Crypto` object which provides cryptographic APIs.
//! Spec: https://w3c.github.io/webcrypto/

use boa_engine::object::ObjectInitializer;
use boa_engine::{Context, JsError, JsNativeError, JsObject, JsString, JsValue, NativeFunction};
use std::cell::Cell;

thread_local! {
    // SplitMix64 PRNG state initialized with a fixed nonzero seed
    static RNG_STATE: Cell<u64> = const { Cell::new(1234567890123456789u64) };
}

// TODO(spec): not cryptographically secure
fn next_u64() -> u64 {
    RNG_STATE.with(|state| {
        let mut x = state.get();
        x = x.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = x;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z = z ^ (z >> 31);
        state.set(x);
        z
    })
}

fn random_hex_char() -> char {
    let val = next_u64() % 16;
    const HEX_CHARS: [char; 16] = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
    ];
    HEX_CHARS[val as usize]
}

fn random_y_char() -> char {
    let val = next_u64() % 4;
    const Y_CHARS: [char; 4] = ['8', '9', 'a', 'b'];
    Y_CHARS[val as usize]
}

/// Native implementation of `crypto.randomUUID()`.
fn crypto_random_uuid(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> Result<JsValue, JsError> {
    let mut uuid = String::with_capacity(36);
    for _ in 0..8 {
        uuid.push(random_hex_char());
    }
    uuid.push('-');
    for _ in 0..4 {
        uuid.push(random_hex_char());
    }
    uuid.push('-');
    uuid.push('4'); // version 4
    for _ in 0..3 {
        uuid.push(random_hex_char());
    }
    uuid.push('-');
    uuid.push(random_y_char()); // variant y: 8, 9, a, b
    for _ in 0..3 {
        uuid.push(random_hex_char());
    }
    uuid.push('-');
    for _ in 0..12 {
        uuid.push(random_hex_char());
    }
    Ok(JsValue::from(JsString::from(uuid)))
}

/// Native implementation of `crypto.getRandomValues()`.
fn crypto_get_random_values(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let typed_array_val = args.first().cloned().unwrap_or(JsValue::undefined());
    let Some(obj) = typed_array_val.as_object() else {
        return Err(JsError::from(
            JsNativeError::typ().with_message("TypeMismatchError: argument is not an object"),
        ));
    };

    let constructor = obj.get(JsString::from("constructor"), context)?;
    let Some(constructor_obj) = constructor.as_object() else {
        return Err(JsError::from(
            JsNativeError::typ().with_message("TypeMismatchError: constructor not found"),
        ));
    };

    let constructor_name = constructor_obj
        .get(JsString::from("name"), context)?
        .as_string()
        .map(|s| s.to_std_string().unwrap_or_default())
        .unwrap_or_default();

    let byte_length_val = obj.get(JsString::from("byteLength"), context)?;
    let byte_length = byte_length_val.as_number().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("TypeMismatchError: byteLength is not a number"),
        )
    })?;

    if byte_length > 65536.0 {
        return Err(JsError::from(JsNativeError::range().with_message(
            "QuotaExceededError: The requested length exceeds 65,536 bytes",
        )));
    }

    let length_val = obj.get(JsString::from("length"), context)?;
    let length = length_val.as_number().map(|n| n as usize).ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("TypeMismatchError: length is not a number"),
        )
    })?;

    match constructor_name.as_str() {
        "Int8Array" | "Uint8Array" | "Uint8ClampedArray" => {
            for i in 0..length {
                let val = next_u64() as u8;
                obj.set(i, JsValue::from(val), true, context)?;
            }
        }
        "Int16Array" | "Uint16Array" => {
            for i in 0..length {
                let val = next_u64() as u16;
                obj.set(i, JsValue::from(val), true, context)?;
            }
        }
        "Int32Array" | "Uint32Array" => {
            for i in 0..length {
                let val = next_u64() as u32;
                obj.set(i, JsValue::from(val), true, context)?;
            }
        }
        "BigInt64Array" | "BigUint64Array" => {
            let bigint_constructor = context
                .global_object()
                .get(JsString::from("BigInt"), context)?;
            let bigint_constructor_obj = bigint_constructor.as_object().ok_or_else(|| {
                JsError::from(JsNativeError::typ().with_message("BigInt constructor not found"))
            })?;
            for i in 0..length {
                let val = next_u64();
                let val_str = JsValue::from(JsString::from(val.to_string()));
                let js_bigint =
                    bigint_constructor_obj.call(&JsValue::undefined(), &[val_str], context)?;
                obj.set(i, js_bigint, true, context)?;
            }
        }
        _ => {
            return Err(JsError::from(JsNativeError::typ().with_message(
                "TypeMismatchError: argument is not an integer TypedArray",
            )));
        }
    }

    Ok(typed_array_val)
}

/// Creates the standard `crypto` object with the `randomUUID` and `getRandomValues` methods.
pub fn create_crypto(context: &mut Context) -> JsObject {
    ObjectInitializer::new(context)
        .function(
            NativeFunction::from_fn_ptr(crypto_random_uuid),
            JsString::from("randomUUID"),
            0,
        )
        .function(
            NativeFunction::from_fn_ptr(crypto_get_random_values),
            JsString::from("getRandomValues"),
            1,
        )
        .build()
}
