//! This module contains compiler internals which should not be used by end-users, but have to be public to be usable from the other compiler crates

use std::{cell::RefCell, collections::HashMap, fmt::{Debug, Display}, hash::{BuildHasher, Hash, Hasher, RandomState}, path::PathBuf};

use hashbrown::HashTable;
use tokens::TokenType;

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
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InternedString(usize);




pub struct Sources {
    pub source_files: Vec<PathBuf>,
    pub source_strings: Vec<String>
}

pub struct LexerData {
    pub tokens: Vec<TokenType>,
    pub spans: Vec<SourceSpan>,
}


pub struct CompilerData {
    // Input data
    pub input: CompilerInput,
    /// Interned strings.
    pub strings: StringTable,
    
    // lexer & parser interleaved data, since submodules lead to further tokenized files.
    pub sources: RefCell<Sources>,
    
    
    
    
    
}



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSpan {
    pub file: usize,
    pub start: usize,
    pub end: usize,
}





pub trait MaybeSpanned {
    fn span(&self) -> Option<SourceSpan>;
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
















