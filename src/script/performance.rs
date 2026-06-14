//! Performance JS object implementation.
//!
//! This module defines the `Performance` object which provides access to performance-related
//! information for the current context.

use boa_engine::class::{Class, ClassBuilder};
use boa_engine::object::{FunctionObjectBuilder, JsObject, ObjectInitializer};
use boa_engine::property::Attribute;
use boa_engine::{
    Context, JsData, JsError, JsNativeError, JsResult, JsString, JsValue, NativeFunction,
};
use boa_gc::{Finalize, GcRefCell, Trace};
use std::sync::OnceLock;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

static TIME_ORIGIN: OnceLock<Instant> = OnceLock::new();
static TIME_ORIGIN_MS: OnceLock<f64> = OnceLock::new();

/// Returns the time origin `Instant`. Initialized on first call.
fn get_time_origin() -> &'static Instant {
    TIME_ORIGIN.get_or_init(Instant::now)
}

/// Returns the time origin Unix epoch high-res timestamp in milliseconds.
fn get_time_origin_ms() -> f64 {
    *TIME_ORIGIN_MS.get_or_init(|| {
        let _ = get_time_origin();
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64()
            * 1000.0
    })
}

/// Representation of a stored performance mark.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct PerformanceMarkEntry {
    pub name: String,
    pub start_time: f64,
}

/// Representation of a stored performance measure.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct PerformanceMeasureEntry {
    pub name: String,
    pub start_time: f64,
    pub duration: f64,
}

/// Performance JS Class host struct.
#[derive(Debug, Trace, Finalize, JsData)]
pub struct Performance {
    pub(crate) marks: GcRefCell<Vec<PerformanceMarkEntry>>,
    pub(crate) measures: GcRefCell<Vec<PerformanceMeasureEntry>>,
}

impl Class for Performance {
    const NAME: &'static str = "Performance";
    const LENGTH: usize = 0;

    fn data_constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<Self> {
        Ok(Performance {
            marks: GcRefCell::new(Vec::new()),
            measures: GcRefCell::new(Vec::new()),
        })
    }

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        let realm = class.context().realm().clone();

        let get_time_origin_fn = FunctionObjectBuilder::new(
            &realm,
            NativeFunction::from_fn_ptr(performance_get_time_origin),
        )
        .name("get timeOrigin")
        .build();

        class
            .accessor(
                JsString::from("timeOrigin"),
                Some(get_time_origin_fn),
                None,
                Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .method(
                JsString::from("now"),
                0,
                NativeFunction::from_fn_ptr(performance_now),
            )
            .method(
                JsString::from("mark"),
                1,
                NativeFunction::from_fn_ptr(performance_mark),
            )
            .method(
                JsString::from("measure"),
                1,
                NativeFunction::from_fn_ptr(performance_measure),
            )
            .method(
                JsString::from("clearMarks"),
                0,
                NativeFunction::from_fn_ptr(performance_clear_marks),
            )
            .method(
                JsString::from("clearMeasures"),
                0,
                NativeFunction::from_fn_ptr(performance_clear_measures),
            );

        Ok(())
    }
}

fn performance_get_time_origin(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    Ok(JsValue::from(get_time_origin_ms()))
}

fn performance_now(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let origin = get_time_origin();
    let elapsed = origin.elapsed();
    let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
    Ok(JsValue::from(elapsed_ms))
}

fn performance_mark(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let performance = obj.downcast_ref::<Performance>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Performance object"))
    })?;

    let name_val = args.first().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("performance.mark: name is required"))
    })?;
    let name = name_val
        .to_string(context)?
        .to_std_string()
        .unwrap_or_default();

    let elapsed = get_time_origin().elapsed();
    let elapsed_ms = elapsed.as_secs_f64() * 1000.0;

    performance.marks.borrow_mut().push(PerformanceMarkEntry {
        name: name.clone(),
        start_time: elapsed_ms,
    });

    let mark_obj = create_performance_mark_object(&name, elapsed_ms, context);
    Ok(mark_obj)
}

