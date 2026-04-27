use std::{collections::HashMap, fmt::Pointer, usize};

use bitflags::bitflags;
use rsl_data::internal::{CompilerData, InternedString, Mutability, StorageClass, StringTable, ir::{Function, GlobalItem, IRBlock, IRID, Primitive, SymbolID, SymbolTable, Type}};
use thiserror::Error;


#[derive(Debug, Clone)]
enum TCType {
    Struct(SymbolID),
    // Function has to be a separate variant because unification doesn't get the symbol table to look up stuff.
    Function {
        sym: SymbolID,
        params: Vec<Type>,
        ret: Type,
    },
    Number,
    Primitive(Primitive),
    Vector {
        components: Option<u8>,
        ty: Box<TCType>,
    },
    Float,
    Int,
    UInt,
    SInt,
    /// Must resolve to a vector or a struct, used for struct property vs vec swizzle differentiation
    VecOrStruct,
    /// Anything indexable really, but there are no operator overload traits yet, so pointers, matrices and arrays.
    Indexable,
    Matrix {
        rows: Option<u8>,
        cols: Option<u8>,
        ty: Box<TCType>,
    },
    Array {
        length: usize,
        ty: Box<TCType>,
    },
    RuntimeArray {
        ty: Box<TCType>,
    },
    Pointer {
        class: StorageClass,
        ty: Box<TCType>,
        mutability: Option<Mutability>,
    },
    Reference {
        class: StorageClass,
        ty: Box<TCType>,
        mutability: Option<Mutability>,
    },
    Unknown,
    /// Points to another type variable
    TypeVariable(TypeID),
}


#[derive(Error, Debug)]
enum TCError {
    #[error("Unknown error: {0}")]
    Unknown(String),
    
}


impl TCType {
    
    fn canonicalize(&self, table: &Vec<TCType>) -> TCType {
        match self {
            TCType::Struct(symbol_id) => todo!(),
            TCType::Function { sym, params , ret} => self.clone(),
            TCType::Number => TCType::Primitive(Primitive::I32),
            TCType::Primitive(primitive) => self.clone(),
            TCType::Vector { components, ty } => TCType::Vector { components: *components, ty: Box::new(ty.canonicalize(table)) },
            TCType::Float => TCType::Primitive(Primitive::F32),
            TCType::Int => TCType::Primitive(Primitive::I32),
            TCType::UInt => TCType::Primitive(Primitive::U32),
            TCType::SInt => TCType::Primitive(Primitive::I32),
            TCType::VecOrStruct => todo!(),
            TCType::Indexable => todo!(),
            TCType::Matrix { rows, cols, ty } => todo!(),
            TCType::Array { length, ty } => todo!(),
            TCType::RuntimeArray { ty } => todo!(),
            TCType::Pointer { class, ty, mutability } => TCType::Pointer { class: *class, ty: Box::new(ty.canonicalize(table)), mutability: *mutability },
            TCType::Reference { class, ty, mutability } => todo!(),
            TCType::Unknown => self.clone(),
            TCType::TypeVariable(type_id) => table[type_id.0 as usize].canonicalize(table),
        }
    }
    
    fn to_ir_type(&self) -> Type {
        match self {
            TCType::Struct(symbol_id) => todo!(),
            TCType::Function { sym, params , ret} => Type::Function { sym: *sym },
            TCType::Number => todo!(),
            TCType::Primitive(primitive) => Type::Primitive(*primitive),
            TCType::Vector { components, ty } => {
                match &**ty {
                    TCType::Primitive(primitive) => Type::Vector { components: components.unwrap(), ty: *primitive },
                    _ => todo!()
                }
            },
            TCType::Float => todo!(),
            TCType::Int => todo!(),
            TCType::UInt => todo!(),
            TCType::SInt => todo!(),
            TCType::VecOrStruct => todo!(),
            TCType::Indexable => todo!(),
            TCType::Matrix { rows, cols, ty } => todo!(),
            TCType::Array { length, ty } => todo!(),
            TCType::RuntimeArray { ty } => todo!(),
            // TODO mutability
            TCType::Pointer { class, ty, mutability } => Type::Pointer { class: *class, ty: Box::new(ty.to_ir_type()), mutability: Mutability::Mutable },
            TCType::Reference { class, ty, mutability } => todo!(),
            TCType::Unknown => Type::Primitive(Primitive::Unit),
            TCType::TypeVariable(type_id) => todo!(),
        }
    }
    
