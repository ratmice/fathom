//! Diagnostics.

use codespan_reporting::diagnostic::{Diagnostic, Label, Severity};
use std::ops::Range;

pub fn non_format_type_as_host_type(
    severity: Severity,
    file_id: usize,
    range: Range<usize>,
) -> Diagnostic<usize> {
    Diagnostic::new(severity)
        .with_message("attempted to compile a non-format type as a host type")
        .with_labels(vec![
            Label::primary(file_id, range).with_message("not a format type")
        ])
}

pub mod error {
    use super::*;

    pub fn unconstrained_int(file_id: usize, range: Range<usize>) -> Diagnostic<usize> {
        Diagnostic::error()
            .with_message("cannot compile unconstrained integer types")
            .with_labels(vec![
                Label::primary(file_id, range).with_message("unconstrained integer type")
            ])
    }
}

pub mod bug {
    pub use super::*;

    pub fn item_name_reused(
        file_id: usize,
        name: &str,
        found: Range<usize>,
        original: Range<usize>,
    ) -> Diagnostic<usize> {
        Diagnostic::bug()
            .with_message(format!(
                "attempted to compile an item named `{}` multiple times",
                name,
            ))
            .with_labels(vec![
                Label::primary(file_id, found).with_message("redefined here"),
                Label::secondary(file_id, original).with_message("original item here"),
            ])
            .with_notes(vec![format!(
                "`{}` must be defined only once in this module",
                name,
            )])
    }

    pub fn oversaturated_fun_elim(file_id: usize, range: Range<usize>) -> Diagnostic<usize> {
        Diagnostic::bug()
            .with_message("attempted to compile an oversaturated function elimination")
            .with_labels(vec![
                Label::primary(file_id, range).with_message("too many eliminations")
            ])
    }

    pub fn unexpected_elim(file_id: usize, range: Range<usize>) -> Diagnostic<usize> {
        Diagnostic::bug()
            .with_message("unexpected elimination")
            .with_labels(vec![
                Label::primary(file_id, range).with_message("unexpected elimination")
            ])
    }

    pub fn integer_out_of_bounds(file_id: usize, range: Range<usize>) -> Diagnostic<usize> {
        Diagnostic::bug()
            .with_message("attempted to compile an out of bounds integer")
            .with_labels(vec![
                Label::primary(file_id, range).with_message("integer out of bounds")
            ])
    }

    pub fn expected_integer(file_id: usize, range: Range<usize>) -> Diagnostic<usize> {
        Diagnostic::bug()
            .with_message("attempted to compile this expression as an integer")
            .with_labels(vec![
                Label::primary(file_id, range).with_message("not an integer")
            ])
    }

    pub fn expected_type(file_id: usize, range: Range<usize>) -> Diagnostic<usize> {
        Diagnostic::bug()
            .with_message("attempted to compile this expression as a type")
            .with_labels(vec![
                Label::primary(file_id, range).with_message("not a type")
            ])
    }

    pub fn unbound_item(file_id: usize, name: &str, range: Range<usize>) -> Diagnostic<usize> {
        Diagnostic::bug()
            .with_message(format!(
                "attempted to compile an item `{}` that was not yet bound",
                name,
            ))
            .with_labels(vec![
                Label::primary(file_id, range).with_message("item not found in this scope")
            ])
        // TODO: provide suggestions
    }
}