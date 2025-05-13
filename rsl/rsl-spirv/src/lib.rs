// Ignore all warnings during development for tests.
#![cfg_attr(test, allow(warnings))]

use std::{collections::{HashMap, HashSet}, iter, mem::discriminant, rc::Rc};

use rsl_data::{ast::{expr::Expression, statement::Block, ItemPath}, mid::{Function, Metadata, ModuleItem, Pointer, Scope, Type, TypeVariant}, passes::resolve_item, GetSpan, Ident, SourceSpan, StorageClass, StructLayout, Uniformity};
use rspirv::{binary::Assemble, dr::{Builder, Operand}, spirv::{Decoration, ExecutionMode, ExecutionModel, FunctionControl}};



fn to_spirv_storage_class(s: StorageClass) -> rspirv::spirv::StorageClass {
    use rspirv::spirv::StorageClass::*;
    match s {
        StorageClass::Private => Private,
        StorageClass::Input => Input,
        StorageClass::Function => Function,
        StorageClass::Workgroup => Workgroup,
        StorageClass::Uniform => Uniform,
        StorageClass::Image => Image,
        StorageClass::Storage => StorageBuffer,
        StorageClass::PhysicalStorage => PhysicalStorageBuffer,
        StorageClass::Push => PushConstant,
    }
}


fn ptr_fn(t: Type) -> TypeVariant {
    TypeVariant::Pointer(Pointer {
        mutable: rsl_data::Mutability::Mutable,
        storage_class: Some(StorageClass::Function),
        ty: Box::new(t),
    })
}