    fn unify_var_val(data: &mut TCData, var: TypeID, val: &TCType) -> Result<TCType, TCError> {
        let t = data.type_vars[var.0 as usize].clone();
        data.type_vars[var.0 as usize] = Self::unify_val_val(data, &t, val)?;
        return Ok(TCType::TypeVariable(var));
    }
    
    fn unify_var_var(data: &mut TCData, v1: TypeID, v2: TypeID) -> Result<TCType, TCError> {
        let t1 = data.type_vars[v1.0 as usize].clone();
        let t2 = data.type_vars[v2.0 as usize].clone();
        let t = Self::unify_val_val(data, &t1, &t2)?;
        data.type_vars[v1.0 as usize] = t.clone();
        data.type_vars[v2.0 as usize] = TCType::TypeVariable(v1);
        return Ok(TCType::TypeVariable(v1));
    }
    
    fn unify_val_val(data: &mut TCData, v1: &TCType, v2: &TCType) -> Result<TCType, TCError> {
        Ok(match (v1, v2) {
            (TCType::TypeVariable(v1), TCType::TypeVariable(v2)) => Self::unify_var_var(data, *v1, *v2)?,
            
            
            (TCType::TypeVariable(v), o) => Self::unify_var_val(data, *v, o)?,
            (o, TCType::TypeVariable(v)) => Self::unify_var_val(data, *v, o)?,
            
            (TCType::Unknown, o) => o.clone(),
            (o, TCType::Unknown) => o.clone(),
            
            
            (TCType::VecOrStruct, TCType::Vector { components, ty }) => v2.clone(),
            (TCType::Vector { components, ty }, TCType::VecOrStruct) => Self::unify_val_val(data, v2, v1)?,
            
            (TCType::Indexable, TCType::Pointer { class, ty, mutability }) => v2.clone(),
            (TCType::Pointer { class, ty, mutability }, TCType::Indexable) => Self::unify_val_val(data, v2, v1)?,
            
            (TCType::Pointer { class: c1, ty: ty1, mutability: m1 }, TCType::Pointer { class: c2, ty: ty2, mutability: m2 }) => {
                let mut c = *c1;
                'a: {
                    if c1 != c2 {
                        let one_logical = *c1 == StorageClass::Logical || *c2 == StorageClass::Logical;
                        if one_logical {
                            if *c1 != StorageClass::Logical {
                                c = *c1;
                            } else {
                                c = *c2;
                            }
                            break 'a;
                        } 
                        return Err(TCError::Unknown("Pointer storage classes don't match".to_string()));
                    }
                }
                // TODO mutability
                let t = Self::unify_val_val(data, ty1, ty2)?;
                TCType::Pointer { class: c, ty: Box::new(t), mutability: *m1 }
            }
            
            
            (TCType::Primitive(p1), TCType::Primitive(p2)) => {
                if *p1 == *p2 {
                    v1.clone()
                } else {
                    return Err(TCError::Unknown(format!("Incompatible primitives: {:#?}, {:#?}", *p1, *p2)));
                }
            }
            
            
            (TCType::Number, TCType::Number) => v1.clone(),
            (TCType::Float, TCType::Float) => v1.clone(),
            (TCType::Int, TCType::Int) => v1.clone(),
            (TCType::SInt, TCType::SInt) => v1.clone(),
            (TCType::UInt, TCType::UInt) => v1.clone(),
            
            
            (TCType::Number, TCType::Primitive(p)) => {
                if p.is_number() {
                    v2.clone()
                } else {
                    return Err(TCError::Unknown(format!("Incompatible primitives wit number: {:#?}", *p)));
                }
            },
            (TCType::Primitive(p), TCType::Number) => Self::unify_val_val(data, v2, v1)?,
            
            (TCType::Int, TCType::Primitive(p)) => {
                if p.is_int() {
                    v2.clone()
                } else {
                    if p.is_float() {
                        v2.clone()
                    } else {
                        return Err(TCError::Unknown(format!("Incompatible primitive with int: {:#?}", *p)));
                    }
                }
            },
            (TCType::Primitive(p), TCType::Int) => Self::unify_val_val(data, v2, v1)?,
            
            (TCType::UInt, TCType::Primitive(p)) => {
                if p.is_uint() {
                    v2.clone()
                } else {
                    if p.is_float() {
                        v2.clone()
                    } else {
                        return Err(TCError::Unknown(format!("Incompatible primitive with uint: {:#?}", *p)));
                    }
                }
            },
            (TCType::Primitive(p), TCType::UInt) => Self::unify_val_val(data, v2, v1)?,
            
            (TCType::SInt, TCType::Primitive(p)) => {
                if p.is_sint() {
                    v2.clone()
                } else {
                    if p.is_float() {
                        v2.clone()
                    } else {
                        return Err(TCError::Unknown(format!("Incompatible primitive with sint: {:#?}", *p)));
                    }
                }
            },
            (TCType::Primitive(p), TCType::SInt) => Self::unify_val_val(data, v2, v1)?,
            
            (TCType::Float, TCType::Primitive(p)) => {
                if p.is_float() {
                    v2.clone()
                } else {
                    return Err(TCError::Unknown(format!("Incompatible primitive with float: {:#?}", *p)));
                }
            },
            (TCType::Primitive(p), TCType::Float) => Self::unify_val_val(data, v2, v1)?,
            
            
            (TCType::Number, TCType::Int) => v2.clone(),
            (TCType::Int, TCType::Number) => Self::unify_val_val(data, v2, v1)?,
            
            (TCType::Number, TCType::Float) => v2.clone(),
            (TCType::Float, TCType::Number) => Self::unify_val_val(data, v2, v1)?,
            
            (TCType::Number, TCType::UInt) => v2.clone(),
            (TCType::UInt, TCType::Number) => Self::unify_val_val(data, v2, v1)?,
            
            (TCType::Number, TCType::SInt) => v2.clone(),
            (TCType::SInt, TCType::Number) => Self::unify_val_val(data, v2, v1)?,
            
            (TCType::Int, TCType::UInt) => v2.clone(),
            (TCType::UInt, TCType::Int) => Self::unify_val_val(data, v2, v1)?,
            
            (TCType::Int, TCType::SInt) => v2.clone(),
            (TCType::SInt, TCType::Int) => Self::unify_val_val(data, v2, v1)?,
            
            
            
            
            _ => {
                todo!("Unify of {:?} and {:?}", v1, v2);
            }
        })
    }
    
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct TypeID(u32);




