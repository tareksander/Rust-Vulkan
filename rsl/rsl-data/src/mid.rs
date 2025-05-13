//! The mid-level representation of the AST.
//! 
//! This is mostly the same as the AST, with some structures lowered and including mapping information (like turning shadowed variable names unique)
//! and structured for further analysis.
//! 
//! 


use std::{cell::{LazyCell, RefCell}, collections::{HashMap, HashSet}, fmt::Display, hash::Hash, path::PathBuf, rc::Rc};

use crate::{ast::{self, expr::Expression, parser::parse, statement::Block, tokenizer::tokenize, Attribute, BuiltinVariable, Entrypoint, GenericArg, GenericArgDefinition, ItemPath, Safety}, GetSpan, Ident, Mutability, SourcePos, SourceSpan, StorageClass, Uniformity, Visibility};



/// Some global metadata about the program for analysis
#[derive(Debug)]
pub struct Metadata {
    /// A list of all functions (including entrypoints)
    pub functions: Vec<ItemPath>,
    /// A list of all entrypoint functions with associated data
    pub entrypoints: HashMap<ItemPath, Entrypoint>,
    /// The set of all functions called from each function
    pub call_set: HashMap<ItemPath, HashSet<ItemPath>>,
    /// The set of all statics used by each function
    pub static_set: HashMap<ItemPath, HashSet<ItemPath>>,
    /// Maps the functions to the entry points they are called from
    pub function_entrypoints: HashMap<ItemPath, HashSet<ItemPath>>
}



#[derive(Debug, Clone)]
pub enum ModuleItem {
    Struct(Struct),
    Function(Function),
    Static(Static),
    Constant(Constant),
    Import(ItemPath),
    Type(Type),
    Primitive(Primitive),
    Module(Scope),
    // TODO trait
}


/// Scopes come from modules, traits, structs or functions (though function scopes are unavailable outside itself)
#[derive(Debug, Clone)]
pub struct Scope {
    pub items: HashMap<Ident, ModuleItem>,
}






impl Scope {
    
    
    
