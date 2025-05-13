use std::{hash::Hash, mem::discriminant};

use crate::{GetSpan, Ident, SourceSpan};





#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub enum Keyword {
    Fn,
    Pub,
    Package,
    Return,
    Use,
    Type,
    Impl,
    For,
    Loop,
    In,
    Mut,
    //Mix,
    //Mixin,
    Struct,
    Super,
    Const,
    SelfValue,
    SelfType,
    //Void,
    Mod,
    Let,
    Break,
    Continue,
    Uni,
    Dyn,
    SUni,
    Nuni,
    Storage,
    Uniform,
    Workgroup,
    Function,
    Private,
    Memory,
    PhysicalStorage,
    Push,
    Static,
    Trait,
    Unsafe,
    Where,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub enum Special {
    Semicolon,
    Colon,
    DoubleColon,
    SquareBracketOpen,
    SquareBracketClose,
    RoundBracketOpen,
    RoundBracketClose,
    CurlyBracketOpen,
    CurlyBracketClose,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    Exclamation,
    Tilde,
    Hash,
    Dot,
    DoubleDot,
    Comma,
    AngleBracketOpen,
    AngleBracketClose,
    Bar,
    DoubleBar,
    And,
    DoubleAnd,
    ThinArrow,
    ThickArrow,
    Equals,
    DoubleEquals,
}


#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Keyword(Keyword),
    Ident(Ident),
    Lifetime(Ident),
    Uniformity(Ident),
    Special(Special),
    Int(u128),
    Float(f64),
    String(String),
    Char(char),
    DocComment(String),
    End,
    Start,
}

impl Hash for TokenType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            TokenType::Keyword(keyword) => keyword.hash(state),
            TokenType::Ident(ident) => ident.hash(state),
            TokenType::Lifetime(ident) => ident.hash(state),
            TokenType::Uniformity(ident) => ident.hash(state),
            TokenType::Special(special) => special.hash(state),
            TokenType::Int(i) => i.hash(state),
            TokenType::Float(f) => f.to_bits().hash(state),
            TokenType::String(s) => s.hash(state),
            TokenType::Char(c) => c.hash(state),
            TokenType::DocComment(d) => d.hash(state),
            TokenType::End => discriminant(self).hash(state),
            TokenType::Start => discriminant(self).hash(state),
        }
    }
}


impl Eq for TokenType {}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub span: SourceSpan,
    pub ty: TokenType,
}





impl GetSpan for &Token {
    fn span(self) -> SourceSpan {
        self.span.clone()
    }
}