#[derive(Debug)]
enum TypeConstraint {
    Number(TypeID),
    Float(TypeID),
    Int(TypeID),
    Sint(TypeID),
    Uint(TypeID),
    Primitive(TypeID),
    Generic(TypeID, SymbolID, Vec<TypeID>),
    Concrete(TypeID, SymbolID),
    Implements {
        ty: TypeID,
        trait_id: SymbolID,
        params: Vec<TypeID>,
        outputs: Vec<TypeID>,
    },
    // This can be solved via unification
    Same(TypeID, TypeID),
    PointerOf(TypeID, TypeID),
    Pointer(TypeID),
    SamePointerMeta(TypeID, TypeID),
    ImmRefOf(TypeID, TypeID),
    MutRefOf(TypeID, TypeID),
    VecOrStruct(TypeID),
    MemberOf(TypeID, TypeID, InternedString),
    Function(TypeID),
}



struct TCData<'a> {
    symbols: &'a SymbolTable,
    strings: &'a StringTable,
    type_table: &'a mut HashMap<IRID, Type>,
    type_vars: Vec<TCType>,
    reverse_type_table: HashMap<TypeID, IRID>,
    tc_type_table: HashMap<IRID, TypeID>,
    constraints: Vec<TypeConstraint>,
    ret_type: TypeID,
}

impl<'a> TCData<'a> {
    
    fn new_ir_type(&mut self, id: IRID, ty: TCType) -> TypeID {
        let tid = TypeID(self.type_vars.len() as u32);
        self.type_vars.push(ty);
        self.reverse_type_table.insert(tid, id);
        self.tc_type_table.insert(*&id, tid);
        tid
    }
    
    fn new_free_type(&mut self, ty: TCType) -> TypeID {
        let tid = TypeID(self.type_vars.len() as u32);
        self.type_vars.push(ty);
        tid
    }
    
}

