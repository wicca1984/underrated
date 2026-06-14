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

static TIME_ORIGINS: OnceLock<(Instant, f64)> = OnceLock::new();

/// Returns the time origins initialized together.
fn get_origins() -> &'static (Instant, f64) {
    TIME_ORIGINS.get_or_init(|| {
        let instant = Instant::now();
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64()
            * 1000.0;
        (instant, ms)
    })
}

/// Returns the time origin `Instant`. Initialized on first call.
fn get_time_origin() -> &'static Instant {
    &get_origins().0
}

/// Returns the time origin Unix epoch high-res timestamp in milliseconds.
fn get_time_origin_ms() -> f64 {
    get_origins().1
}

/// Representation of a stored performance mark.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct PerformanceMarkEntry {
    pub name: String,
    pub start_time: f64,
    pub detail: JsValue,
}

/// Representation of a stored performance measure.
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct PerformanceMeasureEntry {
    pub name: String,
    pub start_time: f64,
    pub duration: f64,
    pub detail: JsValue,
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

        let get_timing_fn =
            FunctionObjectBuilder::new(&realm, NativeFunction::from_fn_ptr(performance_get_timing))
                .name("get timing")
                .build();

        let get_navigation_fn = FunctionObjectBuilder::new(
            &realm,
            NativeFunction::from_fn_ptr(performance_get_navigation),
        )
        .name("get navigation")
        .build();

        class
            .accessor(
                JsString::from("timeOrigin"),
                Some(get_time_origin_fn),
                None,
                Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .accessor(
                JsString::from("timing"),
                Some(get_timing_fn),
                None,
                Attribute::ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .accessor(
                JsString::from("navigation"),
                Some(get_navigation_fn),
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
            )
            .method(
                JsString::from("getEntries"),
                0,
                NativeFunction::from_fn_ptr(performance_get_entries),
            )
            .method(
                JsString::from("getEntriesByType"),
                1,
                NativeFunction::from_fn_ptr(performance_get_entries_by_type),
            )
            .method(
                JsString::from("getEntriesByName"),
                1,
                NativeFunction::from_fn_ptr(performance_get_entries_by_name),
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

fn performance_get_timing(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let origin_ms = get_time_origin_ms();
    let ro = Attribute::ENUMERABLE | Attribute::CONFIGURABLE;
    let timing_obj = ObjectInitializer::new(context)
        .property(
            JsString::from("navigationStart"),
            JsValue::from(origin_ms),
            ro,
        )
        .property(JsString::from("unloadEventStart"), JsValue::from(0), ro)
        .property(JsString::from("unloadEventEnd"), JsValue::from(0), ro)
        .property(JsString::from("redirectStart"), JsValue::from(0), ro)
        .property(JsString::from("redirectEnd"), JsValue::from(0), ro)
        .property(JsString::from("fetchStart"), JsValue::from(origin_ms), ro)
        .property(
            JsString::from("domainLookupStart"),
            JsValue::from(origin_ms),
            ro,
        )
        .property(
            JsString::from("domainLookupEnd"),
            JsValue::from(origin_ms),
            ro,
        )
        .property(JsString::from("connectStart"), JsValue::from(origin_ms), ro)
        .property(JsString::from("connectEnd"), JsValue::from(origin_ms), ro)
        .property(
            JsString::from("secureConnectionStart"),
            JsValue::from(0),
            ro,
        )
        .property(JsString::from("requestStart"), JsValue::from(origin_ms), ro)
        .property(
            JsString::from("responseStart"),
            JsValue::from(origin_ms),
            ro,
        )
        .property(JsString::from("responseEnd"), JsValue::from(origin_ms), ro)
        .property(JsString::from("domLoading"), JsValue::from(origin_ms), ro)
        .property(
            JsString::from("domInteractive"),
            JsValue::from(origin_ms),
            ro,
        )
        .property(
            JsString::from("domContentLoadedEventStart"),
            JsValue::from(origin_ms),
            ro,
        )
        .property(
            JsString::from("domContentLoadedEventEnd"),
            JsValue::from(origin_ms),
            ro,
        )
        .property(JsString::from("domComplete"), JsValue::from(origin_ms), ro)
        .property(
            JsString::from("loadEventStart"),
            JsValue::from(origin_ms),
            ro,
        )
        .property(JsString::from("loadEventEnd"), JsValue::from(origin_ms), ro)
        .build();
    Ok(JsValue::from(timing_obj))
}

fn performance_get_navigation(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let ro = Attribute::ENUMERABLE | Attribute::CONFIGURABLE;
    let nav_obj = ObjectInitializer::new(context)
        .property(JsString::from("type"), JsValue::from(0), ro)
        .property(JsString::from("redirectCount"), JsValue::from(0), ro)
        .build();
    Ok(JsValue::from(nav_obj))
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

    let mut detail = JsValue::undefined();
    let mut start_time = get_time_origin().elapsed().as_secs_f64() * 1000.0;

    if let Some(options_obj) = args.get(1).and_then(|v| {
        if !v.is_undefined() && !v.is_null() {
            v.as_object()
        } else {
            None
        }
    }) {
        let det = options_obj.get(JsString::from("detail"), context)?;
        if !det.is_undefined() {
            detail = det;
        }

        let start_val = options_obj.get(JsString::from("startTime"), context)?;
        if !start_val.is_undefined() && !start_val.is_null() {
            let st = start_val.to_number(context)?;
            if st < 0.0 {
                return Err(JsError::from(
                    JsNativeError::typ()
                        .with_message("performance.mark: startTime cannot be negative"),
                ));
            }
            start_time = st;
        }
    }

    performance.marks.borrow_mut().push(PerformanceMarkEntry {
        name: name.clone(),
        start_time,
        detail: detail.clone(),
    });

    let mark_obj = create_performance_mark_object(&name, start_time, &detail, context);
    Ok(mark_obj)
}

fn resolve_mark_or_value(
    val: &JsValue,
    performance: &Performance,
    context: &mut Context,
) -> JsResult<f64> {
    if val.is_undefined() || val.is_null() {
        Ok(0.0)
    } else if val.is_number() {
        Ok(val.as_number().unwrap_or(0.0))
    } else {
        let name = val.to_string(context)?.to_std_string().unwrap_or_default();
        let marks_borrow = performance.marks.borrow();
        let mark_opt = marks_borrow.iter().rev().find(|m| m.name == name);
        match mark_opt {
            Some(mark) => Ok(mark.start_time),
            None => Err(JsError::from(JsNativeError::syntax().with_message(
                format!("performance.measure: mark '{}' not found", name),
            ))),
        }
    }
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

    let start_or_options_val = args.get(1);

    // Check if start_or_options_val is an options object
    let is_options = start_or_options_val.is_some_and(|val| {
        !val.is_undefined() && !val.is_null() && val.is_object() && !val.is_callable()
    });

    let (start_time, duration, detail) = if is_options {
        // If startOrMeasureOptions is a PerformanceMeasureOptions object and endMark is given, throw a TypeError.
        if args.get(2).filter(|v| !v.is_undefined()).is_some() {
            return Err(JsError::from(JsNativeError::typ().with_message(
                "performance.measure: endMark cannot be specified when options are used",
            )));
        }

        let options_obj = start_or_options_val
            .and_then(|v| v.as_object())
            .ok_or_else(|| {
                JsError::from(
                    JsNativeError::typ()
                        .with_message("performance.measure: options must be an object"),
                )
            })?;

        let mut detail = JsValue::undefined();
        let det = options_obj.get(JsString::from("detail"), context)?;
        if !det.is_undefined() {
            detail = det;
        }

        let start_val = options_obj.get(JsString::from("start"), context)?;
        let end_val = options_obj.get(JsString::from("end"), context)?;
        let duration_val = options_obj.get(JsString::from("duration"), context)?;

        let has_start = !start_val.is_undefined() && !start_val.is_null();
        let has_end = !end_val.is_undefined() && !end_val.is_null();
        let has_duration = !duration_val.is_undefined() && !duration_val.is_null();

        // If start, duration, and end are all present, throw a TypeError
        if has_start && has_duration && has_end {
            return Err(JsError::from(JsNativeError::typ().with_message(
                "performance.measure: start, duration, and end cannot all be specified",
            )));
        }

        // If duration is present and neither start nor end is present, throw a TypeError
        if has_duration && !has_start && !has_end {
            return Err(JsError::from(JsNativeError::typ().with_message(
                "performance.measure: duration cannot be specified without start or end",
            )));
        }

        let duration_f64 = if has_duration {
            let d = duration_val.to_number(context)?;
            if d < 0.0 {
                return Err(JsError::from(
                    JsNativeError::typ()
                        .with_message("performance.measure: duration cannot be negative"),
                ));
            }
            Some(d)
        } else {
            None
        };

        let resolved_start;
        let resolved_duration;

        if has_start && has_end {
            let s = resolve_mark_or_value(&start_val, &performance, context)?;
            let e = resolve_mark_or_value(&end_val, &performance, context)?;
            resolved_start = s;
            resolved_duration = e - s;
        } else if has_start && has_duration {
            let s = resolve_mark_or_value(&start_val, &performance, context)?;
            let dur = duration_f64.unwrap_or(0.0);
            resolved_start = s;
            resolved_duration = dur;
        } else if has_duration && has_end {
            let e = resolve_mark_or_value(&end_val, &performance, context)?;
            let dur = duration_f64.unwrap_or(0.0);
            resolved_start = e - dur;
            resolved_duration = dur;
        } else if has_start {
            let s = resolve_mark_or_value(&start_val, &performance, context)?;
            let e = get_time_origin().elapsed().as_secs_f64() * 1000.0;
            resolved_start = s;
            resolved_duration = e - s;
        } else if has_end {
            let e = resolve_mark_or_value(&end_val, &performance, context)?;
            resolved_start = 0.0;
            resolved_duration = e;
        } else {
            resolved_start = 0.0;
            let e = get_time_origin().elapsed().as_secs_f64() * 1000.0;
            resolved_duration = e;
        }

        (resolved_start, resolved_duration, detail)
    } else {
        let start_val = args.get(1).cloned().unwrap_or(JsValue::undefined());
        let end_val = args.get(2).cloned().unwrap_or(JsValue::undefined());

        let start_time = if start_val.is_undefined() || start_val.is_null() {
            0.0
        } else {
            resolve_mark_or_value(&start_val, &performance, context)?
        };

        let end_time = if end_val.is_undefined() || end_val.is_null() {
            get_time_origin().elapsed().as_secs_f64() * 1000.0
        } else {
            resolve_mark_or_value(&end_val, &performance, context)?
        };

        (start_time, end_time - start_time, JsValue::undefined())
    };

    performance
        .measures
        .borrow_mut()
        .push(PerformanceMeasureEntry {
            name: name.clone(),
            start_time,
            duration,
            detail: detail.clone(),
        });

    let measure_obj =
        create_performance_measure_object(&name, start_time, duration, &detail, context);
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

fn performance_get_entries(
    this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-object"))
    })?;
    let performance = obj.downcast_ref::<Performance>().ok_or_else(|| {
        JsError::from(JsNativeError::typ().with_message("Method called on non-Performance object"))
    })?;

    let mut entries = Vec::new();

    for mark in performance.marks.borrow().iter() {
        entries.push((
            mark.start_time,
            create_performance_mark_object(&mark.name, mark.start_time, &mark.detail, context),
        ));
    }

    for measure in performance.measures.borrow().iter() {
        entries.push((
            measure.start_time,
            create_performance_measure_object(
                &measure.name,
                measure.start_time,
                measure.duration,
                &measure.detail,
                context,
            ),
        ));
    }

    entries.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let js_entries: Vec<JsValue> = entries.into_iter().map(|(_, val)| val).collect();
    let array = boa_engine::object::builtins::JsArray::from_iter(js_entries, context);

    Ok(JsValue::from(array))
}

fn performance_get_entries_by_type(
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

    let type_val = args.first().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("performance.getEntriesByType: type is required"),
        )
    })?;
    let entry_type = type_val
        .to_string(context)?
        .to_std_string()
        .unwrap_or_default();

    let mut entries = Vec::new();

    if entry_type == "mark" {
        for mark in performance.marks.borrow().iter() {
            entries.push((
                mark.start_time,
                create_performance_mark_object(&mark.name, mark.start_time, &mark.detail, context),
            ));
        }
    } else if entry_type == "measure" {
        for measure in performance.measures.borrow().iter() {
            entries.push((
                measure.start_time,
                create_performance_measure_object(
                    &measure.name,
                    measure.start_time,
                    measure.duration,
                    &measure.detail,
                    context,
                ),
            ));
        }
    }

    entries.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let js_entries: Vec<JsValue> = entries.into_iter().map(|(_, val)| val).collect();
    let array = boa_engine::object::builtins::JsArray::from_iter(js_entries, context);

    Ok(JsValue::from(array))
}

