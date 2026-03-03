use std::collections::{HashMap, HashSet};

use rsl_data::internal::{Attribute, Builtin, Mutability, ShaderType, StorageClass, StringTable, ast::TokenRange, ir::{Function, GlobalItem, IRID, IRInstruction, Primitive, SymbolTable, Type}};
use rspirv::{binary::Assemble, dr::Operand, spirv::{self, AddressingModel, Capability, Decoration, ExecutionModel, FunctionControl, MemoryAccess, MemoryModel}};




trait SpirvStorage {
    fn spirv(&self) -> rspirv::spirv::StorageClass;
}

impl SpirvStorage for StorageClass {
    fn spirv(&self) -> rspirv::spirv::StorageClass {
        match self {
            StorageClass::Input => rspirv::spirv::StorageClass::Input,
            StorageClass::Output => rspirv::spirv::StorageClass::Output,
            StorageClass::Function => rspirv::spirv::StorageClass::Function,
            StorageClass::Private => rspirv::spirv::StorageClass::Private,
            StorageClass::Push => rspirv::spirv::StorageClass::PushConstant,
            StorageClass::Storage => rspirv::spirv::StorageClass::StorageBuffer,
            StorageClass::PhysicalStorage => rspirv::spirv::StorageClass::PhysicalStorageBuffer,
            StorageClass::Workgroup => rspirv::spirv::StorageClass::Workgroup,
            StorageClass::Uniform => rspirv::spirv::StorageClass::Uniform,
            StorageClass::UniformConstant => rspirv::spirv::StorageClass::UniformConstant,
            StorageClass::Logical => unreachable!(),
        }
    }
}


#[derive(Clone, Copy)]
struct TypeLayout {
    alignment: u16,
    size: u16
}

struct SpirvTypeCache<'a> {
    cache: HashMap<(Type, bool), u32>,
    layout_cache: HashMap<Type, TypeLayout>,
    struct_cache: HashMap<Vec<Type>, (u32, TypeLayout)>,
    array_cache: HashMap<u32, u32>,
    sym: &'a SymbolTable,
}


impl<'a> SpirvTypeCache<'a> {
    pub fn new(sym: &'a SymbolTable) -> Self {
        Self {
            cache: HashMap::new(),
            layout_cache: HashMap::new(),
            struct_cache: HashMap::new(),
            array_cache: HashMap::new(),
            sym,
        }
    }
    
    pub fn spirv_struct(&mut self, b: &mut rspirv::dr::Builder, ty: &[Type]) -> (u32, TypeLayout) {
        if let Some(s) = self.struct_cache.get(ty) {
            return *s;
        }
        let mut types = Vec::with_capacity(ty.len());
        for t in ty {
            types.push(self.get(b, t, true));
        }
        
        let sty = b.type_struct(types);
        b.decorate(sty, Decoration::Block, []);
        let mut max_align = 0;
        let mut offset: u16 = 0;
        for i in 0..ty.len() {
            let meta = self.layout(b, &ty[i]);
            offset = offset.next_multiple_of(meta.alignment);
            b.member_decorate(sty, i as u32, Decoration::Offset, [Operand::LiteralBit32(offset as u32)]);
            offset += meta.size;
            max_align = max_align.max(meta.alignment);
        }
        let size = offset.next_multiple_of(max_align);
        let ret = (sty, TypeLayout {
            alignment: max_align,
            size,
        });
        self.struct_cache.insert(ty.to_vec(), ret);
        return ret;
    }
    
