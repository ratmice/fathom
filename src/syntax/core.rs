//! The core syntax of the language

use codespan::{ByteIndex, ByteSpan};
use nameless::{
    self, Bind, BoundName, BoundPattern, BoundTerm, Embed, Ignore, Name, ScopeState, Var,
};
use std::collections::HashSet;
use std::fmt;
use std::rc::Rc;

use syntax::pretty::{self, ToDoc};

/// A universe level
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, BoundTerm)]
pub struct Level(pub u32);

impl Level {
    pub fn succ(self) -> Level {
        Level(self.0 + 1)
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Raw primitive constants
///
/// These are either the literal values or the types that describe them.
///
/// We could church encode all the things, but that would be prohibitively
/// expensive computationally!
#[derive(Debug, Clone, PartialEq, PartialOrd, BoundTerm)]
pub enum RawConstant {
    String(String),
    Char(char),
    Int(u64),
    Float(f64),
}

impl fmt::Display for RawConstant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_doc().group().render_fmt(pretty::FALLBACK_WIDTH, f)
    }
}

/// Primitive constants
///
/// These are either the literal values or the types that describe them.
///
/// We could church encode all the things, but that would be prohibitively
/// expensive computationally!
#[derive(Debug, Clone, PartialEq, PartialOrd, BoundTerm)]
pub enum Constant {
    Bool(bool),
    String(String),
    Char(char),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    BoolType,
    StringType,
    CharType,
    U8Type,
    U16Type,
    U32Type,
    U64Type,
    I8Type,
    I16Type,
    I32Type,
    I64Type,
    F32Type,
    F64Type,
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_doc().group().render_fmt(pretty::FALLBACK_WIDTH, f)
    }
}

/// A module definition
pub struct RawModule {
    /// The name of the module
    pub name: String,
    /// The definitions contained in the module
    pub definitions: Vec<RawDefinition>,
}

impl fmt::Display for RawModule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_doc().group().render_fmt(pretty::FALLBACK_WIDTH, f)
    }
}

/// Top level definitions
pub struct RawDefinition {
    /// The name of the declaration
    pub name: String,
    /// The body of the definition
    pub term: Rc<RawTerm>,
    /// An optional type annotation to aid in type inference
    pub ann: Rc<RawTerm>,
}

impl fmt::Display for RawDefinition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_doc().group().render_fmt(pretty::FALLBACK_WIDTH, f)
    }
}

/// A record label
///
/// Labels are significant when comparing for alpha-equality, both in terms and
/// in patterns
#[derive(Debug, Clone, PartialEq)]
pub struct Label(pub Name);

impl BoundTerm for Label {
    fn term_eq(&self, other: &Label) -> bool {
        match (self.0.name(), other.0.name()) {
            (Some(lhs), Some(rhs)) => lhs == rhs,
            (_, _) => Name::term_eq(&self.0, &other.0),
        }
    }
}

impl BoundPattern for Label {
    fn pattern_eq(&self, other: &Label) -> bool {
        Label::term_eq(self, other)
    }

    fn freshen(&mut self) -> Vec<Name> {
        self.0.freshen()
    }

    fn rename(&mut self, perm: &[Name]) {
        self.0.rename(perm)
    }

    fn on_free(&self, state: ScopeState, name: &Name) -> Option<BoundName> {
        self.0.on_free(state, name)
    }

