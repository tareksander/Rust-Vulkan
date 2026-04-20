use std::{cell::RefCell, collections::HashMap};

use crate::internal::{ Attribute, InternedString, Mutability, ShaderType, StorageClass, StringTable, Uniformity, Visibility, ast::{self, BinOp, Block, Expression, FunctionDefinition, ModuleData, SourceRange, Statement, UnOp}, ir::{Function, GlobalItem, IRBlock, IRID, IRInstruction, Primitive, SymbolTable, Type}};




impl SymbolTable {
    
    
    
    /// Turns an AST module into an unresolved symbol table
    pub fn from_module(module: ModuleData, strings: &StringTable) -> SymbolTable {
        let mut s = SymbolTable::new_prelude(strings);
        
        
        for f in module.functions {
            s.insert(f.ident, function_definition_to_ir(f)).unwrap();
        }
        
        
        
        return s;
    }
    
    
    
}


struct Data<'a> {
    blocks: &'a mut Vec<IRBlock>,
    id: &'a mut IRID,
    num_params: usize,
}

fn type_to_ir(t: ast::Type) -> Option<Type> {
    // TODO What about partly inferred variables like array types?
    match t {
        ast::Type::Path(item_path) => 
            Some(Type::Unresolved(item_path)),
        ast::Type::Pointer { star_token, mutability, ty } =>
            Some(Type::Pointer { class: StorageClass::PhysicalStorage, ty: Box::new(type_to_ir(*ty).unwrap()), mutability }),
        ast::Type::Reference { ampersand_token, mutability, ty } => 
            Some(Type::Reference { class: StorageClass::Logical, ty: Box::new(type_to_ir(*ty).unwrap()), mutability: mutability.map(|m| m.0).unwrap_or(Mutability::Immutable) }),
        ast::Type::Array { ty, size } =>
            Some(Type::UnresolvedArray { length: size, ty: Box::new(type_to_ir(*ty).unwrap()) }),
        ast::Type::Unit => Some(Type::Primitive(Primitive::Unit)),
        ast::Type::Inferred { ty } => None,
    }
}

fn function_definition_to_ir(f: FunctionDefinition) -> (Visibility, GlobalItem) {
    let is_compute_entrypoint = f.attrs.contains(&Attribute::Compute);
    
    // TODO error if more than one entrypoint attribute is specified
    
    
    let is_entrypoint = is_compute_entrypoint;
    
    let mut types = HashMap::new();
    
    let vis = f.visibility.map(|v| v.0).unwrap_or(Visibility::Priv);
    let ret = (type_to_ir(f.ret.0).unwrap(), f.ret.1.map(|u| u.0).unwrap_or(Uniformity::Inferred));
    let mut params = Vec::with_capacity(f.params.len());
    
    for p in &f.params {
        params.push((p.0, p.1.clone(), p.2.clone().map(|u| u.0).unwrap_or(if is_entrypoint {
            Uniformity::Dispatch
        } else {
            Uniformity::Inferred
        }), type_to_ir(p.3.clone()).unwrap()));
    }
    let shader_type = if let Some(t) = f.shader_type {
        t.0
    } else {
        if is_compute_entrypoint {
            ShaderType::Compute
        } else {
            ShaderType::Generic
        }
    };
    let num_params = params.len();
    let mut locals = HashMap::with_capacity(num_params);
    for i in 0..num_params{
        types.insert(IRID(i), params[i].3.clone());
        locals.insert(f.params[i].0, IRID(i));
    }
    let mut blocks = vec![IRBlock { instructions: vec![] }];
    
    let mut id = IRID(num_params);
    
    let r = f.block.range.clone();
    
    let mut d = Data {
        blocks: &mut blocks,
        id: &mut id,
        num_params,
    };
    if let Some(id) = block_to_ir(&mut d, f.block, locals) {
        blocks.last_mut().unwrap().instructions.push(IRInstruction::ReturnValue { id, token_id: r });
    }
    
    
    return (vis, GlobalItem::Function(Function {
        attrs: f.attrs,
        ident_token: f.ident_token,
        shader_type,
        num_params,
        ret: ret.into(),
        blocks: RefCell::new(blocks),
        next_id: RefCell::new(id),
        types: RefCell::new(types),
    }));
}


fn insert(blocks: &mut Vec<IRBlock>, i: IRInstruction) {
    blocks.last_mut().unwrap().instructions.push(i);
}



fn block_to_ir(d: &mut Data, b: Block, mut locals: HashMap<InternedString, IRID>) -> Option<IRID> {
    for s in b.statements {
        statement_to_blocks(d, s, &mut locals);
    }
    if let Some(e) = b.value {
        let range = e.range();
        let id = expr_to_blocks(d, e, false, &mut locals);
        return Some(id);
    }
    return None;
}

fn statement_to_blocks(d: &mut Data, stm: Statement, locals: &mut HashMap<InternedString, IRID>) {
    match stm {
        Statement::Expression(e) => {
            expr_to_blocks(d, e, false, locals);
        },
        Statement::Return(r, e) => {
            if let Some(e) = e {
                let range = r.merge(&e.range());
                let id = expr_to_blocks(d, e, false, locals);
                insert(d.blocks, IRInstruction::ReturnValue { id, token_id: range });
            } else {
                insert(d.blocks, IRInstruction::Return { token_id: r });
            }
        },
        Statement::Break { break_token, label, value } => todo!(),
        Statement::Continue(token_range) => todo!(),
        Statement::Let(interned_string, token_range, _, _, expression) => todo!(),
    }
}

