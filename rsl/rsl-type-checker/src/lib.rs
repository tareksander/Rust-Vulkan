use std::{collections::HashMap, fmt::Pointer};

use bitflags::bitflags;
use rsl_data::internal::{CompilerData, InternedString, Mutability, StorageClass, StringTable, ir::{Function, GlobalItem, IRBlock, IRID, Primitive, SymbolID, SymbolTable, Type}};
use ena::unify::{InPlaceUnificationTable, NoError, UnifyKey, UnifyValue};


#[derive(Debug, Clone)]
enum TCType {
    Struct(SymbolID),
    // Function has to be a separate variant because unification doesn't get the symbol table to look up stuff.
    Function {
        sym: SymbolID,
        params: Vec<Type>,
    },
    Primitive(Primitive),
    Vector {
        components: Option<u8>,
        ty: Option<Primitive>,
    },
    /// Must resolve to a vector or a struct, used for struct property vs vec swizzle differentiation
    VecOrStruct,
    /// Anything indexable really, but there are no operator overload traits yet, so pointers, matrices and arrays.
    Indexable,
    Matrix {
        rows: Option<u8>,
        cols: Option<u8>,
        ty: Option<Primitive>,
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
}

impl UnifyValue for TCType {
    type Error = ();

    fn unify_values(v1: &Self, v2: &Self) -> Result<Self, Self::Error> {
        match (v1, v2) {
            // Unconstrained types trivially unify
            (TCType::Unknown, TCType::Unknown) => Ok(TCType::Unknown),
            
            // If one value is unknown, the defined one has precedence
            (TCType::Unknown, a) => Ok(a.clone()),
            (a, TCType::Unknown) => Ok(a.clone()),
            
            (TCType::VecOrStruct, TCType::Vector { components, ty }) => Ok(v2.clone()),
            (TCType::Vector { components, ty }, TCType::VecOrStruct) => Ok(v1.clone()),
            
            (TCType::VecOrStruct, TCType::Struct(_)) => Ok(v2.clone()),
            (TCType::Struct(_), TCType::VecOrStruct) => Ok(v1.clone()),
            
            
            // todo: indexable to concrete type.
            (TCType::Indexable, TCType::Pointer { class, ty, mutability }) => Ok(v2.clone()),
            (TCType::Pointer { class, ty, mutability }, TCType::Indexable) => Ok(v1.clone()),
            
            
            
            (TCType::Pointer { class: c1, ty: ty1, mutability: m1 }, TCType::Pointer { class: c2, ty: ty2, mutability: m2 }) => {
                let mut c = *c1;
                if *c1 != StorageClass::Logical && *c2 != StorageClass::Logical && *c1 != *c2 {
                    return Err(());
                }
                if c == StorageClass::Logical {
                    c = *c2;
                }
                let t = Self::unify_values(&**ty1, &**ty2)?;
                if *m1 != *m2 {
                    return Err(());
                }
                return Ok(TCType::Pointer { class: c, ty: Box::new(t), mutability: *m1 });
            }
            
            
            
            
            
            
            _ => Err(())
        }
    }
}





bitflags! {
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct LengthSet : u8 {
        const One = 1 << 1;
        const Two = 1 << 2;
        const Three = 1 << 3;
        const Four = 1 << 4;
    }
    
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct PrimitiveSet : u16 {
        const U8 = 1 << 0;
        const U16 = 1 << 1;
        const U32 = 1 << 2;
        const U64 = 1 << 3;
        const I8 = 1 << 4;
        const I16 = 1 << 5;
        const I32 = 1 << 6;
        const I64 = 1 << 7;
        const F16 = 1 << 8;
        const F32 = 1 << 9;
        const F64 = 1 << 10;
        const Bool = 1 << 11;
        const Unit = 1 << 12;
    }
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct UniformitySet :u8 {
        const Dispatch = 1 << 0;
        const Workgroup = 1 << 1;
        const Subgroup = 1 << 2;
        const Invocation = 1 << 3;
    }
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct MutabilitySet :u8 {
        const Immutable = 1 << 0;
        const Mutable = 1 << 1;
    }
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct StorageClassSet :u8 {
        const Function = 1 << 0;
        const Private = 1 << 1;
        const Workgroup = 1 << 2;
        const Storage = 1 << 3;
        const PhysicalStorage = 1 << 4;
        const Logical = 1 << 5;
    }
}



#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct TypeID(u32);


impl UnifyKey for TypeID {
    type Value = TCType;

    fn index(&self) -> u32 {
        self.0
    }

    fn from_index(u: u32) -> Self {
        Self(u)
    }

    fn tag() -> &'static str {
        "TypeID"
    }
}



