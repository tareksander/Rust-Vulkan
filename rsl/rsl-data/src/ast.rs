use std::{fmt::Display, hash::Hash};

use expr::Expression;
use token::Token;
use ty::Type;

use crate::{GetSpan, Ident, SourceSpan, Uniformity};

pub mod module;
pub mod function;
pub mod structure;
pub mod implementation;
pub mod token;
pub mod tokenizer;
pub mod parser;
pub mod expr;
pub mod statement;
pub mod ty;
pub mod traits;



#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenericArgDefinition {
    Expr(Ident, SourceSpan),
    Type(Ident, SourceSpan),
    Uni(Ident, SourceSpan),
    Life(Ident, SourceSpan),
}

#[derive(Debug, Clone)]
pub enum GenericArg {
    Expr(Expression),
    Type(Type),
    Lifetime(Ident, SourceSpan),
    Uniformity(Uniformity, SourceSpan),
}

impl Eq for GenericArg {}

impl PartialEq for GenericArg {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Expr(l0), Self::Expr(r0)) => l0 == r0,
            (Self::Type(l0), Self::Type(r0)) => l0 == r0,
            (Self::Lifetime(l0, l1), Self::Lifetime(r0, r1)) => l0 == r0,
            (Self::Uniformity(l0, l1), Self::Uniformity(r0, r1)) => l0 == r0,
            _ => false,
        }
    }
}

impl Hash for GenericArg {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl Display for GenericArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenericArg::Expr(expression) => expression.fmt(f),
            GenericArg::Type(t) => todo!(),
            GenericArg::Lifetime(ident, source_span) => f.write_str(&ident.str),
            GenericArg::Uniformity(uniformity, source_span) => Display::fmt(uniformity, f),
        }
    }
}


impl GetSpan for &GenericArg {
    fn span(self) -> SourceSpan {
        match self {
            GenericArg::Expr(expression) => expression.span(),
            GenericArg::Type(t) => t.span(),
            GenericArg::Lifetime(ident, span) => span.clone(),
            GenericArg::Uniformity(uniformity, span) => span.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Safety {
    Safe,
    Unsafe,
}

#[derive(Debug, Clone)]
pub struct ItemPath {
    /// A vector of identifiers, their SourceSpans and the vector of generic arguments, if any
    pub segments: Vec<(Ident, SourceSpan, Vec<GenericArg>)>,
    pub global: bool,
}

impl Eq for ItemPath {}

impl PartialEq for ItemPath {
    fn eq(&self, other: &Self) -> bool {
        self.segments.iter().map(|s| (&s.0, &s.2)).eq(other.segments.iter().map(|s| (&s.0, &s.2))) && self.global == other.global
    }
}

impl Hash for ItemPath {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.segments.iter().map(|i| (&i.0, &i.2)).for_each(|i| i.hash(state));
        self.global.hash(state);
    }
}

impl ItemPath {
    pub fn globalize(&self, mut prefix: ItemPath) -> ItemPath {
        let span = self.span();
        for seg in &mut prefix.segments {
            seg.1 = span.clone();
        }
        ItemPath {
            segments: prefix.segments.iter().cloned().chain(self.segments.iter().cloned()).collect(),
            global: true,
        }
    }
}

impl Display for ItemPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.global {
            f.write_str("::")?;
        }
        self.segments.iter().map(|e| {
            f.write_str(&e.0.str)?;
            if e.2.len() != 0 {
                f.write_str("<")?;
                write!(f, "{:?}", e.2.iter().map(|e| e as &dyn Display))?;
                f.write_str(">")?;
            }
            Ok(())
        }).collect::<std::fmt::Result>()?;
        Ok(())
    }
}

impl GetSpan for &ItemPath {
    fn span(self) -> SourceSpan {
        self.segments.iter().map(|e| {
            if e.2.len() != 0 {
                e.1.expand(&(&e.2).into_iter().span())
            } else {
                e.1.clone()
            }
        }).span()
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum ValueOrConstant<T> {
    Value(T, SourceSpan),
    Constant(ItemPath),
}




#[derive(Debug, Clone, PartialEq)]
pub enum Attribute {
    Entrypoint(Entrypoint),
    Push(ValueOrConstant<u32>),
    Set(ValueOrConstant<u32>),
    Binding(ValueOrConstant<u32>),
    Spec(u32),
    Builtin(BuiltinVariable),
    
    Custom(String, Vec<Token>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BuiltinVariable {
    GlobalInvocationID,
    
}

#[derive(Debug, Clone, PartialEq)]
pub enum Entrypoint {
    Compute(ValueOrConstant<u32>, ValueOrConstant<u32>, ValueOrConstant<u32>),
}


