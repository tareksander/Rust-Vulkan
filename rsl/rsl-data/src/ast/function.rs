use crate::{ast::{expr::Expression, statement::{Block, Statement}, ty::Type}, ast::Attribute, Ident, ast::ItemPath, SourceSpan, Uniformity, Visibility};

use super::{GenericArgDefinition, Safety};





#[derive(Debug, Clone)]
pub struct Function {
    pub attrs: Vec<(Attribute, SourceSpan)>,
    pub vis: Option<(Visibility, SourceSpan)>,
    pub safety: Safety,
    pub generic_args: Vec<GenericArgDefinition>,
    pub uni: (Uniformity, SourceSpan),
    pub fn_: SourceSpan,
    pub ident: (Ident, SourceSpan),
    pub params: Vec<((Ident, SourceSpan), Type)>,
    pub ret: Type,
    pub block: Block,
}