pub fn type_checking(symbols: &SymbolTable, strings: &StringTable, function: &Function) {
    let ir = function.blocks.borrow_mut();
    let mut table = function.types.borrow_mut();
    let ret_type = to_tctype(symbols, &function.ret.borrow().0);
    let mut d = TCData {
        symbols,
        strings,
        type_table: &mut* table,
        reverse_type_table: HashMap::new(),
        tc_type_table: HashMap::new(),
        type_vars: vec![],
        constraints: vec![],
        ret_type: TypeID(0),
    };
    d.ret_type = d.new_free_type(ret_type);
    for i in 0..function.num_params {
        d.new_ir_type(IRID(i), to_tctype(d.symbols, &d.type_table[&IRID(i)]));
    }
    for b in ir.iter() {
        type_check_block(&mut d, b, symbols);
    }
    //println!("{:#?}", ir);
    //println!("{:#?}", d.reverse_type_table);
    let mut c = d.constraints;
    d.constraints = vec![];
    c.retain(|c| {
        match c {
            TypeConstraint::Same(a, b) => {
                //println!("Unifying variables {}, and {}", a.0, b.0);
                TCType::unify_var_var(&mut d, *a, *b).unwrap();
                return false;
            },
            TypeConstraint::Number(a) => {
                TCType::unify_var_val(&mut d, *a, &TCType::Number).unwrap();
                return false;
            },
            TypeConstraint::Int(a) => {
                TCType::unify_var_val(&mut d, *a, &TCType::Int).unwrap();
                return false;
            },
            TypeConstraint::Sint(a) => {
                TCType::unify_var_val(&mut d, *a, &TCType::SInt).unwrap();
                return false;
            },
            TypeConstraint::Uint(a) => {
                //println!("Unifying variable {} with UInt", a.0);
                TCType::unify_var_val(&mut d, *a, &TCType::UInt).unwrap();
                return false;
            },
            TypeConstraint::VecOrStruct(t) => {
                //println!("Unifying variable {} with VecOrStruct", t.0);
                TCType::unify_var_val(&mut d, *t, &TCType::VecOrStruct).unwrap();
                return false;
            },
            TypeConstraint::MemberOf(st, mt, mn) => {
                // TODO use canonical name, maybe resolve these later and until fixpoint
                // because later constraints could give info on what kind of struct/vector the base is
                match d.type_vars[st.0 as usize].clone() {
                    TCType::Vector { components, ty } => {
                        TCType::unify_var_val(&mut d, *mt, &TCType::Pointer { class: StorageClass::Logical, ty, mutability: None }).unwrap();
                    }
                    _ => todo!(),
                }
                return false;
            },
            TypeConstraint::Pointer(t) => {
                TCType::unify_var_val(&mut d, *t, &TCType::Pointer { class: StorageClass::Logical, ty: Box::new(TCType::Unknown), mutability: None }).unwrap();
                return false;
            },
            TypeConstraint::PointerOf(p, v) => {
                //println!("Solving PointerOf {} with type {}, IDs {}, and {}", p.0, v.0, d.reverse_type_table.get(p).cloned().unwrap_or(IRID(usize::MAX)).0, d.reverse_type_table.get(v).cloned().unwrap_or(IRID(usize::MAX)).0);
                TCType::unify_var_val(&mut d, *p, &TCType::Pointer { class: StorageClass::Logical, ty: Box::new(TCType::TypeVariable(*v)), mutability: None }).unwrap();
                return false;
            },
            TypeConstraint::SamePointerMeta(p1, p2) => {
                let pt1 = match &d.type_vars[p1.0 as usize] {
                    TCType::Pointer { class, ty, mutability } => (*class, *mutability),
                    _ => unreachable!(),
                };
                let pt2 = match &d.type_vars[p2.0 as usize] {
                    TCType::Pointer { class, ty, mutability } => (*class, *mutability),
                    _ => unreachable!(),
                };
                if pt1.0 != StorageClass::Logical && pt2.0 != StorageClass::Logical && pt1.0 != pt2.0 {
                    panic!("Incompatible pointer storage classes")
                }
                let c = if pt1.0 != StorageClass::Logical {
                    pt1.0
                } else {
                    pt2.0
                };
                if pt1.1.is_some_and(|m1| pt2.1.is_some_and(|m2| m1 != m2)) {
                    panic!("Incompatible pointer mutabilities")
                }
                let mut m = None;
                if pt1.1.is_some() {
                    m = pt1.1;
                }
                if pt2.1.is_some() {
                    m = pt2.1;
                }
                match &mut d.type_vars[p1.0 as usize] {
                    TCType::Pointer { class, ty, mutability } => {
                        *class = c;
                        *mutability = m;
                    },
                    _ => unreachable!(),
                }
                match &mut d.type_vars[p2.0 as usize] {
                    TCType::Pointer { class, ty, mutability } => {
                        *class = c;
                        *mutability = m;
                    },
                    _ => unreachable!(),
                }
                return false;
            }
            _ => {
                return true;
            }
        }
    });
    if ! c.is_empty() {
        panic!("unimplemented type constraints")
    }
    
    // println!("{:#?}", c);
    // for (i, v) in d.type_vars.iter().enumerate() {
    //     println!("Type var {} for SSA {}: {:#?}", i, d.reverse_type_table.get(&TypeID(i as u32)).cloned().unwrap_or(IRID(usize::MAX)).0, v);
    // }
    for (id, ty) in &d.tc_type_table {
        let t = &d.type_vars[ty.0 as usize];
        d.type_table.insert(*id, t.canonicalize(&d.type_vars).to_ir_type());
    }
    
}