pub fn gen_spirv(root: &Scope, md: &Metadata) -> Vec<u32> {
    let mut b = rspirv::dr::Builder::new();
    b.set_version(1, 6);
    
    let GLSLStd = b.ext_inst_import("GLSL.std.450");
    
    let statics = md.static_set.iter().flat_map(|v| v.1.iter().cloned()).collect::<HashSet<ItemPath>>();
    
    let statics_map = statics.iter().map(|i| {
        let id = b.id();
        b.name(id, &i.segments.last().unwrap().0.to_string());
        let s = match root.lookup_path(i).unwrap() {
            ModuleItem::Static(s) => {
                s
            },
            _ => unreachable!()
        };
        let mut storage = StorageClass::Private;
        for atr in &s.attrs {
            match &atr.0 {
                rsl_data::ast::Attribute::Push(value_or_constant) => {
                    storage = StorageClass::Push;
                },
                rsl_data::ast::Attribute::Set(value_or_constant) => todo!(),
                rsl_data::ast::Attribute::Binding(value_or_constant) => todo!(),
                rsl_data::ast::Attribute::Builtin(builtin_variable) => {
                    let builtin = Operand::BuiltIn(match builtin_variable {
                        rsl_data::ast::BuiltinVariable::GlobalInvocationID => {
                            storage = StorageClass::Input;
                            rspirv::spirv::BuiltIn::GlobalInvocationId
                        },
                    });
                    b.decorate(id, Decoration::BuiltIn, vec![builtin]);
                },
                _ => {}
            }
        }
        let mut ty = TypeVariant::Pointer(Pointer { mutable: s.mutability, storage_class: Some(storage), ty: Box::new(s.ty.clone()) });
        resolve_item(root, &mut ty);
        return (i, (id, ty, storage));
    }).collect::<HashMap<_, _>>();
    
    
    let mut all_types = statics.iter().map(|sp| {
        match root.lookup_path(sp).unwrap() {
            ModuleItem::Static(s) => {
                let ty = Type {
                    uni: Some(Uniformity::Uniform),
                    ty: statics_map[sp].1.clone(),
                    span: s.ty.span.clone(),
                };
                ty
            },
            _ => unreachable!()
        }
    }).chain(md.functions.iter().flat_map(|fp| {
        match root.lookup_path(fp).unwrap() {
            ModuleItem::Function(f) => {
                iter::once(f.ret.clone())
                .chain(f.params.iter().map(|p| p.1.clone())).chain(f.expr_types.borrow().iter().cloned())
                .chain(iter::once(Type { uni: None, ty: TypeVariant::Function(fp.clone()), span: f.block.span.clone() }))
                // .chain(f.local_types.borrow().iter().map(|t| {
                //     Type { uni: None, ty: ptr_fn(t.1.clone()), span: t.1.span.clone() }
                // }))
                // TODO leave out values that can't be pointed to in functions
                .chain(f.expr_types.borrow().iter().map(|t| {
                    Type { uni: None, ty: ptr_fn(t.clone()), span: t.span.clone() }
                }))
                .collect::<Vec<_>>()
            },
            _ => unreachable!()
        }
    })).collect::<HashSet<Type>>();
    
    fn add_type_recursively(root: &Scope, all_types: &mut HashSet<Type>, mut ty: Type) {
        resolve_item(root, &mut ty.ty);
        all_types.insert(ty.clone());
        match ty.ty {
            rsl_data::mid::TypeVariant::Pointer(mut pointer) => {
                resolve_item(root, &mut pointer.ty.ty);
                add_type_recursively(root, all_types, *pointer.ty);
            },
            rsl_data::mid::TypeVariant::Reference(mut reference) => {
                resolve_item(root, &mut reference.ty.ty);
                add_type_recursively(root, all_types, *reference.ty);
            },
            rsl_data::mid::TypeVariant::Tuple(item_paths) => todo!(),
            rsl_data::mid::TypeVariant::Struct(item_path) => {
                match root.lookup_path(&item_path).unwrap() {
                    ModuleItem::Struct(s) => {
                        for f in &s.fields {
                            let mut ty = f.1.ty.clone();
                            resolve_item(root, &mut ty.ty);
                            add_type_recursively(root, all_types, ty);
                        }
                    },
                    _ => unreachable!()
                }
            },
            _ => {}
        }
    }
    
    for t in all_types.clone() {
        add_type_recursively(root, &mut all_types, t);
    }
    
    
    
    let all_types = all_types.into_iter().map(|mut t| {
        t.ty
    }).collect::<Vec<TypeVariant>>();
    
    
    let mut type_map: HashMap<TypeVariant, u32> = HashMap::new();
    
    for t in &all_types {
        type_map.insert(t.clone(), b.id());
        match t {
            TypeVariant::Struct(_) => {
                // Reserve 2 consecutive IDs for buffer and non-buffer versions of structs.
                // The first one is the buffer version
                b.id();
            },
            _ => {}
        }
    }
    
    //println!("map: {:#?}", &type_map);
    //println!("all: {:#?}", &all_types);
    
    
    
    let functions = md.functions.iter().map(|p| {
        match root.lookup_path(p).unwrap() {
            ModuleItem::Function(f) => {
                return (p.clone(), (f, b.id(), type_map[&TypeVariant::Function(p.clone())]));
            },
            _ => unreachable!()
        }
    }).collect::<HashMap<_, _>>();
    
    
    {
        use rspirv::spirv::Capability;
        b.capability(Capability::Shader);
        b.capability(Capability::Matrix);
        b.capability(Capability::VariablePointers);
        b.capability(Capability::VariablePointersStorageBuffer);
        b.capability(Capability::PhysicalStorageBufferAddresses);
        b.capability(Capability::VulkanMemoryModel);
        b.capability(Capability::Shader);
    }
    
    b.memory_model(rspirv::spirv::AddressingModel::PhysicalStorageBuffer64, rspirv::spirv::MemoryModel::Vulkan);
    
    
    
    for e in &md.entrypoints {
        match &e.1 {
            rsl_data::ast::Entrypoint::Compute(x, y, z) => {
                let (_, id, tid) = functions[&e.0];
                //println!("{:#?}", md.static_set[e.3]);
                b.entry_point(ExecutionModel::GLCompute, id, e.0.segments.last().unwrap().0.to_string(),
                 md.static_set[&e.0].iter().map(|s| statics_map[s].0).collect::<Vec<_>>());
                let dims = (match x {
                    rsl_data::ast::ValueOrConstant::Value(v, _) => *v,
                    rsl_data::ast::ValueOrConstant::Constant(item_path) => todo!(),
                }, match y {
                    rsl_data::ast::ValueOrConstant::Value(v, _) => *v,
                    rsl_data::ast::ValueOrConstant::Constant(item_path) => todo!(),
                }, match z {
                    rsl_data::ast::ValueOrConstant::Value(v, _) => *v,
                    rsl_data::ast::ValueOrConstant::Constant(item_path) => todo!(),
                });
                
                b.execution_mode(id, ExecutionMode::LocalSize, vec![dims.0, dims.1, dims.2]);
            },
        }
    }
    
    
    // TODO make a map for struct size/alignment requirements
    // Then generate decorations based on that
    
    
    // types & globals
    
    
    let mut type_vector = type_map.iter().collect::<Vec<(&TypeVariant, &u32)>>();
    //println!("vector: {:#?}", &type_vector);
    let mut easy_type_vector = type_vector.iter().filter(|t| match t.0 {
        TypeVariant::Function(_) => false,
        TypeVariant::Pointer(_) => false,
        TypeVariant::Reference(_) => false,
        TypeVariant::Tuple(_) => false,
        TypeVariant::Struct(_) => false,
        _ => true
    }).collect::<Vec<_>>();
    easy_type_vector.sort_by(|a, b| {
        fn precedence(t: &TypeVariant) -> u32 {
            match t {
                TypeVariant::Function(_) => unreachable!(),
                TypeVariant::Pointer(_) => unreachable!(),
                TypeVariant::Reference(_) => unreachable!(),
                TypeVariant::Tuple(_) => unreachable!(),
                TypeVariant::Struct(_) => unreachable!(),
                TypeVariant::Matrix(_) => 4,
                TypeVariant::Vector(_) => 3,
                TypeVariant::Primitive(_) => 2,
                TypeVariant::Unit => 1,
                TypeVariant::Error => unreachable!(),
                TypeVariant::AbstractFloat => unreachable!(),
                TypeVariant::AbstractInt => unreachable!(),
                TypeVariant::Item(_) => unreachable!(),
            }
        }
        let pa = precedence(a.0);
        let pb = precedence(b.0);
        return pa.cmp(&pb);
    });
    
    
    
    
    for t in easy_type_vector {
        match t.0 {
            TypeVariant::Matrix(m) => {
                todo!()
            },
            TypeVariant::Vector(v) => {
                b.type_vector_id(Some(*t.1), type_map[&TypeVariant::Primitive(v.ty)], v.components.into());
            },
            TypeVariant::Primitive(p) => {
                match p {
                    rsl_data::mid::Primitive::U8 => b.type_int_id(Some(*t.1), 8, 0),
                    rsl_data::mid::Primitive::U16 => b.type_int_id(Some(*t.1), 16, 0),
                    rsl_data::mid::Primitive::U32 => b.type_int_id(Some(*t.1), 32, 0),
                    rsl_data::mid::Primitive::U64 => b.type_int_id(Some(*t.1), 64, 0),
                    rsl_data::mid::Primitive::I8 => b.type_int_id(Some(*t.1), 8, 1),
                    rsl_data::mid::Primitive::I16 => b.type_int_id(Some(*t.1), 16, 1),
                    rsl_data::mid::Primitive::I32 => b.type_int_id(Some(*t.1), 32, 1),
                    rsl_data::mid::Primitive::I64 => b.type_int_id(Some(*t.1), 64, 1),
                    rsl_data::mid::Primitive::F16 => b.type_float_id(Some(*t.1), 16),
                    rsl_data::mid::Primitive::F32 => b.type_float_id(Some(*t.1), 32),
                    rsl_data::mid::Primitive::F64 => b.type_float_id(Some(*t.1), 64),
                };
            },
            TypeVariant::Unit => {
                b.type_void_id(Some(*t.1));
            },
            _ => {}
        }
    }
    
    let nontrivial_types = type_vector.iter().filter(|t| match t.0 {
        TypeVariant::Function(_) => true,
        TypeVariant::Pointer(_) => true,
        TypeVariant::Reference(_) => true,
        TypeVariant::Tuple(_) => true,
        TypeVariant::Struct(_) => true,
        _ => false
    }).map(|t| *t).collect::<HashMap<_, _>>();
    
    let mut nontrivial_types_declared: HashSet<u32> = HashSet::new();
    
    let mut struct_field_indices: HashMap<u32, HashMap<Ident, u16>> = HashMap::new();
    
    for t in &nontrivial_types {
        match t.0 {
            TypeVariant::Pointer(p) => {
                if p.storage_class.unwrap() == StorageClass::PhysicalStorage && match &p.ty.ty {
                    TypeVariant::Struct(_) => true,
                    _ => false
                } {
                    b.type_forward_pointer(**t.1, rspirv::spirv::StorageClass::PhysicalStorageBuffer);
                }
            },
            TypeVariant::Reference(r) => {
                todo!()
            },
            _ => {}
        }
    }
    
    fn is_physical_storage_pointer_to_struct(ty: &TypeVariant, type_map: &HashMap<TypeVariant, u32>) -> bool {
        match ty {
            TypeVariant::Pointer(p) => p.storage_class.unwrap() == StorageClass::PhysicalStorage && match &p.ty.ty {
                TypeVariant::Struct(_) => true,
                _ => false
            },
            _ => false
        }
    }
    
    // Entry is missing when the struct contains logical pointers
    let mut struct_size_align: HashMap<u32, (u32, u32)> = HashMap::new();
    
    fn type_size_align(ty: &TypeVariant, layout: StructLayout, struct_size_align: &HashMap<u32, (u32, u32)>, type_map: &HashMap<TypeVariant, u32>) -> Option<(u32, u32)> {
        match ty {
            TypeVariant::Pointer(pointer) => {
                if pointer.storage_class.unwrap() != StorageClass::PhysicalStorage {
                    None
                } else {
                    Some((8, 8))
                }
            },
            TypeVariant::Reference(reference) => todo!(),
            TypeVariant::Tuple(item_paths) => todo!(),
            TypeVariant::Item(item_path) => unreachable!(),
            TypeVariant::Struct(item_path) => {
                struct_size_align.get(&type_map[ty]).copied()
            },
            TypeVariant::Primitive(primitive) => {
                Some(match primitive {
                    rsl_data::mid::Primitive::U8 => (1, 1),
                    rsl_data::mid::Primitive::U16 => (2, 2),
                    rsl_data::mid::Primitive::U32 => (4, 4),
                    rsl_data::mid::Primitive::U64 => (8, 8),
                    rsl_data::mid::Primitive::I8 => (1, 1),
                    rsl_data::mid::Primitive::I16 => (2, 2),
                    rsl_data::mid::Primitive::I32 => (4, 4),
                    rsl_data::mid::Primitive::I64 => (8, 8),
                    rsl_data::mid::Primitive::F16 => (2, 2),
                    rsl_data::mid::Primitive::F32 => (4, 4),
                    rsl_data::mid::Primitive::F64 => (8, 8),
                })
            },
            TypeVariant::Function(item_path) => unreachable!(),
            TypeVariant::AbstractInt => unreachable!(),
            TypeVariant::AbstractFloat => unreachable!(),
            TypeVariant::Vector(vector) => {
                
                todo!()
            },
            TypeVariant::Matrix(matrix) => todo!(),
            TypeVariant::Unit => Some((0, 0)),
            TypeVariant::Error => unreachable!(),
        }
    }
    
    
    fn declare_nontrivial(ty: &TypeVariant, root: &Scope, type_map: &HashMap<TypeVariant, u32>, struct_field_indices: &mut HashMap<u32, HashMap<Ident, u16>>,
            nontrivial_types: &HashMap<&TypeVariant, &u32>, nontrivial_types_declared: &mut HashSet<u32>, b: &mut rspirv::dr::Builder,
            struct_size_align: &mut HashMap<u32, (u32, u32)>) {
        let id = *type_map.get(ty).expect(&format!("No type id for: {:#?}", ty));
        if ! nontrivial_types_declared.contains(&id) {
            match ty {
                TypeVariant::Function(f) => {
                    
                },
                TypeVariant::Tuple(t) => todo!(),
                TypeVariant::Pointer(p) => {
                    let pointee = if let Some(pointee_id) = nontrivial_types.get(&p.ty.ty) {
                        if ! is_physical_storage_pointer_to_struct(&p.ty.ty, type_map)  {
                            declare_nontrivial(&p.ty.ty, root, type_map, struct_field_indices, nontrivial_types, nontrivial_types_declared, b, struct_size_align);
                        }
                        **pointee_id
                    } else {
                        type_map[&p.ty.ty]
                    };
                    b.type_pointer(Some(id), to_spirv_storage_class(p.storage_class.unwrap()), pointee);
                },
                TypeVariant::Reference(r) => {
                    todo!()
                },
                TypeVariant::Struct(sp) => {
                    let s = match root.lookup_path(sp).unwrap() {
                        ModuleItem::Struct(s) => s,
                        _ => unreachable!()
                    };
                    let field_types = s.fields.iter().map(|f| {
                        let mut ty = f.1.ty.ty.clone();
                        resolve_item(root, &mut ty);
                        (f.0, ty)
                    }).collect::<Vec<_>>();
                    
                    for f in &field_types {
                        declare_nontrivial(&f.1, root, type_map, struct_field_indices, nontrivial_types, nontrivial_types_declared, b, struct_size_align);
                    }
                    
                    let mut indices = HashMap::new();
                    for (i, (ident, _)) in field_types.iter().enumerate() {
                        indices.insert((*ident).clone(), i as u16);
                    }
                    struct_field_indices.insert(id, indices);
                    
                    let mut field_offsets = Vec::with_capacity(field_types.len());
                    
                    let mut offset = 0;
                    for (i, f) in field_types.iter().enumerate() {
                        if let Some((ms, ma)) = type_size_align(&f.1, StructLayout::Std430, struct_size_align, type_map) {
                            let to_add = ma - offset % ma;
                            if to_add != ma {
                                offset += to_add;
                            }
                            field_offsets.push(offset);
                            offset += ms;
                        } else {
                            break;
                        }
                    }
                    
                    
                    if field_offsets.len() == field_types.len() {
                        for (i, o) in field_offsets.iter().enumerate() {
                            let f = &field_types[i];
                            b.member_decorate(id, i as u32, Decoration::Offset, vec![Operand::LiteralBit32(*o)]);
                        }
                        b.name(id, &sp.segments.last().unwrap().0.to_string());
                        b.decorate(id, Decoration::Block, vec![]);
                        b.type_struct_id(Some(id), field_types.iter().map(|f| type_map[&f.1]));
                    }
                    b.name(id+1, &sp.segments.last().unwrap().0.to_string());
                    b.type_struct_id(Some(id+1), field_types.iter().map(|f| type_map[&f.1]));
                },
                _ => unreachable!()
            }
            nontrivial_types_declared.insert(id);
        }
    }
    
    for t in &nontrivial_types {
        declare_nontrivial(*t.0, root, &type_map, &mut struct_field_indices, &nontrivial_types,
            &mut nontrivial_types_declared, &mut b, &mut struct_size_align);
    }
    
    for t in &nontrivial_types {
        match t.0 {
            TypeVariant::Function(fp) => {
                let f = match root.lookup_path(fp).unwrap() {
                    ModuleItem::Function(f) => f,
                    _ => unreachable!()
                };
                b.name(**t.1, &(fp.segments.last().unwrap().0.to_string() + "T"));
                b.type_function_id(Some(**t.1), type_map[&f.ret.ty], f.params.iter().map(|p| type_map[&p.1.ty]));
            }
            _ => {}
        }
    }
    
    for s in &statics_map {
        b.variable(type_map[&s.1.1], Some(s.1.0), to_spirv_storage_class(s.1.2), None);
    }
    
    
    // functions
    
    
    let mdp = MetadataPlus {
        md: &md,
        functions: &functions,
        type_map: &type_map,
        struct_field_indices: &struct_field_indices,
        struct_size_align: &struct_size_align,
        statics_map: &statics_map,
    };
    
    
    for func in &functions {
        let (p, (_, id, _)) = func;
        b.name(*id, &p.segments.last().unwrap().0.to_string());
        gen_function(func, &mdp, &mut b);
    }
    
    
    let mut m = b.module();
    // TODO request a tool ID & language from Khronos once the language matures a bit
    //m.header.as_mut().unwrap().generator = 1;
    return m.assemble();
}


