//! Core language.

use crate::env::{GlobalVar, LocalVar};
use crate::StringId;

pub mod binary;
pub mod semantics;

/// Information about how entries were bound in the rigid environment. This is
/// used when inserting [flexible variables][Term::FlexibleInsertion] during
/// elaboration.
//
// See also: https://en.wikipedia.org/wiki/Abstract_and_concrete
#[derive(Debug, Copy, Clone)]
pub enum EntryInfo {
    /// The entry was bound as a definition in the environment.
    Definition,
    /// The entry was bound as a parameter in the environment
    Parameter,
}

/// Core language terms.
#[derive(Debug, Clone)]
pub enum Term<'arena> {
    /// Rigid variable occurrences.
    ///
    /// These correspond to variables that were most likely bound as a result of
    /// user code, for example from [let expressions]), [function types] and
    /// [function literals].
    ///
    /// [let expressions]: Term::Let
    /// [function types]: Term::FunType
    /// [function literals]: Term::FunLit
    ///
    /// ## References
    ///
    /// - [A unification algorithm for typed λ-calculus](https://doi.org/10.1016/0304-3975(75)90011-0)
    /// - [Type Classes: Rigid type variables](https://typeclasses.com/rigid-type-variables)
    RigidVar(LocalVar),
    /// Flexible variable occurrences.
    ///
    /// These are inserted during [elaboration] when we have something we want
    /// pattern unification to fill in for us. They are 'flexible' because the
    /// expressions that they correspond to might be updated (from unknown to
    /// known) during unification.
    ///
    /// Also known as: metavariables.
    ///
    /// [elaboration]: crate::surface::elaboration
    FlexibleVar(GlobalVar),
    /// A flexible variable that has been inserted during elaboration, along
    /// with the [entry information] in the rigid environment at the time of
    /// insertion.
    ///
    /// The entry information will let us know what rigidly bound parameters to
    /// apply to the flexible variable during [evaluation]. The applied
    /// parameters will correspond to the [function literals] that will be
    /// added to the flexible solution during unification.
    ///
    /// We clone the entry information and perform the function applications
    /// during evaluation because elaborating to a series of [function
    /// applications] directly would involve expensively [quoting] each
    /// parameter.
    ///
    /// For example, given the following code:
    ///
    /// ```text
    /// let test : fn (A : Type) -> A -> A
    ///   = fn A => fn a =>
    ///        let b : A = a; ?x;
    /// Type
    /// ```
    ///
    /// This would be elaborated to:
    ///
    /// ```text
    /// let test : fn (A : Type) -> A -> A
    ///   = fn A => fn a =>
    /// //     │       │
    /// //     │       ╰────────────╮
    /// //     ╰──────────────────╮ │
    /// //                        │ │
    /// //                        ▼ ▼
    ///        let b : A = a; (?x A a);
    /// //                     ^^^^^^  the flexible insertion
    /// Type
    /// ```
    ///
    /// Notice how `A` and `a` are applied to the flexible variable `?x`,
    /// because they are bound as rigid parameters, where as `b` is _not_
    /// applied, because it is bound as a definition.
    ///
    /// [entry information]: EntryInfo
    /// [function literals]: Term::FunLit
    /// [function applications]: Term::FunApp
    /// [evaluation]: semantics::EvalContext::eval
    /// [quoting]: semantics::QuoteContext::quote
    //
    // TODO: Bit-vectors might make this a bit more compact and cheaper to
    //       construct. For example:
    //
    // - https://lib.rs/crates/smallbitvec
    // - https://lib.rs/crates/bit-vec
    FlexibleInsertion(GlobalVar, &'arena [EntryInfo]),
    /// Annotated expressions.
    Ann(&'arena Term<'arena>, &'arena Term<'arena>),
    /// Let expressions.
    Let(
        Option<StringId>,
        &'arena Term<'arena>,
        &'arena Term<'arena>,
        &'arena Term<'arena>,
    ),

    /// The type of types.
    Universe,

    /// Dependent function types.
    ///
    /// Also known as: pi types, dependent product types.
    FunType(Option<StringId>, &'arena Term<'arena>, &'arena Term<'arena>),
    /// Function literals.
    ///
    /// Also known as: lambda expressions, anonymous functions.
    FunLit(Option<StringId>, &'arena Term<'arena>),
    /// Function applications.
    FunApp(&'arena Term<'arena>, &'arena Term<'arena>),

    /// Dependent record types.
    RecordType(&'arena [StringId], &'arena [Term<'arena>]),
    /// Record literals.
    RecordLit(&'arena [StringId], &'arena [Term<'arena>]),
    /// Record projections.
    RecordProj(&'arena Term<'arena>, StringId),

    /// Array literals.
    ArrayLit(&'arena [Term<'arena>]),

    /// Record formats, consisting of a list of dependent formats.
    FormatRecord(&'arena [StringId], &'arena [Term<'arena>]),
    /// Overlap formats, consisting of a list of dependent formats, overlapping
    /// in memory.
    FormatOverlap(&'arena [StringId], &'arena [Term<'arena>]),

    /// Primitives.
    Prim(Prim),

    /// Constant literals.
    ConstLit(Const),
    /// Match on a constant.
    ///
    /// (head_expr, pattern_branches, default_expr)
    ConstMatch(
        &'arena Term<'arena>,
        &'arena [(Const, Term<'arena>)],
        Option<&'arena Term<'arena>>,
    ),
}

macro_rules! def_prims {
    ($($(#[$prim_attr:meta])* $PrimName:ident => $prim_name:literal),* $(,)?) => {
        /// Primitives.
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        pub enum Prim {
            $($(#[$prim_attr])* $PrimName),*
        }

        impl Prim {
            pub const fn name(&self) -> &'static str {
                match self {
                    $(Prim::$PrimName => $prim_name),*
                }
            }
        }
    };
}

def_prims! {
    /// Void type.
    VoidType => "Void",

    /// Type of booleans.
    BoolType => "Bool",
    /// Type of unsigned, 8-bit integers.
    U8Type => "U8",
    /// Type of unsigned, 16-bit integers.
    U16Type => "U16",
    /// Type of unsigned, 32-bit integers.
    U32Type => "U32",
    /// Type of unsigned, 64-bit integers.
    U64Type => "U64",
    /// Type of signed, two's complement, 8-bit integers.
    S8Type => "S8",
    /// Type of signed, two's complement, 16-bit integers.
    S16Type => "S16",
    /// Type of signed, two's complement, 32-bit integers.
    S32Type => "S32",
    /// Type of signed, two's complement, 64-bit integers.
    S64Type => "S64",
    /// Type of 32-bit, IEEE-754 floating point numbers.
    F32Type => "F32",
    /// Type of 64-bit, IEEE-754 floating point numbers.
    F64Type => "F64",
    /// Type of optional data.
    OptionType => "Option",
    /// Type of dynamically sized arrays.
    ArrayType => "Array",
    /// Type of arrays, with 8-bit indices.
    Array8Type => "Array8",
    /// Type of arrays, with 16-bit indices.
    Array16Type => "Array16",
    /// Type of arrays, with 32-bit indices.
    Array32Type => "Array32",
    /// Type of arrays, with 64-bit indices.
    Array64Type => "Array64",
    /// Type of stream positions.
    PosType => "Pos",
    /// Type of stream references.
    RefType => "Ref",

    /// Type of format descriptions.
    FormatType => "Format",
    /// Unsigned, 8-bit integer formats.
    FormatU8 => "u8",
    /// Unsigned, 16-bit integer formats (big-endian).
    FormatU16Be => "u16be",
    /// Unsigned, 16-bit integer formats (little-endian).
    FormatU16Le => "u16le",
    /// Unsigned, 32-bit integer formats (big-endian).
    FormatU32Be => "u32be",
    /// Unsigned, 32-bit integer formats (little-endian).
    FormatU32Le => "u32le",
    /// Unsigned, 64-bit integer formats (big-endian).
    FormatU64Be => "u64be",
    /// Unsigned, 64-bit integer formats (little-endian).
    FormatU64Le => "u64le",
    /// Signed, two's complement, 8-bit integer formats.
    FormatS8 => "s8",
    /// Signed, two's complement, 16-bit integer formats (big-endian).
    FormatS16Be => "s16be",
    /// Signed, two's complement, 16-bit integer formats (little-endian).
    FormatS16Le => "s16le",
    /// Signed, two's complement, 32-bit integer formats (big-endian).
    FormatS32Be => "s32be",
    /// Signed, two's complement, 32-bit integer formats (little-endian).
    FormatS32Le => "s32le",
    /// Signed, two's complement, 64-bit integer formats (big-endian).
    FormatS64Be => "s64be",
    /// Signed, two's complement, 64-bit integer formats (little-endian).
    FormatS64Le => "s64le",
    /// 32-bit, IEEE-754 floating point formats (big-endian).
    FormatF32Be => "f32be",
    /// 32-bit, IEEE-754 floating point formats (little-endian).
    FormatF32Le => "f32le",
    /// 64-bit, IEEE-754 floating point formats (big-endian).
    FormatF64Be => "f64be",
    /// 64-bit, IEEE-754 floating point formats (little-endian).
    FormatF64Le => "f64le",
    /// Array formats, with unsigned 8-bit indices.
    FormatArray8 => "array8",
    /// Array formats, with unsigned 16-bit indices.
    FormatArray16 => "array16",
    /// Array formats, with unsigned 32-bit indices.
    FormatArray32 => "array32",
    /// Array formats, with unsigned 64-bit indices.
    FormatArray64 => "array64",
    /// Repeat a format until the length of the given parse scope is reached.
    FormatRepeatUntilEnd => "repeat_until_end",
    /// A format which returns the current position in the input stream.
    FormatStreamPos => "stream_pos",
    /// A format that links to another location in the binary data stream,
    /// relative to a base position.
    FormatLink => "link",
    /// A format that forces a reference to be read eagerly.
    FormatDeref => "deref",
    /// A format that always succeeds with some data.
    FormatSucceed => "succeed",
    /// A format that always fails to parse.
    FormatFail => "fail",
    /// Unwrap an option, or fail to parse.
    FormatUnwrap => "unwrap",
    /// Format representations.
    FormatRepr => "Repr",

    /// Reported errors.
    ReportedError => "reported_error",

    BoolEq  => "bool_eq",
    BoolNeq => "bool_neq",
    BoolNot => "bool_not",
    BoolAnd => "bool_and",
    BoolOr  => "bool_or",
    BoolXor => "bool_xor",

    U8Eq  => "u8_eq",
    U8Neq => "u8_neq",
    U8Gt  => "u8_gt",
    U8Lt  => "u8_lt",
    U8Gte => "u8_gte",
    U8Lte => "u8_lte",
    U8Add => "u8_add",
    U8Sub => "u8_sub",
    U8Mul => "u8_mul",
    U8Div => "u8_div",
    U8Not => "u8_not",
    U8Shl => "u8_shl",
    U8Shr => "u8_shr",
    U8And => "u8_and",
    U8Or  => "u8_or",
    U8Xor => "u8_xor",

    U16Eq  => "u16_eq",
    U16Neq => "u16_neq",
    U16Gt  => "u16_gt",
    U16Lt  => "u16_lt",
    U16Gte => "u16_gte",
    U16Lte => "u16_lte",
    U16Add => "u16_add",
    U16Sub => "u16_sub",
    U16Mul => "u16_mul",
    U16Div => "u16_div",
    U16Not => "u16_not",
    U16Shl => "u16_shl",
    U16Shr => "u16_shr",
    U16And => "u16_and",
    U16Or  => "u16_or",
    U16Xor => "u16_xor",

    U32Eq  => "u32_eq",
    U32Neq => "u32_neq",
    U32Gt  => "u32_gt",
    U32Lt  => "u32_lt",
    U32Gte => "u32_gte",
    U32Lte => "u32_lte",
    U32Add => "u32_add",
    U32Sub => "u32_sub",
    U32Mul => "u32_mul",
    U32Div => "u32_div",
    U32Not => "u32_not",
    U32Shl => "u32_shl",
    U32Shr => "u32_shr",
    U32And => "u32_and",
    U32Or  => "u32_or",
    U32Xor => "u32_xor",

    U64Eq  => "u64_eq",
    U64Neq => "u64_neq",
    U64Gt  => "u64_gt",
    U64Lt  => "u64_lt",
    U64Gte => "u64_gte",
    U64Lte => "u64_lte",
    U64Add => "u64_add",
    U64Sub => "u64_sub",
    U64Mul => "u64_mul",
    U64Div => "u64_div",
    U64Not => "u64_not",
    U64Shl => "u64_shl",
    U64Shr => "u64_shr",
    U64And => "u64_and",
    U64Or  => "u64_or",
    U64Xor => "u64_xor",

    S8Eq  => "s8_eq",
    S8Neq => "s8_neq",
    S8Gt  => "s8_gt",
    S8Lt  => "s8_lt",
    S8Gte => "s8_gte",
    S8Lte => "s8_lte",
    S8Neg => "s8_neg",
    S8Add => "s8_add",
    S8Sub => "s8_sub",
    S8Mul => "s8_mul",
    S8Div => "s8_div",
    S8Abs => "s8_abs",
    S8UAbs => "s8_unsigned_abs",

    S16Eq  => "s16_eq",
    S16Neq => "s16_neq",
    S16Gt  => "s16_gt",
    S16Lt  => "s16_lt",
    S16Gte => "s16_gte",
    S16Lte => "s16_lte",
    S16Neg => "s16_neg",
    S16Add => "s16_add",
    S16Sub => "s16_sub",
    S16Mul => "s16_mul",
    S16Div => "s16_div",
    S16Abs => "s16_abs",
    S16UAbs => "s16_unsigned_abs",

    S32Eq  => "s32_eq",
    S32Neq => "s32_neq",
    S32Gt  => "s32_gt",
    S32Lt  => "s32_lt",
    S32Gte => "s32_gte",
    S32Lte => "s32_lte",
    S32Neg => "s32_neg",
    S32Add => "s32_add",
    S32Sub => "s32_sub",
    S32Mul => "s32_mul",
    S32Div => "s32_div",
    S32Abs => "s32_abs",
    S32UAbs => "s32_unsigned_abs",

    S64Eq  => "s64_eq",
    S64Neq => "s64_neq",
    S64Gt  => "s64_gt",
    S64Lt  => "s64_lt",
    S64Gte => "s64_gte",
    S64Lte => "s64_lte",
    S64Neg => "s64_neg",
    S64Add => "s64_add",
    S64Sub => "s64_sub",
    S64Mul => "s64_mul",
    S64Div => "s64_div",
    S64Abs => "s64_abs",
    S64UAbs => "s64_unsigned_abs",

    OptionSome => "some",
    OptionNone => "none",
    OptionFold => "option_fold",

    Array8Find => "array8_find",
    Array16Find => "array16_find",
    Array32Find => "array32_find",
    Array64Find => "array64_find",

    PosAddU8  => "pos_add_u8",
    PosAddU16 => "pos_add_u16",
    PosAddU32 => "pos_add_u32",
    PosAddU64 => "pos_add_u64",
}

/// Formatting style for integers
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum UIntStyle {
    Binary,
    Decimal,
    Hexadecimal,
    /// A [four-character code](https://en.wikipedia.org/wiki/FourCC) (big-endian)
    Ascii,
}

/// Constants
#[derive(Debug, Copy, Clone, PartialOrd)]
pub enum Const {
    Bool(bool),
    U8(u8, UIntStyle),
    U16(u16, UIntStyle),
    U32(u32, UIntStyle),
    U64(u64, UIntStyle),
    S8(i8),
    S16(i16),
    S32(i32),
    S64(i64),
    // TODO: use logical equality for floating point numbers
    F32(f32),
    F64(f64),
    Pos(u64),
    Ref(u64),
}

impl PartialEq for Const {
    fn eq(&self, other: &Self) -> bool {
        match (*self, *other) {
            (Const::Bool(a), Const::Bool(b)) => a == b,
            (Const::U8(a, _), Const::U8(b, _)) => a == b,
            (Const::U16(a, _), Const::U16(b, _)) => a == b,
            (Const::U32(a, _), Const::U32(b, _)) => a == b,
            (Const::U64(a, _), Const::U64(b, _)) => a == b,
            (Const::S8(a), Const::S8(b)) => a == b,
            (Const::S16(a), Const::S16(b)) => a == b,
            (Const::S32(a), Const::S32(b)) => a == b,
            (Const::S64(a), Const::S64(b)) => a == b,
            (Const::F32(a), Const::F32(b)) => a == b,
            (Const::F64(a), Const::F64(b)) => a == b,
            (Const::Pos(a), Const::Pos(b)) => a == b,
            (Const::Ref(a), Const::Ref(b)) => a == b,
            _ => false,
        }
    }
}

pub trait ToBeBytes<const N: usize> {
    fn to_be_bytes(self) -> [u8; N];
}

macro_rules! impl_styled_uint {
    ($($ty:ty),*) => {
        $(
        impl ToBeBytes<{std::mem::size_of::<$ty>()}> for &$ty {
            fn to_be_bytes(self) -> [u8; std::mem::size_of::<$ty>()] {
                <$ty>::to_be_bytes(*self)
            }
        }

        impl UIntStyled<{std::mem::size_of::<$ty>()}> for &$ty {}
        )*
    };
}

impl_styled_uint!(u8, u16, u32, u64);

pub trait UIntStyled<const N: usize>:
    std::fmt::Display + Copy + std::fmt::LowerHex + std::fmt::Binary + ToBeBytes<N>
{
}

impl UIntStyle {
    pub fn format<T: UIntStyled<N>, const N: usize>(&self, number: T) -> String {
        match self {
            UIntStyle::Binary => format!("0b{:b}", number),
            UIntStyle::Decimal => number.to_string(),
            UIntStyle::Hexadecimal => format!("0x{:x}", number),
            UIntStyle::Ascii => {
                let bytes = number.to_be_bytes();
                if bytes.iter().all(|c| c.is_ascii() && !c.is_ascii_control()) {
                    let s = std::str::from_utf8(&bytes).unwrap(); // unwrap safe due to above check
                    format!("\"{}\"", s)
                } else {
                    format!("0x{:x}", number)
                }
            }
        }
    }

    pub fn merge(left: UIntStyle, right: UIntStyle) -> UIntStyle {
        use UIntStyle::*;

        match (left, right) {
            // If one is the default style, then return the other
            (Decimal, style) | (style, Decimal) => style,
            // When both styles are the same. Note: (Decimal, Decimal) is handled above
            (Binary, Binary) => Binary,
            (Hexadecimal, Hexadecimal) => Hexadecimal,
            (Ascii, Ascii) => Ascii,
            // Otherwise use the default style
            (_, _) => Decimal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_drop() {
        assert!(!std::mem::needs_drop::<Term<'_>>());
        assert!(!std::mem::needs_drop::<Term<'_>>());
    }
}