fn performance_get_entries_by_name(
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
        JsError::from(
            JsNativeError::typ().with_message("performance.getEntriesByName: name is required"),
        )
    })?;
    let name = name_val
        .to_string(context)?
        .to_std_string()
        .unwrap_or_default();

    let entry_type_opt = args.get(1).filter(|v| !v.is_undefined() && !v.is_null());
    let entry_type = if let Some(type_val) = entry_type_opt {
        Some(
            type_val
                .to_string(context)?
                .to_std_string()
                .unwrap_or_default(),
        )
    } else {
        None
    };

    let mut entries = Vec::new();

    let check_mark = entry_type.as_deref().is_none_or(|t| t == "mark");
    let check_measure = entry_type.as_deref().is_none_or(|t| t == "measure");

    if check_mark {
        for mark in performance.marks.borrow().iter() {
            if mark.name == name {
                entries.push((
                    mark.start_time,
                    create_performance_mark_object(
                        &mark.name,
                        mark.start_time,
                        &mark.detail,
                        context,
                    ),
                ));
            }
        }
    }

    if check_measure {
        for measure in performance.measures.borrow().iter() {
            if measure.name == name {
                entries.push((
                    measure.start_time,
                    create_performance_measure_object(
                        &measure.name,
                        measure.start_time,
                        measure.duration,
                        &measure.detail,
                        context,
                    ),
                ));
            }
        }
    }

    entries.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let js_entries: Vec<JsValue> = entries.into_iter().map(|(_, val)| val).collect();
    let array = boa_engine::object::builtins::JsArray::from_iter(js_entries, context);

    Ok(JsValue::from(array))
}

