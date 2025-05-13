use crate::{ast::ItemPath, GetSpan, Mutability, SourcePos, SourceSpan, StorageClass, Uniformity};




#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Item(Option<(Uniformity, SourceSpan)>, ItemPath),
    Reference(Reference),
    Pointer(Pointer),
    SelfType(SourceSpan),
    Unit(SourceSpan),
    Tuple(Tuple),
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tuple {
    pub ty: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    pub mutable: Mutability,
    pub uni: Option<(Uniformity, SourceSpan)>,
    pub storage: Option<StorageClass>,
    pub ty: Box<Type>,
    pub start: SourcePos,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pointer {
    pub mutable: Mutability,
    pub uni: Option<(Uniformity, SourceSpan)>,
    pub storage: Option<StorageClass>,
    pub ty: Box<Type>,
    pub start: SourcePos,
}



impl GetSpan for &Type {
    fn span(self) -> SourceSpan {
        match self {
            Type::Item(_, item_path) => item_path.span(),
            Type::Reference(reference) => reference.start.to(&reference.ty.span()),
            Type::Pointer(pointer) => pointer.start.to(&pointer.ty.span()),
            Type::SelfType(source_span) => source_span.clone(),
            Type::Unit(source_span) => source_span.clone(),
            Type::Tuple(tuple) => tuple.ty.iter().span(),
        }
    }
}



