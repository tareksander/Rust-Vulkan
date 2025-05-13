// Ignore all warnings during development for tests.
#![cfg_attr(test, allow(warnings))]

use std::{cmp::{max, min, Ordering}, fmt::Display, path::PathBuf, rc::Rc};

use bitflags::bitflags;



pub mod ast;
pub mod mid;
pub mod passes;
pub mod core;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceSpan {
    pub file: Rc<PathBuf>,
    pub start: SourcePos,
    pub end: SourcePos,
}

impl SourceSpan {
    
    
    fn expand(&self, other: &SourceSpan) -> SourceSpan {
        SourceSpan { file: self.file.clone(), start: min(self.start, other.start), end: max(self.end, other.end) }
    }
    
    
}



#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct SourcePos {
    pub line: u32,
    pub character: u32,
}

impl Ord for SourcePos {
    fn cmp(&self, other: &Self) -> Ordering {
        let o = self.line.cmp(&other.line);
        match o {
            Ordering::Equal => {
                self.character.cmp(&other.character)
            }
            _ => {
                o
            }
        }
    }
}

impl PartialOrd for SourcePos {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


impl SourcePos {
    const ZERO: SourcePos = SourcePos {
        line: 0,
        character: 0,
    };
    
    pub fn to(&self, s: &SourceSpan) -> SourceSpan {
        SourceSpan {
            file: s.file.clone(),
            start: *self,
            end: s.end,
        }
    }
}



#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident {
    pub(crate) str: String,
}

// This implementation is for outside users. The compiler itself violates these rules, e.g. by prefixing variables with numbers during lowering.
impl TryFrom<&str> for Ident {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() == 0 {
            return Err(());
        }
        if ! value.is_ascii() {
            return Err(());
        }
        if value.starts_with(|c: char| c.is_ascii_digit()) {
            return Err(());
        }
        for c in value.chars() {
            if ! c.is_ascii() {
                if c.is_ascii_alphanumeric() || c == '_' {
                    continue;
                }
                return Err(());
            }
        }
        return Ok(Self { str: value.to_owned() });
    }
}

impl ToString for Ident {
    fn to_string(&self) -> String {
        self.str.to_owned()
    }
}

impl PartialEq<str> for Ident {
    fn eq(&self, other: &str) -> bool {
        self.str == other
    }
}



#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StructLayout {
    Std430,
    Std140,
    Scalar,
}



#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StorageClass {
    Private,
    Function,
    Workgroup,
    Uniform,
    Image,
    Storage,
    PhysicalStorage,
    Push,
    Input,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Visibility {
    Pub,
    Priv,
    Pack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mutability {
    Mutable,
    Immutable,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Uniformity {
    /// Uniform at workgroup scope
    Uniform,
    /// Uniform at subgroup scope
    SubUniform,
    /// Not uniform at any scope
    NonUniform,
    /// A generic uniformity
    Generic(Ident)
}

impl Uniformity {
    pub fn limit(&self, max: &Uniformity) -> Self {
        match max {
            Uniformity::Uniform => self.clone(),
            Uniformity::SubUniform => {
                if *self == Uniformity::Uniform {
                    max.clone()
                } else {
                    self.clone()
                }
            },
            Uniformity::NonUniform => {
                max.clone()
            },
            Uniformity::Generic(ident) => todo!(),
        }
    }
}


impl PartialOrd for Uniformity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Generic(_), _) => None,
            (_, Self::Generic(_)) => None,
            _ => {
                let s = match self {
                    Uniformity::Uniform => 2,
                    Uniformity::SubUniform => 1,
                    Uniformity::NonUniform => 0,
                    Uniformity::Generic(_) => -1,
                };
                let o = match other {
                    Uniformity::Uniform => 2,
                    Uniformity::SubUniform => 1,
                    Uniformity::NonUniform => 0,
                    Uniformity::Generic(_) => -1,
                };
                Some(s.cmp(&o))
            }
        }
    }
}



impl Display for Uniformity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Uniformity::Uniform => f.write_str("uni"),
            Uniformity::SubUniform => f.write_str("suni"),
            Uniformity::NonUniform => f.write_str("nuni"),
            Uniformity::Generic(ident) => f.write_str(&ident.str),
        }
    }
}




pub trait GetSpan {
    fn span(self) -> SourceSpan;
}


impl<I, T> GetSpan for I where I: Iterator<Item = T>, T: GetSpan {
    fn span(self) -> SourceSpan {
        self.map(|e| e.span()).reduce(|s, e| s.expand(&e)).unwrap()
    }
}


impl GetSpan for &SourceSpan {
    fn span(self) -> SourceSpan {
        self.clone()
    }
}

impl GetSpan for SourceSpan {
    fn span(self) -> SourceSpan {
        self
    }
}






bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Capabilities : u128 {
        const Pointers = 1 << 0;
        const Storage8 = 1 << 1;
        const Uniform8 = 1 << 2;
        const Push8 = 1 << 3;
        const Storage16 = 1 << 4;
        const Uniform16 = 1 << 5;
        const Push16 = 1 << 6;
        const Int8 = 1 << 7;
        const Int16 = 1 << 8;
        const Float16 = 1 << 9;
        const Int64 = 1 << 10;
        const Float64 = 1 << 11;
        const Int64Atomic = 1 << 12;
        const SubgroupReconvergence = 1 << 13;
        const MaximalReconvergence = 1 << 14;
        const ScalarBlockLayout = 1 << 15;
        const SubgroupExtended = 1 << 16;
        const SubgroupRotate = 1 << 17;
        const SubgroupBallot = 1 << 18;
        const SubgroupVote = 1 << 19;
        const UniformStandard = 1 << 20;
        const RelaxedLayout = 1 << 21;
        const StorageImageFormatlessRead = 1 << 22;
        const StorageImageFormatlessWrite = 1 << 23;
        const UniformBufferArrayDynamic = 1 << 24;
        const StorageBufferArrayDynamic = 1 << 25;
        const StorageImageArrayDynamic = 1 << 26;
        const SampledImageArrayDynamic = 1 << 27;
        const InputAttachmentDynamic = 1 << 28;
        const UniformTexelDynamic = 1 << 29;
        const StorageTexelDynamic = 1 << 30;
        const UniformBufferArrayNonUniform = 1 << 31;
        const StorageBufferArrayNonUniform = 1 << 32;
        const StorageImageArrayNonUniform = 1 << 33;
        const SampledImageArrayNonUniform = 1 << 34;
        const InputAttachmentNonUniform = 1 << 35;
        const UniformTexelNonUniform = 1 << 36;
        const StorageTexelNonUniform = 1 << 37;
    }
    
}

impl Capabilities {
    pub fn wgsl() -> Self {
        Self::empty()
    }
    
    
    
    
    
}



