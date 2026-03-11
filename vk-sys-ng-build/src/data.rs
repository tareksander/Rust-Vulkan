//! Data structures for a parsed Vulkan XML file.

use std::{collections::HashMap, path::PathBuf};

use bitflags::bitflags;


#[derive(Debug)]
pub struct VKRegistry {
    pub platforms: Vec<VKPlatform>,
    // TODO tags
    pub types: HashMap<String, VKTypeDefinition>,
    
    pub commands: HashMap<String, VKCmd>,
    
    pub enums: HashMap<String, VKEnum>,
    
    
    
}


#[derive(Debug, Clone)]
pub struct VKEnum {
    pub ty: VKEnumType,
    pub bitwidth: u16,
    pub variants: Vec<VKEnumVariant>,
}


#[derive(Debug, Clone, Copy)]
pub enum VKEnumType {
    /// Actually not an enum, but a group of constants
    Constants,
    Enum,
    Bitmask
}

#[derive(Debug, Clone)]
pub struct VKEnumVariant {
    pub name: String,
    pub value: Option<String>,
    pub bitpos: Option<u16>,
    pub deprecated: EnumDeprecation,
    pub ty: Option<CPrimitive>,
    pub alias: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VKPlatform {
    /// Platform name identifier
    pub name: String,
    /// C macro define name guarding the definitions
    pub protect: String,
}

#[derive(Debug, Clone)]
pub struct VKTypeDefinition {
    pub requires: Option<String>,
    pub deprecated: TypeDeprecation,
    pub kind: VKTypeDefinitionKind,
}

#[derive(Debug, Clone, Copy)]
pub enum TypeDeprecation {
    /// Not deprecated
    None,
    /// Deprecated as legacy, with another API superseding it
    Legacy,
    /// Deprecated as not following naming conventions
    Aliased,
}

#[derive(Debug, Clone, Copy)]
pub enum EnumDeprecation {
    /// Not deprecated
    None,
    /// Functionality is ignored
    Ignored,
    /// Deprecated as not following naming conventions
    Aliased,
    /// Deprecated without explanation
    Deprecated,
}


#[derive(Debug, Clone)]
pub enum VKTypeDefinitionKind {
    /// The C header include path, as well as the optional arbitrary C code content
    Include(String),
    /// A struct represented as member-type pairs
    Struct(VKStruct),
    /// A union represented as member-type pairs
    Union(VKStruct),
    /// A "base type" according to the spec.
    BaseType(CTypeDef),
    /// The raw C code of the define
    Define(String),
    /// Aliased to the provided type name
    Alias(String),
    /// A handle type
    Handle {
        dispatchable: bool
    },
    /// A function pointer type
    FunctionPointer(CFunc),
    /// The name is used to get the actual enum from the enums list.
    Enum,
    /// A dependency on another type, mostly meaning that a type is defined in an included C header instead of in the spec.
    Dependency
}

#[derive(Debug, Clone)]
pub struct VKStruct {
    /// Whether duplicates of this struct in pNext chains are allowed
    pub allow_duplicate: bool,
    pub required_limit_type: bool,
    pub returned_only: bool,
    pub struct_extends: Vec<String>,
    pub members: Vec<VKStructMember>,
}



#[derive(Debug, Clone)]
pub struct VKStructMember {
    pub name: String,
    pub ty: CType,
    pub stride: Option<String>,
    pub len: Vec<ParamArrayLen>,
    pub optional: Vec<bool>,
    pub selector: Option<String>,
    pub selection: Vec<String>,
    pub externsync: Externsync,
    pub valid_structs: Vec<String>,
    pub feature_link: Option<String>,
    pub limit_type: Option<LimitType>,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum Externsync {
    Yes,
    No,
    Maybe,
}

#[derive(Debug, Clone, Copy)]
pub enum InRenderpass {
    Yes,
    No,
    Both,
}


bitflags! {
    
    #[derive(Debug, Clone, Copy)]
    pub struct LimitType : u16 {
        const Min = 1 << 0;
        const Max = 1 << 1;
        const Pot = 1 << 2;
        const Mul = 1 << 3;
        const Bits = 1 << 4;
        const Bitmask = 1 << 5;
        const Range = 1 << 6;
        const Struct = 1 << 7;
        const Exact = 1 << 8;
        const NoAuto = 1 << 9;
    }
    
    #[derive(Debug, Clone, Copy)]
    pub struct CommandTask: u8 {
        const Action = 1 << 0;
        const Indirect = 1 << 1;
        const State = 1 << 2;
        const Sync = 1 << 3;
    }
}


#[derive(Debug, Clone)]
pub enum ParamArrayLen {
    One,
    Named(String),
    NullTerminated,
    Custom(String),
}


#[derive(Debug, Clone, Copy)]
pub enum CmdBufferLevel {
    Primary,
    Secondary,
    Both,
}

#[derive(Debug, Clone)]
pub struct VKCmdDefinition {
    pub tasks: CommandTask,
    pub queues: Vec<String>,
    pub success_codes: Vec<String>,
    pub error_code: Vec<String>,
    pub in_renderpass: InRenderpass,
    pub level: CmdBufferLevel,
    pub signature: CFunc,
}


#[derive(Debug, Clone)]
pub enum VKCmd {
    Alias(String),
    Definition(VKCmdDefinition),
}


#[derive(Debug, Clone)]
pub struct CFunc {
    pub ret: CType,
    pub params: Vec<VKStructMember>,
}

#[derive(Debug, Clone)]
pub enum CTypeDef {
    Typedef(CType),
    Struct(Vec<CTypeDef>),
    /// An opaque struct.
    Opaque,
    // This isn't used in the spec
    //Union(Vec<CTypeDef>),
}

#[derive(Debug, Clone)]
pub enum CType {
    Ptr {
        mutable: bool,
        ty: Box<CType>,
    },
    Array {
        ty: Box<CType>,
        size: CArraySize,
    },
    Primitive(CPrimitive),
    Bitfield(Box<CType>, u32),
    Named(String),
}

#[derive(Debug, Clone)]
pub enum CArraySize {
    /// Reference to a constant defined in the spec.
    Name(String),
    Value(u64),
}


#[derive(Debug, Clone, Copy)]
pub enum CPrimitive {
    U(u16),
    I(u16),
    F(u16),
    SizeT,
    SSizeT,
    Void,
    Char,
}




































