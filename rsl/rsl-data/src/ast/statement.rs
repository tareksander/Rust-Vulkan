use crate::{ast::{expr::Expression, ty::Type}, Ident, ast::ItemPath, Mutability, SourceSpan};




#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Expression(Expression),
    Return(Option<Expression>),
    Break(SourceSpan),
    Continue(SourceSpan),
    Let(Let),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Let {
    Single(SourceSpan, Mutability, Ident, Option<Type>, Option<Expression>),
    // TODO nested tuple destructuring
    //Destructure(Vec<(Mutability, Ident)>, Option<Tuple>, Expression),
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub value: Option<Expression>,
    pub span: SourceSpan,
}