/// Converts an expression to blocks in a function, returning the IRID of the value.
/// If lvalue is true, instead returns a pointer to the place to store a value.
fn expr_to_blocks(d: &mut Data, e: Expression, lvalue: bool, locals: &mut HashMap<InternedString, IRID>) -> IRID {
    let er = e.range();
    match e {
        Expression::Unary { e, op, op_range } => {
            if lvalue {
                if op != UnOp::Ref && op != UnOp::Ref {
                    panic!("Only indexing and de/referencing is an allowed operation for lvalues");
                }
                if op == UnOp::Deref {
                    let i = d.id.next();
                    let inp = expr_to_blocks(d, *e, lvalue, locals);
                    insert(d.blocks, IRInstruction::Load {
                        ptr: inp,
                        out: i
                    });
                    i
                } else {
                    // Since an lvalue should be a pointer, just omit the ref operation. 
                    expr_to_blocks(d, *e, lvalue, locals)
                }
                
            } else {
                if op == UnOp::Deref {
                    let i = d.id.next();
                    let inp = expr_to_blocks(d, *e, false, locals);
                    insert(d.blocks, IRInstruction::Load {
                        ptr: inp,
                        out: i
                    });
                    i
                } else {
                    if op == UnOp::Ref {
                        expr_to_blocks(d, *e, true, locals)
                    } else {
                        let i = d.id.next();
                        let inp = expr_to_blocks(d, *e, lvalue, locals);
                        insert(d.blocks, IRInstruction::UnOp {
                            inp,
                            op,
                            out: i,
                            span: er });
                            i
                    }
                }
            }
        },
        Expression::Binary { lhs, op, rhs } => {
            match op {
                BinOp::Index => {
                    let i = d.id.next();
                    let lhs = expr_to_blocks(d, *lhs, true, locals);
                    let rhs = expr_to_blocks(d, *rhs, false, locals);
                    insert(d.blocks, IRInstruction::BinOp {
                        lhs,
                        op,
                        rhs,
                        out: i,
                        span: er });
                    if ! lvalue {
                        let ni = d.id.next();
                        insert(d.blocks, IRInstruction::Load { ptr: i, out: ni });
                        ni
                    } else {
                        i
                    }
                },
                BinOp::Assign => {
                    if lvalue {
                        panic!("Only indexing and dereferencing is an allowed operation for lvalues");
                    }
                    let i = d.id.next();
                    insert(d.blocks, IRInstruction::Unit { out: i });
                    let lhs = expr_to_blocks(d, *lhs, true, locals);
                    let rhs = expr_to_blocks(d, *rhs, false, locals);
                    insert(d.blocks, IRInstruction::Store { ptr: lhs, value: rhs });
                    i
                },
                _ => {
                    if lvalue {
                        panic!("Only indexing and dereferencing is an allowed operation for lvalues");
                    }
                    let i = d.id.next();
                    let lhs = expr_to_blocks(d,*lhs, lvalue, locals);
                    let rhs = expr_to_blocks(d,*rhs, lvalue, locals);
                    insert(d.blocks, IRInstruction::BinOp {
                        lhs,
                        op,
                        rhs,
                        out: i,
                        span: er });
                    i
                }
            }
        },
        Expression::Property { e, name, name_token } => {
            let i = d.id.next();
            let base = expr_to_blocks(d,*e, true, locals);
            insert(d.blocks, IRInstruction::Property {
                inp: base,
                name: (name, name_token),
                out: i
            });
            if ! lvalue {
                let ni = d.id.next();
                insert(d.blocks, IRInstruction::Load { ptr: i, out: ni });
                ni
            } else {
                i
            }
        },
        Expression::Item(item_path) => {
            if item_path.global == false && item_path.segments.len() == 1 && item_path.segments[0].generic_args.len() == 0 &&
               let Some(lid) = locals.get(&item_path.segments[0].ident) {
                   if lid.0 < d.num_params {
                    *lid
                } else {
                    let i = d.id.next();
                    insert(d.blocks, IRInstruction::Load { ptr: *lid, out: i });
                    i
                }
            } else {
                let i = d.id.next();
                let r = item_path.range();
                insert(d.blocks, IRInstruction::Path { path: item_path, tokens: r, id: i, lvalue });
                i
            }
        },
        Expression::Group(expression) => expr_to_blocks(d, *expression, lvalue, locals),
        Expression::IntLiteral(v, token_range) => {
            let i = d.id.next();
            insert(d.blocks, IRInstruction::Int { v, id: i, token_id: token_range, ty: None });
            i
        },
        Expression::FloatLiteral(v, token_range) => {
            let i = d.id.next();
            insert(d.blocks, IRInstruction::Float { v, id: i, token_id: token_range, ty: None });
            i
        },
        Expression::Call(item_path, expressions) => {
            let fi = d.id.next();
            insert(d.blocks, IRInstruction::Path { path: item_path.clone(), tokens: item_path.range(), id: fi, lvalue: false });
            let i = d.id.next();
            let mut params = expressions.iter().map(|e| expr_to_blocks(d, e.clone(), false, locals)).collect();
            insert(d.blocks, IRInstruction::Call { func: fi, args: params, out: i, span: item_path.range() });
            i
        },
        Expression::If { condition, then, other } => todo!(),
        Expression::Loop { block } => todo!(),
        // TODO replace unwrap with error reporting
        Expression::Unsafe(block) => block_to_ir(d, *block, todo!()).unwrap(),
    }
    
    
    
    
    
    
    
}





























