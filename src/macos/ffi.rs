//! FFI declarations and safe Rust wrappers for Apple Unified
//! Logging and Activity Tracing.

use std::ffi::{CString, c_char, c_uint, c_void};

/// Opaque `os_log_t` handle.
type RawOsLog = *mut c_void;

/// Opaque `os_activity_t` handle.
type RawOsActivity = *mut c_void;

/// Apple `os_log_type_t` discriminant.
///
/// Values match `<os/log.h>` constants.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OsLogType {
    /// `OS_LOG_TYPE_DEFAULT` -- visible in Console.app by default.
    Default = 0,
    /// `OS_LOG_TYPE_INFO` -- hidden by default, persisted on log
    /// collect.
    Info = 1,
    /// `OS_LOG_TYPE_DEBUG` -- hidden by default, not persisted.
    Debug = 2,
    /// `OS_LOG_TYPE_ERROR` -- always persisted, shown in orange.
    Error = 16,
    /// `OS_LOG_TYPE_FAULT` -- always persisted, shown in red.
    Fault = 17,
}

/// Activity scope state saved on the stack (16 bytes, opaque).
#[repr(C)]
struct ScopeState {
    opaque: [u64; 2],
}

// ── extern "C" ──────────────────────────────────────────────

unsafe extern "C" {
    // Real functions callable directly from Rust.
    fn os_log_create(subsystem: *const c_char, category: *const c_char) -> RawOsLog;
    fn os_log_type_enabled(log: RawOsLog, log_type: u8) -> bool;
    fn os_activity_scope_enter(activity: RawOsActivity, state: *mut ScopeState);
    fn os_activity_scope_leave(state: *mut ScopeState);
    fn os_release(object: *mut c_void);

    // C wrappers for macros (csrc/wrapper.c).
    fn tracing_os_layer_log_with_type(log: RawOsLog, log_type: u8, message: *const c_char);
    fn tracing_os_layer_activity_create(
        description: *const c_char,
        parent: RawOsActivity,
        flags: c_uint,
    ) -> RawOsActivity;
    fn tracing_os_layer_activity_current() -> RawOsActivity;
}

// ── OsLog ───────────────────────────────────────────────────

/// Safe handle to an `os_log_t` instance.
///
/// Calls `os_release` on drop.
pub(crate) struct OsLog {
    raw: RawOsLog,
}

impl OsLog {
    /// Create a log handle for the given subsystem and category.
    pub(crate) fn new(subsystem: &str, category: &str) -> Self {
        let subsystem = CString::new(subsystem).expect("subsystem contains interior NUL");
        let category = CString::new(category).expect("category contains interior NUL");
        let raw = unsafe { os_log_create(subsystem.as_ptr(), category.as_ptr()) };
        Self { raw }
    }

    /// Check whether the given log type is enabled by the system.
    pub(crate) fn type_enabled(&self, log_type: OsLogType) -> bool {
        unsafe { os_log_type_enabled(self.raw, log_type as u8) }
    }

    /// Emit a pre-formatted message at the given log type.
    pub(crate) fn emit(&self, log_type: OsLogType, message: &CString) {
        unsafe {
            tracing_os_layer_log_with_type(self.raw, log_type as u8, message.as_ptr());
        }
    }
}

impl Drop for OsLog {
    fn drop(&mut self) {
        unsafe { os_release(self.raw) };
    }
}

// SAFETY: `os_log_t` is documented as thread-safe by Apple.
unsafe impl Send for OsLog {}
// SAFETY: `os_log_t` is documented as thread-safe by Apple.
unsafe impl Sync for OsLog {}

// ── Activity ────────────────────────────────────────────────

/// Safe handle to an `os_activity_t` instance.
///
/// Stored in span extensions. Calls `os_release` on drop.
pub(crate) struct Activity {
    raw: RawOsActivity,
}

impl Activity {
    /// Create a new activity with the given description and
    /// parent.
    pub(crate) fn create(description: &CString, parent: RawOsActivity) -> Self {
        let raw = unsafe {
            tracing_os_layer_activity_create(
                description.as_ptr(),
                parent,
                0, // OS_ACTIVITY_FLAG_DEFAULT
            )
        };
        Self { raw }
    }

    /// Raw pointer for use as a parent in child activity
    /// creation.
    pub(crate) fn raw(&self) -> RawOsActivity {
        self.raw
    }

    /// Raw pointer for `OS_ACTIVITY_CURRENT`.
    pub(crate) fn current_raw() -> RawOsActivity {
        unsafe { tracing_os_layer_activity_current() }
    }

    /// Execute a closure within this activity's scope.
    ///
    /// Calls `os_activity_scope_enter` before and
    /// `os_activity_scope_leave` after, ensuring the scope is
    /// always cleaned up even if the closure panics.
    pub(crate) fn scoped<F: FnOnce()>(&self, f: F) {
        let mut state = ScopeState { opaque: [0; 2] };
        unsafe { os_activity_scope_enter(self.raw, &raw mut state) };
        f();
        unsafe { os_activity_scope_leave(&raw mut state) };
    }
}

impl Drop for Activity {
    fn drop(&mut self) {
        unsafe { os_release(self.raw) };
    }
}

// SAFETY: `os_activity_t` is reference-counted and thread-safe.
unsafe impl Send for Activity {}
// SAFETY: `os_activity_t` is reference-counted and thread-safe.
unsafe impl Sync for Activity {}