    pub fn layout(&mut self, b: &mut rspirv::dr::Builder, ty: &Type) -> TypeLayout {
        if let Some(layout) = self.layout_cache.get(ty) {
            return *layout;
        }
        match ty {
            Type::Unresolved(item_path) => unreachable!(),
            Type::Resolved(symbol_id) => todo!(),
            Type::Primitive(primitive) => {
                match primitive {
                    Primitive::U8 => todo!(),
                    Primitive::U16 => todo!(),
                    Primitive::U32 => TypeLayout { alignment: 4, size: 4 },
                    Primitive::U64 => todo!(),
                    Primitive::I8 => todo!(),
                    Primitive::I16 => todo!(),
                    Primitive::I32 => todo!(),
                    Primitive::I64 => todo!(),
                    Primitive::F16 => todo!(),
                    Primitive::F32 => todo!(),
                    Primitive::F64 => todo!(),
                    Primitive::Bool => todo!(),
                    Primitive::Unit => unreachable!(),
                }
            },
            Type::Vector { components, ty } => todo!(),
            Type::Matrix { rows, cols, ty } => todo!(),
            Type::Array { length, ty } => todo!(),
            Type::UnresolvedArray { length, ty } => unreachable!(),
            Type::RuntimeArray { ty } => todo!(),
            Type::Pointer { class, ty, mutability } => {
                if *class != StorageClass::PhysicalStorage {
                    unreachable!()
                }
                TypeLayout { alignment: 8, size: 8 }
            },
            Type::Reference { class, ty, mutability } => todo!(),
        }
    }
    
    pub fn get(&mut self, b: &mut rspirv::dr::Builder, ty: &Type, mut block: bool) -> u32 {
        match ty {
            Type::Primitive(primitive) => {
                block = false;
            },
            Type::Vector { components, ty } => {
                block = false;
            },
            Type::Matrix { rows, cols, ty } => {
                block = false;
            },
            Type::Pointer { class, ty, mutability } => {
                block = false;
            }
            _ => {}
        }
        if let Some(id) = self.cache.get(&(ty.clone(), block)) {
            return *id;
        }
        let id = match ty {
            Type::Resolved(symbol_id) => todo!(),
            Type::Primitive(primitive) => {
                match primitive {
                    Primitive::U8 => b.type_int(8, 0),
                    Primitive::U16 => b.type_int(16, 0),
                    Primitive::U32 => b.type_int(32, 0),
                    Primitive::U64 => b.type_int(64, 0),
                    Primitive::I8 => b.type_int(8, 1),
                    Primitive::I16 => b.type_int(16, 1),
                    Primitive::I32 => b.type_int(32, 1),
                    Primitive::I64 => b.type_int(64, 1),
                    Primitive::F16 => b.type_float(16, None),
                    Primitive::F32 => b.type_float(32, None),
                    Primitive::F64 => b.type_float(64, None),
                    Primitive::Bool => b.type_bool(),
                    Primitive::Unit => b.type_void(),
                }
            },
            Type::Vector { components, ty } => {
                let ct = self.get(b, &Type::Primitive(*ty), block);
                b.type_vector(ct, *components as u32)
            },
            Type::Matrix { rows, cols, ty } => todo!(),
            Type::Array { length, ty } => todo!(),
            Type::RuntimeArray { ty } => todo!(),
            Type::Pointer { class, ty, mutability } => {
                let pointee_block = class.explicit_layout();
                let pointee = self.get(b, &**ty, pointee_block);
                b.type_pointer(None, class.spirv(), pointee)
            },
            Type::Reference { class, ty, mutability } => todo!(),
            Type::UnresolvedArray { length, ty } => unreachable!(),
            Type::Unresolved(item_path) => unreachable!(),
        };
        self.cache.insert((ty.clone(), block), id);
        
        return id;
    }
    
    
    fn spirv_rtarrayp(&mut self, b: &mut rspirv::dr::Builder, ty: u32, meta: TypeLayout) -> u32 {
        if let Some(id) =  self.array_cache.get(&ty) {
            return *id;
        }
        
        let at = b.type_runtime_array(ty);
        b.decorate(at, Decoration::ArrayStride, [Operand::LiteralBit32(meta.size.next_multiple_of(meta.alignment) as u32)]);
        let apt = b.type_pointer(None, spirv::StorageClass::PhysicalStorageBuffer, at);
        
        self.array_cache.insert(ty, apt);
        return apt;
    }
    
    
}



struct GLSLInsts {
    
    
    
    
    
    
    
    
}



struct EmitData<'a> {
    b: &'a mut rspirv::dr::Builder,
    sym: &'a SymbolTable,
    types: &'a mut SpirvTypeCache<'a>,
    builtins: &'a mut HashMap<Builtin, u32>,
    strings: &'a StringTable,
    u32zero: u32,
}

