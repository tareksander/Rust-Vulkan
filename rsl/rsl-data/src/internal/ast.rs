use std::{collections::HashMap, fmt::Display, ops::Range};

use crate::internal::{ShaderType, tokens::Token};

use super::{Attribute, InternedString, Mutability, StorageClass, Uniformity, Visibility};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokenRange {
    pub file: usize,
    pub range: Range<usize>
}

impl TokenRange {
    pub fn point(file: usize, index: usize) -> Self {
        Self {
            file,
            range: index..(index+1)
        }
    }
    
    pub fn merge(&self, other: &TokenRange) -> TokenRange {
        return TokenRange { file: self.file, range: self.range.start.min(other.range.start)..self.range.end.min(other.range.end) };
    }
    
    pub fn merge_iter<I, T: Iterator<Item = I>>(&self, iter: T) -> TokenRange where I: SourceRange {
        iter.fold(self.clone(), |r, i| r.merge(&i.range()))
    }
}


impl<T> SourceRange for &T where T: SourceRange {
    fn range(&self) -> TokenRange {
        (*self).range()
    }
}


pub trait SourceRange {
    fn range(&self) -> TokenRange;
}



#[derive(Debug, Clone)]
pub struct ItemPath {
    pub segments: Vec<ItemPathSegment>,
    pub global: bool,
}

impl SourceRange for ItemPath {
    fn range(&self) -> TokenRange {
        self.segments[1..].iter().fold(self.segments.first().unwrap().range(), |r, s| r.merge(&s.range()))
    }
}

#[derive(Debug, Clone)]
pub struct ItemPathSegment {
    pub ident: InternedString,
    pub ident_token: TokenRange,
    pub generic_args: Vec<GenericArg>
}

impl SourceRange for ItemPathSegment {
    fn range(&self) -> TokenRange {
        let mut r = self.ident_token.clone();
        r = r.merge_iter(self.generic_args.iter());
        return r;
    }
}

#[derive(Debug, Clone)]
pub enum GenericArg {
    /// Can be another generic or a concrete type.
    Type(Type),
    Expression(Expression),
    Uniformity(Uniformity, TokenRange),
    Lifetime(InternedString, TokenRange),
}

impl SourceRange for GenericArg {
    fn range(&self) -> TokenRange {
        todo!()
    }
}

#[derive(Debug)]
pub enum GenericArgDefinition {
    Type(InternedString, TokenRange),
    Expression(InternedString, TokenRange),
    Lifetime(InternedString, TokenRange),
}


#[derive(Debug, Clone)]
pub enum Type {
    Path(ItemPath),
    Pointer{
        star_token: TokenRange,
        mutability: Mutability,
        ty: Box<Type>
    },
    Reference{
        ampersand_token: TokenRange,
        mutability: Option<(Mutability, TokenRange)>,
        ty: Box<Type>
    },
    Array {
        ty: Box<Type>,
        size: Expression,
    },
    Unit,
    Inferred {
        ty: Option<Box<Type>>,
    }
}


#[derive(Debug, Clone)]
pub enum Expression {
    Unary{ e: Box<Expression>, op: UnOp, op_range: TokenRange},
    Binary{ lhs: Box<Expression>, op: BinOp, rhs: Box<Expression>},
    Property { e: Box<Expression>, name: InternedString, name_token: TokenRange },
    Item(ItemPath),
    Group(Box<Expression>),
    IntLiteral(u128, TokenRange),
    FloatLiteral(f64, TokenRange),
    Call(ItemPath, Vec<Expression>),
    If {
        condition: Box<Expression>,
        then: Box<Block>,
        other: Option<Box<Block>>,
    },
    Loop {
        block: Box<Block>,
    },
    Unsafe(Box<Block>),
}


impl SourceRange for Expression {
    fn range(&self) -> TokenRange {
        match self {
            Expression::Unary { e, op, op_range } => e.range().merge(op_range),
            Expression::Binary { lhs, op, rhs } => lhs.range().merge(&rhs.range()),
            Expression::Property { e, name, name_token } => e.range().merge(name_token),
            Expression::Item(item_path) => item_path.range(),
            Expression::Group(expression) => expression.range(),
            Expression::IntLiteral(_, token_range) => token_range.clone(),
            Expression::FloatLiteral(_, token_range) => token_range.clone(),
            Expression::Call(item_path, expressions) => item_path.range().merge_iter(expressions.iter()),
            Expression::If { condition, then, other } => {
                let mut r = condition.range().merge(&then.range());
                if let Some(o) = other {
                    r = r.merge(&o.range());
                }
                r
            },
            Expression::Loop { block } => block.range(),
            Expression::Unsafe(block) => block.range(),
        }
    }
}



impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Unary { e, op , op_range: _} => {
                f.write_str("(")?;
                Display::fmt(op, f)?;
                f.write_str(" ")?;
                Display::fmt(&*e, f)?;
                f.write_str(")")
            },
            Expression::Binary { lhs, op, rhs } => {
                f.write_str("(")?;
                Display::fmt(&*lhs, f)?;
                f.write_str(" ")?;
                Display::fmt(op, f)?;
                f.write_str(" ")?;
                Display::fmt(&*rhs, f)?;
                f.write_str(")")
            },
            Expression::Property { e, name, name_token } => {
                
                
                
                todo!()
            },
            Expression::Item(item_path) => {
                
                
                
                todo!()
            },
            Expression::Group(expression) => {
                Display::fmt(&*expression, f)
            },
            Expression::IntLiteral(v, token_range) => {
                Display::fmt(v, f)
            },
            Expression::FloatLiteral(v, token_range) => {
                Display::fmt(v, f)
            },
            Expression::Call(item_path, expressions) => {
                
                
                
                todo!()
            },
            Expression::If { condition, then, other } => {
                
                
                
                todo!()
            },
            Expression::Loop { block } => {
                
                
                
                todo!()
            },
            Expression::Unsafe(block) => {
                
                
                
                todo!()
            },
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Negate,
    BinNot,
    LogNot,
    Deref,
    Ref,
    RefMut,
}

impl Display for UnOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnOp::Negate => f.write_str("-"),
            UnOp::BinNot => f.write_str("~"),
            UnOp::LogNot => f.write_str("!"),
            UnOp::Deref => f.write_str("*"),
            UnOp::Ref => f.write_str("&"),
            UnOp::RefMut => f.write_str("&mut"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    BinAnd,
    LogAnd,
    BinOr,
    LogOr,
    BinXor,
    Index,
    Assign,
    Equals,
    NotEquals,
    Less,
    LessEquals,
    Greater,
    GreaterEquals,
}

impl Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinOp::Add => f.write_str("+"),
            BinOp::Sub => f.write_str("-"),
            BinOp::Mul => f.write_str("*"),
            BinOp::Div => f.write_str("/"),
            BinOp::Mod => f.write_str("%"),
            BinOp::BinAnd => f.write_str("&"),
            BinOp::LogAnd => f.write_str("&&"),
            BinOp::BinOr => f.write_str("|"),
            BinOp::LogOr => f.write_str("||"),
            BinOp::BinXor => f.write_str("^"),
            BinOp::Index => f.write_str("[]"),
            BinOp::Assign => f.write_str("="),
            BinOp::Equals => f.write_str("=="),
            BinOp::NotEquals => f.write_str("!="),
            BinOp::Less => f.write_str("<"),
            BinOp::LessEquals => f.write_str("<="),
            BinOp::Greater => f.write_str(">"),
            BinOp::GreaterEquals => f.write_str(">="),
        }
    }
}

#[derive(Debug)]
pub struct StructDefinition {
    pub attrs: Vec<Attribute>,
    pub visibility: Option<(Visibility, TokenRange)>,
    pub struct_token: TokenRange,
    pub ident: InternedString,
    pub ident_token: TokenRange,
    pub generics: Vec<GenericArgDefinition>,
    pub generics_constraints: Vec<GenericsConstraint>,
    pub fields: Vec<StructField>
}

#[derive(Debug)]
pub struct StructField {
    pub attrs: Vec<Attribute>,
    pub visibility: Option<(Visibility, TokenRange)>,
    pub ident: InternedString,
    pub ident_token: TokenRange,
    pub ty: Type,
}


#[derive(Debug)]
pub enum GenericsConstraint {
    Implements {
        var: (InternedString, TokenRange),
        trait_: ItemPath,
    },
    Outlives {
        longer: (InternedString, TokenRange),
        shorter: (InternedString, TokenRange),
    },
    UniformitySum((InternedString, TokenRange), Vec<(InternedString, TokenRange)>),
}