fn create_performance_mark_object(
    name: &str,
    start_time: f64,
    detail: &JsValue,
    context: &mut Context,
) -> JsValue {
    let ro = Attribute::ENUMERABLE | Attribute::CONFIGURABLE;
    let mark_obj = ObjectInitializer::new(context)
        .property(JsString::from("name"), JsString::from(name), ro)
        .property(JsString::from("entryType"), JsString::from("mark"), ro)
        .property(JsString::from("startTime"), JsValue::from(start_time), ro)
        .property(JsString::from("duration"), JsValue::from(0.0), ro)
        .property(JsString::from("detail"), detail.clone(), ro)
        .build();
    JsValue::from(mark_obj)
}

fn create_performance_measure_object(
    name: &str,
    start_time: f64,
    duration: f64,
    detail: &JsValue,
    context: &mut Context,
) -> JsValue {
    let ro = Attribute::ENUMERABLE | Attribute::CONFIGURABLE;
    let measure_obj = ObjectInitializer::new(context)
        .property(JsString::from("name"), JsString::from(name), ro)
        .property(JsString::from("entryType"), JsString::from("measure"), ro)
        .property(JsString::from("startTime"), JsValue::from(start_time), ro)
        .property(JsString::from("duration"), JsValue::from(duration), ro)
        .property(JsString::from("detail"), detail.clone(), ro)
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

    #[test]
    fn test_performance_get_entries_api() {
        let mut context = Context::default();
        let performance = create_performance(&mut context);
        let _ = context.register_global_property(
            JsString::from("performance"),
            performance,
            Attribute::all(),
        );

        // Add some marks and measures
        context
            .eval(Source::from_bytes("performance.mark('mark1');"))
            .unwrap();
        context
            .eval(Source::from_bytes("performance.mark('mark2');"))
            .unwrap();
        context
            .eval(Source::from_bytes(
                "performance.measure('measure1', 'mark1', 'mark2');",
            ))
            .unwrap();

        // 1. Check getEntries returns all 3 entries
        let res = context
            .eval(Source::from_bytes("performance.getEntries().length === 3"))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // Check chronological ordering of getEntries()
        let res = context
            .eval(Source::from_bytes(
                r#"
                const entries = performance.getEntries();
                entries[0].name === "mark1" && entries[1].name === "measure1" && entries[2].name === "mark2"
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 2. Check getEntriesByType('mark') returns only marks
        let res = context
            .eval(Source::from_bytes(
                r#"
                const marks = performance.getEntriesByType("mark");
                marks.length === 2 && marks[0].name === "mark1" && marks[1].name === "mark2"
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // Check getEntriesByType('measure') returns only measures
        let res = context
            .eval(Source::from_bytes(
                r#"
                const measures = performance.getEntriesByType("measure");
                measures.length === 1 && measures[0].name === "measure1"
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // Check getEntriesByType with unsupported type returns empty array
        let res = context
            .eval(Source::from_bytes(
                "performance.getEntriesByType('invalid_type').length === 0",
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 3. Check getEntriesByName('mark1') returns the specific mark
        let res = context
            .eval(Source::from_bytes(
                r#"
                const named = performance.getEntriesByName("mark1");
                named.length === 1 && named[0].name === "mark1" && named[0].entryType === "mark"
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // Check getEntriesByName with correct type
        let res = context
            .eval(Source::from_bytes(
                r#"
                const named_typed = performance.getEntriesByName("mark1", "mark");
                named_typed.length === 1 && named_typed[0].name === "mark1"
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // Check getEntriesByName with non-matching type returns empty array
        let res = context
            .eval(Source::from_bytes(
                r#"
                performance.getEntriesByName("mark1", "measure").length === 0
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));
    }

    #[test]
    fn test_performance_timing_and_navigation_api_surface() {
        let mut context = Context::default();
        let performance = create_performance(&mut context);
        let _ = context.register_global_property(
            JsString::from("performance"),
            performance,
            Attribute::all(),
        );

        // Check timing object exists and has valid numbers
        let res = context
            .eval(Source::from_bytes(
                r#"
                performance.timing &&
                typeof performance.timing.navigationStart === "number" &&
                performance.timing.navigationStart > 0 &&
                performance.timing.unloadEventStart === 0 &&
                performance.timing.fetchStart === performance.timing.navigationStart
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // Check navigation object exists and has valid type and redirectCount
        let res = context
            .eval(Source::from_bytes(
                r#"
                performance.navigation &&
                performance.navigation.type === 0 &&
                performance.navigation.redirectCount === 0
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));
    }

    #[test]
    fn test_performance_user_timing_l3_mark_options() {
        let mut context = Context::default();
        let performance = create_performance(&mut context);
        let _ = context.register_global_property(
            JsString::from("performance"),
            performance,
            Attribute::all(),
        );

        // 1. Mark with startTime and detail
        let res = context
            .eval(Source::from_bytes(
                r#"
                {
                    const m = performance.mark("custom-mark", {
                        detail: { foo: "bar" },
                        startTime: 123.45
                    });
                    m.name === "custom-mark" &&
                    m.startTime === 123.45 &&
                    m.detail.foo === "bar"
                }
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 2. Mark with negative startTime should throw TypeError
        let res = context
            .eval(Source::from_bytes(
                r#"
                {
                    let threw = false;
                    try {
                        performance.mark("negative", { startTime: -5 });
                    } catch (e) {
                        threw = e instanceof TypeError;
                    }
                    threw
                }
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));
    }

    #[test]
    fn test_performance_user_timing_l3_measure_options() {
        let mut context = Context::default();
        let performance = create_performance(&mut context);
        let _ = context.register_global_property(
            JsString::from("performance"),
            performance,
            Attribute::all(),
        );

        // Setup custom marks
        context
            .eval(Source::from_bytes(
                r#"
                performance.mark("m1", { startTime: 100 });
                performance.mark("m2", { startTime: 250 });
                "#,
            ))
            .unwrap();

        // 1. Measure with start and end
        let res = context
            .eval(Source::from_bytes(
                r#"
                {
                    const m = performance.measure("meas-1", {
                        start: "m1",
                        end: "m2",
                        detail: "metadata-1"
                    });
                    m.startTime === 100 &&
                    m.duration === 150 &&
                    m.detail === "metadata-1"
                }
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 2. Measure with start and duration
        let res = context
            .eval(Source::from_bytes(
                r#"
                {
                    const m = performance.measure("meas-2", {
                        start: "m1",
                        duration: 50
                    });
                    m.startTime === 100 &&
                    m.duration === 50
                }
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 3. Measure with duration and end
        let res = context
            .eval(Source::from_bytes(
                r#"
                {
                    const m = performance.measure("meas-3", {
                        end: "m2",
                        duration: 75
                    });
                    m.startTime === 175 &&
                    m.duration === 75
                }
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 4. Measure with negative duration throws TypeError
        let res = context
            .eval(Source::from_bytes(
                r#"
                {
                    let threw = false;
                    try {
                        performance.measure("meas-neg", {
                            start: "m1",
                            duration: -10
                        });
                    } catch (e) {
                        threw = e instanceof TypeError;
                    }
                    threw
                }
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 5. Measure with start, duration, and end throws TypeError
        let res = context
            .eval(Source::from_bytes(
                r#"
                {
                    let threw = false;
                    try {
                        performance.measure("meas-all", {
                            start: "m1",
                            end: "m2",
                            duration: 100
                        });
                    } catch (e) {
                        threw = e instanceof TypeError;
                    }
                    threw
                }
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 6. Measure with duration and neither start nor end throws TypeError
        let res = context
            .eval(Source::from_bytes(
                r#"
                {
                    let threw = false;
                    try {
                        performance.measure("meas-dur-only", {
                            duration: 100
                        });
                    } catch (e) {
                        threw = e instanceof TypeError;
                    }
                    threw
                }
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // 7. Measure with options and endMark throws TypeError
        let res = context
            .eval(Source::from_bytes(
                r#"
                {
                    let threw = false;
                    try {
                        performance.measure("meas-conflict", { start: "m1" }, "m2");
                    } catch (e) {
                        threw = e instanceof TypeError;
                    }
                    threw
                }
                "#,
            ))
            .unwrap();
        assert_eq!(res.as_boolean(), Some(true));
    }
}
