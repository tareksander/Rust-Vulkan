


use std::{cell::RefCell, collections::HashMap};


use crate::internal::{Builtin, Mutability, ShaderType, StringTable, Visibility, ast::ItemPathSegment};

use super::{ast::{self, BinOp, GenericArgDefinition, GenericsConstraint, ItemPath, TokenRange, UnOp}, Attribute, InternedString, StorageClass, Uniformity};



pub mod astconvert;


/// Resolved item paths are global paths where generics information is already resolved and included
/// in the path via name mangling. The path as a whole is then joined with "::" and forms the resolved path.
pub struct ResolvedItemPath(pub InternedString);


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolID(pub usize);



#[derive(Debug)]
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
    
    #[allow(non_snake_case)]
    pub fn new_prelude(strings: &StringTable) -> Self {
        let mut m = Self {
            map: HashMap::with_capacity(1024),
            items: Vec::with_capacity(1024),
        };
        
        let c = strings.insert_get("core");
        
        
        let globalInvocationID = strings.insert_get("globalInvocationID");
        m.insert(globalInvocationID, (Visibility::Priv, GlobalItem::Import { path: ItemPath { segments: vec![
            ItemPathSegment {
                ident: c,
                ident_token: TokenRange { file: 0, range: 0..0 },
                generic_args: vec![]
            },
            ItemPathSegment {
                ident: globalInvocationID,
                ident_token: TokenRange { file: 0, range: 0..0 },
                generic_args: vec![]
            }
        ], global: true } })).unwrap();
        
        let tu32 = strings.insert_get("u32");
        m.insert(tu32, (Visibility::Priv, GlobalItem::Import { path: ItemPath { segments: vec![
            ItemPathSegment {
                ident: c,
                ident_token: TokenRange { file: 0, range: 0..0 },
                generic_args: vec![]
            },
            ItemPathSegment {
                ident: tu32,
                ident_token: TokenRange { file: 0, range: 0..0 },
                generic_args: vec![]
            }
        ], global: true } })).unwrap();
        
        
        
        return m;
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
    
    pub fn core(strings: &StringTable) -> Self {
        let mut t = SymbolTable::new();
        
        
        t.insert(strings.insert_get("globalInvocationID"), (Visibility::Pub, GlobalItem::Static {
            attrs: vec![Attribute::Builtin(Builtin::GlobalInvocationId)],
            ident_token: TokenRange { file: 0, range: 0..0 },
            uni: Uniformity::Invocation,
            ty: Type::Vector { components: 3, ty: Primitive::U32 },
        })).unwrap();
        
        // TODO make declarative macro to help
        t.insert(strings.insert_get("u32"), (Visibility::Pub, GlobalItem::Type(Type::Primitive(Primitive::U32)))).unwrap();
        
        
        
        
        
        return t;
    }
    
    
    pub fn eval_constexprs(&mut self) {
        // evaluate definitive constexprs, that is constant initializers, generic value arguments, and array and vector lengths
        
        
        todo!()
    }
    
    pub fn resolve_paths(&mut self) {
        
        
        
        todo!()
    }
    
}

#[derive(Debug)]
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
        uni: Uniformity,
        ty: Type,
    },
    FunctionTemplate {
        
        args: Vec<GenericArgDefinition>,
        constraints: Vec<GenericsConstraint>,
    },
    Function(Function),
    Import {
        path: ItemPath,
    },
    // TODO import from another symbol table, to make symbol tables for packages more disposable for an LSP.
    ResolvedImport {
        public: bool,
        path: ItemPath,
        id: SymbolID,
    },
    
    Placeholder,
    Type(Type),
    Module(SymbolTable),
}

#[derive(Debug)]
pub struct Function {
    pub attrs: Vec<Attribute>,
    pub ident_token: TokenRange,
    pub shader_type: ShaderType,
    pub params: Vec<(InternedString, TokenRange, Uniformity, Type)>,
    pub ret: (Type, Uniformity),
    pub blocks: RefCell<Vec<IRBlock>>,
}


