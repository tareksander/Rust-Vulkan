use crate::{Ident, SourceSpan, Visibility};

use super::{function::Function, module::{ConstantDefinition, StaticDefinition}, ty::Type, Attribute, GenericArgDefinition};




#[derive(Debug, Clone)]
pub struct Structure {
    pub attrs: Vec<(Attribute, SourceSpan)>,
    pub ident: (Ident, SourceSpan),
    pub args: Vec<GenericArgDefinition>,
    pub fields: Vec<StructField>,
    pub span: SourceSpan,
}


#[derive(Debug, Clone)]
pub struct StructField {
    pub vis: Visibility,
    pub ident: (Ident, SourceSpan),
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct Implementation {
    pub attrs: Vec<(Attribute, SourceSpan)>,
    pub ident: (Ident, SourceSpan),
    // Optional trait type
    pub tr: Option<Type>,
    pub args: Vec<GenericArgDefinition>,
    pub types: Vec<(Ident, SourceSpan)>,
    pub functions: Vec<Function>,
    // no associated constants for now, to only have to worry about associated functions
    //pub constants: Vec<ConstantDefinition>,
}









