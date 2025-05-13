use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::rc::Rc;

use crate::ast::expr::Expression;
use crate::ast::statement::{Block, Let, Statement};
use crate::ast::{ty, Attribute, Entrypoint, ItemPath};
use crate::mid::{Function, Metadata, ModuleItem, Primitive, Scope, Type, TypeVariant};

use crate::ast::module::Module;
use crate::{Ident, SourcePos, SourceSpan, Uniformity};




/// Runs all passes on the modules in the specified order
pub fn run_passes(root: &mut Scope) -> Metadata {
    let mut md  = simplify(root);
    let order = get_order_canonicalize(root);
    constant_evaluation(root, &order);
    infer_types(root, &order, &mut md);
    visibility_check(root, &order);
    limits_check(root, &order, &mut md);
    return md;
}



/// Computes the required order of the modules, errors on cyclic dependencies.
pub fn get_order_canonicalize(root: &mut Scope) -> Vec<Ident> {
    let mut dependencies: HashMap<Ident, HashSet<Ident>> = HashMap::new();
    for s in root.scopes() {
        dependencies.insert(s.0.clone(), HashSet::new());
    }
    
    
    fn walk_type_paths<F>(path: &ItemPath, t: &mut Type, in_function: bool, f: &mut F) where F: FnMut(&ItemPath) {
        match &mut t.ty {
            TypeVariant::Pointer(pointer) => walk_type_paths(path, &mut pointer.ty, in_function, f),
            TypeVariant::Reference(reference) => walk_type_paths(path, &mut reference.ty, in_function, f),
            TypeVariant::Tuple(item_paths) => todo!(),
            TypeVariant::Item(item_path) => {
                //println!("item path: {:#?}", item_path);
                //println!("globalize with: {:#?}", path);
                if ! item_path.global {
                    *item_path = item_path.globalize(path.clone());
                    item_path.global = true;
                }
                //println!("item path after: {:#?}", item_path);
                f(&item_path);
            },
            TypeVariant::Struct(item_path) => {},
            TypeVariant::Primitive(primitive) => {}
            TypeVariant::Function(item_path) => {},
            TypeVariant::AbstractInt => {},
            TypeVariant::AbstractFloat => {},
            TypeVariant::Vector(vector) => {},
            TypeVariant::Matrix(matrix) => {},
            TypeVariant::Unit => {},
            TypeVariant::Error => {},
        }
    }
    
    fn walk_block_paths<F>(path: &ItemPath, b: &mut Block, in_function: bool, f: &mut F) where F: FnMut(&ItemPath) {
        for s in &mut b.statements {
            match s {
                Statement::Expression(expression) => walk_expr_paths(path, expression, in_function, f),
                Statement::Return(expression) => {
                    if let Some(e) = expression {
                        walk_expr_paths(path, e, in_function, f)
                    }
                },
                Statement::Break(source_span) => {},
                Statement::Continue(source_span) => {},
                Statement::Let(l) => {
                    match l {
                        Let::Single(source_span, mutability, ident, ty, expression) => {
                            if let Some(e) = expression {
                                walk_expr_paths(path, e, in_function, f);
                            }
                            // TODO canonicalize type path (also looking in the type recursively, so that primitive types are resolved)
                        },
                    }
                },
            }
        }
        if let Some(v) = &mut b.value {
            walk_expr_paths(path, v, in_function, f);
        }
    }
    
    fn walk_expr_paths<F>(path: &ItemPath, expr: &mut Expression, in_function: bool, f: &mut F) where F: FnMut(&ItemPath) {
        match expr {
            Expression::Item(item_path) => {
                if ! in_function && ! path.global {
                    *item_path = item_path.globalize(path.clone());
                    item_path.global = true;
                }
                f(&item_path);
            },
            Expression::UnOp(source_span, un_op, expression) => {
                walk_expr_paths(path, expression, in_function, f);
            },
            Expression::BinOp(expression, bin_op, expression1) => {
                walk_expr_paths(path, expression, in_function, f);
                walk_expr_paths(path, expression1, in_function, f);
            },
            Expression::If(i) => {
                walk_block_paths(path, &mut i.then, in_function, f);
                if let Some(otherwise) = &mut i.otherwise {
                    walk_block_paths(path, otherwise, in_function, f);
                }
            },
            Expression::Tuple(source_span, expressions) => todo!(),
            Expression::Property(expression, ident, source_span) => {
                walk_expr_paths(path, expression, in_function, f);
            },
            Expression::Call(expression, expressions) => {
                walk_expr_paths(path, expression, in_function, f);
                for e in expressions {
                    walk_expr_paths(path, e, in_function, f);
                }
            },
            Expression::Index(expression, expression1) => {
                walk_expr_paths(path, expression, in_function, f);
                walk_expr_paths(path, expression1, in_function, f);
            },
            Expression::Cast(expression, t) => {
                walk_expr_paths(path, expression, in_function, f);
                todo!()
                //walk_type_paths(path, t, f);
            },
            Expression::Unsafe(block) => walk_block_paths(path, block, in_function, f),
            _ => {}
        }
    }
    
    fn walk_scope_paths<F>(path: &ItemPath, scope: &mut Scope, f: &mut F) where F: FnMut(&ItemPath) {
        let keys: Vec<Ident> = scope.items.keys().cloned().collect();
        for i in &mut scope.items {
            match i.1 {
                crate::mid::ModuleItem::Struct(s) => {
                    for field in &mut s.fields {
                        walk_type_paths(path, &mut field.1.ty, false, f);
                    }
                },
                crate::mid::ModuleItem::Function(func) => {
                    for p in &mut func.params {
                        walk_type_paths(path, &mut p.1, true, f);
                    }
                    walk_type_paths(path, &mut func.ret, true, f);
                    
                },
                crate::mid::ModuleItem::Static(s) => {
                    if let Some(init) = &mut s.value {
                        walk_expr_paths(path, &mut init.0, false, f);
                    }
                    walk_type_paths(path, &mut s.ty, false, f);
                },
                crate::mid::ModuleItem::Constant(constant) => walk_expr_paths(path, &mut constant.init, false, f),
                crate::mid::ModuleItem::Import(item_path) => {
                    if ! item_path.global {
                        if keys.contains(&item_path.segments[0].0) {
                            *item_path = item_path.globalize(path.clone());
                        } else {
                            item_path.global = true;
                        }
                    }
                    f(&item_path);
                },
                crate::mid::ModuleItem::Type(ty) => todo!(),
                crate::mid::ModuleItem::Primitive(_) => {},
                ModuleItem::Module(s) => {
                    let mut p = path.clone();
                    p.segments.push((i.0.clone(), path.segments[0].1.clone(), vec![]));
                    walk_scope_paths(&p, s, f);
                }
            }
        }
    }
    
    
    for s in &mut root.scopes_mut() {
        // TODO get dependencies by recursively walking everything and checking every ItemPath.
        // Also canonicalize the paths while we're at it.
        let dummy_span = SourceSpan {
            file: Rc::new(PathBuf::from("compiler.rsl")),
            start: SourcePos::ZERO,
            end: SourcePos::ZERO,
        };
        walk_scope_paths(&ItemPath { segments: vec![(s.0.clone(), dummy_span, vec![])], global: true }, &mut* s.1, &mut |p| {
            
        });
    }
    
    fn has_cycle(map: &HashMap<Ident, HashSet<Ident>>, current: Ident, visited: &mut HashSet<Ident>) -> bool {
        visited.insert(current.clone());
        for dep in &map[&current]{
            if visited.contains(dep) {
                return true;
            } else {
                if has_cycle(map, current.clone(), visited) {
                    return true;
                }
            }
        }
        return false;
    }
    
    for s in root.scopes() {
        if has_cycle(&mut dependencies, s.0.clone(), &mut HashSet::new()) {
            panic!("Cycle in package graph detected");
        }
    }
    
    // for now just return an arbitrary order
    let mut order = vec![];
    for s in root.scopes() {
        order.push(s.0.clone());
    }
    return order;
}