struct MetadataPlus<'a> {
    md: &'a Metadata,
    functions: &'a HashMap<ItemPath, (&'a Function, u32, u32)>,
    type_map: &'a HashMap<TypeVariant, u32>,
    struct_field_indices: &'a HashMap<u32, HashMap<Ident, u16>>,
    struct_size_align: &'a HashMap<u32, (u32, u32)>,
    statics_map: &'a HashMap<&'a ItemPath, (u32, TypeVariant, StorageClass)>
}



fn gen_function(f: (&ItemPath, &(&Function, u32, u32)), mdp: &MetadataPlus, b: &mut Builder) {
    let (p, (f, id, tid)) = f;
    let rett = mdp.type_map[&f.ret.ty];
    b.begin_function(rett, Some(*id), FunctionControl::NONE, *tid).unwrap();
    let mut type_index = 0;
    let expr_types = f.expr_types.borrow();
    let mut locals = HashMap::new();
    gen_block(&f.block, mdp, b, &expr_types, &mut type_index, &mut locals, &f.local_types.borrow());
    // Add an empty return to Unit functions. Passes higher up should ensure function blocks always return a value otherwise.
    if b.selected_block().is_some() && f.ret.ty.is_unit() {
        b.ret().unwrap();
    }
    
    b.end_function().unwrap();
}