#[derive(Debug)]
enum TypeConstraint {
    Number(TypeID),
    Float(TypeID),
    Int(TypeID),
    Sint(TypeID),
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
    ImmRefOf(TypeID, TypeID),
    MutRefOf(TypeID, TypeID),
    VecOrStruct(TypeID),
    MemberOf(TypeID, TypeID, InternedString),
}



struct TCData<'a> {
    symbols: &'a SymbolTable,
    strings: &'a StringTable,
    unify: InPlaceUnificationTable<TypeID>,
    type_table: &'a mut HashMap<IRID, Type>,
    unify_table: HashMap<TypeID, IRID>,
    reverse_unify_table: HashMap<IRID, TypeID>,
    constraints: Vec<TypeConstraint>,
}

impl<'a> TCData<'a> {
    
    fn new_ir_type(&mut self, id: IRID, ty: TCType) -> TypeID {
        let tid = self.unify.new_key(ty);
        self.unify_table.insert(tid, id);
        self.reverse_unify_table.insert(id, tid);
        tid
    }
    
}

pub fn type_checking(symbols: &SymbolTable, strings: &StringTable, function: &Function) {
    let ir = function.blocks.borrow_mut();
    let unify = InPlaceUnificationTable::<TypeID>::new();
    let mut table = function.types.borrow_mut();
    let unify_table = HashMap::with_capacity(function.next_id.borrow().0);
    let mut d = TCData {
        symbols,
        strings,
        unify,
        type_table: &mut* table,
        unify_table,
        reverse_unify_table: HashMap::new(),
        constraints: vec![],
    };
    for b in ir.iter() {
        type_check_block(&mut d, b, symbols);
    }
    d.constraints.retain(|c| {
        match c {
            TypeConstraint::Same(a, b) => {
                d.unify.unify_var_var(*a, *b).unwrap();
                return false;
            },
            _ => {
                return true;
            }
        }
    });
    println!("{:#?}", d.constraints);
    
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
        Type::Unresolved(item_path) => todo!(),
        Type::Resolved(symbol_id) => lookup_type(symbols, *symbol_id),
        Type::Primitive(primitive) => TCType::Primitive(*primitive),
        Type::Vector { components, ty } => TCType::Vector { components: Some(*components), ty: Some(*ty) },
        Type::Matrix { rows, cols, ty } => todo!(),
        Type::Array { length, ty } => todo!(),
        Type::UnresolvedArray { length, ty } => todo!(),
        Type::RuntimeArray { ty } => todo!(),
        Type::Pointer { class, ty, mutability } => TCType::Pointer { class: *class, ty: Box::new(to_tctype(symbols, &*ty)), mutability: Some(*mutability) },
        Type::Reference { class, ty, mutability } => todo!(),
    }
}