    fn on_bound(&self, state: ScopeState, name: BoundName) -> Option<Name> {
        self.0.on_bound(state, name)
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Raw terms, unchecked and with implicit syntax that needs to be elaborated
///
/// For now the only implicit syntax we have is holes and lambdas that lack a
/// type annotation.
#[derive(Debug, Clone, PartialEq, BoundTerm)]
pub enum RawTerm {
    /// A term annotated with a type
    Ann(Ignore<ByteSpan>, Rc<RawTerm>, Rc<RawTerm>),
    /// Universes
    Universe(Ignore<ByteSpan>, Level),
    /// Constants
    Constant(Ignore<ByteSpan>, RawConstant),
    /// A hole
    Hole(Ignore<ByteSpan>),
    /// A variable
    Var(Ignore<ByteSpan>, Var),
    /// Dependent function types
    Pi(
        Ignore<ByteSpan>,
        Bind<(Name, Embed<Rc<RawTerm>>), Rc<RawTerm>>,
    ),
    /// Lambda abstractions
    Lam(
        Ignore<ByteSpan>,
        Bind<(Name, Embed<Rc<RawTerm>>), Rc<RawTerm>>,
    ),
    /// Term application
    App(Rc<RawTerm>, Rc<RawTerm>),
    /// If expression
    If(Ignore<ByteIndex>, Rc<RawTerm>, Rc<RawTerm>, Rc<RawTerm>),
    /// Dependent record types
    RecordType(
        Ignore<ByteSpan>,
        Bind<(Label, Embed<Rc<RawTerm>>), Rc<RawTerm>>,
    ),
    /// Dependent record
    Record(
        Ignore<ByteSpan>,
        Bind<(Label, Embed<Rc<RawTerm>>), Rc<RawTerm>>,
    ),
    /// The unit type
    EmptyRecordType(Ignore<ByteSpan>),
    /// The element of the unit type
    EmptyRecord(Ignore<ByteSpan>),
    /// Field projection
    Proj(Ignore<ByteSpan>, Rc<RawTerm>, Ignore<ByteSpan>, Label),
}

impl RawTerm {
    pub fn span(&self) -> ByteSpan {
        match *self {
            RawTerm::Ann(span, _, _)
            | RawTerm::Universe(span, _)
            | RawTerm::Hole(span)
            | RawTerm::Constant(span, _)
            | RawTerm::Var(span, _)
            | RawTerm::Pi(span, _)
            | RawTerm::Lam(span, _)
            | RawTerm::RecordType(span, _)
            | RawTerm::Record(span, _)
            | RawTerm::EmptyRecordType(span)
            | RawTerm::EmptyRecord(span)
            | RawTerm::Proj(span, _, _, _) => span.0,
            RawTerm::App(ref fn_term, ref arg) => fn_term.span().to(arg.span()),
            RawTerm::If(start, _, _, ref if_false) => ByteSpan::new(start.0, if_false.span().end()),
        }
    }
}

impl fmt::Display for RawTerm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_doc().group().render_fmt(pretty::FALLBACK_WIDTH, f)
    }
}

impl RawTerm {
    // TODO: Move to nameless crate
    fn visit_vars<F: FnMut(&Var)>(&self, on_var: &mut F) {
        match *self {
            RawTerm::Ann(_, ref expr, ref ty) => {
                expr.visit_vars(on_var);
                ty.visit_vars(on_var);
            },
            RawTerm::Universe(_, _) | RawTerm::Hole(_) | RawTerm::Constant(_, _) => {},
            RawTerm::Var(_, ref var) => on_var(var),
            RawTerm::Pi(_, ref scope) => {
                (scope.unsafe_pattern.1).0.visit_vars(on_var);
                scope.unsafe_body.visit_vars(on_var);
            },
            RawTerm::Lam(_, ref scope) => {
                (scope.unsafe_pattern.1).0.visit_vars(on_var);
                scope.unsafe_body.visit_vars(on_var);
            },
            RawTerm::App(ref fn_expr, ref arg_expr) => {
                fn_expr.visit_vars(on_var);
                arg_expr.visit_vars(on_var);
            },
            RawTerm::If(_, ref cond, ref if_true, ref if_false) => {
                cond.visit_vars(on_var);
                if_true.visit_vars(on_var);
                if_false.visit_vars(on_var);
            },
            RawTerm::RecordType(_, ref scope) => {
                (scope.unsafe_pattern.1).0.visit_vars(on_var);
                scope.unsafe_body.visit_vars(on_var);
            },
            RawTerm::Record(_, ref scope) => {
                (scope.unsafe_pattern.1).0.visit_vars(on_var);
                scope.unsafe_body.visit_vars(on_var);
            },
            RawTerm::EmptyRecordType(_) | RawTerm::EmptyRecord(_) => {},
            RawTerm::Proj(_, ref expr, _, _) => {
                expr.visit_vars(on_var);
            },
        };
    }

    // TODO: move to nameless crate
    pub fn free_vars(&self) -> HashSet<Name> {
        let mut free_vars = HashSet::new();
        self.visit_vars(&mut |var| match *var {
            Var::Bound(_, _) => {},
            Var::Free(ref name) => {
                free_vars.insert(name.clone());
            },
        });
        free_vars
    }
}

