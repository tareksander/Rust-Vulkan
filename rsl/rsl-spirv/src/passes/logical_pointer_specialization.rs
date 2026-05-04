use std::collections::HashMap;

use rsl_data::internal::{InternedString, StorageClass, StringTable, ir::{GlobalItem, IRID, IRInstruction, SymbolID, SymbolTable, Type}};


struct SpecializationRequest {
    sym: SymbolID,
    classes: Vec<Option<StorageClass>>,
    rename: bool,
}


struct Fixup {
    sym: SymbolID,
    block: usize,
    inst: usize,
    new_func: InternedString,
}


/// - Converts entrypoint parameters to physical pointers.
/// - Converts locals and non-entrypoint parameters to function pointers
/// - Converts builtins to input pointers
/// - Propagates the storage classes and specializes functions that are called if necessary
/// 
/// TODO:
/// - Specialize called functions with storage classes for parameter pointers
/// - Handle vertex and fragment shader IO storage classes
/// 
pub fn logical_pointer_specialization(sym: &mut SymbolTable, strings: &StringTable) {
    
    // TODO handle builtins in a generic way without traversing the whole table
    // Maybe keep the names in a list?
    
    let globalInvocationID = sym.lookup_id(&strings.insert_get("::core::globalInvocationID")).unwrap();
    //println!("{:#?}", globalInvocationID);
    
    let mut requests = vec![];
    
    for s in sym.iter() {
        let i = sym.get(s);
        if let GlobalItem::Function(f) = &i.1 {
            // TODO other entrypoints
            if f.attrs.contains(&rsl_data::internal::Attribute::Compute) {
                if f.num_params != 0 {
                    let mut types = f.types.borrow_mut();
                    let mut classes = Vec::with_capacity(f.num_params);
                    for i in 0..f.num_params {
                        match &mut types.get_mut(&IRID(i)).unwrap() {
                            Type::Pointer { class, ty, mutability } => {
                                classes.push(Some(StorageClass::PhysicalStorage));
                            },
                            _ => {
                                classes.push(None);
                            }
                        }
                    }
                    if classes.iter().any(|v| v.is_some()) {
                        requests.push(SpecializationRequest {
                            sym: s,
                            classes,
                            rename: false
                        });
                    }
                }
            }
        }
    }
    
    let mut fixups = vec![];
    
    fn specialized_name(sym: &SymbolTable, strings: &StringTable, s: SymbolID, classes: &[Option<StorageClass>]) -> InternedString {
        strings.insert_get(&("%".to_string() + &strings.lookup(sym.get_name(s)) + "%" + &classes.iter().filter_map(|v| if let Some(sc) = v {
            Some(sc.to_string().to_owned())
        } else {
            None
        }).reduce(|p, n| p + "&" + &n).unwrap()))
    }
    
    loop {
        let mut new_requests: Vec<SpecializationRequest> = vec![];
        
        for r in &requests {
            let s = if r.rename {
                let name = specialized_name(sym, strings, r.sym, &r.classes);
                let f = sym.get(r.sym).clone();
                sym.insert(name, f).unwrap();
                sym.lookup_id(&name).unwrap()
            } else {
                r.sym
            };
            let i = sym.get(s);
            if let GlobalItem::Function(f) = &i.1 {
                let mut storage_classes: HashMap<IRID, StorageClass> = HashMap::new();
                let mut blocks = f.blocks.borrow_mut();
                let mut types = f.types.borrow_mut();
                let mut funcs = HashMap::new();
                for (i, c) in r.classes.iter().enumerate() {
                    if let Some(c) = c {
                        storage_classes.insert(IRID(i), *c);
                        match &mut types.get_mut(&IRID(i)).unwrap() {
                            Type::Pointer { class, ty, mutability } => {
                                *class = *c;
                            },
                            _ => unreachable!()
                        }
                    }
                }
                
                for (block_index, b) in blocks.iter_mut().enumerate() {
                    for (inst_index, mut i) in b.instructions.iter_mut().enumerate() {
                        match &mut i {
                            IRInstruction::ResolvedPath { path, tokens, id } => {
                                let path = sym.follow_imports(*path);
                                if path == globalInvocationID {
                                    match &mut types.get_mut(id).unwrap() {
                                        Type::Pointer { class, ty, mutability } => {
                                            storage_classes.insert(*id, StorageClass::Input);
                                            *class = StorageClass::Input;
                                        },
                                        _ => panic!("{:#?},:{:#?}", id, types.get_mut(id).unwrap())
                                    }
                                }
                                match &sym.get(path).1 {
                                    GlobalItem::Function(f) => {
                                        funcs.insert(*id, (path, block_index, inst_index));
                                    },
                                    _ => {}
                                }
                            },
                            IRInstruction::Local { ident, ident_token, id, ty, uni, mutable } => {
                                storage_classes.insert(*id, StorageClass::Function);
                                match &mut types.get_mut(id).unwrap() {
                                    Type::Pointer { class, ty, mutability } => {
                                        if *class == StorageClass::Logical {
                                            *class = StorageClass::Function;
                                        }
                                    },
                                    _ => unreachable!()
                                }
                            },
                            IRInstruction::UnOp { inp, op, out, span } => todo!(),
                            IRInstruction::BinOp { lhs, op, rhs, out, span } => {
                                match op {
                                    rsl_data::internal::ast::BinOp::Add => {
                                        // TODO pointer arithmetic
                                    },
                                    rsl_data::internal::ast::BinOp::Sub => {
                                        // TODO pointer arithmetic
                                    },
                                    rsl_data::internal::ast::BinOp::Mul => {},
                                    rsl_data::internal::ast::BinOp::Div => {},
                                    rsl_data::internal::ast::BinOp::Mod => {},
                                    rsl_data::internal::ast::BinOp::BinAnd => todo!(),
                                    rsl_data::internal::ast::BinOp::LogAnd => {},
                                    rsl_data::internal::ast::BinOp::BinOr => todo!(),
                                    rsl_data::internal::ast::BinOp::LogOr => {},
                                    rsl_data::internal::ast::BinOp::BinXor => todo!(),
                                    rsl_data::internal::ast::BinOp::Index => {
                                        match &mut types.get_mut(lhs).unwrap() {
                                            Type::Pointer { class, ty, mutability } => {
                                                // TODO conflicting storage classes?
                                                if *class == StorageClass::Logical {
                                                    panic!("logical storage classin index lhs");
                                                }
                                                storage_classes.insert(*out, storage_classes[lhs]);
                                            },
                                            _ => unreachable!()
                                        }
                                    },
                                    rsl_data::internal::ast::BinOp::Assign => todo!(),
                                    rsl_data::internal::ast::BinOp::Equals => {},
                                    rsl_data::internal::ast::BinOp::NotEquals => {},
                                    rsl_data::internal::ast::BinOp::Less => {},
                                    rsl_data::internal::ast::BinOp::LessEquals => {},
                                    rsl_data::internal::ast::BinOp::Greater => {},
                                    rsl_data::internal::ast::BinOp::GreaterEquals => {},
                                }
                            },
                            IRInstruction::Property { inp, name, out } => {
                                println!("{}", inp.0);
                                storage_classes.insert(*out, storage_classes[inp]);
                                match &mut types.get_mut(out).unwrap() {
                                    Type::Pointer { class, ty, mutability } => {
                                        // TODO conflicting storage classes?
                                        if *class == StorageClass::Logical {
                                            *class = storage_classes[inp];
                                        }
                                    },
                                    _ => unreachable!()
                                }
                            },
                            IRInstruction::Call { func, args, out, span } => {
                                let (called, path_block, path_inst) = funcs[func];
                                match &sym.get(called).1 {
                                    GlobalItem::Function(f) => {
                                        if f.num_params != 0 {
                                            let mut called_types = f.types.borrow_mut();
                                            let mut classes = Vec::with_capacity(f.num_params);
                                            for i in 0..f.num_params {
                                                match &mut called_types.get_mut(&IRID(i)).unwrap() {
                                                    Type::Pointer { class, ty, mutability } => {
                                                        let c = match &types[&args[i]] {
                                                            Type::Pointer { class, ty, mutability } => {
                                                                *class
                                                            },
                                                            _ => unreachable!()
                                                        };
                                                        classes.push(Some(c));
                                                    },
                                                    _ => {
                                                        classes.push(None);
                                                    }
                                                }
                                            }
                                            if classes.iter().any(|v| v.is_some()) {
                                                fixups.push(Fixup {
                                                    sym: s,
                                                    block: path_block,
                                                    inst: path_inst,
                                                    new_func: specialized_name(sym, strings, called, &classes),
                                                });
                                                new_requests.push(SpecializationRequest { sym: called, classes, rename: true });
                                            }
                                        }
                                    },
                                    _ => unreachable!()
                                }
                            },
                            IRInstruction::Int { v, id, token_id, ty } => {},
                            IRInstruction::Float { v, id, token_id, ty } => {},
                            IRInstruction::Cast { inp, out, ty } => todo!(),
                            IRInstruction::Spread { inp, out, uni } => todo!(),
                            IRInstruction::ReturnValue { id, token_id } => {},
                            IRInstruction::Loop { header, body, cont, merge, construct } => todo!(),
                            IRInstruction::Branch { target_block } => {},
                            IRInstruction::If { inp, true_target_block, false_target_block, merge, construct } => {},
                            IRInstruction::Phi { out, sources } => todo!(),
                            _ => {}
                        }
                    }
                }
            }
        }
        
        if new_requests.is_empty() {
            break;
        }
        
        requests = new_requests;
    }
    
    for f in fixups {
        match &sym.get(f.sym).1 {
            GlobalItem::Function(func) => {
                let mut blocks = func.blocks.borrow_mut();
                match &mut blocks[f.block].instructions[f.inst] {
                    IRInstruction::ResolvedPath { path, tokens, id } => {
                        *path = sym.lookup_id(&f.new_func).unwrap();
                    }
                    _ => panic!()
                }
            }
            _ => panic!()
        }
    }
    
}









