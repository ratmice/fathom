//! The core type theory of the data description language.

use codespan::{ByteIndex, FileId, Span};
use codespan_reporting::diagnostic::Diagnostic;
use num_bigint::BigInt;
use pretty::{DocAllocator, DocBuilder};
use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use crate::lexer::SpannedToken;
use crate::{diagnostics, ieee754};

mod grammar {
    include!(concat!(env!("OUT_DIR"), "/core/grammar.rs"));
}

pub mod compile;
pub mod semantics;
pub mod validate;

/// A module of items.
#[derive(Debug, Clone)]
pub struct Module {
    /// The file in which this module was defined.
    pub file_id: FileId,
    /// Doc comment.
    pub doc: Arc<[String]>,
    /// The items in this module.
    pub items: Vec<Item>,
}

impl Module {
    pub fn parse(
        file_id: FileId,
        tokens: impl IntoIterator<Item = Result<SpannedToken, Diagnostic>>,
        report: &mut dyn FnMut(Diagnostic),
    ) -> Module {
        grammar::ModuleParser::new()
            .parse(file_id, report, tokens)
            .unwrap_or_else(|error| {
                report(diagnostics::error::parse(file_id, error));
                Module {
                    file_id,
                    doc: Arc::new([]),
                    items: Vec::new(),
                }
            })
    }

    pub fn doc<'core, D>(&'core self, alloc: &'core D) -> DocBuilder<'core, D>
    where
        D: DocAllocator<'core>,
        D::Doc: Clone,
    {
        let docs = match self.doc.as_ref() {
            [] => None,
            doc => Some(alloc.intersperse(
                doc.iter().map(|line| format!("//!{}", line)),
                alloc.hardline(),
            )),
        };
        let items = self.items.iter().map(|item| item.doc(alloc));

        (alloc.nil())
            .append(alloc.intersperse(
                docs.into_iter().chain(items),
                alloc.hardline().append(alloc.hardline()),
            ))
            .append(alloc.hardline())
    }
}

impl PartialEq for Module {
    fn eq(&self, other: &Module) -> bool {
        self.items == other.items
    }
}

/// Items in a module.
#[derive(Debug, Clone)]
pub enum Item {
    /// Alias definitions
    Alias(Alias),
    /// Struct definitions.
    Struct(StructType),
}

impl Item {
    pub fn span(&self) -> Span {
        match self {
            Item::Struct(struct_ty) => struct_ty.span,
            Item::Alias(alias) => alias.span,
        }
    }

    pub fn doc<'core, D>(&'core self, alloc: &'core D) -> DocBuilder<'core, D>
    where
        D: DocAllocator<'core>,
        D::Doc: Clone,
    {
        match self {
            Item::Alias(alias) => alias.doc(alloc),
            Item::Struct(struct_ty) => struct_ty.doc(alloc),
        }
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Item) -> bool {
        match (self, other) {
            (Item::Alias(alias0), Item::Alias(alias1)) => *alias0 == *alias1,
            (Item::Struct(struct_ty0), Item::Struct(struct_ty1)) => *struct_ty0 == *struct_ty1,
            (_, _) => false,
        }
    }
}

/// An alias definition.
#[derive(Debug, Clone)]
pub struct Alias {
    /// The full span of this definition.
    pub span: Span,
    /// Doc comment.
    pub doc: Arc<[String]>,
    /// Name of this definition.
    pub name: String,
    /// The term that is aliased.
    pub term: Term,
}

impl Alias {
    pub fn doc<'core, D>(&'core self, alloc: &'core D) -> DocBuilder<'core, D>
    where
        D: DocAllocator<'core>,
        D::Doc: Clone,
    {
        let docs = alloc.concat(self.doc.iter().map(|line| {
            (alloc.nil())
                .append(format!("///{}", line))
                .append(alloc.hardline())
        }));

        (alloc.nil())
            .append(docs)
            .append(alloc.as_string(&self.name))
            .append(alloc.space())
            .append("=")
            .group()
            .append(
                (alloc.nil())
                    .append(alloc.space())
                    .append(self.term.doc(alloc))
                    .group()
                    .append(";")
                    .nest(4),
            )
    }
}

impl PartialEq for Alias {
    fn eq(&self, other: &Alias) -> bool {
        self.name == other.name && self.term == other.term
    }
}

/// A struct type definition.
#[derive(Debug, Clone)]
pub struct StructType {
    /// The full span of this definition.
    pub span: Span,
    /// Doc comment.
    pub doc: Arc<[String]>,
    /// Name of this definition.
    pub name: String,
    /// Fields in the struct.
    pub fields: Vec<TypeField>,
}

