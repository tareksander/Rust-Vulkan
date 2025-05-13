use crate::{Ident, SourceSpan};

use super::{function::Function, Attribute, GenericArgDefinition};




#[derive(Debug, Clone)]
pub struct Trait {
    pub attrs: Vec<(Attribute, SourceSpan)>,
    pub ident: (Ident, SourceSpan),
    pub args: Vec<GenericArgDefinition>,
    pub types: Vec<(Ident, SourceSpan)>,
    pub functions: Vec<Function>,
    
    
    
    
    
    
    
    
}


