fn gen_block(block: &Block, mdp: &MetadataPlus, b: &mut Builder, expr_types: &[Type], type_index: &mut usize,
        locals: &mut HashMap<Ident, u32>, local_types: &HashMap<Ident, Type>) -> (u32, Option<u32>) {
    let id = b.begin_block(None).unwrap();
    for s in &block.statements {
        match s {
            rsl_data::ast::statement::Statement::Expression(expression) => {
                gen_expr(expression, mdp, b, expr_types, type_index, locals, local_types);
            },
            rsl_data::ast::statement::Statement::Return(expression) => {
                todo!();
                break;
            },
            rsl_data::ast::statement::Statement::Break(source_span) => todo!(),
            rsl_data::ast::statement::Statement::Continue(source_span) => todo!(),
            rsl_data::ast::statement::Statement::Let(l) => {
                match l {
                    rsl_data::ast::statement::Let::Single(source_span, mutability, ident, _, expression) => {
                        *type_index += 1;
                        let local_id = b.variable(mdp.type_map[&ptr_fn(local_types[&ident].clone())], None, to_spirv_storage_class(StorageClass::Function), None);
                        let local_type = mdp.type_map[&local_types[&ident].ty];
                        locals.insert(ident.clone(), local_id);
                        if let Some(e) = expression {
                            let val = gen_expr(e, mdp, b, expr_types, type_index, locals, local_types).unwrap();
                            b.store(local_id, val, None, vec![]).unwrap();
                        } else {
                            let undef = b.undef(local_type, None);
                            b.store(local_id, undef, None, vec![]).unwrap();
                        }
                    },
                }
            },
        }
    }
    
    
    return (id, None);
}