fn lookup_type(symbols: &SymbolTable, ty: SymbolID) -> TCType {
    match &symbols.get(ty).1 {
        GlobalItem::Type(ty) => {
            to_tctype(symbols, &ty)
        }
        _ => {todo!()}
    }
}

fn to_tctype(symbols: &SymbolTable, ty: &Type) -> TCType {
    match ty {
        Type::Unresolved(item_path) => panic!("Unresolved path: {:#?}", item_path),
        Type::Resolved(symbol_id) => lookup_type(symbols, *symbol_id),
        Type::Primitive(primitive) => TCType::Primitive(*primitive),
        Type::Vector { components, ty } => TCType::Vector { components: Some(*components), ty: Box::new(TCType::Primitive(*ty)) },
        Type::Matrix { rows, cols, ty } => todo!(),
        Type::Array { length, ty } => todo!(),
        Type::UnresolvedArray { length, ty } => todo!(),
        Type::RuntimeArray { ty } => todo!(),
        Type::Pointer { class, ty, mutability } => TCType::Pointer { class: *class, ty: Box::new(to_tctype(symbols, &*ty)), mutability: Some(*mutability) },
        Type::Reference { class, ty, mutability } => todo!(),
        Type::Function { sym } => to_tctype(symbols, &Type::Resolved(*sym)),
    }
}


