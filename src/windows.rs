//! Windows implementation using the Windows Event Log Application log.

use std::collections::BTreeMap;
use std::fmt::Debug;

use eventlog::EventLog;
use log::{Level as LogLevel, Log, Record};
use tracing_core::field::{Field, Visit};
use tracing_core::{Event, Level, Subscriber};
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::registry::LookupSpan;

/// [`Layer`] that emits events to the Windows Event Log Application log.
pub struct OsLogLayer {
    source: EventLog,
    category: String,
}

impl OsLogLayer {
    /// Create a Windows Event Log layer for the given subsystem and category.
    ///
    /// The `subsystem` must be registered as an event source under the
    /// Windows Event Log `Application` log before this method is called.
    /// Installers commonly perform that registration because it normally
    /// requires administrator permission. Returns `None` when the source is
    /// unavailable.
    ///
    /// The `category` is included in every event message because the classic
    /// Event Log API uses the registered source name as its grouping key.
    ///
    /// # Panics
    ///
    /// Panics when `subsystem` or `category` contains an interior NUL.
    #[must_use]
    pub fn try_new(subsystem: &str, category: &str) -> Option<Self> {
        assert!(
            !subsystem.contains('\0'),
            "subsystem contains an interior NUL"
        );
        assert!(
            !category.contains('\0'),
            "category contains an interior NUL"
        );

        let source = EventLog::new(subsystem, LogLevel::Trace).ok()?;

        Some(Self {
            source,
            category: category.to_owned(),
        })
    }
}

impl<S> Layer<S> for OsLogLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut attributes = AttributeMap::default();
        event.record(&mut FieldVisitor::new(&mut attributes));
        let message = format_event_message(&self.category, &attributes);
        let level = level_to_log_level(*event.metadata().level());
        let args = format_args!("{message}");
        let record = Record::builder()
            .args(args)
            .level(level)
            .target(event.metadata().target())
            .build();
        self.source.log(&record);
    }
}

// ── Formatting ─────────────────────────────────────────────

type AttributeMap = BTreeMap<String, String>;

struct FieldVisitor<'a> {
    output: &'a mut AttributeMap,
}

impl<'a> FieldVisitor<'a> {
    fn new(output: &'a mut AttributeMap) -> Self {
        Self { output }
    }
}

impl Visit for FieldVisitor<'_> {
    fn record_i64(&mut self, field: &Field, value: i64) {
        self.output
            .insert(field.name().to_owned(), value.to_string());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.output
            .insert(field.name().to_owned(), value.to_string());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.output
            .insert(field.name().to_owned(), value.to_string());
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.output
            .insert(field.name().to_owned(), value.to_owned());
    }

    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        self.output
            .insert(field.name().to_owned(), format!("{value:?}"));
    }
}

fn format_event_message(category: &str, attributes: &AttributeMap) -> String {
    let mut message = format!("[{category}]");
    if let Some(value) = attributes.get("message") {
        message.push(' ');
        message.push_str(value);
    }
    for (key, value) in attributes {
        if key == "message" {
            continue;
        }
        message.push(' ');
        message.push_str(key);
        message.push('=');
        message.push_str(value);
    }
    message.retain(|character| character != '\0');
    message
}

fn level_to_log_level(level: Level) -> LogLevel {
    match level {
        Level::ERROR => LogLevel::Error,
        Level::WARN => LogLevel::Warn,
        Level::INFO => LogLevel::Info,
        Level::DEBUG => LogLevel::Debug,
        Level::TRACE => LogLevel::Trace,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_category_message_first() {
        let attributes = BTreeMap::from([
            ("records".to_owned(), "12".to_owned()),
            ("message".to_owned(), "processed".to_owned()),
        ]);
        assert_eq!(
            format_event_message("worker", &attributes),
            "[worker] processed records=12"
        );
    }

    #[test]
    fn formats_message_without_message_field() {
        let attributes = BTreeMap::from([("records".to_owned(), "12".to_owned())]);
        assert_eq!(
            format_event_message("worker", &attributes),
            "[worker] records=12"
        );
    }

    #[test]
    fn strips_nul_bytes_before_wide_encoding() {
        let attributes = BTreeMap::from([("message".to_owned(), "a\0b".to_owned())]);
        assert_eq!(format_event_message("worker", &attributes), "[worker] ab");
    }

    #[test]
    fn maps_tracing_levels_to_log_levels() {
        assert_eq!(level_to_log_level(Level::ERROR), LogLevel::Error);
        assert_eq!(level_to_log_level(Level::WARN), LogLevel::Warn);
        assert_eq!(level_to_log_level(Level::INFO), LogLevel::Info);
        assert_eq!(level_to_log_level(Level::DEBUG), LogLevel::Debug);
        assert_eq!(level_to_log_level(Level::TRACE), LogLevel::Trace);
    }
}