    /// Creates a compilation root scope. The root scope itself shouldn't be modified, but sub-scopes are added for the top-level shader module and libraries.
    pub fn root() -> Scope {
        
        
        let mut root = Scope {
            items: HashMap::new(),
        };
        let core_span = SourceSpan {
            file: Rc::new(PathBuf::from("compiler.rsl")),
            start: SourcePos { line: 0, character: 0 },
            end: SourcePos { line: 0, character: 0 },
        };
        
        
        root.items.insert(Ident::try_from("u8").unwrap(), ModuleItem::Primitive(Primitive::U8));
        root.items.insert(Ident::try_from("u16").unwrap(), ModuleItem::Primitive(Primitive::U16));
        root.items.insert(Ident::try_from("u32").unwrap(), ModuleItem::Primitive(Primitive::U32));
        root.items.insert(Ident::try_from("u64").unwrap(), ModuleItem::Primitive(Primitive::U64));
        root.items.insert(Ident::try_from("i8").unwrap(), ModuleItem::Primitive(Primitive::I8));
        root.items.insert(Ident::try_from("i16").unwrap(), ModuleItem::Primitive(Primitive::I16));
        root.items.insert(Ident::try_from("i32").unwrap(), ModuleItem::Primitive(Primitive::I32));
        root.items.insert(Ident::try_from("i64").unwrap(), ModuleItem::Primitive(Primitive::I64));
        root.items.insert(Ident::try_from("f16").unwrap(), ModuleItem::Primitive(Primitive::F16));
        root.items.insert(Ident::try_from("f32").unwrap(), ModuleItem::Primitive(Primitive::F32));
        root.items.insert(Ident::try_from("f64").unwrap(), ModuleItem::Primitive(Primitive::F64));
        
        {
            use Primitive::*;
            for p in [
                    U8, U16, U32, U64,
                    I8, I16, I32, I64,
                    F16, F32, F64,
                ] {
                for n in 2..4 {
                    root.items.insert(Ident::try_from(format!("vec{n}{p}").as_str()).unwrap(), ModuleItem::Type(
                        Type {
                            uni: None,
                            ty: TypeVariant::Vector(Vector {
                                components: n,
                                ty: p,
                            }),
                            span: core_span.clone(),
                        },
                    ));
                }
                for n in 2..4 {
                    for m in 2..4 {
                        root.items.insert(Ident::try_from(format!("mat{n}{m}{p}").as_str()).unwrap(), ModuleItem::Type(
                            Type {
                                uni: None,
                                ty: TypeVariant::Matrix(Matrix {
                                    cols: n,
                                    rows: m,
                                    ty: p,
                                }),
                                span: core_span.clone(),
                            },
                        ));
                    }
                }
            }
        }
        
        
        
        root.items.insert(Ident::try_from("globalInvocationId").unwrap(), ModuleItem::Static(Static {
            attrs: vec![(Attribute::Builtin(BuiltinVariable::GlobalInvocationID), core_span.clone())],
            mutability: Mutability::Immutable,
            ty: Type { uni: Some(Uniformity::NonUniform), ty: TypeVariant::Vector(Vector {
                components: 3,
                ty: Primitive::U32
            }), span: core_span.clone() },
            value: None,
        }));
        
        
        
        
        
        
        
        return root;
    }
    
    
    pub fn from_ast(m: ast::module::Module) -> Self {
        let mut s = Scope {
            items: HashMap::new(),
        };
        
        {
            let prelude_span = SourceSpan {
                file: Rc::new(PathBuf::from("prelude.rsl")),
                start: SourcePos { line: 0, character: 0 },
                end: SourcePos { line: 0, character: 0 },
            };
            use Primitive::*;
            for p in [
                    U8, U16, U32, U64,
                    I8, I16, I32, I64,
                    F16, F32, F64,
                ] {
                let ident = Ident::try_from(format!("{p}").as_str()).unwrap();
                s.items.insert(ident.clone(), ModuleItem::Import(ItemPath { segments: vec![(ident, prelude_span.clone(), vec![])], global: true }));
                for n in 2..4 {
                    let ident = Ident::try_from(format!("vec{n}{p}").as_str()).unwrap();
                    s.items.insert(ident.clone(), ModuleItem::Import(ItemPath { segments: vec![(ident, prelude_span.clone(), vec![])], global: true }));
                }
                for n in 2..4 {
                    for m in 2..4 {
                        let ident = Ident::try_from(format!("mat{n}{m}{p}").as_str()).unwrap();
                        s.items.insert(ident.clone(), ModuleItem::Import(ItemPath { segments: vec![(ident, prelude_span.clone(), vec![])], global: true }));
                    }
                }
            }
        }
        
        for st in m.structs {
            s.items.insert(st.ident.0.clone(), ModuleItem::Struct(Struct {
                attrs: st.attrs,
                fields: st.fields.iter().map(|f| (f.ident.0.clone(), StructField {
                    vis: f.vis,
                    ident: f.ident.1.clone(),
                    ty: Type::from(&f.ty),
                })).collect(),
                generic_args: st.args.clone(),
                constraints: vec![], // TODO parse and 
            }));
        }
        
        for i in m.imports {
            s.items.insert(i.path.segments.last().unwrap().0.clone(), ModuleItem::Import(i.path.clone()));
        }
        
        for st in m.statics {
            let mut allow_init = true;
            let mut allow_mut = true;
            let attrs = st.attrs.iter().map(|a| &a.0).collect::<Vec<_>>();
            for a in attrs {
                match a {
                    Attribute::Entrypoint(Entrypoint::Compute(_, _1, _2)) => {
                        panic!("compute attribute is only valid for functions")
                    },
                    Attribute::Push(_) => {
                        allow_init = false;
                        allow_mut = false;
                    },
                    Attribute::Set(_) => {
                        allow_init = false;
                        allow_mut = false;
                    },
                    Attribute::Binding(_) => {
                        allow_init = false;
                        allow_mut = false;
                    },
                    Attribute::Builtin(_) => {
                        allow_init = false;
                        allow_mut = false;
                    },
                    _ => {}
                }
            }
            // TODO check initializer when initializers are added for statics
            if ! allow_mut && st.mutability == Mutability::Mutable {
                panic!("Mutability not allowed by attributes");
            }
            let mut ty = Type::from(&st.ty);
            if ! allow_init {
                ty.uni = Some(Uniformity::Uniform);
            }
            s.items.insert(st.ident, ModuleItem::Static(Static {
                attrs: st.attrs.clone(),
                mutability: st.mutability,
                ty,
                value: None,
            }));
        }
        
        for c in m.constants {
            let attrs = c.attrs.iter().map(|a| &a.0).collect::<Vec<_>>();
            for a in attrs {
                match a {
                    Attribute::Entrypoint(Entrypoint::Compute(_, _1, _2)) => {
                        panic!("compute attribute is only valid for functions")
                    },
                    _ => {}
                }
            }
            s.items.insert(c.ident, ModuleItem::Constant(Constant {
                attrs: c.attrs.clone(),
                ty: Type::from(&c.ty),
                init: c.init,
                expr_types: vec![]
            }));
        }
        
        for f in m.functions {
            let attrs = f.attrs.iter().map(|a| &a.0).collect::<Vec<_>>();
            s.items.insert(f.ident.0, ModuleItem::Function(Function {
                attrs: f.attrs,
                uni: f.uni.0,
                safety: f.safety,
                params: f.params.iter().map(|p| (p.0.clone(), Type::from(&p.1))).collect(),
                ret: Type::from(&f.ret),
                block: f.block,
                expr_types: RefCell::new(vec![]),
                local_types: RefCell::new(HashMap::new()),
            }));
        }
        
        return s;
    }
    
