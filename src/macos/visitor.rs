//! Field visitor for collecting tracing span/event attributes
//! into a string map.

use std::collections::BTreeMap;
use std::fmt::Debug;

use tracing_core::field::{Field, Visit};

/// Ordered map of attribute names to their string values.
pub(crate) type AttributeMap = BTreeMap<String, String>;

/// Collects tracing field values into an [`AttributeMap`].
pub(crate) struct FieldVisitor<'a> {
    output: &'a mut AttributeMap,
}

impl<'a> FieldVisitor<'a> {
    /// Create a visitor that writes into `output`.
    pub(crate) fn new(output: &'a mut AttributeMap) -> Self {
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
