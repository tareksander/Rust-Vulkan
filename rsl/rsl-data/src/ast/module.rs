use crate::{ast::{expr::Expression, function::Function, ty::Type}, ast::Attribute, Ident, Mutability, SourceSpan};

use super::{structure::{Implementation, Structure}, traits::Trait, ItemPath};






#[derive(Debug, Clone)]
pub struct Module {
    pub span: SourceSpan,
    pub inline_modules: Vec<Module>,
    
    pub functions: Vec<Function>,
    pub statics: Vec<StaticDefinition>,
    pub constants: Vec<ConstantDefinition>,
    
    pub structs: Vec<Structure>,
    pub impls: Vec<Implementation>,
    pub traits: Vec<Trait>,
    
    pub imports: Vec<Import>,
    
    
}



#[derive(Debug, Clone)]
pub struct Import {
    pub path: ItemPath,
}



#[derive(Debug, Clone)]
pub struct StaticDefinition {
    pub attrs: Vec<(Attribute, SourceSpan)>,
    pub mutability: Mutability,
    pub span: SourceSpan,
    pub ident: Ident,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct ConstantDefinition {
    pub attrs: Vec<(Attribute, SourceSpan)>,
    pub span: SourceSpan,
    pub ident: Ident,
    pub ty: Type,
    pub init: Expression
}