    fn lookup_segments(&self, path: &[(Ident, SourceSpan, Vec<GenericArg>)]) -> Option<&ModuleItem> {
        if path.len() == 0 {
            return None;
        }
        if path.len() == 1 {
            //println!("looking up {}", path[0].0.str);
            return self.items.get(&path[0].0);
        }
        match self.items.get(&path[0].0) {
            Some(i) => {
                match i {
                    ModuleItem::Module(scope) => scope.lookup_segments(&path[1..]),
                    ModuleItem::Struct(st) => {
                        todo!("resolve associated functions")
                    }
                    _ => None
                }
            },
            None => None,
        }
        //return self..get(&path[0].0).and_then(|v| v.1.lookup_segments(&path[1..]));
    }
    
    
    pub fn lookup_path(&self, path: &ItemPath) -> Option<&ModuleItem> {
        let mut path = path.clone();
        let mut res;
        loop {
            res = self.lookup_segments(&path.segments);
            if let Some(r) = &res {
                match r {
                    ModuleItem::Import(item_path) => {
                        path = item_path.clone();
                    },
                    _ => break
                }
            } else {
                break;
            }
        }
        return res;
    }
    
    pub fn lookup_type(&self, path: &ItemPath) -> TypeVariant {
        match self.lookup_path(path).expect(&format!("Path not found: {}", path)) {
            ModuleItem::Struct(_) => TypeVariant::Struct(path.clone()),
            ModuleItem::Type(t) => t.ty.clone(),
            ModuleItem::Primitive(primitive) => TypeVariant::Primitive(*primitive),
            _ => panic!("Module item isn't a type")
        }
    }
    
    pub fn scopes(&self) -> impl Iterator<Item = (&Ident, &Scope)> {
        self.items.iter().filter_map(|i| {
            match i {
                (id, ModuleItem::Module(s)) => Some((id, s)),
                _ => None,
            }
        })
    }
    