/// A typechecked and elaborated module
pub struct Module {
    /// The name of the module
    pub name: String,
    /// The definitions contained in the module
    pub definitions: Vec<Definition>,
}

/// A typechecked and elaborated definition
pub struct Definition {
    /// The name of the definition
    pub name: String,
    /// The elaborated value
    pub term: Rc<Term>,
    /// The type of the definition
    pub ann: Rc<Type>,
}

/// The core term syntax
#[derive(Debug, Clone, PartialEq, BoundTerm)]
pub enum Term {
    /// A term annotated with a type
    Ann(Ignore<ByteSpan>, Rc<Term>, Rc<Term>),
    /// Universes
    Universe(Ignore<ByteSpan>, Level),
    /// Constants
    Constant(Ignore<ByteSpan>, Constant),
    /// A variable
    Var(Ignore<ByteSpan>, Var),
    /// Dependent function types
    Pi(Ignore<ByteSpan>, Bind<(Name, Embed<Rc<Term>>), Rc<Term>>),
    /// Lambda abstractions
    Lam(Ignore<ByteSpan>, Bind<(Name, Embed<Rc<Term>>), Rc<Term>>),
    /// Term application
    App(Rc<Term>, Rc<Term>),
    /// If expression
    If(Ignore<ByteIndex>, Rc<Term>, Rc<Term>, Rc<Term>),
    /// Dependent record types
    RecordType(Ignore<ByteSpan>, Bind<(Label, Embed<Rc<Term>>), Rc<Term>>),
    /// Dependent record
    Record(Ignore<ByteSpan>, Bind<(Label, Embed<Rc<Term>>), Rc<Term>>),
    /// The unit type
    EmptyRecordType(Ignore<ByteSpan>),
    /// The element of the unit type
    EmptyRecord(Ignore<ByteSpan>),
    /// Field projection
    Proj(Ignore<ByteSpan>, Rc<Term>, Ignore<ByteSpan>, Label),
}

impl Term {
    pub fn span(&self) -> ByteSpan {
        match *self {
            Term::Ann(span, _, _)
            | Term::Universe(span, _)
            | Term::Constant(span, _)
            | Term::Var(span, _)
            | Term::Lam(span, _)
            | Term::Pi(span, _)
            | Term::RecordType(span, _)
            | Term::Record(span, _)
            | Term::EmptyRecordType(span)
            | Term::EmptyRecord(span)
            | Term::Proj(span, _, _, _) => span.0,
            Term::App(ref fn_term, ref arg) => fn_term.span().to(arg.span()),
            Term::If(start, _, _, ref if_false) => ByteSpan::new(start.0, if_false.span().end()),
        }
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_doc().group().render_fmt(pretty::FALLBACK_WIDTH, f)
    }
}

impl Term {
    // TODO: Move to nameless crate
    fn visit_vars<F: FnMut(&Var)>(&self, on_var: &mut F) {
        match *self {
            Term::Ann(_, ref expr, ref ty) => {
                expr.visit_vars(on_var);
                ty.visit_vars(on_var);
            },
            Term::Universe(_, _) | Term::Constant(_, _) => {},
            Term::Var(_, ref var) => on_var(var),
            Term::Pi(_, ref scope) => {
                (scope.unsafe_pattern.1).0.visit_vars(on_var);
                scope.unsafe_body.visit_vars(on_var);
            },
            Term::Lam(_, ref scope) => {
                (scope.unsafe_pattern.1).0.visit_vars(on_var);
                scope.unsafe_body.visit_vars(on_var);
            },
            Term::App(ref fn_expr, ref arg_expr) => {
                fn_expr.visit_vars(on_var);
                arg_expr.visit_vars(on_var);
            },
            Term::If(_, ref cond, ref if_true, ref if_false) => {
                cond.visit_vars(on_var);
                if_true.visit_vars(on_var);
                if_false.visit_vars(on_var);
            },
            Term::RecordType(_, ref scope) => {
                (scope.unsafe_pattern.1).0.visit_vars(on_var);
                scope.unsafe_body.visit_vars(on_var);
            },
            Term::Record(_, ref scope) => {
                (scope.unsafe_pattern.1).0.visit_vars(on_var);
                scope.unsafe_body.visit_vars(on_var);
            },
            Term::EmptyRecordType(_) | Term::EmptyRecord(_) => {},
            Term::Proj(_, ref expr, _, _) => {
                expr.visit_vars(on_var);
            },
        };
    }

