


use core::slice;
use std::{cell::RefCell, collections::HashMap, fmt::{Debug, Pointer}, hash::Hash, iter::Enumerate, mem::replace, rc::Rc};


use crate::internal::{Builtin, Mutability, ShaderType, StringTable, Visibility, ast::ItemPathSegment};

use super::{ast::{self, BinOp, GenericArgDefinition, GenericsConstraint, ItemPath, TokenRange, UnOp}, Attribute, InternedString, StorageClass, Uniformity};



pub mod astconvert;



#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolID(pub usize);



#[derive(Debug, Clone)]
pub struct SymbolTable {
    map: HashMap<InternedString, usize>,
    mapr: HashMap<usize, InternedString>,
    items: Vec<(Visibility, GlobalItem)>,
}


impl SymbolTable {
    pub fn new() -> Self {
        Self {
            map: HashMap::with_capacity(1024),
            mapr: HashMap::with_capacity(1024),
            items: Vec::with_capacity(1024),
        }
    }
    
    #[allow(non_snake_case)]
    pub fn new_prelude(strings: &StringTable) -> Self {
        let mut m = Self {
            map: HashMap::with_capacity(1024),
            mapr: HashMap::with_capacity(1024),
            items: Vec::with_capacity(1024),
        };
        
        let c = strings.insert_get("core");
        
        let dummy_range = TokenRange { file: 0, range: 0..0 };
        
        let globalInvocationID = strings.insert_get("globalInvocationID");
        m.insert(globalInvocationID, (Visibility::Priv, GlobalItem::Import { path: ItemPath { segments: vec![
            ItemPathSegment {
                ident: c,
                ident_token: dummy_range.clone(),
                generic_args: vec![]
            },
            ItemPathSegment {
                ident: globalInvocationID,
                ident_token: dummy_range.clone(),
                generic_args: vec![]
            }
        ], global: true }, span: dummy_range.clone() })).unwrap();
        
        let t = strings.insert_get("u32");
        m.insert(t, (Visibility::Priv, GlobalItem::Import { path: ItemPath { segments: vec![
            ItemPathSegment {
                ident: c,
                ident_token: dummy_range.clone(),
                generic_args: vec![]
            },
            ItemPathSegment {
                ident: t,
                ident_token: dummy_range.clone(),
                generic_args: vec![]
            }
        ], global: true }, span: dummy_range.clone() })).unwrap();
        let t = strings.insert_get("f32");
        m.insert(t, (Visibility::Priv, GlobalItem::Import { path: ItemPath { segments: vec![
            ItemPathSegment {
                ident: c,
                ident_token: dummy_range.clone(),
                generic_args: vec![]
            },
            ItemPathSegment {
                ident: t,
                ident_token: dummy_range.clone(),
                generic_args: vec![]
            }
        ], global: true }, span: dummy_range.clone() })).unwrap();
        
        
        return m;
    }
    
    
    pub fn insert_module(&mut self, other: SymbolTable, name: InternedString) {
        self.insert(name, (Visibility::Priv, GlobalItem::Module(other))).unwrap();
    }
    
    pub fn lookup(&self, path: &InternedString) -> Option<&(Visibility, GlobalItem)> {
        self.map.get(path).and_then(|i| Some(&self.items[*i]))
    }
    
    pub fn lookup_id(&self, path: &InternedString) -> Option<SymbolID> {
        self.map.get(path).and_then(|i| Some(SymbolID(*i)))
    }
    
    pub fn get(&self, id: SymbolID) -> &(Visibility, GlobalItem) {
        match &self.items[id.0].1 {
            GlobalItem::ResolvedImport { path: _, id } => self.get(*id),
            _ => &self.items[id.0]
        }
    }
    
    pub fn get_mut(&mut self, id: SymbolID) -> &(Visibility, GlobalItem) {
        match &mut self.items[id.0].1 {
            GlobalItem::ResolvedImport { path: _, id } => {
                let id = *id;
                self.get_mut(id)
            },
            _ => {
                &mut self.items[id.0]
            }
        }
    }
    
    pub fn follow_imports(&self, id: SymbolID) -> SymbolID {
        match &self.items[id.0].1 {
            GlobalItem::ResolvedImport { path: _, id } => self.follow_imports(*id),
            _ => id
        }
    }
    
    pub fn get_name(&self, id: SymbolID) -> InternedString {
        *self.mapr.get(&id.0).unwrap()
    }
    