impl StructType {
    pub fn doc<'core, D>(&'core self, alloc: &'core D) -> DocBuilder<'core, D>
    where
        D: DocAllocator<'core>,
        D::Doc: Clone,
    {
        let docs = alloc.concat(self.doc.iter().map(|line| {
            (alloc.nil())
                .append(format!("///{}", line))
                .append(alloc.hardline())
        }));

        let struct_prefix = (alloc.nil())
            .append("struct")
            .append(alloc.space())
            .append(alloc.as_string(&self.name))
            .append(alloc.space());

        let struct_ty = if self.fields.is_empty() {
            (alloc.nil()).append(struct_prefix).append("{}").group()
        } else {
            (alloc.nil())
                .append(struct_prefix)
                .append("{")
                .group()
                .append(alloc.concat(self.fields.iter().map(|field| {
                    (alloc.nil())
                        .append(alloc.hardline())
                        .append(field.doc(alloc))
                        .nest(4)
                        .group()
                })))
                .append(alloc.hardline())
                .append("}")
        };

        (alloc.nil()).append(docs).append(struct_ty)
    }
}

impl PartialEq for StructType {
    fn eq(&self, other: &StructType) -> bool {
        self.name == other.name && self.fields == other.fields
    }
}

/// A field in a struct type definition.
#[derive(Debug, Clone)]
pub struct TypeField {
    pub doc: Arc<[String]>,
    pub start: ByteIndex,
    pub name: String,
    pub term: Term,
}

impl TypeField {
    pub fn span(&self) -> Span {
        Span::new(self.start, self.term.span().end())
    }

    pub fn doc<'core, D>(&'core self, alloc: &'core D) -> DocBuilder<'core, D>
    where
        D: DocAllocator<'core>,
        D::Doc: Clone,
    {
        let docs = alloc.concat(self.doc.iter().map(|line| {
            (alloc.nil())
                .append(format!("///{}", line))
                .append(alloc.hardline())
        }));

        (alloc.nil())
            .append(docs)
            .append(
                (alloc.nil())
                    .append(alloc.as_string(&self.name))
                    .append(alloc.space())
                    .append(":")
                    .group(),
            )
            .append(
                (alloc.nil())
                    .append(alloc.space())
                    .append(self.term.doc(alloc))
                    .append(","),
            )
    }
}

impl PartialEq for TypeField {
    fn eq(&self, other: &TypeField) -> bool {
        self.name == other.name && self.term == other.term
    }
}

/// Universes.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Universe {
    Type,
    Format,
    Kind,
}

impl Universe {
    pub fn doc<'core, D>(&'core self, alloc: &'core D) -> DocBuilder<'core, D>
    where
        D: DocAllocator<'core>,
        D::Doc: Clone,
    {
        match self {
            Universe::Type => alloc.text("Type"),
            Universe::Format => alloc.text("Format"),
            Universe::Kind => alloc.text("Kind"),
        }
    }
}

/// Universes.
#[derive(Debug, Clone)]
pub enum Constant {
    /// Host integer constants.
    Int(BigInt),
    /// Host IEEE-754 single-precision floating point constants.
    F32(f32),
    /// Host IEEE-754 double-precision floating point constants.
    F64(f64),
}

impl Constant {
    pub fn doc<'core, D>(&'core self, alloc: &'core D) -> DocBuilder<'core, D>
    where
        D: DocAllocator<'core>,
        D::Doc: Clone,
    {
        use num_traits::Float;
        use std::borrow::Cow;

        // Workaround -0.0 ridiculousness
        fn format_float<T: Float + From<u8> + fmt::Display>(value: T) -> Cow<'static, str> {
            if value == <T as From<u8>>::from(0) && value.is_sign_negative() {
                "-0".into()
            } else {
                value.to_string().into()
            }
        }

        match self {
            Constant::Int(value) => (alloc.nil())
                .append("int")
                .append(alloc.space())
                .append(alloc.as_string(value)),
            Constant::F32(value) => (alloc.nil())
                .append("f32")
                .append(alloc.space())
                .append(format_float(*value)),
            Constant::F64(value) => (alloc.nil())
                .append("f64")
                .append(alloc.space())
                .append(format_float(*value)),
        }
    }
}

impl PartialEq for Constant {
    fn eq(&self, other: &Constant) -> bool {
        match (self, other) {
            (Constant::Int(val0), Constant::Int(val1)) => val0 == val1,
            (Constant::F32(val0), Constant::F32(val1)) => ieee754::logical_eq(*val0, *val1),
            (Constant::F64(val0), Constant::F64(val1)) => ieee754::logical_eq(*val0, *val1),
            (_, _) => false,
        }
    }
}

/// Terms.
#[derive(Debug, Clone)]
pub enum Term {
    /// Item references
    Item(Span, String),
    /// Terms annotated with types.
    Ann(Arc<Term>, Arc<Term>),
    /// Universes.
    Universe(Span, Universe),
    /// Constants.
    Constant(Span, Constant),
    /// A boolean elimination.
    BoolElim(Span, Arc<Term>, Arc<Term>, Arc<Term>),
    /// A integer elimination.
    IntElim(Span, Arc<Term>, BTreeMap<BigInt, Arc<Term>>, Arc<Term>),