#[derive(Debug)]
pub struct FunctionDefinition {
    pub attrs: Vec<Attribute>,
    pub visibility: Option<(Visibility, TokenRange)>,
    pub unsafe_token: Option<TokenRange>,
    pub shader_type: Option<(ShaderType, TokenRange)>,
    pub uniformity: Option<(Uniformity, TokenRange)>,
    pub fn_token: TokenRange,
    pub ident: InternedString,
    pub ident_token: TokenRange,
    pub generics: Vec<GenericArgDefinition>,
    pub generics_constraints: Vec<GenericsConstraint>,
    pub params: Vec<(InternedString, TokenRange, Option<(Uniformity, TokenRange)>, Type)>,
    pub block: Block,
    pub ret: (Type, Option<(Uniformity, TokenRange)>),
}

#[derive(Debug)]
pub struct TraitFunction {
    pub attrs: Vec<Attribute>,
    pub unsafe_token: Option<TokenRange>,
    pub uniformity: Option<(Uniformity, TokenRange)>,
    pub fn_token: TokenRange,
    pub ident: InternedString,
    pub ident_token: TokenRange,
    pub generics: Vec<GenericArgDefinition>,
    pub generics_constraints: Vec<GenericsConstraint>,
    pub params: Vec<(InternedString, TokenRange, Type)>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub value: Option<Expression>,
    pub label: Option<(InternedString, TokenRange)>,
    pub range: TokenRange,
}

impl SourceRange for Block {
    fn range(&self) -> TokenRange {
        self.range.clone()
    }
}

#[derive(Debug, Clone)]
pub enum Statement {
    Expression(Expression),
    Return(TokenRange, Option<Expression>),
    Break {
        break_token: TokenRange,
        label :Option<(InternedString, TokenRange)>,
        value: Option<Expression>
    },
    Continue(TokenRange),
    Let(InternedString, TokenRange, Option<(Uniformity, TokenRange)>, Type, Option<Expression>),
}


#[derive(Debug)]
pub struct StaticDefinition {
    pub attrs: Vec<Attribute>,
    pub visibility: Option<(Visibility, TokenRange)>,
    pub static_token: TokenRange,
    pub ident: InternedString,
    pub ident_token: TokenRange,
    pub ty: Type,
    pub init: Expression,
}

#[derive(Debug)]
pub struct ConstantDefinition {
    pub attrs: Vec<Attribute>,
    pub visibility: Option<(Visibility, TokenRange)>,
    pub const_token: TokenRange,
    pub ident: InternedString,
    pub ident_token: TokenRange,
    pub ty: Type,
    pub init: Expression,
}


#[derive(Debug)]
pub struct StructImplBlock {
    pub functions: Vec<FunctionDefinition>,
    pub consts: Vec<ConstantDefinition>,
}


#[derive(Debug)]
pub struct TraitImplBlock {
    pub unsafe_token: Option<TokenRange>,
    pub trait_name: InternedString,
    pub name_token: TokenRange,
    pub functions: Vec<FunctionDefinition>,
    pub types: Vec<TypeAlias>,
}


#[derive(Debug)]
pub struct TypeAlias {
    pub type_token: TokenRange,
    pub args: Vec<GenericArgDefinition>,
    pub ty: Type,
}

#[derive(Debug)]
pub struct TraitDefinition {
    pub unsafe_token: Option<TokenRange>,
    pub trait_name: InternedString,
    pub name_token: TokenRange,
    pub functions: Vec<TraitFunction>,
    pub types: Vec<(InternedString, TokenRange)>,
}


#[derive(Debug)]
pub struct ModuleData {
    pub attrs: Vec<Attribute>,
    pub structs: Vec<StructDefinition>,
    pub traits: Vec<TraitDefinition>,
    pub functions: Vec<FunctionDefinition>,
    pub statics: Vec<StaticDefinition>,
    pub consts: Vec<ConstantDefinition>,
    pub struct_impls: Vec<StructImplBlock>,
    pub trait_impls: Vec<TraitImplBlock>,
    pub inline_modules: Vec<ModuleData>,
    pub outline_modules:Vec<(InternedString, Vec<Attribute>)>,
    
    
    pub span: TokenRange,
}