    pub fn insert(&mut self, path: InternedString, item: (Visibility, GlobalItem)) -> Result<(), ()> {
        if let Some(i) = self.map.get(&path) {
            return Err(());
        }
        let i = self.items.len();
        self.items.push(item);
        self.map.insert(path, i);
        self.mapr.insert(i, path);
        return Ok(());
    }
    
    pub fn reserve(&mut self, path: InternedString, vis: Visibility) -> Result<SymbolID, ()> {
        if let Some(i) = self.map.get(&path) {
            return Err(());
        }
        let i = self.items.len();
        self.items.push((vis, GlobalItem::Placeholder));
        self.map.insert(path, i);
        self.mapr.insert(i, path);
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
        t.insert(strings.insert_get("f32"), (Visibility::Pub, GlobalItem::Type(Type::Primitive(Primitive::F32)))).unwrap();
        
        
        
        
        return t;
    }
    
    pub fn iter(&self) -> impl Iterator<Item = SymbolID> {
        (0..self.items.len()).map(|e| SymbolID(e))
    }
    
    
    
    
    pub fn eval_constexprs(&mut self) {
        // evaluate definitive constexprs, that is constant initializers, generic value arguments, and array and vector lengths
        
        
        todo!()
    }
    
    
    /// Should only be called on the top level symbol table.
    /// This merges all child symbol tables recursively, adjusts the paths of items, and resolves all imports.
    /// This can be called a second time after adding more modules to the symbol table again though.
    /// Doesn't work with adding resolved modules though, since symbol ids between items aren't touched even though they change.
    pub fn resolve_paths(&mut self, strings: &StringTable) {
        let mut modules = vec![];
        
        fn find_modules(modules: &mut Vec<(InternedString, SymbolTable)>, table: &mut SymbolTable, prefix: InternedString, strings: &StringTable) {
            for (id, i) in table.items.iter_mut().enumerate() {
                match &mut i.1 {
                    GlobalItem::Module(symbol_table) => {
                        let name = *table.mapr.get(&id).unwrap();
                        let prefix = strings.insert_get(&(strings.lookup(prefix) + "::" + &strings.lookup(name)));
                        find_modules(modules, symbol_table, prefix, strings);
                        let s = replace(&mut i.1, GlobalItem::RemovedModule);
                        modules.push((prefix, match s {
                            GlobalItem::Module(s) => s,
                            _ => {unreachable!()}
                        }));
                    },
                    _ => {}
                }
            }
        }
        
        let double_colon = strings.insert_get("::");
        let empty = strings.insert_get("");
        
        find_modules(&mut modules, self, empty, strings);
        
        // TODO resolve all outstanding symbols, generating errors for symbols still not found
        
        
        
        
        for m in modules {
            self.items.reserve(m.1.items.len());
            self.map.reserve(m.1.items.len());
            self.mapr.reserve(m.1.items.len());
            for (id, mut i) in m.1.items.into_iter().enumerate() {
                let new_path = strings.insert_get(&(strings.lookup(m.0) + "::" + &strings.lookup(m.1.mapr[&id])));
                self.insert(new_path, i).unwrap();
            }
        }
        
        
        for (id, i) in self.items.iter_mut().enumerate() {
            match i {
                (v, GlobalItem::Import { path , span}) => {
                    // for now the only imports are the prelude into core, which are global paths
                    if path.global {
                        let p = path.interned(strings, double_colon);
                        // TODO visibility calculations
                        if let Some(id) = self.map.get(&p) {
                            let ri = GlobalItem::ResolvedImport { path: path.clone(), id: SymbolID(*id) };
                            i.1 = ri;
                        } else {
                            panic!("Could not find import: {}", strings.lookup(p));
                        }
                    } else {
                        todo!()
                    }
                    
                    // TODO support paths from local modules and from imported submodules, that is use foo::bar and then use bar::baz
                    
                    
                    
                },
                _ => {
                    //println!("symbol path: {}", strings.lookup(self.mapr[&id]));
                }
            }
        }
        
        fn resolve_type(table: &SymbolTable, t: &mut Type, m: InternedString, strings: &StringTable) {
            match t {
                Type::Unresolved(item_path) => {
                    let mut sym = None;
                    if ! item_path.global {
                        let lp = item_path.segments[0].ident;
                        let gp = strings.insert_get(&(strings.lookup(m) + "::" + &strings.lookup(lp)));
                        if table.lookup_id(&gp).is_some() {
                            if let Some(s) = table.lookup_id(&item_path.interned(strings, m)) {
                                sym = Some(s);
                            } else {
                                panic!()
                            }
                        }
                    }
                    if sym.is_none() {
                        item_path.global = true;
                        if let Some(s) = table.lookup_id(&item_path.interned(strings, m)) {
                            sym = Some(s);
                        } else {
                            panic!()
                        } 
                    }
                    let s = sym.unwrap();
                    match &table.get(s).1 {
                        GlobalItem::Type(ty) => {
                            *t = ty.clone();
                            
                        }
                        _ => {
                            todo!()
                        }
                    }
                },
                Type::Resolved(symbol_id) => {},
                Type::Primitive(primitive) => {},
                Type::Vector { components, ty } => {},
                Type::Matrix { rows, cols, ty } => {},
                Type::Array { length, ty } => todo!(),
                Type::UnresolvedArray { length, ty } => todo!(),
                Type::RuntimeArray { ty } => todo!(),
                Type::Pointer { class, ty, mutability } => resolve_type(table, &mut*ty, m, strings),
                Type::Reference { class, ty, mutability } => resolve_type(table, &mut*ty, m, strings),
                Type::Function { sym } => {},
            }
        }
        
        fn resolve_item(table: &SymbolTable, p: &mut ItemPath, m: InternedString, strings: &StringTable) -> SymbolID {
            let mut sym = None;
            if ! p.global {
                let lp = p.segments[0].ident;
                let gp = strings.insert_get(&(strings.lookup(m) + "::" + &strings.lookup(lp)));
                if table.lookup_id(&gp).is_some() {
                    if let Some(s) = table.lookup_id(&p.interned(strings, m)) {
                        sym = Some(s);
                    } else {
                        panic!()
                    }
                }
            }
            if sym.is_none() {
                p.global = true;
                if let Some(s) = table.lookup_id(&p.interned(strings, m)) {
                    sym = Some(s);
                } else {
                    panic!("could not find symbol: {}", strings.lookup(p.interned(strings, m)));
                } 
            }
            return sym.unwrap();
        }
        
        
        for (id, i) in self.items.iter().enumerate() {
            match i {
                (v, GlobalItem::Function(f)) => {
                    let m = self.mapr[&id].base(strings);
                    let mut blocks = f.blocks.borrow_mut();
                    let mut types = f.types.borrow_mut();
                    for i in 0..f.num_params {
                        let t = types.get_mut(&IRID(i)).unwrap();
                        resolve_type(self, t, m, strings);
                    }
                    let mut rt = f.ret.borrow_mut();
                    resolve_type(self, &mut rt.0, m, strings);
                    for b in blocks.iter_mut() {
                        for inst in &mut b.instructions {
                            match inst {
                                IRInstruction::Path { path, tokens, id, lvalue } => {
                                    let pid  = resolve_item(self, path, m, strings);
                                    *inst = IRInstruction::ResolvedPath { path: pid, tokens: tokens.clone(), id: *id, lvalue: *lvalue  };
                                },
                                IRInstruction::Local { ident, ident_token, id, ty, uni, mutable } => {
                                    if let Some(ty) = ty {
                                        resolve_type(self, ty, m, strings);
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                    // TODO load constants into function variables for lvalue positions
                },
                _ => {}
            }
        }
        
    }
    
}

#[derive(Debug, Clone)]
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
        span: TokenRange,
    },
    ResolvedImport {
        path: ItemPath,
        id: SymbolID,
    },
    
    Placeholder,
    Type(Type),
    Module(SymbolTable),
    RemovedModule,
    Removed,
}

#[derive(Clone)]
pub struct Function {
    pub attrs: Vec<Attribute>,
    pub ident_token: TokenRange,
    pub shader_type: ShaderType,
    //pub params: Vec<(InternedString, TokenRange, Uniformity, Type)>,
    pub num_params: usize,
    pub ret: RefCell<(Type, Uniformity)>,
    pub blocks: RefCell<Vec<IRBlock>>,
    pub next_id: RefCell<IRID>,
    pub types: RefCell<HashMap<IRID, Type>>,
}

impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Function").field("attrs", &self.attrs).field("ident_token", &self.ident_token).field("shader_type", &self.shader_type).field("num_params", &self.num_params).field("ret", &self.ret)
        .field("next_id", &self.next_id).field("params", &(0..self.num_params).map(|i| self.types.borrow()[&IRID(i)].clone()).collect::<Vec<_>>()).finish()?;
        let b = self.blocks.borrow();
        let t = self.types.borrow();
        
        let mut l = f.debug_list();
        for b in b.iter() {
            for i in &b.instructions {
                l.entry(&i);
                let mut print_type = |id: &IRID| {
                    if let Some(t) = t.get(id) {
                        l.entry(t);
                    } else {
                        l.entry(&"Missing type");
                    }
                };
                match i {
                    IRInstruction::Local { ident, ident_token, id, ty, uni, mutable } => {
                        print_type(id)
                    },
                    IRInstruction::ResolvedPath { path, tokens, id, lvalue } => {
                        print_type(id)
                    },
                    IRInstruction::UnOp { inp, op, out, span } => {
                        print_type(out)
                    },
                    IRInstruction::BinOp { lhs, op, rhs, out, span } => {
                        print_type(out)
                    },
                    IRInstruction::Property { inp, name, out } => {
                        print_type(out)
                    },
                    IRInstruction::Load { ptr, out } => {
                        print_type(out)
                    },
                    IRInstruction::Store { ptr, value } => {
                        print_type(value)
                    },
                    _ => {}
                }
            }
        }
        l.finish()?;
        Ok(())
    }
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
        cols: u8,
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
    },
    Function {
        sym: SymbolID,
    },
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Unresolved(l0), Self::Unresolved(r0)) => unreachable!(),
            (Self::Resolved(l0), Self::Resolved(r0)) => l0 == r0,
            (Self::Primitive(l0), Self::Primitive(r0)) => l0 == r0,
            (Self::Vector { components: l_components, ty: l_ty }, Self::Vector { components: r_components, ty: r_ty }) => l_components == r_components && l_ty == r_ty,
            (Self::Matrix { rows: l_rows, cols: l_cols, ty: l_ty }, Self::Matrix { rows: r_rows, cols: r_cols, ty: r_ty }) => l_rows == r_rows && l_cols == r_cols && l_ty == r_ty,
            (Self::Array { length: l_length, ty: l_ty }, Self::Array { length: r_length, ty: r_ty }) => l_length == r_length && l_ty == r_ty,
            (Self::UnresolvedArray { length: l_length, ty: l_ty }, Self::UnresolvedArray { length: r_length, ty: r_ty }) => unreachable!(),
            (Self::RuntimeArray { ty: l_ty }, Self::RuntimeArray { ty: r_ty }) => l_ty == r_ty,
            (Self::Pointer { class: l_class, ty: l_ty, mutability: l_mutability }, Self::Pointer { class: r_class, ty: r_ty, mutability: r_mutability }) => l_class == r_class && l_ty == r_ty && l_mutability == r_mutability,
            (Self::Reference { class: l_class, ty: l_ty, mutability: l_mutability }, Self::Reference { class: r_class, ty: r_ty, mutability: r_mutability }) => l_class == r_class && l_ty == r_ty && l_mutability == r_mutability,
            _ => false,
        }
    }
}