fn type_check_block(data: &mut TCData, block: &IRBlock, symbols: &SymbolTable) {
    for inst in &block.instructions {
        match inst {
            rsl_data::internal::ir::IRInstruction::Ident { name, token, global, id } => todo!(),
            rsl_data::internal::ir::IRInstruction::ResolvedPath { path, tokens, id } => {
                let s = symbols.get(*path);
                match &s.1 {
                    rsl_data::internal::ir::GlobalItem::StructTemplate { args, constraints } => todo!(),
                    rsl_data::internal::ir::GlobalItem::Struct { attrs, ident_token } => todo!(),
                    rsl_data::internal::ir::GlobalItem::Trait { attrs, ident_token, args, constraints, types, functions } => todo!(),
                    rsl_data::internal::ir::GlobalItem::Static { attrs, ident_token, uni, ty } => {
                        //println!("{:#?}", to_tctype(symbols, ty));
                        let tct = TCType::Pointer { class: StorageClass::Logical, ty: Box::new(to_tctype(symbols, ty)), mutability: None };
                        data.new_ir_type(*id, tct);
                    },
                    rsl_data::internal::ir::GlobalItem::FunctionTemplate { args, constraints } => todo!(),
                    rsl_data::internal::ir::GlobalItem::Function(function) => {
                        let fty = function.types.borrow();
                        data.new_ir_type(*id, TCType::Function { sym: *path, params: (0..function.num_params).map(|i| fty[&IRID(i)].clone()).collect(), ret: function.ret.borrow().0.clone()});
                    },
                    rsl_data::internal::ir::GlobalItem::Import { path, span } => todo!(),
                    rsl_data::internal::ir::GlobalItem::ResolvedImport { path, id } => todo!(),
                    rsl_data::internal::ir::GlobalItem::Placeholder => todo!(),
                    rsl_data::internal::ir::GlobalItem::Type(ty) => {
                        todo!()
                    },
                    rsl_data::internal::ir::GlobalItem::Module(symbol_table) => todo!(),
                    rsl_data::internal::ir::GlobalItem::RemovedModule => todo!(),
                    rsl_data::internal::ir::GlobalItem::Removed => todo!(),
                }
            },
            rsl_data::internal::ir::IRInstruction::Local { ident, ident_token, id, ty, uni, mutable } => {
                if let Some(ty) = ty {
                    data.new_ir_type(*id, TCType::Pointer{ class: StorageClass::Logical, ty: Box::new(to_tctype(symbols, ty)), mutability: None});
                } else {
                    data.new_ir_type(*id, TCType::Pointer{ class: StorageClass::Logical, ty: Box::new(TCType::Unknown), mutability: None});
                }
            },
            rsl_data::internal::ir::IRInstruction::UnOp { inp, op, out, span } => {
                match *op {
                    rsl_data::internal::ast::UnOp::Deref => {
                        todo!()
                    }
                    _ => todo!("{:#?}", op)
                }
            },
            rsl_data::internal::ir::IRInstruction::BinOp { lhs, op, rhs, out, span } => {
                let lhsty = data.tc_type_table[lhs];
                let rhsty = data.tc_type_table[rhs];
                let outty = data.new_ir_type(*out, TCType::Unknown);
                use rsl_data::internal::ast::BinOp;
                match *op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                        data.constraints.push(TypeConstraint::Number(outty));
                        data.constraints.push(TypeConstraint::Number(lhsty));
                        data.constraints.push(TypeConstraint::Number(rhsty));
                        data.constraints.push(TypeConstraint::Same(outty, lhsty));
                        data.constraints.push(TypeConstraint::Same(rhsty, lhsty));
                    },
                    BinOp::Mod => todo!(),
                    BinOp::BinAnd => todo!(),
                    BinOp::BinOr => todo!(),
                    BinOp::LogAnd | BinOp::LogOr => {
                        data.type_vars[outty.0 as usize] = TCType::Primitive(Primitive::Bool);
                        data.constraints.push(TypeConstraint::Same(outty, lhsty));
                        data.constraints.push(TypeConstraint::Same(rhsty, lhsty));
                    },
                    BinOp::BinXor => todo!(),
                    BinOp::Index => {
                        let mt = data.new_free_type(TCType::Unknown);
                        data.constraints.push(TypeConstraint::PointerOf(lhsty, mt));
                        data.constraints.push(TypeConstraint::PointerOf(outty, mt));
                        data.constraints.push(TypeConstraint::Uint(rhsty));
                        data.constraints.push(TypeConstraint::SamePointerMeta(lhsty, outty));
                    },
                    BinOp::Assign => unreachable!(),
                    BinOp::Equals | BinOp::NotEquals | BinOp::Less | BinOp::LessEquals | BinOp::Greater | BinOp::GreaterEquals => {
                        data.type_vars[outty.0 as usize] = TCType::Primitive(Primitive::Bool);
                        data.constraints.push(TypeConstraint::Same(rhsty, lhsty));
                    },
                }
            },
            rsl_data::internal::ir::IRInstruction::Unit { out } => {
                data.new_ir_type(*out, TCType::Primitive(Primitive::Unit));
            },
            rsl_data::internal::ir::IRInstruction::Load { ptr, out } => {
                let oty = data.new_ir_type(*out, TCType::Unknown);
                data.constraints.push(TypeConstraint::PointerOf(data.tc_type_table[ptr], oty));
            },
            rsl_data::internal::ir::IRInstruction::Store { ptr, value } => {
                let vty = data.new_ir_type(*value, TCType::Unknown);
                data.constraints.push(TypeConstraint::PointerOf(data.tc_type_table[ptr], vty));
            },
            rsl_data::internal::ir::IRInstruction::Property { inp, name, out } => {
                let ity = data.tc_type_table[inp];
                let pt = data.new_free_type(TCType::VecOrStruct);
                data.constraints.push(TypeConstraint::PointerOf(ity, pt));
                let mty = data.new_ir_type(*out, TCType::Unknown);
                data.constraints.push(TypeConstraint::MemberOf(pt, mty, name.0));
            },
            rsl_data::internal::ir::IRInstruction::Call { func, args, out, span } => {
                let (sym, params, ret) = match &data.type_vars[data.tc_type_table[func].0 as usize] {
                    TCType::Function { sym, params , ret} => (sym, params, ret),
                    _ => panic!("call id is not a function")
                };
                let args = args.clone();
                let params = params.clone();
                let ret = ret.clone();
                data.new_ir_type(*out, to_tctype(symbols, &ret));
                for (a, p) in args.iter().zip(params) {
                    let pt = data.new_free_type(to_tctype(symbols, &p));
                    data.constraints.push(TypeConstraint::Same(data.tc_type_table[a], pt));
                }
            },
            rsl_data::internal::ir::IRInstruction::Int { v, id, token_id, ty } => {
                data.new_ir_type(*id, TCType::Int);
            },
            rsl_data::internal::ir::IRInstruction::Float { v, id, token_id, ty } => {
                data.new_ir_type(*id, TCType::Float);
            },
            rsl_data::internal::ir::IRInstruction::Cast { inp, out, ty } => todo!(),
            rsl_data::internal::ir::IRInstruction::Spread { inp, out, uni } => todo!(),
            rsl_data::internal::ir::IRInstruction::ReturnValue { id, token_id } => {
                data.constraints.push(TypeConstraint::Same(data.tc_type_table[id], data.ret_type));
            },
            rsl_data::internal::ir::IRInstruction::Return { token_id } => {},
            rsl_data::internal::ir::IRInstruction::Loop { header, body, cont, merge, construct } => todo!(),
            rsl_data::internal::ir::IRInstruction::Branch { target_block } => {},
            rsl_data::internal::ir::IRInstruction::If { inp, true_target_block, false_target_block, merge, construct } => {
                let inty = data.tc_type_table[inp];
                let oty = data.new_free_type(TCType::Primitive(Primitive::Bool));
                data.constraints.push(TypeConstraint::Same(inty, oty));
            },
            rsl_data::internal::ir::IRInstruction::Phi { out, sources } => todo!(),
            rsl_data::internal::ir::IRInstruction::NOP => {},
        }
    }
}



