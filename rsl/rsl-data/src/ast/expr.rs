use std::fmt::{Debug, Display};

use crate::{ast::statement::Block, GetSpan, Ident, ast::ItemPath, SourceSpan};

use super::ty::Type;

#[derive(Debug, Clone, PartialEq)]

pub enum Expression {
    Int(SourceSpan, u128),
    Float(SourceSpan, f64),
    Item(ItemPath),
    UnOp(SourceSpan, UnOp, Box<Expression>),
    BinOp(Box<Expression>, BinOp, Box<Expression>),
    If(Box<If>),
    Unit(SourceSpan),
    Tuple(SourceSpan, Vec<Expression>),
    Property(Box<Expression>, Ident, SourceSpan),
    Call(Box<Expression>, Vec<Expression>),
    Index(Box<Expression>, Box<Expression>),
    Cast(Box<Expression>, Type),
    Unsafe(Box<Block>),
}

impl Eq for Expression {}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct If {
    pub condition: Expression,
    pub then: Block,
    pub otherwise: Option<Block>,
    pub span: SourceSpan,
    
}



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
}

impl Display for UnOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnOp::Neg => f.write_str("-"),
            UnOp::Not => f.write_str("!"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Assign,
}


impl Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinOp::Add => f.write_str("+"),
            BinOp::Sub => f.write_str("-"),
            BinOp::Mul => f.write_str("*"),
            BinOp::Div => f.write_str("/"),
            BinOp::Assign => f.write_str("="),
        }
    }
}



impl GetSpan for &Expression {
    fn span(self) -> SourceSpan {
        match self {
            Expression::Int(source_span, _) => source_span.clone(),
            Expression::UnOp(span, _un_op, expression) => span.expand(&expression.span()),
            Expression::BinOp(expression, _bin_op, expression1) => expression.span().expand(&expression1.span()),
            Expression::If(i) => i.span.clone(),
            Expression::Float(source_span, _) => source_span.clone(),
            Expression::Unit(s) => s.clone(),
            Expression::Item(item_path) => item_path.span(),
            Expression::Tuple(source_span, _expressions) => source_span.clone(),
            Expression::Property(lhs, _id, s) => lhs.span().expand(&s),
            Expression::Call(lhs, args) => lhs.span().expand(&args.iter().span()),
            Expression::Index(lhs, i) => lhs.span().expand(&i.span()),
            Expression::Cast(expression, ty) => expression.span().expand(&ty.span()),
            Expression::Unsafe(block) => block.span.clone(),
                    }
    }
}


impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Int(_source_span, v) => f.write_str(&v.to_string()),
            Expression::Float(_source_span, v) => f.write_str(&v.to_string()),
            Expression::Item(item_path) => Display::fmt(item_path, f),
            Expression::UnOp(_source_span, un_op, expression) => {
                f.write_str("(")?;
                Display::fmt(un_op, f)?;
                f.write_str(" ")?;
                Display::fmt(expression, f)?;
                f.write_str(")")
            },
            Expression::BinOp(lhs, bin_op, rhs) => {
                f.write_str("(")?;
                Display::fmt(lhs, f)?;
                f.write_str(" ")?;
                Display::fmt(bin_op, f)?;
                f.write_str(" ")?;
                Display::fmt(rhs, f)?;
                f.write_str(")")
            },
            Expression::If(_) => todo!(),
            Expression::Unit(_source_span) => f.write_str("()"),
            Expression::Tuple(_source_span, expressions) => {
                f.write_str("(")?;
                for i in 0..(expressions.len()-1) {
                    Display::fmt(&expressions[i], f)?;
                    f.write_str(", ")?;
                }
                Display::fmt(expressions.last().unwrap(), f)?;
                f.write_str(")")
            },
            Expression::Call(lhs, expressions) => {
                Display::fmt(lhs, f)?;
                f.write_str("(")?;
                for i in 0..(expressions.len()-1) {
                    Display::fmt(&expressions[i], f)?;
                    f.write_str(", ")?;
                }
                Display::fmt(expressions.last().unwrap(), f)?;
                f.write_str(")")
            },
            Expression::Property(lhs, ident, _) => {
                Display::fmt(lhs, f)?;
                f.write_str(".")?;
                f.write_str(&ident.str)
            },
            Expression::Index(lhs, i) => {
                Display::fmt(lhs, f)?;
                f.write_str("[")?;
                Display::fmt(&i, f)?;
                f.write_str("]")
            }
            Expression::Cast(lhs, rhs) => {
                f.write_str("(")?;
                Display::fmt(lhs, f)?;
                f.write_str(" as ")?;
                //Display::fmt(rhs, f)?;
                f.write_str("todo )")
            },
            Expression::Unsafe(block) => {
                todo!()
            }
        }
    }
}