fn type_check_block(data: &mut TCData, block: &IRBlock, symbols: &SymbolTable) {
    for inst in &block.instructions {
        match inst {
            rsl_data::internal::ir::IRInstruction::Path { path, tokens, id, lvalue } => todo!(),
            rsl_data::internal::ir::IRInstruction::ResolvedPath { path, tokens, id, lvalue } => {
                let s = symbols.get(*path);
                match &s.1 {
                    rsl_data::internal::ir::GlobalItem::StructTemplate { args, constraints } => todo!(),
                    rsl_data::internal::ir::GlobalItem::Struct { attrs, ident_token } => todo!(),
                    rsl_data::internal::ir::GlobalItem::Trait { attrs, ident_token, args, constraints, types, functions } => todo!(),
                    rsl_data::internal::ir::GlobalItem::Static { attrs, ident_token, uni, ty } => {
                        data.new_ir_type(*id, to_tctype(symbols, ty));
                    },
                    rsl_data::internal::ir::GlobalItem::FunctionTemplate { args, constraints } => todo!(),
                    rsl_data::internal::ir::GlobalItem::Function(function) => todo!(),
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
                    data.new_ir_type(*id, to_tctype(symbols, ty));
                } else {
                    data.new_ir_type(*id, TCType::Unknown);
                }
            },
            rsl_data::internal::ir::IRInstruction::UnOp { inp, op, out, span } => todo!(),
            rsl_data::internal::ir::IRInstruction::BinOp { lhs, op, rhs, out, span } => {
                let lhsty = data.reverse_unify_table[lhs];
                let rhsty = data.reverse_unify_table[lhs];
                let outty = data.new_ir_type(*out, TCType::Unknown);
                match *op {
                    rsl_data::internal::ast::BinOp::Add => {
                        data.constraints.push(TypeConstraint::Number(outty));
                        data.constraints.push(TypeConstraint::Number(lhsty));
                        data.constraints.push(TypeConstraint::Number(rhsty));
                        data.constraints.push(TypeConstraint::Same(outty, lhsty));
                        data.constraints.push(TypeConstraint::Same(rhsty, lhsty));
                    },
                    rsl_data::internal::ast::BinOp::Sub => todo!(),
                    rsl_data::internal::ast::BinOp::Mul => todo!(),
                    rsl_data::internal::ast::BinOp::Div => todo!(),
                    rsl_data::internal::ast::BinOp::Mod => todo!(),
                    rsl_data::internal::ast::BinOp::BinAnd => todo!(),
                    rsl_data::internal::ast::BinOp::LogAnd => todo!(),
                    rsl_data::internal::ast::BinOp::BinOr => todo!(),
                    rsl_data::internal::ast::BinOp::LogOr => todo!(),
                    rsl_data::internal::ast::BinOp::BinXor => todo!(),
                    rsl_data::internal::ast::BinOp::Index => {
                        data.constraints.push(TypeConstraint::Pointer(lhsty));
                        data.constraints.push(TypeConstraint::Pointer(outty));
                        data.constraints.push(TypeConstraint::Int(rhsty));
                    },
                    rsl_data::internal::ast::BinOp::Assign => todo!(),
                    rsl_data::internal::ast::BinOp::Equals => todo!(),
                    rsl_data::internal::ast::BinOp::NotEquals => todo!(),
                    rsl_data::internal::ast::BinOp::Less => todo!(),
                    rsl_data::internal::ast::BinOp::LessEquals => todo!(),
                    rsl_data::internal::ast::BinOp::Greater => todo!(),
                    rsl_data::internal::ast::BinOp::GreaterEquals => todo!(),
                }
            },
            rsl_data::internal::ir::IRInstruction::Unit { out } => {
                data.new_ir_type(*out, TCType::Primitive(Primitive::Unit));
            },
            rsl_data::internal::ir::IRInstruction::Load { ptr, out } => {
                let oty = data.new_ir_type(*out, TCType::Unknown);
                data.constraints.push(TypeConstraint::PointerOf(data.reverse_unify_table[ptr], oty));
            },
            rsl_data::internal::ir::IRInstruction::Store { ptr, value } => {
                let vty = data.new_ir_type(*value, TCType::Unknown);
                data.constraints.push(TypeConstraint::PointerOf(data.reverse_unify_table[ptr], vty));
            },
            rsl_data::internal::ir::IRInstruction::Property { inp, name, out } => {
                let ity = data.reverse_unify_table[inp];
                data.constraints.push(TypeConstraint::VecOrStruct(ity));
                let mty = data.new_ir_type(*out, TCType::Unknown);
                data.constraints.push(TypeConstraint::MemberOf(ity, mty, name.0));
            },
            rsl_data::internal::ir::IRInstruction::Call { func, args, out, span } => todo!(),
            rsl_data::internal::ir::IRInstruction::Int { v, id, token_id, ty } => todo!(),
            rsl_data::internal::ir::IRInstruction::Float { v, id, token_id, ty } => todo!(),
            rsl_data::internal::ir::IRInstruction::Cast { inp, out, ty } => todo!(),
            rsl_data::internal::ir::IRInstruction::Spread { inp, out, uni } => todo!(),
            rsl_data::internal::ir::IRInstruction::ReturnValue { id, token_id } => todo!(),
            rsl_data::internal::ir::IRInstruction::Return { token_id } => {},
            rsl_data::internal::ir::IRInstruction::Loop { header, body, cont, merge, construct } => todo!(),
            rsl_data::internal::ir::IRInstruction::Branch { target_block } => {},
            rsl_data::internal::ir::IRInstruction::If { inp, true_target_block, false_target_block, merge, construct } => todo!(),
            rsl_data::internal::ir::IRInstruction::Phi { out, sources } => todo!(),
        }
    }
}



#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::{path::PathBuf, time::Instant};

    use rsl_data::internal::SourceSpan;
    use rsl_data::internal::{ReportSourceCache, Sources, StringTable, ir::SymbolTable};
    use rsl_lexer::tokenize;
    use rsl_parser::parse_file;

    use super::*;

    #[test]
    fn simple() -> Result<(), ()> {
        let strings = StringTable::new();
        let code = "#[compute] fn test(a: *const u32, b: *const u32, c: *mut u32) { c[globalInvocationID.x] = a[globalInvocationID.x] + b[globalInvocationID.x]; }";
        let mut cache = ReportSourceCache::new(&Sources {
            source_files: vec![PathBuf::from("test.rsl")],
            source_strings: vec![code.to_string()]
        });
        let res = tokenize(code, 0, &strings);
        match res {
            Ok((tokens, spans)) => {
                let spans = spans.iter().map(|r| SourceSpan {
                    file: 0,
                    start: r.start,
                    end: r.end,
                }).collect::<Vec<_>>();
                
                let t1 = Instant::now();
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
                
                
                
                type_checking(&toplevel, &strings, match &toplevel.lookup(&strings.insert_get("::test::test")).unwrap().1 {
                    GlobalItem::Function(function) => function,
                    _ => panic!()
                });
                
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