fn performance_measure(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let performance = obj.downcast_ref::<Performance>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Performance object"))
    })?;

    let name_val = args.first().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("performance.measure: name is required"))
    })?;
    let name = name_val
        .to_string(context)?
        .to_std_string()
        .unwrap_or_default();

    let mut start_time = 0.0;
    if let Some(start_mark_val) = args.get(1) {
        if start_mark_val.is_undefined() || start_mark_val.is_null() {
            start_time = 0.0;
        } else if start_mark_val.is_number() {
            start_time = start_mark_val.as_number().unwrap_or(0.0);
        } else {
            let start_mark_name = start_mark_val
                .to_string(context)?
                .to_std_string()
                .unwrap_or_default();
            let marks_borrow = performance.marks.borrow();
            let mark_opt = marks_borrow
                .iter()
                .rev()
                .find(|m| m.name == start_mark_name);
            match mark_opt {
                Some(mark) => start_time = mark.start_time,
                None => {
                    return Err(JsError::from(JsNativeError::syntax().with_message(
                        format!(
                            "performance.measure: startMark '{}' not found",
                            start_mark_name
                        ),
                    )));
                }
            }
        }
    }

    let mut end_time = get_time_origin().elapsed().as_secs_f64() * 1000.0;
    if let Some(end_mark_val) = args.get(2) {
        if end_mark_val.is_undefined() || end_mark_val.is_null() {
            // Keep default end_time
        } else if end_mark_val.is_number() {
            end_time = end_mark_val.as_number().unwrap_or(0.0);
        } else {
            let end_mark_name = end_mark_val
                .to_string(context)?
                .to_std_string()
                .unwrap_or_default();
            let marks_borrow = performance.marks.borrow();
            let mark_opt = marks_borrow.iter().rev().find(|m| m.name == end_mark_name);
            match mark_opt {
                Some(mark) => end_time = mark.start_time,
                None => {
                    return Err(JsError::from(JsNativeError::syntax().with_message(
                        format!("performance.measure: endMark '{}' not found", end_mark_name),
                    )));
                }
            }
        }
    }

    let duration = end_time - start_time;

    performance
        .measures
        .borrow_mut()
        .push(PerformanceMeasureEntry {
            name: name.clone(),
            start_time,
            duration,
        });

    let measure_obj = create_performance_measure_object(&name, start_time, duration, context);
    Ok(measure_obj)
}

fn performance_clear_marks(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let performance = obj.downcast_ref::<Performance>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Performance object"))
    })?;

    if let Some(name_val) = args.first().filter(|v| !v.is_undefined() && !v.is_null()) {
        let name = name_val
            .to_string(context)?
            .to_std_string()
            .unwrap_or_default();
        performance.marks.borrow_mut().retain(|m| m.name != name);
        return Ok(JsValue::undefined());
    }

    performance.marks.borrow_mut().clear();
    Ok(JsValue::undefined())
}

fn performance_clear_measures(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let performance = obj.downcast_ref::<Performance>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Performance object"))
    })?;

    if let Some(name_val) = args.first().filter(|v| !v.is_undefined() && !v.is_null()) {
        let name = name_val
            .to_string(context)?
            .to_std_string()
            .unwrap_or_default();
        performance.measures.borrow_mut().retain(|m| m.name != name);
        return Ok(JsValue::undefined());
    }

    performance.measures.borrow_mut().clear();
    Ok(JsValue::undefined())
}

fn create_performance_mark_object(name: &str, start_time: f64, context: &mut Context) -> JsValue {
    let ro = Attribute::ENUMERABLE | Attribute::CONFIGURABLE;
    let mark_obj = ObjectInitializer::new(context)
        .property(JsString::from("name"), JsString::from(name), ro)
        .property(JsString::from("entryType"), JsString::from("mark"), ro)
        .property(JsString::from("startTime"), JsValue::from(start_time), ro)
        .property(JsString::from("duration"), JsValue::from(0.0), ro)
        .build();
    JsValue::from(mark_obj)
}

fn create_performance_measure_object(
    name: &str,
    start_time: f64,
    duration: f64,
    context: &mut Context,
) -> JsValue {
    let ro = Attribute::ENUMERABLE | Attribute::CONFIGURABLE;
    let measure_obj = ObjectInitializer::new(context)
        .property(JsString::from("name"), JsString::from(name), ro)
        .property(JsString::from("entryType"), JsString::from("measure"), ro)
        .property(JsString::from("startTime"), JsValue::from(start_time), ro)
        .property(JsString::from("duration"), JsValue::from(duration), ro)
        .build();
    JsValue::from(measure_obj)
}

