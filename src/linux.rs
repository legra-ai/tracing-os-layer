//! Linux implementation using systemd journal via
//! `tracing-journald`.

use tracing_core::span::{Attributes, Id, Record};
use tracing_core::{Event, Subscriber};
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::registry::LookupSpan;

/// [`Layer`] that emits events to the systemd journal.
///
/// Wraps [`tracing_journald::Layer`] with a constructor that
/// matches the cross-platform [`OsLogLayer`](crate::OsLogLayer)
/// API.
pub struct OsLogLayer {
    inner: tracing_journald::Layer,
}

impl OsLogLayer {
    /// Create a journald layer.
    ///
    /// Returns `None` when the journald socket is unavailable
    /// (e.g. inside a minimal container without systemd).
    ///
    /// The `subsystem` and `category` parameters are accepted for
    /// API parity with the macOS variant but are not used --
    /// journald identifies the source by unit/PID.
    #[must_use]
    pub fn try_new(_subsystem: &str, _category: &str) -> Option<Self> {
        tracing_journald::layer().ok().map(|inner| Self { inner })
    }
}

impl<S> Layer<S> for OsLogLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        self.inner.on_new_span(attrs, id, ctx);
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>) {
        self.inner.on_record(id, values, ctx);
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        self.inner.on_event(event, ctx);
    }
}