#[derive(Debug, Clone)]
pub enum Type {
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
    UnresolvedArray {
        length: ast::Expression,
        ty: Box<Type>,
    },
    RuntimeArray {
        ty: Box<Type>,
    },
    Pointer {
        class: StorageClass,
        ty: Box<Type>,
        mutability: Mutability,
    },
    Reference {
        class: StorageClass,
        ty: Box<Type>,
        mutability: Mutability,
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

impl IRID {
    /// Increments the current id by 1 and returns a copy of the last value.
    pub fn next(&mut self) -> IRID {
        let i = *self;
        self.0 += 1;
        return i;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockID(pub usize);


#[derive(Debug)]
/// A small SSA instruction set.
pub enum IRInstruction {
    /// An unresolved path. Should be eliminated by path canonicalization.
    /// Can be turned either into a ResolvedPath or eliminated via replacing with the local SSA ID.
    Path {
        path: ItemPath,
        tokens: TokenRange,
        id: IRID,
        /// Whether an lvalue or an rvalue is needed. If an rvalue is needed, a load instruction will be inserted for the resolved path.
        /// If an lvalue is needed for a constant, a temporary variable is created and the value stored there, and its id replaces the one of the path.
        lvalue: bool,
    },
    
    /// A path that has been resolved into a global symbol ID.
    /// Only functions and statics can be used like this.
    /// Type information is stored in the symbol.
    /// In any case, if the symbol isn't a constant, it is a pointer to the memory location, not the value itself.
    ResolvedPath {
        path: SymbolID,
        tokens: TokenRange,
        id: IRID,
    },
    
    Local {
        ident: InternedString,
        ident_token: TokenRange,
        /// Optimization passes may convert local variables to SSA, but only after error-generating stages so that mapping info is still readily available.
        /// For SPIR-V debug info, mapping info should still be kept in a side-table.
        id: IRID,
        /// None for a type to be inferred. Technically the type is a pointer to the supplied type with the function storage class, but that is implied.
        ty: Option<Type>,
        uni: Uniformity
    },
    
    // TODO trait method invocation
    
    
    UnOp {
        inp: IRID,
        op: UnOp,
        out: IRID,
        span: TokenRange,
        // no type, unary and binary operators have the same input as output types.
    },
    
    BinOp {
        lhs: IRID,
        op: BinOp,
        rhs: IRID,
        out: IRID,
        span: TokenRange,
        // no type, unary and binary operators have the same input as output types, or a predefined output (bool for comparisons).
        // assignments shouldn't be an IR operation, they are immediately desugared to a store operation.
    },
    
    
    Unit {
        out: IRID
    },
    
    Load {
        ptr: IRID,
        out: IRID,
    },
    
    Store {
        ptr: IRID,
        value: IRID,
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
    
    /// Constant integer value.
    Int {
        v: u128,
        id: IRID,
        token_id: TokenRange,
        /// None for a type to be inferred.
        ty: Option<Type>,
    },
    
    /// Constant float value
    Float {
        v: f64,
        id: IRID,
        token_id: TokenRange,
        /// None for a type to be inferred.
        ty: Option<Type>,
    },
    
    
    /// Performs a cast from one type to another. Only valid for numbers and pointers, for pointers to references, and for mutable to immutable references.
    Cast {
        inp: IRID,
        out: IRID,
        ty: Type,
    },
    
    /// Reduces the uniformity of a value by spreading it out over the lower scope.
    Spread {
        inp: IRID,
        out: IRID,
        uni: Uniformity
    },
    
    ReturnValue {
        id: IRID,
        token_id: TokenRange,
    },
    
    Return {
        token_id: TokenRange,
    },
    
    Loop {
        header: BlockID,
        body: BlockID,
        cont: BlockID,
        merge: BlockID,
        construct: TokenRange,
    },
    Branch {
        target_block: BlockID,
    },
    If {
        inp: IRID,
        true_target_block: BlockID,
        false_target_block: BlockID,
        merge: BlockID,
        construct: TokenRange,
    },
    Phi {
        out: IRID,
        sources: Vec<(IRID, BlockID)>
    }
    
    
    
    
}

#[derive(Debug)]
pub struct IRBlock {
    pub instructions: Vec<IRInstruction>,
}






