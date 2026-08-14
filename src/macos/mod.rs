//! macOS implementation using Apple Unified Logging (`os_log`)
//! and Activity Tracing (`os_activity`).

mod ffi;
mod visitor;

use ffi::{Activity, OsLog, OsLogType};
use std::ffi::CString;
use tracing_core::span::{Attributes, Id, Record};
use tracing_core::{Event, Level, Subscriber};
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::registry::LookupSpan;
use visitor::{AttributeMap, FieldVisitor};

/// Async-safe [`Layer`] that emits events to Apple Unified
/// Logging.
///
/// Activity scoping happens entirely within [`Layer::on_event`]:
/// scope-enter, log, scope-leave -- all synchronous, all on the
/// same thread. This avoids the thread-local corruption that
/// occurs when `on_enter`/`on_exit` span across tokio task
/// migrations.
pub struct OsLogLayer {
    log: OsLog,
}

impl OsLogLayer {
    /// Create a layer for the given subsystem and category.
    ///
    /// Always returns `Some` on macOS -- `os_log_create` does not
    /// fail.
    ///
    /// # Arguments
    ///
    /// * `subsystem` -- reverse-DNS identifier (e.g. `"com.example.app"`).
    /// * `category` -- category within the subsystem (e.g. `"worker"`).
    #[must_use]
    pub fn try_new(subsystem: &str, category: &str) -> Option<Self> {
        Some(Self {
            log: OsLog::new(subsystem, category),
        })
    }
}

impl<S> Layer<S> for OsLogLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let Some(span) = ctx.span(id) else { return };
        let mut extensions = span.extensions_mut();
        if extensions.get_mut::<Activity>().is_some() {
            return;
        }

        // Determine parent activity: walk up the span tree, fall
        // back to OS_ACTIVITY_CURRENT if no parent has one.
        let parent_activity = span
            .parent()
            .and_then(|p| p.extensions().get::<Activity>().map(Activity::raw))
            .unwrap_or_else(Activity::current_raw);

        // Build a human-readable description for Console.app.
        let metadata = span.metadata();
        let mut attributes = AttributeMap::default();
        let mut v = FieldVisitor::new(&mut attributes);
        attrs.record(&mut v);
        let description = format_span_description(metadata.target(), metadata.name(), &attributes);

        let c_name = CString::new(description.replace('\0', ""))
            .expect("sanitised string cannot contain NUL");
        let activity = Activity::create(&c_name, parent_activity);
        extensions.insert(activity);
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let metadata = event.metadata();
        let log_type = level_to_os_log_type(*metadata.level());

        // Skip formatting when the system has this level disabled.
        if !self.log.type_enabled(log_type) {
            return;
        }

        let mut attributes = AttributeMap::default();
        let mut v = FieldVisitor::new(&mut attributes);
        event.record(&mut v);
        let message = format_event_message(&attributes);
        let Ok(c_message) = CString::new(message) else {
            return;
        };

        // Scope-enter the current span's activity for just the
        // duration of the os_log call.
        if let Some(span_ref) = ctx.event_span(event) {
            let extensions = span_ref.extensions();
            if let Some(activity) = extensions.get::<Activity>() {
                activity.scoped(|| {
                    self.log.emit(log_type, &c_message);
                });
                return;
            }
        }

        // No parent span or no activity -- emit without scope.
        self.log.emit(log_type, &c_message);
    }

    fn on_record(&self, _id: &Id, _values: &Record<'_>, _ctx: Context<'_, S>) {
        // Span descriptions are set at creation time.
    }

    fn on_close(&self, id: Id, ctx: Context<'_, S>) {
        let Some(span) = ctx.span(&id) else { return };
        let mut extensions = span.extensions_mut();
        // Remove triggers Activity::drop -> os_release.
        extensions.remove::<Activity>();
    }

    // Deliberately NO on_enter / on_exit -- this is the key
    // async-safety property.
}

// ── Helpers ─────────────────────────────────────────────────

/// Map tracing severity to `os_log_type_t`.
///
/// Matches the semantics of Console.app colour coding:
///
/// | tracing | os_log  | Console.app |
/// |---------|---------|-------------|
/// | TRACE   | DEBUG   | hidden      |
/// | DEBUG   | INFO    | hidden      |
/// | INFO    | DEFAULT | visible     |
/// | WARN    | ERROR   | orange      |
/// | ERROR   | FAULT   | red         |
fn level_to_os_log_type(level: Level) -> OsLogType {
    match level {
        Level::TRACE => OsLogType::Debug,
        Level::DEBUG => OsLogType::Info,
        Level::INFO => OsLogType::Default,
        Level::WARN => OsLogType::Error,
        Level::ERROR => OsLogType::Fault,
    }
}

/// Format a span description for `os_activity_create`.
///
/// Produces `target::name(key=val, ...)` or `target::name` when
/// there are no attributes.
fn format_span_description(target: &str, name: &str, attributes: &AttributeMap) -> String {
    if attributes.is_empty() {
        return format!("{target}::{name}");
    }
    let pairs: Vec<String> = attributes.iter().map(|(k, v)| format!("{k}={v}")).collect();
    format!("{target}::{name}({})", pairs.join(", "))
}

/// Format an event message for `os_log_with_type`.
///
/// The `message` field (if present) leads; remaining fields are
/// appended as `key=value` pairs.
fn format_event_message(attributes: &AttributeMap) -> String {
    let mut msg = String::new();
    if let Some(m) = attributes.get("message") {
        msg.push_str(m);
    }
    for (k, v) in attributes {
        if k == "message" {
            continue;
        }
        if !msg.is_empty() {
            msg.push(' ');
        }
        msg.push_str(k);
        msg.push('=');
        msg.push_str(v);
    }
    // Strip interior NULs so CString::new doesn't fail.
    msg.retain(|c| c != '\0');
    msg
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn formats_span_without_attributes() {
        let attributes = BTreeMap::new();
        assert_eq!(
            format_span_description("com.example.app", "worker", &attributes),
            "com.example.app::worker"
        );
    }

    #[test]
    fn formats_span_with_sorted_attributes() {
        let attributes = BTreeMap::from([
            ("records".to_owned(), "12".to_owned()),
            ("mode".to_owned(), "batch".to_owned()),
        ]);
        assert_eq!(
            format_span_description("app", "worker", &attributes),
            "app::worker(mode=batch, records=12)"
        );
    }

    #[test]
    fn formats_event_message_with_message_first() {
        let attributes = BTreeMap::from([
            ("records".to_owned(), "12".to_owned()),
            ("message".to_owned(), "processed".to_owned()),
        ]);
        assert_eq!(format_event_message(&attributes), "processed records=12");
    }

    #[test]
    fn strips_nul_bytes_from_event_message() {
        let attributes = BTreeMap::from([("message".to_owned(), "a\0b".to_owned())]);
        assert_eq!(format_event_message(&attributes), "ab");
    }

    #[test]
    fn maps_tracing_levels_to_native_levels() {
        assert_eq!(level_to_os_log_type(Level::TRACE), OsLogType::Debug);
        assert_eq!(level_to_os_log_type(Level::DEBUG), OsLogType::Info);
        assert_eq!(level_to_os_log_type(Level::INFO), OsLogType::Default);
        assert_eq!(level_to_os_log_type(Level::WARN), OsLogType::Error);
        assert_eq!(level_to_os_log_type(Level::ERROR), OsLogType::Fault);
    }
}