/// Creates the standard `performance` object.
pub fn create_performance(context: &mut Context) -> JsObject {
    let _ = context.register_global_class::<Performance>();

    let performance_obj = context
        .global_object()
        .get(boa_engine::JsString::from("Performance"), context)
        .ok()
        .and_then(|constructor| constructor.as_object());

    if let Some(inst) = performance_obj.and_then(|obj| obj.construct(&[], None, context).ok()) {
        return inst;
    }

    ObjectInitializer::new(context).build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::Source;

    #[test]
    fn test_performance_time_origin_and_now() {
        let mut context = Context::default();
        let performance = create_performance(&mut context);
        let _ = context.register_global_property(
            JsString::from("performance"),
            performance.clone(),
            Attribute::all(),
        );

        let res = context
            .eval(Source::from_bytes("performance.timeOrigin > 0"))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        let res = context
            .eval(Source::from_bytes("performance.now() >= 0"))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));
    }

    #[test]
    fn test_performance_user_timing_mark_measure_clear() {
        let mut context = Context::default();
        let performance = create_performance(&mut context);
        let _ = context.register_global_property(
            JsString::from("performance"),
            performance.clone(),
            Attribute::all(),
        );

        // 1. Mark creation
        let source = Source::from_bytes(
            r#"
            const m1 = performance.mark("start");
            const m2 = performance.mark("end");
            m1.name === "start" && m1.entryType === "mark" && m1.startTime >= 0 && m1.duration === 0 &&
            m2.name === "end" && m2.entryType === "mark" && m2.startTime >= m1.startTime
            "#,
        );
        let res = context.eval(source).unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // Verify state via Rust downcast
        {
            let perf = performance.downcast_ref::<Performance>().unwrap();
            let marks = perf.marks.borrow();
            assert_eq!(marks.len(), 2);
            assert_eq!(marks[0].name, "start");
            assert_eq!(marks[1].name, "end");
        }

        // 2. Measure creation
        let source = Source::from_bytes(
            r#"
            const m = performance.measure("my-duration", "start", "end");
            m.name === "my-duration" && m.entryType === "measure" && m.startTime === m1.startTime && m.duration >= 0
            "#,
        );
        let res = context.eval(source).unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        {
            let perf = performance.downcast_ref::<Performance>().unwrap();
            let measures = perf.measures.borrow();
            assert_eq!(measures.len(), 1);
            assert_eq!(measures[0].name, "my-duration");
            assert!(measures[0].duration >= 0.0);
        }

        // 3. Clear marks by name
        let source = Source::from_bytes(
            r#"
            performance.clearMarks("start");
            "#,
        );
        let _ = context.eval(source).unwrap();

        {
            let perf = performance.downcast_ref::<Performance>().unwrap();
            let marks = perf.marks.borrow();
            assert_eq!(marks.len(), 1);
            assert_eq!(marks[0].name, "end");
        }

        // 4. Clear all marks
        let source = Source::from_bytes(
            r#"
            performance.clearMarks();
            "#,
        );
        let _ = context.eval(source).unwrap();

        {
            let perf = performance.downcast_ref::<Performance>().unwrap();
            let marks = perf.marks.borrow();
            assert!(marks.is_empty());
        }

        // 5. Clear measures by name
        let source = Source::from_bytes(
            r#"
            performance.clearMeasures("my-duration");
            "#,
        );
        let _ = context.eval(source).unwrap();

        {
            let perf = performance.downcast_ref::<Performance>().unwrap();
            let measures = perf.measures.borrow();
            assert!(measures.is_empty());
        }
    }

    #[test]
    fn test_performance_measure_error_handling() {
        let mut context = Context::default();
        let performance = create_performance(&mut context);
        let _ = context.register_global_property(
            JsString::from("performance"),
            performance,
            Attribute::all(),
        );

        // measure with unknown mark name throws an error
        let source = Source::from_bytes(
            r#"
            let threw = false;
            try {
                performance.measure("test", "non-existent");
            } catch (e) {
                threw = true;
            }
            threw
            "#,
        );
        let res = context.eval(source).unwrap();
        assert_eq!(res.as_boolean(), Some(true));
    }
}