impl Eq for Type {}

impl Hash for Type {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Primitive {
    U8, U16, U32, U64,
    I8, I16, I32, I64,
    F16, F32, F64,
    Bool,
    Unit
}

impl Primitive {
    
    pub fn is_number(&self) -> bool {
        use Primitive::*;
        match *self {
            U8 | U16 | U32 | U64 |
            I8 | I16 | I32 | I64 |
            F16 | F32 | F64 => true,
            _ => false,
        }
    }
    
    pub fn is_float(&self) -> bool {
        use Primitive::*;
        match *self {
            F16 | F32 | F64 => true,
            _ => false,
        }
    }
    
    pub fn is_int(&self) -> bool {
        use Primitive::*;
        match *self {
            U8 | U16 | U32 | U64 |
            I8 | I16 | I32 | I64 => true,
            _ => false,
        }
    }
    
    pub fn is_uint(&self) -> bool {
        use Primitive::*;
        match *self {
            U8 | U16 | U32 | U64 => true,
            _ => false,
        }
    }
    
    pub fn is_sint(&self) -> bool {
        use Primitive::*;
        match *self {
            I8 | I16 | I32 | I64 => true,
            _ => false,
        }
    }
    
    
    pub fn is_unit(&self) -> bool {
        *self == Primitive::Unit
    }
    
    pub fn is_bool(&self) -> bool {
        *self == Primitive::Bool
    }
    
    
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


#[derive(Debug, Clone)]
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
        lvalue: bool,
    },
    
    Local {
        ident: InternedString,
        ident_token: TokenRange,
        /// Optimization passes may convert local variables to SSA, but only after error-generating stages so that mapping info is still readily available.
        /// For SPIR-V debug info, mapping info should still be kept in a side-table.
        id: IRID,
        /// None for a type to be inferred. Technically the type is a pointer to the supplied type with the function storage class, but that is implied.
        ty: Option<Type>,
        uni: Uniformity,
        mutable: Mutability,
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

#[derive(Debug, Clone)]
pub struct IRBlock {
    pub instructions: Vec<IRInstruction>,
}






