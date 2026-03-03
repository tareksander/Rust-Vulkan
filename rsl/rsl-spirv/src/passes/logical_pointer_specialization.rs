use std::collections::HashMap;

use rsl_data::internal::{Mutability, StorageClass, StringTable, ir::{GlobalItem, IRID, IRInstruction, SymbolTable, Type}};




/// - Converts entrypoint parameters to push constant pointers.
/// - Converts locals and non-entrypoint parameters to function pointers
/// - Converts builtins to input pointers
/// - Propagates the storage classes and specializes functions that are called if necessary
/// 
/// TODO:
/// - Store push constant values in function variables if accessed non-uniformly (or maybe if passed to functions to avoid code bloat)
/// - Specialize called functions with storage classes for parameter pointers
/// - Handle vertex and fragment shader IO storage classes
/// 
pub fn logical_pointer_specialization(sym: &mut SymbolTable, strings: &StringTable) {
    
    // TODO handle builtins in a generic way without traversing the whole table
    // Maybe keep the names in a list?
    
    let globalInvocationID = sym.lookup_id(&strings.insert_get("::core::globalInvocationID")).unwrap();
    println!("{:#?}", globalInvocationID);
    
    let mut eps = vec![];
    
    
    
    for s in sym.iter() {
        let i = sym.get(s);
        if let GlobalItem::Function(f) = &i.1 {
            // TODO other entrypoints
            if f.attrs.contains(&rsl_data::internal::Attribute::Compute) {
                eps.push(s);
            }
        }
    }
    
    for s in eps {
        let i = sym.get(s);
        if let GlobalItem::Function(f) = &i.1 {
            let mut storage_classes: HashMap<IRID, StorageClass> = HashMap::new();
            let mut blocks = f.blocks.borrow_mut();
            let mut types = f.types.borrow_mut();
            
            if f.num_params != 0 {
                let first = blocks.first_mut().unwrap();
                for i in 0..f.num_params {
                    storage_classes.insert(IRID(i), StorageClass::Push);
                    match &mut types.get_mut(&IRID(i)).unwrap() {
                        Type::Pointer { class, ty, mutability } => {
                            *class = StorageClass::Push;
                        },
                        _ => unreachable!()
                    }
                }
            }
            
            for b in blocks.iter_mut() {
                for mut i in b.instructions.iter_mut() {
                    match &mut i {
                        IRInstruction::ResolvedPath { path, tokens, id, lvalue } => {
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
                                    match &mut types.get_mut(lhs).unwrap() {
                                        Type::Pointer { class, ty, mutability } => {
                                            // TODO conflicting storage classes?
                                            if *class == StorageClass::Logical {
                                                storage_classes.insert(*rhs, storage_classes[lhs]);
                                                *class = storage_classes[lhs];
                                            }
                                        },
                                        _ => unreachable!()
                                    }
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
                        IRInstruction::Property { inp, name, out } => {
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
                        IRInstruction::Call { func, args, out, span } => todo!(),
                        IRInstruction::Int { v, id, token_id, ty } => todo!(),
                        IRInstruction::Float { v, id, token_id, ty } => todo!(),
                        IRInstruction::Cast { inp, out, ty } => todo!(),
                        IRInstruction::Spread { inp, out, uni } => todo!(),
                        IRInstruction::ReturnValue { id, token_id } => todo!(),
                        IRInstruction::Loop { header, body, cont, merge, construct } => todo!(),
                        IRInstruction::Branch { target_block } => todo!(),
                        IRInstruction::If { inp, true_target_block, false_target_block, merge, construct } => todo!(),
                        IRInstruction::Phi { out, sources } => todo!(),
                        _ => {}
                    }
                }
            }
        }
    }
    
    
    
    
}