    pub fn scopes_mut(&mut self) -> impl Iterator<Item = (&Ident, &mut Scope)> {
        self.items.iter_mut().filter_map(|i| {
            match i {
                (id, ModuleItem::Module(s)) => Some((id, s)),
                _ => None,
            }
        })
    }
    
}




impl TypeVariant {
    pub fn is_vector(&self) -> Option<&Vector> {
        match &self {
            TypeVariant::Vector(vector) => Some(vector),
            _ => None,
        }
    }
    
    pub fn is_matrix(&self) -> Option<&Matrix> {
        match &self {
            TypeVariant::Matrix(matrix) => Some(matrix),
            _ => None,
        }
    }
    
    pub fn is_unit(&self) -> bool {
        match &self {
            TypeVariant::Unit => true,
            _ => false,
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeVariant {
    Pointer(Pointer),
    Reference(Reference),
    Tuple(Vec<ItemPath>),
    Item(ItemPath),
    Struct(ItemPath),
    Primitive(Primitive),
    Function(ItemPath),
    /// Should be resolved to a proper type in type inference
    AbstractInt,
    /// Should be resolved to a proper type in type inference
    AbstractFloat,
    Vector(Vector),
    Matrix(Matrix),
    Unit,
    /// Type not inferred yet/could not be inferred
    Error,
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Vector {
    pub components: u8,
    pub ty: Primitive
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Matrix {
    pub rows: u8,
    pub cols: u8,
    pub ty: Primitive
}

#[derive(Debug, Clone)]
pub struct Type {
    pub uni: Option<Uniformity>,
    pub ty: TypeVariant,
    pub span: SourceSpan,
}

impl Hash for Type {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uni.hash(state);
        self.ty.hash(state);
    }
}

impl Eq for Type {}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        self.uni == other.uni && self.ty == other.ty
    }
}

impl From<&ast::ty::Type> for Type {
    fn from(v: &ast::ty::Type) -> Self {
        let span = v.span();
        match v {
            ast::ty::Type::Item(uni, item_path) => Self {
                uni: uni.as_ref().map(|u| u.0.clone()),
                ty: TypeVariant::Item(item_path.clone()),
                span,
            },
            ast::ty::Type::Reference(reference) => Self {
                uni: reference.uni.as_ref().map(|u| u.0.clone()),
                ty: TypeVariant::Reference(Reference {
                    mutable: reference.mutable,
                    storage_class: reference.storage,
                    ty: Box::new(Self::from(&*reference.ty)),
                }),
                span,
            },
            ast::ty::Type::Pointer(pointer) => {
                Self {
                    uni: pointer.uni.clone().and_then(|v| Some(v.0)),
                    ty: TypeVariant::Pointer(Pointer {
                        mutable: pointer.mutable,
                        storage_class: pointer.storage,
                        ty: Box::new(Self::from(&*pointer.ty)),
                    }),
                    span,
                }
            },
            ast::ty::Type::SelfType(source_span) => todo!(),
            ast::ty::Type::Unit(source_span) => Self {
                uni: Some(Uniformity::Uniform),
                ty: TypeVariant::Unit,
                span,
            },
            ast::ty::Type::Tuple(tuple) => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Static {
    pub attrs: Vec<(Attribute, SourceSpan)>,
    pub mutability: Mutability,
    pub ty: Type,
    pub value: Option<(Expression, Vec<Type>)>,
}

#[derive(Debug, Clone)]
pub struct Constant {
    pub attrs: Vec<(Attribute, SourceSpan)>,
    pub ty: Type,
    pub init: Expression,
    pub expr_types: Vec<Type>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub attrs: Vec<(Attribute, SourceSpan)>,
    pub uni: Uniformity,
    pub safety: Safety,
    pub params: Vec<((Ident, SourceSpan), Type)>,
    pub ret: Type,
    pub block: Block,
    pub expr_types: RefCell<Vec<Type>>,
    pub local_types: RefCell<HashMap<Ident, Type>>,
}


#[derive(Debug, Clone)]
pub struct Struct {
    pub attrs: Vec<(Attribute, SourceSpan)>,
    pub generic_args: Vec<GenericArgDefinition>,
    pub constraints: Vec<GenericArgConstraint>,
    pub fields: HashMap<Ident, StructField>,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub vis: Visibility,
    pub ident: SourceSpan,
    pub ty: Type,
}


#[derive(Debug, Clone)]
pub enum GenericArgConstraint {
    /// A outlives B
    Outlives(Ident, Ident),
    /// A implements trait B
    Implements(Ident, ItemPath),
    /// A is the minimum uniformity of B and C
    UniformityMin(Ident, Ident, Ident),
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Primitive {
    U8, U16, U32, U64,
    I8, I16, I32, I64,
    F16, F32, F64,
}

impl Primitive {
    pub fn is_int(&self) -> bool {
        match self {
            Primitive::U8 => true,
            Primitive::U16 => true,
            Primitive::U32 => true,
            Primitive::U64 => true,
            Primitive::I8 => true,
            Primitive::I16 => true,
            Primitive::I32 => true,
            Primitive::I64 => true,
            _ => false
        }
    }
    
    pub fn is_sint(&self) -> bool {
        match self {
            Primitive::I8 => true,
            Primitive::I16 => true,
            Primitive::I32 => true,
            Primitive::I64 => true,
            _ => false
        }
    }
    
    pub fn is_uint(&self) -> bool {
        match self {
            Primitive::U8 => true,
            Primitive::U16 => true,
            Primitive::U32 => true,
            Primitive::U64 => true,
            _ => false
        }
    }
    
    pub fn size(&self) -> u8 {
        match self {
            Primitive::U8 => 1,
            Primitive::U16 => 2,
            Primitive::U32 => 4,
            Primitive::U64 => 8,
            Primitive::I8 => 1,
            Primitive::I16 => 2,
            Primitive::I32 => 4,
            Primitive::I64 => 8,
            Primitive::F16 => 2,
            Primitive::F32 => 4,
            Primitive::F64 => 8,
        }
    }
    
    pub fn is_float(&self) -> bool {
        match self {
            Primitive::F16 => true,
            Primitive::F32 => true,
            Primitive::F64 => true,
            _ => false
        }
    }
    
    
}

impl Display for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Primitive::U8 => f.write_str("u8"),
            Primitive::U16 => f.write_str("u16"),
            Primitive::U32 => f.write_str("u32"),
            Primitive::U64 => f.write_str("u64"),
            Primitive::I8 => f.write_str("i8"),
            Primitive::I16 => f.write_str("i16"),
            Primitive::I32 => f.write_str("i32"),
            Primitive::I64 => f.write_str("i64"),
            Primitive::F16 => f.write_str("f16"),
            Primitive::F32 => f.write_str("f32"),
            Primitive::F64 => f.write_str("f64"),
        }
    }
}

#[derive(Debug, Clone, Hash)]
pub struct Reference {
    pub mutable: Mutability,
    pub storage_class: Option<StorageClass>,
    pub ty: Box<Type>,
}

impl Eq for Reference {}

impl PartialEq for Reference {
    fn eq(&self, other: &Self) -> bool {
        self.mutable == other.mutable && self.storage_class == other.storage_class && self.ty == other.ty
    }
}

#[derive(Debug, Clone, Hash)]
pub struct Pointer {
    pub mutable: Mutability,
    pub storage_class: Option<StorageClass>,
    pub ty: Box<Type>,
}

impl Eq for Pointer {}

impl PartialEq for Pointer {
    fn eq(&self, other: &Self) -> bool {
        self.mutable == other.mutable && self.storage_class == other.storage_class && self.ty == other.ty
    }
}




