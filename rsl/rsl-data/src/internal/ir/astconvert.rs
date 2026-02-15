use std::cell::RefCell;

use crate::internal::{ Attribute, Mutability, ShaderType, StorageClass, StringTable, Uniformity, Visibility, ast::{self, BinOp, Block, Expression, FunctionDefinition, ModuleData, SourceRange, Statement, UnOp}, ir::{Function, GlobalItem, IRBlock, IRID, IRInstruction, Primitive, SymbolTable, Type}};




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
    
    
    let vis = f.visibility.map(|v| v.0).unwrap_or(Visibility::Priv);
    let ret = (type_to_ir(f.ret.0).unwrap(), f.ret.1.map(|u| u.0).unwrap_or(Uniformity::Inferred));
    let mut params = Vec::with_capacity(f.params.len());
    for p in &f.params {
        params.push((p.0, p.1.clone(), p.2.clone().map(|u| u.0).unwrap_or(Uniformity::Inferred), type_to_ir(p.3.clone()).unwrap()));
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
    let mut blocks = vec![IRBlock { instructions: vec![] }];
    let mut id = IRID(0);
    
    let r = f.block.range.clone();
    if let Some(id) = block_to_ir(&mut blocks, f.block, &mut id) {
        blocks.last_mut().unwrap().instructions.push(IRInstruction::ReturnValue { id, token_id: r });
    }
    
    
    return (vis, GlobalItem::Function(Function {
        attrs: f.attrs,
        ident_token: f.ident_token,
        shader_type,
        params,
        ret,
        blocks: RefCell::new(blocks),
    }));
}


fn insert(blocks: &mut Vec<IRBlock>, i: IRInstruction) {
    blocks.last_mut().unwrap().instructions.push(i);
}



fn block_to_ir(blocks: &mut Vec<IRBlock>, b: Block, id: &mut IRID) -> Option<IRID> {
    for s in b.statements {
        statement_to_blocks(blocks, s, id);
    }
    if let Some(e) = b.value {
        let range = e.range();
        let id = expr_to_blocks(blocks, e, false, id);
        return Some(id);
    }
    return None;
}

fn statement_to_blocks(blocks: &mut Vec<IRBlock>, stm: Statement, id: &mut IRID) {
    match stm {
        Statement::Expression(e) => {
            expr_to_blocks(blocks, e, false, id);
        },
        Statement::Return(r, e) => {
            if let Some(e) = e {
                let range = r.merge(&e.range());
                let id = expr_to_blocks(blocks, e, false, id);
                insert(blocks, IRInstruction::ReturnValue { id, token_id: range });
            } else {
                insert(blocks, IRInstruction::Return { token_id: r });
            }
        },
        Statement::Break { break_token, label, value } => todo!(),
        Statement::Continue(token_range) => todo!(),
        Statement::Let(interned_string, token_range, _, _, expression) => todo!(),
    }
}

/// Converts an expression to blocks in a function, returning the IRID of the value.
/// If lvalue is true, instead returns a pointer to the place to store a value.
fn expr_to_blocks(blocks: &mut Vec<IRBlock>, e: Expression, lvalue: bool, id: &mut IRID) -> IRID {
    let er = e.range();
    match e {
        Expression::Unary { e, op, op_range } => {
            if lvalue {
                if op != UnOp::Deref {
                    panic!("Only indexing and dereferencing is an allowed operation for lvalues");
                }
                // Since an lvalue should be a pointer, just omit the deref operation. 
                expr_to_blocks(blocks, *e, lvalue, id)
            } else {
                let i = id.next();
                let inp = expr_to_blocks(blocks, *e, lvalue, id);
                insert(blocks, IRInstruction::UnOp {
                    inp,
                    op,
                    out: i,
                    span: er });
                i
            }
        },
        Expression::Binary { lhs, op, rhs } => {
            match op {
                BinOp::Index => {
                    let i = id.next();
                    let lhs = expr_to_blocks(blocks, *lhs, true, id);
                    let rhs = expr_to_blocks(blocks, *rhs, false, id);
                    insert(blocks, IRInstruction::BinOp {
                        lhs,
                        op,
                        rhs,
                        out: i,
                        span: er });
                    if ! lvalue {
                        let ni = id.next();
                        insert(blocks, IRInstruction::Load { ptr: i, out: ni });
                        ni
                    } else {
                        i
                    }
                },
                BinOp::Assign => {
                    if lvalue {
                        panic!("Only indexing and dereferencing is an allowed operation for lvalues");
                    }
                    let i = id.next();
                    insert(blocks, IRInstruction::Unit { out: i });
                    let lhs = expr_to_blocks(blocks, *lhs, true, id);
                    let rhs = expr_to_blocks(blocks, *rhs, false, id);
                    insert(blocks, IRInstruction::Store { ptr: lhs, value: rhs });
                    i
                },
                _ => {
                    if lvalue {
                        panic!("Only indexing and dereferencing is an allowed operation for lvalues");
                    }
                    let i = id.next();
                    let lhs = expr_to_blocks(blocks,*lhs, lvalue, id);
                    let rhs = expr_to_blocks(blocks,*rhs, lvalue, id);
                    insert(blocks, IRInstruction::BinOp {
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
            let i = id.next();
            let base = expr_to_blocks(blocks,*e, true, id);
            insert(blocks, IRInstruction::Property {
                inp: base,
                name: (name, name_token),
                out: i
            });
            if ! lvalue {
                let ni = id.next();
                insert(blocks, IRInstruction::Load { ptr: i, out: ni });
                ni
            } else {
                i
            }
        },
        Expression::Item(item_path) => {
            let i = id.next();
            let r = item_path.range();
            insert(blocks, IRInstruction::Path { path: item_path, tokens: r, id: i, lvalue });
            i
        },
        Expression::Group(expression) => expr_to_blocks(blocks, *expression, lvalue, id),
        Expression::IntLiteral(_, token_range) => todo!(),
        Expression::FloatLiteral(_, token_range) => todo!(),
        Expression::Call(item_path, expressions) => todo!(),
        Expression::If { condition, then, other } => todo!(),
        Expression::Loop { block } => todo!(),
        // TODO replace unwrap with error reporting
        Expression::Unsafe(block) => block_to_ir(blocks, *block, id).unwrap(),
    }
    
    
    
    
    
    
    
}





























