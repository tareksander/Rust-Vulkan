//! This module contains compiler internals which should not be used by end-users, but have to be public to be usable from the other compiler crates

use std::{cell::RefCell, collections::HashMap, fmt::{Debug, Display}, hash::{BuildHasher, Hash, Hasher, RandomState}, ops::{Add, Range}, path::PathBuf};

use ast::ModuleData;
use hashbrown::HashTable;
use ir::SymbolTable;
use tokens::Token;

use crate::input::CompilerInput;


pub mod tokens;

pub mod ast;

pub mod ir;





struct StringTableInner {
    /// Hash table containing the index corresponding to a string hash.
    map: HashTable<usize>,
    /// Underlying storage for interned strings.
    strings: Vec<String>,
    hasher: RandomState,
}

pub struct StringTable(RefCell<StringTableInner>);

impl Debug for StringTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("StringTable").field(&self.0.borrow().strings).finish()
    }
}

impl StringTable {
    pub fn new() -> Self {
        Self(RefCell::new(StringTableInner { map: HashTable::with_capacity(1024), strings: Vec::with_capacity(1024), hasher: RandomState::new() }))
    }
    
    pub fn lookup(&self, s: InternedString) -> String {
        self.0.borrow().strings[s.0].clone()
    }
    
    pub fn insert_get(&self, s: &str) -> InternedString {
        let mut d = self.0.borrow_mut();
        let d = &mut*d;
        let hash = {
            let mut h = d.hasher.build_hasher();
            s.hash(&mut h);
            h.finish()
        };
        
        if let Some(i) =  d.map.find(hash, |v| d.strings[*v] == s) {
            return InternedString(*i);
        } else {
            let i = d.strings.len();
            d.strings.push(s.to_string());
            d.map.insert_unique(hash, i, |v| {
                let mut h = d.hasher.build_hasher();
                d.strings[*v].hash(&mut h);
                h.finish()
            });
            return InternedString(i);
        }
    }
    
    pub fn memory(&self) -> usize {
        let d = self.0.borrow();
        return size_of::<usize>() * (d.strings.capacity() + d.map.capacity()) + d.strings.iter().map(|s| s.capacity()).sum::<usize>();
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternedString(usize);


impl InternedString {
    pub fn get(&self, strings: &StringTable) -> String {
        strings.lookup(*self)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StorageClass {
    Input,
    Output,
    Function,
    Private,
    Push,
    Storage,
    PhysicalStorage,
    Workgroup,
    Uniform,
    UniformConstant,
    // The umbrella storage class for anything not physical. Will be converted into concrete storage classes via monomorphization.
    Logical,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Visibility {
    Pub,
    Priv,
    Pack,
}

pub struct Sources {
    pub source_files: Vec<PathBuf>,
    pub source_strings: Vec<String>
}

pub struct LexedFile {
    pub tokens: Vec<Token>,
    pub token_spans: Vec<Range<usize>>,
}


pub struct CompilerData {
    // Input data
    pub input: CompilerInput,
    /// Interned strings.
    pub strings: StringTable,
    
    // lexer & parser interleaved data, since submodules lead to further tokenized files.
    pub sources: RefCell<Sources>,
    
    /// Lexer generated data
    pub lexed: RefCell<Vec<LexedFile>>,
    
    /// Parsed modules, with their global path as the key
    pub parsed_modules: RefCell<HashMap<InternedString, ModuleData>>,
    
    // The global symbol table directly converted from the AST and unresolved, which makes replacing modules easy
    pub symbol_tables: RefCell<Option<SymbolTable>>,
    
    // Further data doesn't need to be cached for now, so end of struct. caching passes over symbol tables
    // could be a good idea, but since they may depend on each other, analysis could be a bit complex, so something for the future.
    // This incremental design should be sufficient for an LSP server with the low amounts of code a new language is going to have.
    // The compiler can be rewritten in a query-based way later.
    
    
}



#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Uniformity {
    Dispatch,
    Workgroup,
    Subgroup,
    Invocation,
    Generic(InternedString),
    Inferred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderType {
    /// Can be called from any shader type
    Generic,
    Compute,
    Vertex,
    Fragment,
    Task,
    Mesh,
    // TODO ray tracing
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutability {
    Immutable,
    Mutable
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Safety {
    Safe,
    Unsafe
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSpan {
    pub file: usize,
    pub start: usize,
    pub end: usize,
}



impl ariadne::Span for SourceSpan {
    type SourceId = usize;

    fn source(&self) -> &Self::SourceId {
        &self.file
    }

    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }
}

pub struct ReportSourceCache(Vec<ariadne::Source>, Vec<PathBuf>);

impl ReportSourceCache {
    pub fn new(s: &Sources) -> Self {
        Self(s.source_strings.iter().map(|s| ariadne::Source::from(s.clone())).collect(), s.source_files.clone())
    }
}


struct DisplayPathBuf(PathBuf);

impl Display for DisplayPathBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0.display(), f)
    }
}

impl ariadne::Cache<usize> for ReportSourceCache {
    type Storage = String;

    fn fetch(&mut self, id: &usize) -> Result<&ariadne::Source<Self::Storage>, impl std::fmt::Debug> {
        if let Some(s) = self.0.get(*id) {
            return Ok(s);
        } else {
            return Err(());
        }
    }

    fn display<'b>(&self, id: &'b usize) -> Option<impl std::fmt::Display + 'b> {
        if let Some(p) = self.1.get(*id) {
            Some(DisplayPathBuf(p.clone()))
        } else {
            None
        }
    }
}




#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Builtin {
    GlobalInvocationId,
    
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Layout {
    Auto,
    Std140,
    Std430,
    Scalar,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Attribute {
    Layout(Layout),
    Builtin(Builtin),
    Compute,
    Exported,
    Unsafe(UnsafeAttribute),
    Lang(LangAttribute),
}



#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnsafeAttribute {
    
}



#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LangAttribute {
    /// Module attribute, enables all other lang attributes.
    Core,
    
    
    
    
    
    
    
    
    
    
    
    
    
    
}