    // TODO: move to nameless crate
    pub fn free_vars(&self) -> HashSet<Name> {
        let mut free_vars = HashSet::new();
        self.visit_vars(&mut |var| match *var {
            Var::Bound(_, _) => {},
            Var::Free(ref name) => {
                free_vars.insert(name.clone());
            },
        });
        free_vars
    }
}

/// Values
///
/// These are either in _normal form_ (they cannot be reduced further) or are
/// _neutral terms_ (there is a possibility of reducing further depending
/// on the bindings given in the context)
#[derive(Debug, Clone, PartialEq, BoundTerm)]
pub enum Value {
    /// Universes
    Universe(Level),
    /// Constants
    Constant(Constant),
    /// A pi type
    Pi(Bind<(Name, Embed<Rc<Value>>), Rc<Value>>),
    /// A lambda abstraction
    Lam(Bind<(Name, Embed<Rc<Value>>), Rc<Value>>),
    /// Dependent record types
    RecordType(Bind<(Label, Embed<Rc<Value>>), Rc<Value>>),
    /// Dependent record
    Record(Bind<(Label, Embed<Rc<Value>>), Rc<Value>>),
    /// The unit type
    EmptyRecordType,
    /// The element of the unit type
    EmptyRecord,
    /// Neutral terms
    Neutral(Rc<Neutral>),
}

impl Value {
    pub fn record_ty(&self) -> Option<Bind<(Label, Embed<Rc<Value>>), Rc<Value>>> {
        match *self {
            Value::RecordType(ref scope) => Some(scope.clone()),
            _ => None,
        }
    }

    pub fn lookup_record_ty(&self, label: &Label) -> Option<Rc<Value>> {
        let mut current_scope = self.record_ty();

        while let Some(scope) = current_scope {
            let ((current_label, Embed(value)), body) = nameless::unbind(scope);
            if Label::pattern_eq(&current_label, label) {
                return Some(value);
            }
            current_scope = body.record_ty();
        }

        None
    }

    pub fn record(&self) -> Option<Bind<(Label, Embed<Rc<Value>>), Rc<Value>>> {
        match *self {
            Value::Record(ref scope) => Some(scope.clone()),
            _ => None,
        }
    }

    pub fn lookup_record(&self, label: &Label) -> Option<Rc<Value>> {
        let mut current_scope = self.record();

        while let Some(scope) = current_scope {
            let ((current_label, Embed(value)), body) = nameless::unbind(scope);
            if Label::pattern_eq(&current_label, label) {
                return Some(value);
            }
            current_scope = body.record();
        }

        None
    }

    /// Returns `true` if the value is in weak head normal form
    pub fn is_whnf(&self) -> bool {
        match *self {
            Value::Universe(_)
            | Value::Constant(_)
            | Value::Pi(_)
            | Value::Lam(_)
            | Value::RecordType(_)
            | Value::Record(_)
            | Value::EmptyRecordType
            | Value::EmptyRecord => true,
            Value::Neutral(_) => false,
        }
    }

    /// Returns `true` if the value is in normal form (ie. it contains no neutral terms within it)
    pub fn is_nf(&self) -> bool {
        match *self {
            Value::Universe(_)
            | Value::Constant(_)
            | Value::EmptyRecordType
            | Value::EmptyRecord => true,
            Value::Pi(ref scope) | Value::Lam(ref scope) => {
                (scope.unsafe_pattern.1).0.is_nf() && scope.unsafe_body.is_nf()
            },
            Value::RecordType(ref scope) | Value::Record(ref scope) => {
                (scope.unsafe_pattern.1).0.is_nf() && scope.unsafe_body.is_nf()
            },
            Value::Neutral(_) => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_doc().group().render_fmt(pretty::FALLBACK_WIDTH, f)
    }
}

/// The head of an application
#[derive(Debug, Clone, PartialEq, BoundTerm)]
pub enum Head {
    /// Variables that have not yet been replaced with a definition
    Var(Var),
    // TODO: Metavariables
}

/// The spine of a neutral term
///
/// These are arguments that are awaiting application
pub type Spine = Vec<Rc<Value>>;

/// Neutral terms
///
/// These might be able to be reduced further depending on the bindings in the
/// context
#[derive(Debug, Clone, PartialEq, BoundTerm)]
pub enum Neutral {
    /// Term application
    App(Head, Spine),
    /// If expression
    If(Rc<Neutral>, Rc<Value>, Rc<Value>, Spine),
    /// Field projection
    Proj(Rc<Neutral>, Label, Spine),
}

impl fmt::Display for Neutral {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_doc().group().render_fmt(pretty::FALLBACK_WIDTH, f)
    }
}

