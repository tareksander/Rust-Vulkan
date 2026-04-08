use std::{ffi::{CStr, CString}, panic::catch_unwind, sync::LazyLock};

use regex::Regex;

use crate::data::{CFunc, CPrimitive, CType, VKEnum, VKEnumVariant, VKRegistry, VKTypeDefinition, VKTypeDefinitionKind};



pub struct Config {
    /// Determines whether the raw bindings are generates as public (to reduce symbol clutter and help automatic imports).
    /// They are still generated because they are used internally by the high-level variant.
    pub raw_bindings: bool,
}

impl CType {
    fn to_rust(&self) -> String {
        match self {
            CType::Ptr { mutable, ty } => {
                let mut out = "*".to_string();
                out += if *mutable {
                    "mut"
                } else {
                    "const"
                };
                out += " ";
                out += &ty.to_rust();
                out
            },
            CType::Array { ty, size } => {
                let mut out = "[".to_string();
                out += &ty.to_rust();
                out += "; ";
                match size {
                    crate::data::CArraySize::Name(s) => {
                        out += s;
                    },
                    crate::data::CArraySize::Value(v) => {
                        out += &v.to_string();
                    },
                }
                out += " as usize]";
                out
            },
            CType::Primitive(cprimitive) => {
                let tmp;
                match cprimitive {
                    crate::data::CPrimitive::U(s) => {
                        tmp = format!("u{}", s);
                        tmp.as_str()
                    },
                    crate::data::CPrimitive::I(s) => {
                        tmp = format!("i{}", s);
                        tmp.as_str()
                    },
                    crate::data::CPrimitive::F(s) => {
                        tmp = format!("f{}", s);
                        tmp.as_str()
                    },
                    crate::data::CPrimitive::SizeT => "usize",
                    crate::data::CPrimitive::SSizeT => "isize",
                    crate::data::CPrimitive::Void => "::std::ffi::c_void",
                    crate::data::CPrimitive::Char => "::std::ffi::c_char",
                }.to_string()
            },
            CType::Bitfield(ctype, _) => "() /* TODO: Bitfields */".to_string(),
            CType::Named(s) => s.clone(),
        }
    }
}

fn append_type(name: &String, ty: &VKTypeDefinition, out: &mut String) {
    if name.starts_with("VkVideo") {
        return;
    }
    match &ty.kind {
        VKTypeDefinitionKind::Include(_) => {},
        VKTypeDefinitionKind::Struct(s) => {
            *out += "\n#[repr(C)]\n";
            *out += &format!("pub struct {} {{\n", name);
            for f in &s.members {
                let mut name = f.name.as_str();
                // rename rust keywords
                if f.name == "type" {
                    name = "ty";
                }
                *out += &format!("\tpub {}: ", name);
                if f.name == "pnext" && s.returned_only {
                    let ty = CType::Ptr { mutable: false, ty: match &f.ty {
                        CType::Ptr { mutable, ty } => ty.clone(),
                        _ => panic!("pnext field isn't a pointer")
                    } };
                    *out += &ty.to_rust();
                } else {
                    *out += &f.ty.to_rust();
                }
                *out += ",\n";
            }
            *out += "}\n";
        },
        VKTypeDefinitionKind::Union(u) => {
            todo!()
        },
        VKTypeDefinitionKind::BaseType(base) => {
            //println!("base: {}", name);
            match base {
                crate::data::CTypeDef::Typedef(ctype) => {
                    *out += &format!("pub type {} = {};", name, ctype.to_rust());
                },
                crate::data::CTypeDef::Struct(ctype_defs) => todo!(),
                crate::data::CTypeDef::Opaque => {
                    *out += &format!("pub struct {} {{ _d: ()}}", name);
                },
            }
        },
        VKTypeDefinitionKind::Define(content) => {
            //println!("macro: {}", name);
        },
        VKTypeDefinitionKind::Alias(alias) => {
            *out += &format!("pub type {} = {};\n", name, alias);
        },
        VKTypeDefinitionKind::Handle { dispatchable } => {
            if *dispatchable {
                *out += &format!("pub type {} = *const ::std::ffi::c_void;\n", name);
            } else {
                *out += &format!("pub type {} = u32;\n", name);
            }
        },
        VKTypeDefinitionKind::FunctionPointer(f) => {
            *out += &format!("pub type {} = {};\n\n", name, f.to_rust());
        },
        VKTypeDefinitionKind::Enum => {},
        VKTypeDefinitionKind::Dependency => {},
    }
    
    
}

impl CFunc {
    fn to_rust(&self) -> String {
        format!("unsafe extern \"C\" fn({}) -> {}", self.params.iter().map(|p| p.ty.to_rust()).collect::<Vec<String>>().join(", "), self.ret.to_rust())
    }
    
    fn to_fn_rust(&self, name: &str) -> String {
        format!("unsafe extern \"C\" fn {}({}) -> {}", name, self.params.iter().map(|p| p.ty.to_rust()).collect::<Vec<String>>().join(", "), self.ret.to_rust())
    }
}

static CONST_BIN_NOT_REWRITE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\(\~([0-9])UL?L?\)").unwrap()
});