fn visibility_check(root: &mut Scope, order: &[Ident]) {
    
}

fn constant_evaluation(root: &mut Scope, order: &[Ident]) {
    // TODO check dependency graph for constant initializers for cycles
    // compute constant values
    // resolve all expressions between compile-time numbers
}


/// Checks:
/// - TODO no return in control flow constructs without maximal reconvergence
/// - call graph cycles
/// - TODO check that push constants can only be used directly in entrypoints, otherwise code sharing is severely undermined
/// - TODO Only one push constant static per entrypoint used
/// - TODO Check for infinite struct sizes by composition without pointers (arrays are also included in the size calculations, except RuntimeArrays)
/// - TODO binding overlap for each entrypoint
/// - TODO check builtin usage for stages
fn limits_check(root: &mut Scope, order: &[Ident], md: &mut Metadata) {
    
    // call graph cycle check
    for (e, _) in &md.entrypoints {
        let mut call_set = HashSet::new();
        let mut to_visit = Vec::new();
        to_visit.push(e.clone());
        
        while let Some(f) = to_visit.pop() {
            md.function_entrypoints.get_mut(&f).unwrap().insert(e.clone());
            if ! call_set.insert(f.clone()) {
                panic!("Call graph cycle detected");
            }
            to_visit.append(&mut md.call_set[&f].iter().cloned().collect());
        }
    }
    
    
    
    
    
    
}


