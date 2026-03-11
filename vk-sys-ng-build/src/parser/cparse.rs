//! Parsing C types and type definitions found in the spec.

use logos::Logos;

use crate::data::{CArraySize, CPrimitive, CType, CTypeDef};



#[derive(Debug, Default, Clone, PartialEq)]
pub enum CLexerError {
    #[default]
    Other
}

/// Limited C lexer for what's needed for type definitions
#[derive(Logos, PartialEq, Eq, Debug)]
#[logos(skip r"[ \t]+")]
#[logos(error(CLexerError))]
pub enum CToken<'a> {
    #[token("typedef")]
    Typedef,
    #[token("struct")]
    Struct,
    #[token("union")]
    Union,
    #[token("const")]
    Const,
    
    #[regex("[a-zA-Z_][a-zA-Z_0-9]*")]
    Ident(&'a str),
    
    
    #[regex("[0-9]+", |lex| lex.slice().parse::<u64>().unwrap())]
    Int(u64),
    
    #[token("*")]
    Star,
    #[token(";")]
    Semicolon,
    #[token(":")]
    Colon,
    #[token("[")]
    SquareBracketOpen,
    #[token("]")]
    SquareBracketClose,
}


/// Lexes C type definitions to tokens.
/// 
/// IMPORTANT: add elements where you call this to the exclusion list in xmlparse::remove_whitespace_only_nodes. Otherwise identifiers can get combined.
pub fn lex_c<'a>(text: &'a str) -> Vec<CToken<'a>> {
    CToken::lexer(text).collect::<Result<Vec<CToken>, CLexerError>>().unwrap()
}

/// Parses a C type definition (currently only typedefs and opaque structs, since that's all the spec uses (structs and unions are described in xml))
pub fn parse_c_type_def(tokens: &[CToken]) -> CTypeDef {
    let first = tokens.first().expect("Empty C type");
    match first {
        CToken::Typedef => {
            match tokens.get(tokens.len()-1) {
                Some(CToken::Semicolon) => {}
                _ => panic!("Invalid typedef")
            }
            match tokens.get(tokens.len()-2) {
                Some(CToken::Ident(_)) => {}
                _ => panic!("Invalid typedef")
            }
            //println!("{:#?}", tokens);
            let tokens = &tokens[1..(tokens.len()-2)];
            return CTypeDef::Typedef(parse_c_type(tokens));
        },
        CToken::Struct => {
            // opaque struct definition
            match tokens.get(1) {
                Some(CToken::Ident(_)) => {}
                _ => panic!("Invalid opaque struct definition")
            }
            match tokens.get(2) {
                Some(CToken::Semicolon) => {}
                _ => panic!("Invalid opaque struct definition")
            }
            return CTypeDef::Opaque;
        },
        _ => {
            panic!("Invalid type definition start: {:#?}", first);
        }
    }
}


pub fn parse_c_type(tokens: &[CToken]) -> CType {
    let mut i = 0;
    let mut constant = false;
    let mut base: Option<CType> = None;
    while i < tokens.len() {
        if tokens[i] == CToken::Struct {
            i += 1;
            continue;
        }
        if tokens[i] == CToken::Const {
            if ! constant {
                i += 1;
                constant = true;
                continue;
            } else {
                panic!("double const");
            }
        }
        if let CToken::Ident(name) = tokens[i] {
            i += 1;
            let ty = match name {
                "void" => CType::Primitive(CPrimitive::Void),
                
                "size_t" => CType::Primitive(CPrimitive::SizeT),
                "ssize_t" => CType::Primitive(CPrimitive::SSizeT),
                
                "int8_t" => CType::Primitive(CPrimitive::I(8)),
                "int16_t" => CType::Primitive(CPrimitive::I(16)),
                "int32_t" => CType::Primitive(CPrimitive::I(32)),
                "int64_t" => CType::Primitive(CPrimitive::I(64)),
                
                "uint8_t" => CType::Primitive(CPrimitive::U(8)),
                "uint16_t" => CType::Primitive(CPrimitive::U(16)),
                "uint32_t" => CType::Primitive(CPrimitive::U(32)),
                "uint64_t" => CType::Primitive(CPrimitive::U(64)),
                
                "float" => CType::Primitive(CPrimitive::F(32)),
                "double" => CType::Primitive(CPrimitive::F(64)),
                
                _ => CType::Named(name.to_string()),
            };
            if base.is_some() {
                panic!("Base type already there")
            }
            base = Some(ty);
            continue;
        }
        if tokens[i] == CToken::Colon {
            if let Some(ty) = &mut base {
                i += 1;
                let size = match tokens[i] {
                    CToken::Int(i) => i as u32,
                    _ => panic!("no literal after bitfied start")
                };
                i += 1;
                base = Some(CType::Bitfield(Box::new(ty.clone()), size));
                constant = false;
            } else {
                panic!("bitfield without primitive base type");
            }
            continue;
        }
        if tokens[i] == CToken::SquareBracketOpen {
            if let Some(ty) = &mut base {
                i += 1;
                let size;
                match &tokens[i] {
                    CToken::Ident(name) => {
                        size = CArraySize::Name(name.to_string());
                    },
                    CToken::Int(v) => {
                        size = CArraySize::Value(*v);
                    }
                    _ => panic!("array size not an identifier or constant")
                }
                i += 1;
                if tokens[i] == CToken::SquareBracketClose {
                    i += 1;
                } else {
                    panic!("Expression or unclosed array size");
                }
                base = Some(CType::Array { ty: Box::new(ty.clone()), size });
                continue;
            } else {
                panic!("array without base type");
            }
        }
        if tokens[i] == CToken::Star {
            if let Some(ty) = &mut base {
                base = Some(CType::Ptr { mutable: ! constant, ty: Box::new(ty.clone()) });
                constant = false;
            } else {
                panic!("pointer without base type");
            }
            i += 1;
            continue;
        }
        panic!("Invalid C token: {:#?}", tokens[i]);
    }
    //println!("{:#?}", tokens);
    return base.expect("No type found");
}