/// Types are at the term level, so this is just an alias
pub type Type = Value;

impl From<Var> for Neutral {
    fn from(src: Var) -> Neutral {
        Neutral::App(Head::Var(src), vec![])
    }
}

impl From<Var> for Value {
    fn from(src: Var) -> Value {
        Value::from(Neutral::from(src))
    }
}

impl From<Neutral> for Value {
    fn from(src: Neutral) -> Value {
        Value::Neutral(Rc::new(src))
    }
}

impl<'a> From<&'a Value> for Term {
    fn from(src: &'a Value) -> Term {
        match *src {
            Value::Universe(level) => Term::Universe(Ignore::default(), level),
            Value::Constant(ref c) => Term::Constant(Ignore::default(), c.clone()),
            Value::Pi(ref scope) => {
                let ((name, Embed(param_ann)), body) = nameless::unbind(scope.clone());
                let param = (name, Embed(Rc::new(Term::from(&*param_ann))));

                Term::Pi(
                    Ignore::default(),
                    nameless::bind(param, Rc::new(Term::from(&*body))),
                )
            },
            Value::Lam(ref scope) => {
                let ((name, Embed(param_ann)), body) = nameless::unbind(scope.clone());
                let param = (name, Embed(Rc::new(Term::from(&*param_ann))));

                Term::Lam(
                    Ignore::default(),
                    nameless::bind(param, Rc::new(Term::from(&*body))),
                )
            },
            Value::RecordType(ref scope) => {
                let ((name, Embed(param_ann)), body) = nameless::unbind(scope.clone());
                let param = (name, Embed(Rc::new(Term::from(&*param_ann))));

                Term::RecordType(
                    Ignore::default(),
                    nameless::bind(param, Rc::new(Term::from(&*body))),
                )
            },
            Value::Record(ref scope) => {
                let ((name, Embed(param_value)), body) = nameless::unbind(scope.clone());
                let param = (name, Embed(Rc::new(Term::from(&*param_value))));

                Term::Record(
                    Ignore::default(),
                    nameless::bind(param, Rc::new(Term::from(&*body))),
                )
            },
            Value::EmptyRecordType => Term::EmptyRecordType(Ignore::default()).into(),
            Value::EmptyRecord => Term::EmptyRecord(Ignore::default()).into(),
            Value::Neutral(ref n) => Term::from(&**n),
        }
    }
}

impl<'a> From<&'a Neutral> for Term {
    fn from(src: &'a Neutral) -> Term {
        let (head, spine) = match *src {
            Neutral::App(ref head, ref spine) => (Term::from(head), spine),
            Neutral::If(ref cond, ref if_true, ref if_false, ref spine) => {
                let head = Term::If(
                    Ignore::default(),
                    Rc::new(Term::from(&**cond)),
                    Rc::new(Term::from(&**if_true)),
                    Rc::new(Term::from(&**if_false)),
                );
                (head, spine)
            },
            Neutral::Proj(ref expr, ref name, ref spine) => {
                let head = Term::Proj(
                    Ignore::default(),
                    Rc::new(Term::from(&**expr)),
                    Ignore::default(),
                    name.clone(),
                );
                (head, spine)
            },
        };

        spine.iter().fold(head, |acc, arg| {
            Term::App(Rc::new(acc), Rc::new(Term::from(&**arg)))
        })
    }
}

impl<'a> From<&'a Head> for Term {
    fn from(src: &'a Head) -> Term {
        match *src {
            Head::Var(ref var) => Term::Var(Ignore::default(), var.clone()),
        }
    }
}