fn gen_lvalue_expr(expr: &Expression, mdp: &MetadataPlus, b: &mut Builder, expr_types: &[Type], type_index: &mut usize,
    locals: &mut HashMap<Ident, u32>, local_types: &HashMap<Ident, Type>) -> u32 {
    match expr {
        Expression::Property(expression, ident, source_span) => todo!(),
        Expression::Index(lhs, rhs) => {
            let res_t = &expr_types[*type_index];
            // TODO determine mutablilty?
            let res_t_id = mdp.type_map[&TypeVariant::Pointer(Pointer { mutable: rsl_data::Mutability::Mutable, storage_class: Some(StorageClass::PhysicalStorage), ty: Box::new(Type {
                uni: Some(Uniformity::NonUniform),
                ty: res_t.ty.clone(),
                span: expr.span().clone(),
            }) })];
            *type_index += 1;
            let p_t = &expr_types[*type_index];
            let p_id = gen_expr(&lhs, mdp, b, expr_types, type_index, locals, local_types).unwrap();
            let i_t = &expr_types[*type_index];
            let i_id = gen_expr(&rhs, mdp, b, expr_types, type_index, locals, local_types).unwrap();
            
            //let p_t_id = mdp.type_map[&p_t.ty];
            //let i_t_id = mdp.type_map[&i_t.ty];
            
            
            return b.access_chain(res_t_id, None, p_id, vec![i_id]).unwrap();
        },
        Expression::Item(item_path) => todo!(),
        _ => unreachable!()
    }
}