/// simplify complex control flow constructs and other things:
/// - Resolve shadowed variable names in functions different names prefixed by a number
/// - Resolves variables in functions to global scope if they aren't local
///     - canonicalization pass is needed for global Initializers
/// - TODO turning for and while loops into loop
fn simplify(root: &mut Scope) -> Metadata {
    let mut md = Metadata {
        functions: vec![],
        entrypoints: HashMap::new(),
        call_set: HashMap::new(),
        static_set: HashMap::new(),
        function_entrypoints: HashMap::new()
    };
    
    fn simplify_module(module: &mut Scope, path: ItemPath, md: &mut Metadata) {
        
        struct VariableScopeList<'a> {
            // Maps the identifier to the number of times it has been shadowed in this block. This is, for each time it is encountered in an expression, the number has to be prepended.
            // If a path is encountered, but isn't found in the scope, look one up higher. If it's found in no scope, resolve it into the module.
            // If the name is encountered in a let increment the max entry and update the current vars entry.
            vars: HashMap<Ident, u16>,
            max: &'a mut HashMap<Ident, u16>,
            parent: Option<&'a VariableScopeList<'a>>
        }
        
        impl<'a> VariableScopeList<'a> {
            fn process_let(&mut self, l: &mut Let) {
                match l {
                    Let::Single(_, _, id, _, _) => {
                        let num = self.max.get(id).and_then(|v| Some(*v)).unwrap_or_default() + 1;
                        self.max.insert(id.clone(), num);
                        self.vars.insert(id.clone(), num);
                        *id = Ident { str: num.to_string() + &id.str };
                    },
                }
            }
            
            fn resolve(&mut self, path: &mut ItemPath, scope: &ItemPath) -> bool {
                //println!("Resolving {:#?}", path);
                if ! path.global && path.segments.len() == 1 {
                    let id = &mut path.segments[0].0;
                    if let Some(num) = self.find(id) {
                        id.str = num.to_string() + &id.str;
                        return false;
                        //println!("resolved as local variable");
                    } else {
                        *path = path.globalize(scope.clone());
                        path.global = true;
                        //println!("resolved as global variable: {:#?}", path);
                    }
                }
                return true;
            }
            
            fn find(&self, id: &Ident) -> Option<u16> {
                if let Some(res) = self.vars.get(id) {
                    return Some(*res);
                } else {
                    if let Some(p) = self.parent {
                        return p.find(id);
                    } else {
                        return None;
                    }
                }
            }
        }
        
        fn walk_statements<'a, F, F2>(b: &mut Block, scope: &ItemPath, f: &F, f2: &F2, variables: &mut VariableScopeList<'a>, md: &mut Metadata, function_path: &ItemPath) where F: Fn(&mut Let, &mut VariableScopeList<'a>), F2: Fn(&mut ItemPath, &mut VariableScopeList<'a>) {
            fn walk_expr_statements<'a, F, F2>(e: &mut Expression, scope: &ItemPath, f: &F, f2: &F2, variables: &mut VariableScopeList<'a>, md: &mut Metadata, function_path: &ItemPath) where F: Fn(&mut Let, &mut VariableScopeList<'a>), F2: Fn(&mut ItemPath, &mut VariableScopeList<'a>) {
                match e {
                    Expression::Int(_, _) => {},
                    Expression::Float(_, _) => {},
                    Expression::Item(item_path) => f2(item_path, variables),
                    Expression::UnOp(_, _, expression) => walk_expr_statements(expression, scope, f, f2, variables, md, function_path),
                    Expression::BinOp(lhs, _, rhs) => {
                        walk_expr_statements(lhs, scope, f, f2, variables, md, function_path);
                        walk_expr_statements(rhs, scope, f, f2, variables, md, function_path);
                    },
                    Expression::If(i) => {
                        walk_expr_statements(&mut i.condition, scope, f, f2, variables, md, function_path);
                        walk_statements(&mut i.then, scope, f, f2, variables, md, function_path);
                        if let Some(otherwise) = &mut i.otherwise {
                            walk_statements(otherwise, scope, f, f2, variables, md, function_path);
                        }
                    },
                    Expression::Unit(_) => {},
                    Expression::Tuple(_, expressions) => {
                        for e in expressions {
                            walk_expr_statements(e, scope, f, f2, variables, md, function_path);
                        }
                    },
                    Expression::Property(expression, _, _) => {
                        walk_expr_statements(expression, scope, f, f2, variables, md, function_path);
                    },
                    Expression::Call(expression, expressions) => {
                        walk_expr_statements(expression, scope, f, f2, variables, md, function_path);
                        for e in expressions {
                            walk_expr_statements(e, scope, f, f2, variables, md, function_path);
                        }
                    },
                    Expression::Index(lhs, rhs) => {
                        walk_expr_statements(lhs, scope, f, f2, variables, md, function_path);
                        walk_expr_statements(rhs, scope, f, f2, variables, md, function_path);
                    },
                    Expression::Cast(expression, _) => walk_expr_statements(expression, scope, f, f2, variables, md, function_path),
                    Expression::Unsafe(block) => walk_statements(block, scope, f, f2, variables, md, function_path),
                }
            }
            for s in &mut b.statements {
                match s {
                    Statement::Expression(expression) => walk_expr_statements(expression, scope, f, f2, variables, md, function_path),
                    Statement::Return(expression) => {
                        if let Some(e) = expression {
                            walk_expr_statements(e, scope, f, f2, variables, md, function_path)
                        }
                    },
                    Statement::Break(_) => {},
                    Statement::Continue(_) => {},
                    Statement::Let(l) => {
                        match l {
                            crate::ast::statement::Let::Single(_, _, _, _, Some(expression)) => walk_expr_statements(expression, scope, f, f2, variables, md, function_path),
                            _ => {}
                        }
                        f(l, variables);
                    }
                }
            }
            if let Some(e) = &mut b.value {
                walk_expr_statements(e, scope, f, f2, variables, md, function_path);
            }
        }
        let file = Rc::new(PathBuf::from("compiler.rsl"));
    
        
        for i in &mut module.items {
            match i.1 {
                crate::mid::ModuleItem::Function(function) => {
                    let mut fn_path = path.clone();
                    fn_path.segments.push((i.0.clone(), SourceSpan {
                        file: file.clone(),
                        start: SourcePos::ZERO,
                        end: SourcePos::ZERO,
                    }, vec![]));
                    for a in &function.attrs {
                        match a {
                            (Attribute::Entrypoint(e), _) => {
                                md.entrypoints.insert(fn_path.clone(), e.clone());
                            },
                            _ => {}
                        }
                    }
                    md.functions.push(fn_path.clone());
                    md.call_set.insert(fn_path.clone(), HashSet::new());
                    md.static_set.insert(fn_path.clone(), HashSet::new());
                    md.function_entrypoints.insert(fn_path.clone(), HashSet::new());
                    
                    let scope = path.clone();
                    // scope.segments.push((i.0.clone(), SourceSpan {
                    //     file: file.clone(),
                    //     start: SourcePos::ZERO,
                    //     end: SourcePos::ZERO,
                    // }, vec![]));
                    walk_statements(&mut function.block, &scope, &mut |l, vars| {
                        vars.process_let(l);
                    }, &|path, vars| {
                        //println!("resolving {:#?}", path);
                        vars.resolve(path, &scope);
                    }, &mut VariableScopeList { vars: HashMap::new(), max: &mut HashMap::new(), parent: None }, md, &fn_path);
                },
                crate::mid::ModuleItem::Static(_) => {},
                _ => {}
            }
        }
        
        
        for module in &mut module.scopes_mut() {
            let m = &mut*module.1;
            let mut scope = path.clone();
            scope.segments.push((module.0.clone(), SourceSpan {
                file: file.clone(),
                start: SourcePos::ZERO,
                end: SourcePos::ZERO,
            }, vec![]));
            simplify_module(m, scope, md);
        }
    }
    let file = Rc::new(PathBuf::from("compiler.rsl"));
    for module in &mut root.scopes_mut() {
        let scope = ItemPath {
            global: true,
            segments: vec![(module.0.clone(), SourceSpan {
                file: file.clone(),
                start: SourcePos::ZERO,
                end: SourcePos::ZERO,
            }, vec![])]
        };
        simplify_module(&mut*module.1, scope, &mut md);
    }
    return md;
}


