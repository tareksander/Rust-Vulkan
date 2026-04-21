use std::{hash::Hash, mem::discriminant};

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
    Static,
    Trait,
    Unsafe,
    Where,
    Type,
    If,
    Else,
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
    Dot,
    DoubleDot,
    Comma,
    Less,
    LessEquals,
    Greater,
    GreaterEquals,
    Bar,
    DoubleBar,
    And,
    DoubleAnd,
    ThinArrow,
    ThickArrow,
    Equals,
    ExclamationEquals,
    DoubleEquals,
    Hash,
}


#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Keyword(Keyword),
    Ident(InternedString),
    Lifetime(InternedString),
    Special(Special),
    Int(u128),
    Float(f64),
    String(InternedString),
    Char(char),
    DocComment(String),
    End,
    Start,
}

impl Eq for Token {}

impl Hash for Token {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Token::Keyword(keyword) => keyword.hash(state),
            Token::Ident(interned_string) => interned_string.hash(state),
            Token::Lifetime(interned_string) => interned_string.hash(state),
            Token::Special(special) => special.hash(state),
            Token::Int(i) => i.hash(state),
            Token::Float(_) => panic!("Float tokens shouldn't be put in HashMaps!"),
            Token::String(s) => s.hash(state),
            Token::Char(c) => c.hash(state),
            Token::End => discriminant(self).hash(state),
            Token::Start => discriminant(self).hash(state),
            Token::DocComment(s) => s.hash(state),
        }
    }
}



