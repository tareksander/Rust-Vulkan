


use std::{cell::RefCell, collections::HashMap, sync::LazyLock};


use crate::internal::{Builtin, StringTable, Visibility};

use super::{ast::{self, BinOp, GenericArgDefinition, GenericsConstraint, ItemPath, TokenRange, UnOp}, Attribute, InternedString, StorageClass, Uniformity};




pub mod astconvert;


/// Resolved item paths are global paths where generics information is already resolved and included
/// in the path via name mangling. The path as a whole is then joined with "::" and forms the resolved path.
pub struct ResolvedItemPath(pub InternedString);


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolID(pub usize);


pub struct SymbolTable {
    map: HashMap<InternedString, usize>,
    items: Vec<(Visibility, GlobalItem)>,
}


impl SymbolTable {
    pub fn new() -> Self {
        Self {
            map: HashMap::with_capacity(1024),
            items: Vec::with_capacity(1024),
        }
    }
    
    pub fn lookup(&self, path: &InternedString) -> Option<&(Visibility, GlobalItem)> {
        self.map.get(path).and_then(|i| Some(&self.items[*i]))
    }
    
    pub fn lookup_id(&self, path: &InternedString) -> Option<SymbolID> {
        self.map.get(path).and_then(|i| Some(SymbolID(*i)))
    }
    
    pub fn get(&self, id: SymbolID) -> &(Visibility, GlobalItem) {
        &self.items[id.0]
    }
    
    pub fn insert(&mut self, path: InternedString, item: (Visibility, GlobalItem)) -> Result<(), ()> {
        if let Some(i) = self.map.get(&path) {
            return Err(());
        }
        let i = self.items.len();
        self.items.push(item);
        self.map.insert(path, i);
        return Ok(());
    }
    
    pub fn reserve(&mut self, path: InternedString, vis: Visibility) -> Result<SymbolID, ()> {
        if let Some(i) = self.map.get(&path) {
            return Err(());
        }
        let i = self.items.len();
        self.items.push((vis, GlobalItem::Placeholder));
        self.map.insert(path, i);
        return Ok(SymbolID(i));
    }
    
    pub fn set_reserved(&mut self, id: SymbolID, item: (Visibility, GlobalItem)) -> Result<(), ()> {
        match self.items[id.0] {
            (_, GlobalItem::Placeholder) => {
                self.items[id.0] = item;
                return Ok(());
            },
            _ => {
                return Err(());
            }
        }
    }
    
    
    pub fn replace_symbol(&mut self, id: SymbolID, item: (Visibility, GlobalItem)) {
        self.items[id.0] = item;
    }
    
    pub fn core(strings: StringTable) -> Self {
        let mut t = SymbolTable::new();
        
        
        t.insert(strings.insert_get("globalInvocationID"), (Visibility::Pub, GlobalItem::Static {
            attrs: vec![Attribute::Builtin(Builtin::GlobalInvocationId)],
            ident_token: TokenRange { file: 0, range: 0..0 },
            ty: Type {
                uni: Some(Uniformity::Invocation),
                var: TypeVariant::Vector { components: 3, ty: Primitive::U32 },
            },
        })).unwrap();
        
        // TODO make declarative macro to help
        t.insert(strings.insert_get("u32"), (Visibility::Pub, GlobalItem::Type(TypeVariant::Primitive(Primitive::U32)))).unwrap();
        
        
        
        
        
        return t;
    }
    
    
}


pub enum GlobalItem {
    /// Generic struct. Should be resolved after type checking and can then be ignored.
    StructTemplate {
        args: Vec<GenericArgDefinition>,
        constraints: Vec<GenericsConstraint>,
    },
    Struct {
        attrs: Vec<Attribute>,
        ident_token: TokenRange,
        
    },
    Trait {
        attrs: Vec<Attribute>,
        ident_token: TokenRange,
        args: Vec<GenericArgDefinition>,
        constraints: Vec<GenericsConstraint>,
        types: Vec<InternedString>,
        /// Trait functions contain no blocks, only implementation functions do.
        functions: Vec<Function>,
        
    },
    Static {
        attrs: Vec<Attribute>,
        ident_token: TokenRange,
        ty: Type,
    },
    FunctionTemplate {
        
        args: Vec<GenericArgDefinition>,
        constraints: Vec<GenericsConstraint>,
    },
    Function(Function),
    Import {
        public: bool,
        path: ItemPath,
    },
    // TODO import from another symbol table, to make symbol tables for packages more disposable for an LSP.
    ResolvedImport {
        public: bool,
        path: ItemPath,
        id: SymbolID,
    },
    
