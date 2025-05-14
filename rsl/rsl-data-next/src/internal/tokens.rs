use std::{hash::Hash};

use super::{InternedString};






#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub enum Keyword {
    Fn,
    Pub,
    Package,
    Return,
    Use,
    Impl,
    For,
    Loop,
    In,
    Mut,
    Struct,
    Super,
    Const,
    SelfValue,
    SelfType,
    Mod,
    Let,
    Break,
    Continue,
    Uni,
    SUni,
    Nuni,
    Storage,
    Uniform,
    Workgroup,
    Function,
    Private,
    PhysicalStorage,
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
    Ident(InternedString),
    Lifetime(InternedString),
    Uniformity(InternedString),
    Special(Special),
    Int(u128),
    Float(f64),
    String(String),
    Char(char),
    DocComment(String),
    End,
    Start,
}



