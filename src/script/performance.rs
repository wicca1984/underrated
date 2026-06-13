//! Performance JS object implementation.
//!
//! This module defines the `Performance` object which provides access to performance-related
//! information for the current context.

use boa_engine::object::ObjectInitializer;
use boa_engine::{Context, JsError, JsObject, JsValue, NativeFunction};
use std::sync::OnceLock;
use std::time::Instant;

static TIME_ORIGIN: OnceLock<Instant> = OnceLock::new();

/// Returns the time origin `Instant`. Initialized on first call.
fn get_time_origin() -> &'static Instant {
    TIME_ORIGIN.get_or_init(Instant::now)
}

/// Native implementation of `performance.now()`.
///
/// Returns a `DOMHighResTimeStamp` representing the milliseconds elapsed since the time origin.
fn performance_now(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> Result<JsValue, JsError> {
    let origin = get_time_origin();
    let elapsed = origin.elapsed();
    let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
    Ok(JsValue::from(elapsed_ms))
}

/// Creates the standard `performance` object with the `now` method.
pub fn create_performance(context: &mut Context) -> JsObject {
    ObjectInitializer::new(context)
        .function(
            NativeFunction::from_fn_ptr(performance_now),
            boa_engine::JsString::from("now"),
            0,
        )
        .build()
}