/// Generates an expression and returns the result ID (if the type is not unit)
fn gen_expr(expr: &Expression, mdp: &MetadataPlus, b: &mut Builder, expr_types: &[Type], type_index: &mut usize,
    locals: &mut HashMap<Ident, u32>, local_types: &HashMap<Ident, Type>) -> Option<u32> {
    match expr {
        Expression::Int(source_span, _) => todo!(),
        Expression::Float(source_span, _) => todo!(),
        Expression::Item(item_path) => {
            let t = &expr_types[*type_index];
            let t_id = mdp.type_map[&t.ty];
            *type_index += 1;
            let v_id;
            if item_path.segments.len() == 1 {
                let ident = &item_path.segments.first().unwrap().0;
                v_id = locals[ident];
            } else {
                v_id = mdp.statics_map[item_path].0
            }
            return Some(b.load(t_id, None, v_id, None, vec![]).unwrap());
        },
        Expression::UnOp(source_span, un_op, expression) => todo!(),
        Expression::BinOp(lhs, op, rhs) => {
            match op {
                rsl_data::ast::expr::BinOp::Add => {
                    let t = &expr_types[*type_index];
                    *type_index += 1;
                    let t_id = mdp.type_map[&t.ty];
                    let lhs = gen_expr(&lhs, mdp, b, expr_types, type_index, locals, local_types).unwrap();
                    let rhs = gen_expr(&rhs, mdp, b, expr_types, type_index, locals, local_types).unwrap();
                    // TODO floating point add and type conversions
                    return Some(b.i_add(t_id, None, lhs, rhs).unwrap());
                },
                rsl_data::ast::expr::BinOp::Sub => todo!(),
                rsl_data::ast::expr::BinOp::Mul => todo!(),
                rsl_data::ast::expr::BinOp::Div => todo!(),
                rsl_data::ast::expr::BinOp::Assign => {
                    *type_index += 1;
                    let lhs = gen_lvalue_expr(&lhs, mdp, b, expr_types, type_index, locals, local_types);
                    let rhs = gen_expr(&rhs, mdp, b, expr_types, type_index, locals, local_types).unwrap();
                    b.store(lhs, rhs, None, vec![]).unwrap();
                    return None;
                },
            }
        },
        Expression::If(_) => todo!(),
        Expression::Unit(source_span) => {},
        Expression::Tuple(source_span, expressions) => todo!(),
        Expression::Property(expression, ident, source_span) => {
            let t = &expr_types[*type_index];
            *type_index += 1;
            let et = &expr_types[*type_index];
            let lhs_id = gen_expr(&expression, mdp, b, expr_types, type_index, locals, local_types).unwrap();
            let index;
            match &et.ty {
                TypeVariant::Pointer(pointer) => todo!(),
                TypeVariant::Reference(reference) => todo!(),
                TypeVariant::Tuple(item_paths) => todo!(),
                TypeVariant::Struct(item_path) => {
                    index = mdp.struct_field_indices[&mdp.type_map[&et.ty]][ident] as u32;
                },
                TypeVariant::Vector(vector) => {
                    index = match ident.to_string().as_str() {
                        "x" => 0,
                        "y" => 1,
                        "z" => 2,
                        "w" => 3,
                        _ => todo!()
                    };
                },
                _ => unreachable!()
            }
            let res = b.composite_extract(mdp.type_map[&t.ty], None, lhs_id, vec![index]).unwrap();
            return Some(res);
        },
        Expression::Call(expression, expressions) => todo!(),
        Expression::Index(lhs, rhs) => {
            let res_t = &expr_types[*type_index];
            let res_t_id = mdp.type_map[&res_t.ty];
            // TODO determine mutablilty?
            let res_ptr_t_id = mdp.type_map[&TypeVariant::Pointer(Pointer { mutable: rsl_data::Mutability::Immutable, storage_class: Some(StorageClass::PhysicalStorage), ty: Box::new(Type {
                uni: res_t.uni.clone(),
                ty: res_t.ty.clone(),
                span: expr.span().clone(),
            }) })];
            *type_index += 1;
            let p_t = &expr_types[*type_index];
            let p_id = gen_expr(&lhs, mdp, b, expr_types, type_index, locals, local_types).unwrap();
            let i_t = &expr_types[*type_index];
            let i_id = gen_expr(&rhs, mdp, b, expr_types, type_index, locals, local_types).unwrap();
            
            //let p_t_id = mdp.type_map[&p_t.ty];
            //let i_t_id = mdp.type_map[&i_t.ty];
            let res_ptr_id = b.access_chain(res_ptr_t_id, None, p_id, vec![i_id]).unwrap();
            return Some(b.load(res_t_id, None, res_ptr_id, None, vec![]).unwrap());
            
            
            //return b.access_chain(res_t_id, None, p_id, vec![i_id]).unwrap();
        },
        Expression::Cast(expression, _) => todo!(),
        Expression::Unsafe(block) => todo!(),
    }
    
    return None;
}