/// resolves all item type variants recursively
pub fn resolve_item(root: &Scope, ty: &mut TypeVariant) {
    match ty {
        TypeVariant::Item(path) => {
            *ty = root.lookup_type(path);
        }
        TypeVariant::Pointer(pointer) => {
            resolve_item(root, &mut pointer.ty.ty);
        },
        TypeVariant::Reference(reference) => {
            resolve_item(root, &mut reference.ty.ty);
        },
        TypeVariant::Tuple(item_paths) => todo!("resolve tuple"),
        TypeVariant::Function(item_path) => {},
        
        _ => {}
    }
    
}


/// Infers type information if possible and checks types.
/// By this point the constant evaluation has to have run.
fn infer_types(root: &mut Scope, order: &[Ident], md: &mut Metadata) {
    let dummy_span = SourceSpan {
        file: Rc::new(PathBuf::from("compiler.rsl")),
        start: SourcePos::ZERO,
        end: SourcePos::ZERO,
    };
    // TODO
    // Keep a table with variable types in a function. When a variable is assigned, set it to that type and error if another usage contradicts it.
    // Limit the maximal uniformity of an operation to the control flow uniformity, by passing that to the type check functions.
    // Bubble up the error type
    
    fn infer_block(block: &Block, variables: &mut HashMap<Ident, Type>, types: &mut Vec<Type>, dummy_span: &SourceSpan, root: &Scope, max_uniformity: &Uniformity, md: &mut Option<(&mut Metadata, &ItemPath)>) {
        let i = types.len();
        types.push(Type { uni: Some(Uniformity::Uniform), ty: TypeVariant::Error, span: dummy_span.clone() });
        for s in &block.statements {
            match s {
                Statement::Expression(expression) => infer_expr(expression, variables, types, dummy_span, root, max_uniformity, md),
                Statement::Return(expression) => todo!(),
                Statement::Break(source_span) => todo!(),
                Statement::Continue(source_span) => todo!(),
                Statement::Let(l) => {
                    match l {
                        Let::Single(source_span, mutability, ident, ty, expression) => {
                            
                            if let Some(ty) = ty {
                                todo!("let with type annotation (requires canonicalization)")
                            } else {
                                if let Some(e) = expression {
                                    let i = types.len();
                                    infer_expr(e, variables, types, dummy_span, root, max_uniformity, md);
                                    if types[i].ty == TypeVariant::Error {
                                        panic!("Could not infer variable type");
                                    }
                                    
                                    variables.insert(ident.clone(), types[i].clone());
                                } else {
                                    variables.insert(ident.clone(), Type { uni: None, ty: TypeVariant::Error, span: dummy_span.clone() });
                                }
                            }
                        },
                    }
                },
            }
        }
        if let Some(e) = &block.value {
            let vi = types.len();
            infer_expr(&e, variables, types, dummy_span, root, max_uniformity, md);
            types[i] = types[vi].clone();
        } else {
            types[i].ty = TypeVariant::Unit;
        }
    }
    
    fn infer_expr(expr: &Expression, variables: &mut HashMap<Ident, Type>, types: &mut Vec<Type>, dummy_span: &SourceSpan, root: &Scope, max_uniformity: &Uniformity, md: &mut Option<(&mut Metadata, &ItemPath)>) {
        let i = types.len();
        types.push(Type { uni: Some(Uniformity::Uniform), ty: TypeVariant::Error, span: dummy_span.clone() });
        match expr {
            Expression::Int(_, _) => {
                types[i].ty = TypeVariant::AbstractInt;
                types[i].uni = Some(Uniformity::Uniform);
            },
            Expression::Float(_, _) => {
                types[i].ty = TypeVariant::AbstractFloat;
                types[i].uni = Some(Uniformity::Uniform);
            },
            Expression::Item(item_path) => {
                if item_path.global {
                    if let Some(item) = root.lookup_path(item_path) {
                        match item {
                            ModuleItem::Static(stat) => {
                                //println!("static type: {:#?}", stat.ty);
                                types[i] = stat.ty.clone();
                                if let Some(md) = md {
                                    md.0.static_set.get_mut(md.1).unwrap().insert(item_path.clone());
                                }
                            },
                            ModuleItem::Constant(constant) => todo!(),
                            ModuleItem::Function(f) => {
                                types[i].ty = TypeVariant::Function(item_path.clone());
                                types[i].uni = Some(Uniformity::Uniform);
                                if let Some(md) = md {
                                    md.0.call_set.get_mut(md.1).unwrap().insert(item_path.clone());
                                }
                            }
                            _ => {
                                println!("non-value module item: {:#?}", item);
                            }
                        }
                    } else {
                        println!("Item not found: {:#?}", item_path);
                    }
                } else {
                    //println!("{:#?}", item_path.segments);
                    types[i] = variables[&item_path.segments[0].0].clone();
                }
                resolve_item(root, &mut types[i].ty);
            },
            Expression::UnOp(_, un_op, expression) => {
                infer_expr(&*expression, variables, types, dummy_span, root, max_uniformity, md);
                if types[i+1].ty == TypeVariant::Error {
                    return;
                }
                match un_op {
                    crate::ast::expr::UnOp::Neg => {
                        
                    },
                    crate::ast::expr::UnOp::Not => {
                        
                    },
                }
            },
            Expression::BinOp(lhs, bin_op, rhs) => {
                infer_expr(&lhs, variables, types, dummy_span, root, max_uniformity, md);
                let rhsi = types.len();
                infer_expr(&rhs, variables, types, dummy_span, root, max_uniformity, md);
                if types[i+1].ty == TypeVariant::Error {
                    return;
                }
                if types[rhsi].ty == TypeVariant::Error {
                    return;
                }
                let lhst = &types[i+1];
                let rhst = &types[rhsi];
                if lhst.ty == TypeVariant::AbstractInt || lhst.ty == TypeVariant::AbstractFloat &&
                   rhst.ty == TypeVariant::AbstractInt || rhst.ty == TypeVariant::AbstractFloat {
                    panic!("Found 2 abstract numbers, which should have been resolved by constant evaluation");
                }
                match bin_op {
                    crate::ast::expr::BinOp::Assign => {
                        match &**lhs {
                            Expression::Item(item_path) => {
                                //todo!("allow item in assignment lhs")
                            },
                            Expression::UnOp(source_span, un_op, expression) => {
                                todo!("allow dereference in assignment lhs")
                            },
                            Expression::Property(expression, ident, source_span) => {
                                //todo!("allow property access in assignment lhs")
                            },
                            Expression::Index(expression, expression1) => {
                                //todo!("allow indexing in assignment lhs")
                            },
                            _ => {
                                panic!("Invalid left hand side of an assignment");
                            }
                        }
                        if lhst.uni.as_ref().unwrap() > rhst.uni.as_ref().unwrap() {
                            panic!("Unable to assign less uniform value to more uniform variable")
                        }
                        types[i].ty = TypeVariant::Unit;
                        types[i].uni = Some(Uniformity::Uniform);
                    },
                    _ => {
                        // TODO scalar multiplication and division
                        match (&lhst.ty, &rhst.ty) {
                            (TypeVariant::Primitive(lhsp), TypeVariant::Primitive(rhsp)) => {
                                let resp;
                                match (lhsp.is_float(), rhsp.is_float()) {
                                    (true, true) => {
                                        if lhsp.size() > rhsp.size() {
                                            resp = lhsp;
                                        } else {
                                            resp = rhsp;
                                        }
                                    },
                                    (true, false) => {
                                        // TODO size check when coercing
                                        resp = lhsp;
                                    }
                                    (false, true) => {
                                        // TODO size check when coercing
                                        resp = rhsp;
                                    }
                                    (false, false) => {
                                        if lhsp.size() > rhsp.size() {
                                            resp = lhsp;
                                        } else {
                                            resp = rhsp;
                                        }
                                    }
                                }
                                let lhsu = lhst.uni.clone().unwrap();
                                let rhsu = rhst.uni.clone().unwrap();
                                types[i].ty = TypeVariant::Primitive(*resp);
                                types[i].uni = Some((lhsu.limit(&rhsu)).limit(max_uniformity));
                            }
                            /*
                            (TypeVariant::Item(lhspath), TypeVariant::Item(rhspath)) => {
                                let lhsi = root.lookup_path(&lhspath).unwrap();
                                let rhsi = root.lookup_path(&rhspath).unwrap();
                                match (lhsi, rhsi) {
                                    (ModuleItem::Primitive(lhsp), ModuleItem::Primitive(rhsp)) => {
                                        if *lhsp == Primitive::Unit || *rhsp == Primitive::Unit {
                                            panic!("Cannot use binary operations on the unit type");
                                        }
                                        let mut resp;
                                        match (lhsp.is_float(), rhsp.is_float()) {
                                            (true, true) => {
                                                if lhsp.size() > rhsp.size() {
                                                    resp = lhspath;
                                                } else {
                                                    resp = rhspath;
                                                }
                                            },
                                            (true, false) => {
                                                // TODO size check when coercing
                                                resp = lhspath;
                                            }
                                            (false, true) => {
                                                // TODO size check when coercing
                                                resp = rhspath;
                                            }
                                            (false, false) => {
                                                if lhsp.size() > rhsp.size() {
                                                    resp = lhspath;
                                                } else {
                                                    resp = rhspath;
                                                }
                                            }
                                        }
                                        let lhsu = lhst.uni.clone().unwrap();
                                        let rhsu = rhst.uni.clone().unwrap();
                                        types[i].ty = TypeVariant::Item(resp.clone());
                                        types[i].uni = Some((lhsu.limit(&rhsu)).limit(max_uniformity));
                                    },
                                    _ => panic!("Binary operations can only operate on primitives, matrices and vectors")
                                }
                                
                            }
                            */
                            _ => {
                                let lhs_allowed = match lhst.ty {
                                    TypeVariant::Matrix(_) => true,
                                    TypeVariant::Vector(_) => true,
                                    _ => false
                                };
                                let rhs_allowed = match lhst.ty {
                                    TypeVariant::Matrix(_) => true,
                                    TypeVariant::Vector(_) => true,
                                    _ => false
                                };
                                if lhs_allowed && rhs_allowed {
                                    todo!("vector and matrix ops");
                                    
                                    let lhsu = lhst.uni.clone().unwrap();
                                    let rhsu = rhst.uni.clone().unwrap();
                                    
                                    types[i].uni = Some((lhsu.limit(&rhsu)).limit(max_uniformity));
                                } else {
                                    panic!("Binary operations can only be used on primitives, matrices or vectors: {:#?}, {:#?}", lhst, rhst)
                                }
                            }
                        }
                    }
                }
            },
            Expression::If(_) => todo!(),
            Expression::Unit(_) => todo!(),
            Expression::Tuple(_, expressions) => todo!(),
            Expression::Property(expression, ident, _) => {
                //println!("property access");
                infer_expr(&*expression, variables, types, dummy_span, root, max_uniformity, md);
                //println!("lhs type: {:#?}", types[i+1]);
                if types[i+1].ty == TypeVariant::Error {
                    return;
                }
                let real_type;
                match &types[i+1].ty {
                    TypeVariant::Pointer(pointer) => {
                        real_type = &*pointer.ty;
                    },
                    TypeVariant::Reference(reference) => {
                        real_type = &*reference.ty;
                    },
                    TypeVariant::Vector(_) => {
                        real_type = &types[i+1];
                    },
                    TypeVariant::Struct(_) => {
                        real_type = &types[i+1]
                    },
                    // TypeVariant::Item(item_path) => {
                    //     match root.lookup_path(item_path).as_ref().unwrap() {
                    //         ModuleItem::Struct(st) => {
                    //             real_type = &types[i+1];
                    //         },
                    //         ModuleItem::Type(ty) => {
                    //             if let Some(v) = ty.ty.is_vector() {
                    //                 real_type = &ty;
                    //             } else {
                    //                 panic!("Property access can only be used on structs and vectors")
                    //             }
                    //         },
                    //         _ => panic!("Property access can only be used on structs and vectors")
                    //     }
                    // },
                    _ => {
                        panic!("Property access can only be used on structs and vectors")
                    }
                }
                match &real_type.ty {
                    TypeVariant::Struct(s) => {
                        let s = root.lookup_path(s).unwrap();
                        let s = match s {
                            ModuleItem::Struct(s) => s,
                            _ => panic!("Member access can only be used on structs and vectors")
                        };
                        let fty = &s.fields.get(ident).expect("Invalid struct field").ty;
                        types[i] = fty.clone();
                        types[i].uni = types[i+1].uni.clone();
                        
                        //todo!("struct member access")
                    },
                    TypeVariant::Vector(vector) => {
                        if ident.str.len() != 1 {
                            todo!("vector subselection & swizzling")
                        } else {
                            match ident.str.chars().next().unwrap() {
                                'x' | 'y' | 'z' | 'w' => {
                                    let uni = real_type.uni.clone();
                                    types[i].ty = TypeVariant::Primitive(vector.ty);
                                    types[i].uni = uni;
                                },
                                _ => panic!("Invalid vector component")
                            }
                        }
                    },
                    TypeVariant::Error => {},
                    _ => panic!("Property access can only be used on structs and vectors")
                }
                resolve_item(root, &mut types[i].ty);
            },
            Expression::Call(expression, expressions) => {
                todo!();
                
                resolve_item(root, &mut types[i].ty);
            },
            Expression::Index(expr, index) => {
                infer_expr(expr, variables, types, dummy_span, root, max_uniformity, md);
                let indexi = types.len();
                infer_expr(index, variables, types, dummy_span, root, max_uniformity, md);
                
                let exprt = &types[i+1];
                
                let rest;
                match &exprt.ty {
                    TypeVariant::Pointer(pointer) => {
                        rest = &*pointer.ty
                    },
                    // TODO arrays
                    TypeVariant::Error => {
                        return;
                    },
                    _ => {
                        panic!("only pointers and arrays can be indexed")
                    }
                }
                
                
                
                let indext = &types[indexi];
                if indext.ty == TypeVariant::Error {
                    return;
                }
                match indext.ty {
                    TypeVariant::Primitive(primitive) => {},
                    TypeVariant::Error => return,
                    _ => {return}
                }
                types[i] = rest.clone();
                resolve_item(root, &mut types[i].ty);
            },
            Expression::Cast(expression, _) => todo!(),
            Expression::Unsafe(block) => {
                infer_block(&block, variables, types, dummy_span, root, max_uniformity, md);
                types[i] = types[i+1].clone();
            },
        }
        //resolve_item(root, &mut types[i].ty);
    }
    
    fn infer_scope(scope: &Scope, path: &ItemPath, root: &Scope, mut md: &mut Metadata) {
        let dummy_span = SourceSpan {
            file: Rc::new(PathBuf::from("compiler.rsl")),
            start: SourcePos::ZERO,
            end: SourcePos::ZERO,
        };
        
        for i in &scope.items {
            if let ModuleItem::Function(f) = i.1 {
                let mut variables: HashMap<Ident, Type> = HashMap::new();
                let mut types = vec![];
                let mut path = path.clone();
                path.segments.push((i.0.clone(), dummy_span.clone(), vec![]));
                let mut mdo = Some((md, &path));
                let ret = infer_block(&f.block, &mut variables, &mut types, &dummy_span, root, &f.uni, &mut mdo);
                md = mdo.unwrap().0;
                *f.expr_types.borrow_mut() = types;
                *f.local_types.borrow_mut() = variables;
            }
        }
        for module in scope.scopes() {
            let m = &*module.1;
            // TODO check submodules
            let mut path = path.clone();
            path.segments.push((module.0.clone(), dummy_span.clone(), vec![]));
            infer_scope(m, &path, root, md);
        }
    }
    
    
    // TODO resolve types of constants in one pass
    // TODO resolve types of statics in one pass
    // Or require their type to be fully specified
    
    
    //let functions = vec![];
    
    // the root scope is always unchecked, since that should only contain the magic compiler builtins
    let scopes = root.scopes().collect::<HashMap<&Ident, &Scope>>();
    for module in order {
        let m = scopes.get(module).unwrap();
        // TODO check submodules
        let path = ItemPath {
            global: true,
            segments: vec![(module.clone(), dummy_span.clone(), vec![])]
        };
        infer_scope(m, &path, root, md);
    }
    
    
    
    
    
}



