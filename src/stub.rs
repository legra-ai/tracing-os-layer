//! No-op stub for platforms without a supported OS-native
//! logging backend.

use tracing_core::Subscriber;
use tracing_subscriber::layer::Layer;
use tracing_subscriber::registry::LookupSpan;

/// No-op [`Layer`] for unsupported platforms.
///
/// [`OsLogLayer::try_new`] always returns `None` on platforms
/// other than macOS and Linux.
pub struct OsLogLayer;

impl OsLogLayer {
    /// Always returns `None` on this platform.
    #[must_use]
    pub fn try_new(_subsystem: &str, _category: &str) -> Option<Self> {
        None
    }
}

impl<S> Layer<S> for OsLogLayer where S: Subscriber + for<'a> LookupSpan<'a> {}