fn generate_enums(out: &mut String, registry: &VKRegistry) {
    for (name, e) in &registry.enums {
        match e.ty {
            crate::data::VKEnumType::Constants => {
                for v in &e.variants {
                    //println!("{}", v.name);
                    if v.name.ends_with("_NAME") {
                        if let Some(alias) = &v.alias {
                            *out += &format!("pub const {}: &::std::ffi::CStr = {};\n\n", v.name, alias);
                            continue;
                        }
                        *out += &format!("pub const {}: &::std::ffi::CStr = c{};\n\n", v.name, v.value.as_ref().unwrap());
                        continue;
                    }
                    
                    *out += &format!("pub const {}: {} = {};\n\n", v.name,
                        v.ty.map(|v| CType::Primitive(v)).unwrap_or(CType::Primitive(CPrimitive::U(32))).to_rust(),
                        v.value.clone().map(|v| {
                            if let Some(m) = CONST_BIN_NOT_REWRITE_REGEX.captures(v.as_str()) {
                                let num = m.get(1).unwrap();
                                return format!("!{}", num.as_str());
                            } else {
                                v
                            }
                        }).unwrap_or_else(|| v.alias.clone().unwrap()));
                }
            },
            crate::data::VKEnumType::Enum => {
                let sign = if e.variants.iter().any(|v| v.value.as_ref().is_some_and(|v| v.contains("-"))) {
                    "i"
                } else {
                    "u"
                };
                
                *out += &format!("#[repr({}{})]\n", sign, e.bitwidth);
                *out += &format!("pub enum {} {{\n", name);
                
                for v in &e.variants {
                    if v.alias.is_some() {
                        let aliased = find_enum_rec(e, v);
                        *out += &format!("{} = {},\n", v.name, aliased.value.as_ref().unwrap());
                    } else {
                        *out += &format!("{} = {},\n", v.name, v.value.as_ref().unwrap());
                    }
                }
                
                *out += "}\n";
                
            },
            crate::data::VKEnumType::Bitmask => {
                *out += "::bitflags::bitflags! {\n";
                
                *out += &format!("pub struct {} : u{} {{\n", name, e.bitwidth);
                
                for v in &e.variants {
                    if v.alias.is_some() {
                        let aliased = find_enum_rec(e, v);
                        if let Some(bitpos) = aliased.bitpos.as_ref() {
                            *out += &format!("const {} = 1 << {};\n", v.name, bitpos);
                        } else {
                            *out += &format!("const {} = {};\n", v.name, aliased.value.as_ref().unwrap());
                        }
                    } else {
                        if let Some(bitpos) = v.bitpos.as_ref() {
                            *out += &format!("const {} = 1 << {};\n", v.name, bitpos);
                        } else {
                            *out += &format!("const {} = {};\n", v.name, v.value.as_ref().unwrap());
                        }
                    }
                }
                
                *out += "}\n";
                *out += "}\n";
            },
        }
    }
    
    
}

fn find_enum_rec<'a>(e: &'a VKEnum, v: &'a VKEnumVariant) -> &'a VKEnumVariant {
    let alias = v.alias.as_ref().unwrap().as_str();
    let aliased = e.variants.iter().find(|e| e.name == alias).unwrap();
    if aliased.alias.is_some() {
        return find_enum_rec(e, aliased);
    }
    return aliased;
}

pub fn generate_vk(config: &Config, registry: &VKRegistry) -> String {
    // start with a capacity of 1mb to be sure reallocation isn't going to be too heavy
    let mut out = String::with_capacity(1024 * 1024);
    
    
    out += "/// Raw Vulkan bindings corresponding exactly to the C definitions.\n";
    out += "#[allow(non_snake_case)]\n";
    if config.raw_bindings {
        out += "pub ";
    }
    out += "mod raw {\n";
    out += "const RVK_VARIANT_OFFSET: u32 = 29;\n";
    out += "const RVK_MAJOR_OFFSET: u32 = 22;\n";
    out += "const RVK_MINOR_OFFSET: u32 = 12;\n";
    out += "const RVK_PATCH_OFFSET: u32 = 0;\n\n";
    
    out += "pub const fn VK_MAKE_API_VERSION(variant: u32, major: u32, minor: u32, patch: u32) -> u32 {(variant << RVK_VARIANT_OFFSET) | (major << RVK_MAJOR_OFFSET) | (minor << RVK_MINOR_OFFSET) | patch}\n";
    out += "pub const VK_VERSION_1_4: u32 = VK_MAKE_API_VERSION(0, 1, 4, 0);\n\n";
    
    
    
    for (name, ty) in &registry.types {
        append_type(name, ty, &mut out);
    }
    out += "\n\n";
    generate_enums(&mut out, registry);
    out += "\n\n";
    for (name, cmd) in &registry.commands {
        
        match cmd {
            crate::data::VKCmd::Alias(alias) => {
                out += &format!("pub type PFN_{} = PFN_{};\n\n", name, alias);
            },
            crate::data::VKCmd::Definition(cmd) => {
                out += &format!("pub type PFN_{} = {};\n\n", name, cmd.signature.to_rust());
            },
        }
    }
    
    out += "}";
    
    
    out += "/// Slightly more high-level Vulkan bindings with automatic structure types and pointer chains and handle type safety.\n";
    out += "pub mod vk {\n";
    out += "use super::raw::*;\n\n";
    
    out += "}";
    
    
    
    
    
    
    return out;
}