impl<'a> EmitData<'a> {
    
    fn new(b: &'a mut rspirv::dr::Builder, sym: &'a SymbolTable, types: &'a mut SpirvTypeCache<'a>, builtins:&'a mut HashMap<Builtin, u32>, strings: &'a StringTable ) -> Self {
        let u32t = types.get(b, &Type::Primitive(Primitive::U32), false);
        let u32zero = b.constant_bit32(u32t, 0);
        EmitData {
            b,
            sym,
            types,
            builtins,
            strings,
            u32zero,
        }
    }
    
    fn get_type(&mut self, ty: &Type, block: bool) -> u32 {
        self.types.get(self.b, ty, block)
    }
    
    fn spirv_struct(&mut self, ty: &[Type]) -> (u32, TypeLayout) {
        self.types.spirv_struct(self.b, ty)
    }
    
    fn layout(&mut self, ty: &Type) -> TypeLayout {
        self.types.layout(self.b, ty)
    }
    
    fn spirv_rtarrayp(&mut self, ty: u32, meta: TypeLayout) -> u32 {
        self.types.spirv_rtarrayp(self.b, ty, meta)
    }
}


pub fn emit_spirv(sym: &mut SymbolTable, strings: &StringTable) -> Vec<u32> {
    // Module setup. A lot of the capabilities could be optional in the future, but they're all required for now.
    let mut b = rspirv::dr::Builder::new();
    {
        b.memory_model(AddressingModel::PhysicalStorageBuffer64, MemoryModel::Vulkan);
        
        b.extension("SPV_KHR_maximal_reconvergence");
        b.extension("SPV_KHR_subgroup_rotate");
        b.extension("SPV_KHR_subgroup_vote");
        b.extension("SPV_KHR_shader_ballot");
        b.extension("SPV_KHR_workgroup_memory_explicit_layout");
        b.extension("SPV_KHR_untyped_pointers");
        b.extension("SPV_KHR_float_controls2");
        //b.extension("SPV_KHR_compute_shader_derivatives");
        //b.extension("SPV_KHR_quad_control");
        //b.extension("SPV_KHR_shader_clock");
        
        
        // Todo: import GLSL math instructions
        
        
        
        
        b.capability(Capability::Shader);
        b.capability(Capability::Matrix);
        b.capability(Capability::VulkanMemoryModel);
        b.capability(Capability::PhysicalStorageBufferAddresses);
        b.capability(Capability::UniformDecoration);
        b.capability(Capability::ShaderNonUniform);
        b.capability(Capability::DrawParameters);
        b.capability(Capability::VariablePointersStorageBuffer);
        b.capability(Capability::VariablePointers);
        b.capability(Capability::UntypedPointersKHR);
        b.capability(Capability::FloatControls2);
        
        b.capability(Capability::Int16);
        b.capability(Capability::Int8);
        b.capability(Capability::Float16);
        
        b.capability(Capability::Groups);
        b.capability(Capability::SubgroupBallotKHR);
        b.capability(Capability::SubgroupVoteKHR);
        b.capability(Capability::GroupNonUniform);
        b.capability(Capability::GroupNonUniformArithmetic);
        b.capability(Capability::GroupNonUniformVote);
        b.capability(Capability::GroupNonUniformBallot);
        b.capability(Capability::GroupNonUniformRotateKHR);
        b.capability(Capability::GroupNonUniformShuffle);
        b.capability(Capability::GroupNonUniformShuffleRelative);
        
        b.capability(Capability::WorkgroupMemoryExplicitLayoutKHR);
        b.capability(Capability::WorkgroupMemoryExplicitLayout16BitAccessKHR);
        b.capability(Capability::WorkgroupMemoryExplicitLayout8BitAccessKHR);
        b.capability(Capability::UniformAndStorageBuffer16BitAccess);
        b.capability(Capability::UniformAndStorageBuffer8BitAccess);
        b.capability(Capability::StoragePushConstant16);
        b.capability(Capability::StoragePushConstant8);
        b.capability(Capability::StorageInputOutput16);
        b.capability(Capability::StorageUniform16);
        
        b.capability(Capability::UniformBufferArrayDynamicIndexing);
        b.capability(Capability::UniformBufferArrayNonUniformIndexing);
        b.capability(Capability::StorageBufferArrayDynamicIndexing);
        b.capability(Capability::StorageBufferArrayNonUniformIndexing);
        b.capability(Capability::StorageImageArrayDynamicIndexing);
        b.capability(Capability::StorageImageArrayNonUniformIndexing);
        b.capability(Capability::SampledImageArrayDynamicIndexing);
        b.capability(Capability::SampledImageArrayNonUniformIndexing);
    }
    
    // TODO simply declare all builtins that are valid for each stage for each entrypoint and see if spirv-opt filters them out
    
    let mut types = SpirvTypeCache::new(sym);
    let mut builtins = HashMap::new();
    builtins.insert(Builtin::GlobalInvocationId, b.id());
    
    let mut builtins_defined: HashSet<Builtin> = HashSet::new();
    
    let mut d = EmitData::new(&mut b, sym, &mut types, &mut builtins, strings);
    
    
    let mut entrypoints: Vec<((u32, Option<u32>), ShaderType)> = vec![];
    
    for s in sym.iter() {
        match &sym.get(s).1 {
            GlobalItem::Static { attrs, ident_token, uni, ty } => {
                // TODO handle other builtins
                let mut handle_builtin = |builtin: Builtin| {
                    if attrs.contains(&Attribute::Builtin(builtin.clone())) && ! builtins_defined.contains(&builtin) {
                        let ty = d.types.get(d.b, &Type::Pointer { class: StorageClass::Input, ty: Box::new(ty.clone()), mutability: Mutability::Immutable }, false);
                        d.b.variable(ty, Some(d.builtins[&builtin]), spirv::StorageClass::Input, None);
                        builtins_defined.insert(builtin);
                    }
                };
                
                handle_builtin(Builtin::GlobalInvocationId);
            }
            GlobalItem::Function(f) => {
                let is_compute_entry = f.attrs.contains(&Attribute::Compute) ;
                
                let id = emit_function(f, &mut d, is_compute_entry);
                if is_compute_entry {
                    let mut interface = vec![d.builtins[&Builtin::GlobalInvocationId]];
                    if let Some(push) = id.1 {
                        interface.push(push);
                    }
                    d.b.entry_point(ExecutionModel::GLCompute, id.0, "test", interface);
                }
            }
            
            _ => {}
        }
    }
    
    
    
    
    return b.module().assemble();
}