#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::hint::black_box;
    use std::{path::PathBuf, time::Instant};

    use rsl_data::internal::SourceSpan;
    use rsl_data::internal::{ReportSourceCache, Sources, StringTable, ir::SymbolTable};
    use rsl_lexer::tokenize;
    use rsl_parser::parse_file;

    use super::*;

    #[test]
    fn simple() -> Result<(), ()> {
        let strings = StringTable::new();
        let code_template = "#[compute] fn test(a: *const u32, b: *const u32, c: *mut u32) { c[globalInvocationID.x] = a[globalInvocationID.x] + b[globalInvocationID.x]; }";
        let mut code = String::new();
        const N: usize = 1;
        for i in 0..N {
            code += &code_template.replace("test", &("test".to_string() + &i.to_string()));
        }
        let code = black_box(code);
        let mut cache = ReportSourceCache::new(&Sources {
            source_files: vec![PathBuf::from("test.rsl")],
            source_strings: vec![code.to_string()]
        });
        let t0 = Instant::now();
        let res = tokenize(&code, 0, &strings);
        match res {
            Ok((tokens, spans)) => {
                let spans = spans.iter().map(|r| SourceSpan {
                    file: 0,
                    start: r.start,
                    end: r.end,
                }).collect::<Vec<_>>();
                
                let (m, e) = parse_file(&tokens, &spans, 0, vec![], &strings);
                if ! e.is_empty() {
                    e.iter().for_each(|e| e.eprint(&mut cache).unwrap());
                    return Err(());
                }
                let m = SymbolTable::from_module(m, &strings);
                
                let core = SymbolTable::core(&strings);
                let mut toplevel = SymbolTable::new();
                toplevel.insert_module(core, strings.insert_get("core"));
                toplevel.insert_module(m, strings.insert_get("test"));
                
                toplevel.resolve_paths(&strings);
                //toplevel.eval_constexprs();
                
                
                let t1 = Instant::now();
                println!("Time: {} ms", (t1- t0).as_millis());
                for i in 0..N {
                    let f = match &toplevel.lookup(&strings.insert_get(&("::test::test".to_string() + &i.to_string()))).unwrap().1 {
                        GlobalItem::Function(function) => function,
                        _ => panic!()
                    };
                    type_checking(&toplevel, &strings, f);
                    if i == 0 {
                        println!("{:#?}", f);
                    }
                }
                
                let t2 = Instant::now();
                println!("Time: {} ms", (t2- t1).as_millis());
            },
            Err(r) => {
                r.print(cache).unwrap();
                return Err(());
            },
        }
        return Ok(());
        
        
        
        
        
    }
}