#[cfg(test)]
mod tests {
    use std::{fs::{self, OpenOptions}, io::Write, path::PathBuf, process::{exit, Command, Stdio}, rc::Rc};

    use rsl_data::{ast::{parser, tokenizer::tokenize}, mid::Scope, passes::run_passes, Ident, Visibility};

    use super::*;

    #[test]
    fn it_works() {
        let file: Rc<PathBuf> = PathBuf::new().into();
        let tokens = tokenize(r"
        use ::globalInvocationId;
        
        
        struct PushConstants {
            a: *const PhysicalStorage uni u32,
            b: *const PhysicalStorage uni u32,
            c: *mut PhysicalStorage nuni u32,
        }
        
        
        #[push(0)]
        static PUSH: PushConstants;
        
        
        #[compute(1, 1, 1)]
        fn unsafe add() {
            let i = globalInvocationId.x;
            PUSH.c[i] = PUSH.a[i] + PUSH.b[i];
        }
        ", file.clone());
        let scope = Scope::from_ast(parser::parse(tokens.as_slice()));
        let mut root = Scope::root();
        root.items.insert(Ident::try_from("test").unwrap(), ModuleItem::Module(scope));
        let md = run_passes(&mut root);
        let spv = gen_spirv(&root, &md);
        let mut dis = Command::new("spirv-dis");
        dis.stdin(Stdio::piped());
        let mut dis = dis.spawn().unwrap();
        dis.stdin.as_mut().unwrap().write_all(bytemuck::cast_slice(spv.as_slice())).unwrap();
        let res = dis.wait().unwrap();
        if ! res.success() {
            exit(res.code().unwrap());
        }
        let mut val = Command::new("spirv-val");
        val.arg("--target-env");
        val.arg("vulkan1.3");
        val.stdin(Stdio::piped());
        let mut val = val.spawn().unwrap();
        val.stdin.as_mut().unwrap().write_all(bytemuck::cast_slice(spv.as_slice())).unwrap();
        let res = val.wait().unwrap();
        if ! res.success() {
            exit(res.code().unwrap());
        }
        
        let mut f = OpenOptions::new().create(true).write(true).open("test.spv").unwrap();
        f.write_all(bytemuck::cast_slice(&spv)).unwrap();
        
    }
}