fn emit_function(f: &Function, d: &mut EmitData, entrypoint: bool) -> (u32, Option<u32>) {
    let blocks = f.blocks.borrow();
    let types = f.types.borrow();
    
    // TODO remove parameters from function type for entrypoints
    
    let rty = d.get_type(&f.ret.0, false);
    let ret_unit = match f.ret.0 {
        Type::Primitive(Primitive::Unit) => true,
        _ => false,
    };
    
    let mut param_types_spirv = Vec::with_capacity(f.num_params);
    let mut param_types = Vec::with_capacity(f.num_params);
    // let mut param_offsets = Vec::with_capacity(if entrypoint {
    //     f.num_params
    // } else {
    //     0
    // });
    if f.num_params != 0 {
        for i in 0..f.num_params {
            let t = match &types[&IRID(i)] {
                Type::Pointer { class, ty, mutability } => {
                    param_types.push(ty.clone());
                    d.get_type(&*ty, entrypoint)
                }
                _ => unreachable!()
            };
            param_types_spirv.push(t);
        }
    }
    
    let fty = if entrypoint {
        d.b.type_function(rty, vec![])
    } else {
        d.b.type_function(rty, param_types_spirv.iter().cloned())
    };
    
    let mut push_var = None;
    let mut idmap: HashMap<IRID, u32> = HashMap::new();
    
    if entrypoint && f.num_params != 0 {
        let sty = d.b.type_struct(param_types_spirv.iter().cloned());
        let styp = d.b.type_pointer(None, spirv::StorageClass::PushConstant, sty);
        let ps = d.b.variable(styp, None, spirv::StorageClass::PushConstant, None);
        push_var = Some(ps);
        // TODO proper struct calculation
        d.b.decorate(sty, Decoration::Block, []);
        for i in 0..f.num_params {
            d.b.member_decorate(sty, i as u32, Decoration::Offset, [Operand::LiteralBit32((i*8) as u32)]);
        }
    }
    
    let fid = d.b.begin_function(rty, None, FunctionControl::empty(), fty).unwrap();
    let mut blockids = Vec::with_capacity(blocks.len());
    for _ in 0..blocks.len() {
        blockids.push(d.b.id());
    }
    
    
    
    for i in 0..blocks.len() {
        d.b.begin_block(Some(blockids[i])).unwrap();
        let start = if entrypoint && i == 0 && f.num_params != 0 {
            let u32t = d.get_type(&Type::Primitive(Primitive::U32), false);
            for i in 0..f.num_params {
                let index = d.b.constant_bit32(u32t, i as u32);
                let ty = d.b.type_pointer(None, spirv::StorageClass::PushConstant, param_types_spirv[i]);
                idmap.insert(IRID(i), d.b.access_chain(ty, None, push_var.clone().unwrap(), [index]).unwrap());
            }
            f.num_params
        } else {
            0
        };
        for inst in start..blocks[i].instructions.len() {
            emit_instruction(&blocks[i].instructions[inst], d, &blockids, &mut idmap, &types);
        }
        if ret_unit {
            emit_instruction(&IRInstruction::Return { token_id: TokenRange::point(0, 0) }, d, &blockids, &mut idmap, &types);
        }
    }
    
    
    
    d.b.end_function().unwrap();
    return (fid, push_var);
}


