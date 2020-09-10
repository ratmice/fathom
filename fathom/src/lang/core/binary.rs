//! Binary interpreter for Fathom.
//!
//! This is only a naive implementation, and intended for getting a better idea
//! of whether our compiled back-ends actually meet the specification.

use num_bigint::BigInt;
use std::collections::BTreeMap;

pub mod read;

/// Terms that can be produced as a result of reading a binary file, or used as
/// a source from which to write binary data.
#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    /// Integers.
    Int(BigInt),
    /// IEEE-754 single-precision floating point numbers.
    F32(f32),
    /// IEEE-754 double-precision floating point numbers.
    F64(f64),
    /// Sequences
    Seq(Vec<Term>),
    /// Structure values
    Struct(BTreeMap<String, Term>),
}

impl Term {
    pub fn int(data: impl Into<BigInt>) -> Term {
        Term::Int(data.into())
    }
}