    /// Error sentinel.
    Error(Span),
}

impl Term {
    pub fn span(&self) -> Span {
        match self {
            Term::Item(span, _)
            | Term::Universe(span, _)
            | Term::Constant(span, _)
            | Term::BoolElim(span, _, _, _)
            | Term::IntElim(span, _, _, _)
            | Term::Error(span) => *span,
            Term::Ann(term, ty) => Span::merge(term.span(), ty.span()),
        }
    }

    pub fn doc<'core, D>(&'core self, alloc: &'core D) -> DocBuilder<'core, D>
    where
        D: DocAllocator<'core>,
        D::Doc: Clone,
    {
        self.doc_prec(alloc, 0)
    }

    pub fn doc_prec<'core, D>(&'core self, alloc: &'core D, prec: u8) -> DocBuilder<'core, D>
    where
        D: DocAllocator<'core>,
        D::Doc: Clone,
    {
        let show_paren = |cond, doc| match cond {
            true => alloc.text("(").append(doc).append(")"),
            false => doc,
        };

        match self {
            Term::Item(_, name) => (alloc.nil())
                .append("item")
                .append(alloc.space())
                .append(alloc.as_string(name)),
            Term::Ann(term, ty) => show_paren(
                prec > 0,
                (alloc.nil())
                    .append(term.doc_prec(alloc, prec + 1))
                    .append(alloc.space())
                    .append(":")
                    .group()
                    .append(
                        (alloc.space())
                            .append(ty.doc_prec(alloc, prec + 1))
                            .group()
                            .nest(4),
                    ),
            ),
            Term::Universe(_, universe) => universe.doc(alloc),
            Term::Constant(_, constant) => constant.doc(alloc),
            Term::BoolElim(_, head, if_true, if_false) => (alloc.nil())
                .append("bool_elim")
                .append(alloc.space())
                .append(head.doc(alloc))
                .append(alloc.space())
                .append("{")
                .append(alloc.space())
                .append(if_true.doc(alloc))
                .append(",")
                .append(alloc.space())
                .append(if_false.doc(alloc))
                .append(alloc.space())
                .append("}"),
            Term::IntElim(_, head, branches, default) => (alloc.nil())
                .append("int_elim")
                .append(alloc.space())
                .append(head.doc(alloc))
                .append(alloc.space())
                .append("{")
                .append(alloc.concat(branches.iter().map(|(value, term)| {
                    (alloc.nil())
                        .append(alloc.space())
                        .append(alloc.as_string(value))
                        .append(alloc.space())
                        .append("=>")
                        .append(alloc.space())
                        .append(term.doc(alloc))
                        .append(",")
                })))
                .append(alloc.space())
                .append(default.doc(alloc))
                .append(alloc.space())
                .append("}"),
            Term::Error(_) => alloc.text("!"),
        }
    }
}

impl PartialEq for Term {
    fn eq(&self, other: &Term) -> bool {
        match (self, other) {
            (Term::Item(_, name0), Term::Item(_, name1)) => name0 == name1,
            (Term::Ann(term0, ty0), Term::Ann(term1, ty1)) => term0 == term1 && ty0 == ty1,
            (Term::Universe(_, universe0), Term::Universe(_, universe1)) => universe0 == universe1,
            (Term::Constant(_, constant0), Term::Constant(_, constant1)) => constant0 == constant1,
            (
                Term::BoolElim(_, head0, if_true0, if_false0),
                Term::BoolElim(_, head1, if_true1, if_false1),
            ) => head0 == head1 && if_true0 == if_true1 && if_false0 == if_false1,
            (
                Term::IntElim(_, head0, branches0, default0),
                Term::IntElim(_, head1, branches1, default1),
            ) => head0 == head1 && branches0 == branches1 && default0 == default1,
            (Term::Error(_), Term::Error(_)) => true,
            (_, _) => false,
        }
    }
}

/// The head of a neutral term.
#[derive(Debug, Clone, PartialEq)]
pub enum Head {
    /// Item references.
    Item(String),
    /// Errors.
    Error,
}

/// An eliminator that is 'stuck' on some head.
#[derive(Debug, Clone, PartialEq)]
pub enum Elim {
    // FIXME: environment?
    Bool(Arc<Term>, Arc<Term>),
    Int(BTreeMap<BigInt, Arc<Term>>, Arc<Term>),
}

/// Values.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Neutral terms
    Neutral(Head, Vec<Elim>),
    /// Universes.
    Universe(Universe),
    /// Constants.
    Constant(Constant),

    /// Error sentinel.
    Error,
}