fn emit_instruction(inst: &IRInstruction, d: &mut EmitData, blockids: &Vec<u32>, idmap: &mut HashMap<IRID, u32>, types: &HashMap<IRID, Type>) {
    match inst {
        IRInstruction::ResolvedPath { path, tokens, id, lvalue } => {
            // TODO: for now just insert the invocationid builtin
            idmap.insert(*id, d.builtins[&Builtin::GlobalInvocationId]);
        },
        IRInstruction::Local { ident, ident_token, id, ty, uni, mutable } => {
            let t = d.get_type(&types[id], false);
            idmap.insert(*id, d.b.variable(t, None, spirv::StorageClass::Function, None));
        },
        IRInstruction::UnOp { inp, op, out, span } => todo!(),
        IRInstruction::BinOp { lhs, op, rhs, out, span } => {
            match op {
                rsl_data::internal::ast::BinOp::Add => {
                    let t = d.get_type(&types[out], false);
                    idmap.insert(*out, d.b.i_add(t, None, idmap[lhs], idmap[rhs]).unwrap());
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
                    let t = d.get_type(&types[out], false);
                    let et;
                    let meta;
                    match &types[lhs] {
                        Type::Pointer { class, ty, mutability } => {
                            meta = d.layout(&**ty);
                            et = d.get_type(&**ty, class.explicit_layout());
                        },
                        _ => unreachable!()
                    };
                    let apt = d.spirv_rtarrayp(et, meta);
                    let base = d.b.bitcast(apt, None, idmap[lhs]).unwrap();
                    idmap.insert(*out, d.b.access_chain(t, None, base, [idmap[rhs]]).unwrap());
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
        IRInstruction::Unit { out } => {},
        IRInstruction::Load { ptr, out } => {
            let meta;
            let sc;
            match &types[ptr] {
                Type::Pointer { class, ty, mutability } => {
                    sc = *class;
                    meta = d.layout(&**ty);
                },
                _ => unreachable!()
            };
            let t = d.get_type(&types[out], false);
            
            if sc != StorageClass::PhysicalStorage {
                idmap.insert(*out, d.b.load(t, None, idmap[ptr],  if sc == StorageClass::Storage {
                    let mut a = MemoryAccess::NON_PRIVATE_POINTER;
                    Some(a)
                } else {
                    None
                }, []).unwrap());
            } else {
                idmap.insert(*out, d.b.load(t, None, idmap[ptr],{
                    // TODO nonprivate pointer x physical storage
                    let mut a = MemoryAccess::ALIGNED | MemoryAccess::NON_PRIVATE_POINTER;
                    
                    Some(a)
                }, [Operand::LiteralBit32(meta.size.next_multiple_of(meta.alignment) as u32)]).unwrap());
            }
        },
        IRInstruction::Store { ptr, value } => {
            let meta;
            let sc;
            match &types[ptr] {
                Type::Pointer { class, ty, mutability } => {
                    sc = *class;
                    meta = d.layout(&**ty);
                },
                _ => unreachable!()
            };
            
            if sc != StorageClass::PhysicalStorage {
                d.b.store(idmap[ptr], idmap[value],   if sc == StorageClass::Storage {
                    let mut a = MemoryAccess::NON_PRIVATE_POINTER;
                    Some(a)
                } else {
                    None
                }, []).unwrap();
            } else {
                d.b.store(idmap[ptr], idmap[value], {
                    // TODO nonprivate pointer x physical storage
                    let mut a = MemoryAccess::ALIGNED | MemoryAccess::NON_PRIVATE_POINTER;
                    
                    Some(a)
                }, [Operand::LiteralBit32(meta.size.next_multiple_of(meta.alignment) as u32)]).unwrap();
            }
        },
        IRInstruction::Property { inp, name, out } => {
            match &types[inp] {
                Type::Unresolved(item_path) => unreachable!(),
                Type::Primitive(primitive) => unreachable!(),
                Type::Resolved(symbol_id) => todo!(),
                Type::Vector { components, ty } => todo!(),
                Type::Matrix { rows, cols, ty } => todo!(),
                Type::Array { length, ty } => todo!(),
                Type::UnresolvedArray { length, ty } => todo!(),
                Type::RuntimeArray { ty } => todo!(),
                Type::Pointer { class, ty, mutability } => {
                    match &**ty {
                        Type::Unresolved(item_path) => unreachable!(),
                        Type::Primitive(primitive) => unreachable!(),
                        Type::Resolved(symbol_id) => todo!(),
                        Type::Vector { components, ty: _ } => {
                            let index = match name.0.get(d.strings).as_str() {
                                "x" => 0,
                                "y" => 1,
                                "z" => 2,
                                "w" => 4,
                                _ => todo!("bad property string: {}", name.0.get(d.strings))
                            };
                            let u32t = d.get_type(&Type::Primitive(Primitive::U32), false);
                            let index = d.b.constant_bit32(u32t, index);
                            let t = d.get_type(&types[out], false);
                            idmap.insert(*out, d.b.access_chain(t, None, idmap[inp], [index]).unwrap());
                        },
                        Type::Matrix { rows, cols, ty } => todo!(),
                        Type::Array { length, ty } => todo!(),
                        Type::UnresolvedArray { length, ty } => todo!(),
                        Type::RuntimeArray { ty } => todo!(),
                        Type::Pointer { class, ty, mutability } => todo!(),
                        Type::Reference { class, ty, mutability } => todo!(),
                    }
                },
                Type::Reference { class, ty, mutability } => todo!(),
            }
        },
        IRInstruction::Call { func, args, out, span } => todo!(),
        IRInstruction::Int { v, id, token_id, ty } => todo!(),
        IRInstruction::Float { v, id, token_id, ty } => todo!(),
        IRInstruction::Cast { inp, out, ty } => todo!(),
        IRInstruction::Spread { inp, out, uni } => todo!(),
        IRInstruction::ReturnValue { id, token_id } => todo!(),
        IRInstruction::Return { token_id } => {
            d.b.ret().unwrap();
        },
        IRInstruction::Loop { header, body, cont, merge, construct } => todo!(),
        IRInstruction::Branch { target_block } => todo!(),
        IRInstruction::If { inp, true_target_block, false_target_block, merge, construct } => todo!(),
        IRInstruction::Phi { out, sources } => todo!(),
        IRInstruction::Path { path, tokens, id, lvalue } => unreachable!(),
    }
}