    Placeholder,
    Type(TypeVariant),
    Module(SymbolTable),
}


pub struct Function {
    pub attrs: Vec<Attribute>,
    pub ident_token: TokenRange,
    pub params: Vec<(InternedString, TokenRange, Type)>,
    pub ret: Type,
    pub blocks: RefCell<Vec<IRBlock>>,
}


#[derive(Debug, Clone)]
pub struct Type {
    pub uni: Option<Uniformity>,
    pub var: TypeVariant,
}

#[derive(Debug, Clone)]
pub enum TypeVariant {
    Unresolved(ItemPath),
    /// Only valid for concrete structs and traits. Should be generated by the type checking stage via monomorphisation.
    Resolved(SymbolID),
    Primitive(Primitive),
    Vector {
        components: u8,
        ty: Primitive,
    },
    Matrix {
        rows: u8,
        cold: u8,
        ty: Primitive,
    },
    Array {
        length: usize,
        ty: Box<Type>,
    },
    RuntimeArray {
        ty: Box<Type>,
    },
    Pointer {
        class: StorageClass,
        ty: Box<Type>,
    },
    Reference {
        class: StorageClass,
        ty: Box<Type>,
    }
}

#[derive(Debug, Clone)]
pub enum Primitive {
    U8, U16, U32, U64,
    I8, I16, I32, I64,
    F16, F32, F64,
    Bool,
    Unit
}



#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IRID(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockID(pub usize);

/// A small SSA instruction set.
pub enum IRInstruction {
    /// An unresolved path. Should be eliminated by path canonicalization.
    /// Can be turned either into a ResolvedPath or eliminated via replacing with the local SSA ID.
    Path {
        path: ItemPath,
        tokens: TokenRange,
        id: IRID,
    },
    
    /// A path that has been resolved into a global symbol ID.
    /// Only functions and statics can be used like this.
    /// Type information is stored in the symbol.
    ResolvedPath {
        path: SymbolID,
        token: TokenRange,
        id: IRID,
    },
    
    Local {
        ident: InternedString,
        ident_token: TokenRange,
        id: IRID,
        /// None for a type to be inferred.
        ty: Option<Type>,
    },
    
    // TODO trait method invocation
    
    
    UnOp {
        inp: IRID,
        op: UnOp,
        out: IRID,
        span: TokenRange,
        // no type, unary and binary operators have the same input as output types, except assignment (which has Unit).
    },
    
    BinOp {
        lhs: IRID,
        op: BinOp,
        rhs: IRID,
        out: IRID,
        span: TokenRange,
        // no type, unary and binary operators have the same input as output types, or a predefined output (bool for comparisons), except assignment (which has Unit).
    },
    
    Property {
        inp: IRID,
        name: (InternedString, TokenRange),
        out: IRID,
        // No type, can be read from the type of the input and the field
    },
    
    Call {
        func: IRID,
        args: Vec<IRID>,
        out: IRID,
        span: TokenRange,
    },
    
    Int {
        v: u128,
        id: IRID,
        token_id: TokenRange,
        /// None for a type to be inferred.
        ty: Option<Type>,
    },
    
    Float {
        v: f64,
        id: IRID,
        token_id: TokenRange,
        /// None for a type to be inferred.
        ty: Option<Type>,
    },
    
    
    Cast {
        inp: IRID,
        out: IRID,
        ty: Type,
    },
    
    
    SelectionMerge {
        merge: BlockID,
        construct: TokenRange,
    },
    LoopMerge {
        merge: BlockID,
        cont: BlockID,
        construct: TokenRange,
    },
    Branch {
        target_block: BlockID,
    },
    BranchConditional {
        inp: IRID,
        true_target_block: BlockID,
        false_target_block: BlockID,
    },
    Phi {
        out: IRID,
        sources: Vec<(IRID, BlockID)>
    }
    
    
    
    
}

pub struct IRBlock {
    pub instructions: Vec<IRInstruction>,
}






