use std::{collections::HashMap, fmt::Display, ops::Range};

use super::{Attribute, InternedString, Mutability, StorageClass, Uniformity, Visibility};


#[derive(Debug, Clone)]
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
}


#[derive(Debug, Clone)]
pub struct ItemPath {
    pub segments: Vec<ItemPathSegment>,
    pub global: bool,
}

#[derive(Debug, Clone)]
pub struct ItemPathSegment {
    pub ident: InternedString,
    pub ident_token: TokenRange,
    pub generic_args: Vec<GenericArg>
}



#[derive(Debug, Clone)]
pub enum GenericArg {
    /// Can be another generic or a concrete type.
    Type(Type),
    Expression(Expression),
    Uniformity(Uniformity, TokenRange),
    GenericUniformity(InternedString, TokenRange),
    Lifetime(InternedString, TokenRange),
}

#[derive(Debug)]
pub enum GenericArgDefinition {
    Type(InternedString, TokenRange),
    Expression(InternedString, TokenRange),
    Uniformity(InternedString, TokenRange),
    Lifetime(InternedString, TokenRange),
}


#[derive(Debug, Clone)]
pub enum Type {
    Path(ItemPath, Option<(Uniformity, TokenRange)>),
    Pointer{
        star_token: TokenRange,
        uni: Option<(Uniformity, TokenRange)>,
        mutability: Mutability,
        class: Option<(StorageClass, TokenRange)>,
        ty: Box<Type>
    },
    Reference{
        star_token: TokenRange,
        uni: Option<(Uniformity, TokenRange)>,
        mutability: Option<(Mutability, TokenRange)>,
        ty: Box<Type>
    },
    Array {
        ty: Box<Type>,
        uni: Option<(Uniformity, TokenRange)>,
        size: Expression,
    }
}


#[derive(Debug, Clone)]
pub enum Expression {
    Unary{ e: Box<Expression>, op: UnOp},
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

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Unary { e, op } => {
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
    // TODO non-short-circuiting and/or? Should lead to better GPU performance due to less branches
    // Solution: Just support binary and/or for bools
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
    pub uniformity: Option<(Uniformity, TokenRange)>,
    pub fn_token: TokenRange,
    pub ident: InternedString,
    pub ident_token: TokenRange,
    pub generics: Vec<GenericArgDefinition>,
    pub generics_constraints: Vec<GenericsConstraint>,
    pub params: Vec<(InternedString, TokenRange, Type)>,
    pub block: Block,
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
    pub label: Option<(InternedString, TokenRange)>
}

#[derive(Debug, Clone)]
pub enum Statement {
    Expression(Expression),
    Return(Option<Expression>),
    Break {
        break_token: TokenRange,
        label :Option<(InternedString, TokenRange)>,
        value: Option<Expression>
    },
    Continue(TokenRange),
    Let(InternedString, TokenRange, Option<Expression>),
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









