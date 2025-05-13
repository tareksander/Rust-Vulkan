use std::{cell::{Cell, RefCell}, collections::HashMap, fs, mem::{discriminant, take}, path::{Path, PathBuf}, rc::Rc, sync::LazyLock};


use crate::{ast::{expr::{BinOp, Expression, UnOp}, function::Function, module::{ConstantDefinition, Module, StaticDefinition}, statement::{Block, Let, Statement}, token::*, ty::{Pointer, Reference, Type}, Attribute, Entrypoint, GenericArg, ItemPath, Safety, ValueOrConstant}, Ident, Mutability, SourcePos, SourceSpan, StorageClass, Uniformity, Visibility};

use super::{module::Import, structure::{Implementation, StructField, Structure}, traits::Trait, GenericArgDefinition};



struct ParserData<'a> {
    tokens: &'a[Token],
    index: u32,
}


impl<'a> ParserData<'a> {
    
    
    fn take(&mut self) -> &Token {
        let t = &self.tokens[self.index as usize];
        self.index += 1;
        t
    }
    
    fn peek(&self) -> &Token {
        &self.tokens[self.index as usize]
    }
    
    
    
    
    
    
    
    
    
    
}




pub fn parse(tokens: &[Token]) -> Module {
    let mut data = ParserData {
        tokens,
        index: 0,
    };
    parse_module_file(&mut data)
}

fn parse_module(data: &mut ParserData, start: SourceSpan) -> Module {
    let mut inline_modules = vec![];
    let mut functions = vec![];
    let mut statics = vec![];
    let mut constants = vec![];
    let mut vis: Option<(Visibility, SourceSpan)> = None;
    let mut attrs = vec![];
    let mut structs = vec![];
    let mut impls = vec![];
    let mut traits = vec![];
    let mut imports = vec![];
    loop {
        let t = data.peek();
        match t.ty {
            TokenType::End | TokenType::Special(Special::CurlyBracketClose) => {
                let span = start.expand(&t.span);
                //data.take();
                return Module {
                    span,
                    inline_modules,
                    functions,
                    statics,
                    constants,
                    structs,
                    impls,
                    traits,
                    imports,
                }
            },
            TokenType::Keyword(k) => {
                match k {
                    Keyword::Use => {
                        imports.push(parse_import(data));
                    },
                    Keyword::Pub => {
                        if vis.is_some() {
                            panic!("Duplicate visibility");
                        }
                        let t = data.take();
                        vis = Some((Visibility::Pub, t.span.clone()));
                    },
                    Keyword::Package => {
                        if vis.is_some() {
                            panic!("Duplicate visibility");
                        }
                        let t = data.take();
                        vis = Some((Visibility::Pack, t.span.clone()));
                    },
                    Keyword::Fn => {
                        functions.push(parse_function(data, vis.clone(), attrs.clone()));
                        attrs.clear();
                    },
                    Keyword::Struct => {
                        structs.push(parse_struct(data, vis.clone(), attrs.clone()));
                        attrs.clear();
                    },
                    Keyword::Impl => {
                        impls.push(parse_impl(data, vis.clone(), attrs.clone()));
                        attrs.clear();
                    },
                    Keyword::Trait => {
                        traits.push(parse_trait(data, vis.clone(), attrs.clone()));
                        attrs.clear();
                    }
                    Keyword::Mod => {
                        let start = data.take().span.clone();
                        let name = parse_ident(data).0;
                        assert!(data.take().ty == TokenType::Special(Special::CurlyBracketOpen), "Expected '{{'");
                        inline_modules.push(parse_module(data, start));
                        assert!(data.take().ty == TokenType::Special(Special::CurlyBracketClose), "Expected '}}'");
                    }
                    Keyword::Const => {
                        let span = data.take().span.clone();
                        let ident = parse_ident(data).0;
                        let ty;
                        let t = data.take();
                        if t.ty == TokenType::Special(Special::Colon) {
                            ty = parse_type(data);
                        } else {
                            panic!("Expected type");
                        }
                        let init;
                        let t = data.peek();
                        if t.ty == TokenType::Special(Special::Equals) {
                            data.take();
                            init = parse_expr(data);
                        } else {
                            panic!("Expected initializer");
                        }
                        let t = data.take();
                        assert!(t.ty == TokenType::Special(Special::Semicolon), "Expected ';'");
                        constants.push(ConstantDefinition {
                            attrs: attrs.clone(),
                            ident,
                            span: span.expand(&t.span),
                            ty,
                            init,
                        });
                        attrs.clear();
                    },
                    Keyword::Static => {
                        let span = data.take().span.clone();
                        let m = parse_mutability(data);
                        let ident = parse_ident(data).0;
                        let ty;
                        let t = data.take();
                        if t.ty == TokenType::Special(Special::Colon) {
                            ty = parse_type(data);
                        } else {
                            panic!("Expected type");
                        }
                        let t = data.take();
                        assert!(t.ty == TokenType::Special(Special::Semicolon), "Expected ';'");
                        statics.push(StaticDefinition {
                            attrs: attrs.clone(),
                            mutability: m,
                            ident,
                            span: span.expand(&t.span),
                            ty,
                        });
                        attrs.clear();
                    }
                    _ => {
                        panic!("Invalid token: {:#?}", t);
                    }
                }
            },
            TokenType::Special(Special::Hash) => {
                let mut span = data.take().span.clone();
                assert!(data.take().ty == TokenType::Special(Special::SquareBracketOpen));
                let atr = data.take();
                let attribute;
                if let TokenType::Ident(id) = &atr.ty {
                    let atr_span = atr.span.clone();
                    match id.str.as_str() {
                        "compute" => {
                            assert!(data.take().ty == TokenType::Special(Special::RoundBracketOpen));
                            let workgroup_size = parse_delimited(data, 
                                parse_u32_or_constant,
                                TokenType::Special(Special::Comma),
                                TokenType::Special(Special::RoundBracketClose));
                            if workgroup_size.len() > 3 {
                                panic!("Workgroup size is 3D at most");
                            }
                            for s in &workgroup_size {
                                match &s {
                                    ValueOrConstant::Value(v, _) => {
                                        if *v == 0 {
                                            panic!("Workgroup size can't be 0");
                                        }
                                    },
                                    _ => {}
                                }
                            }
                            let default = ValueOrConstant::Value(1, atr_span);
                            attribute = Attribute::Entrypoint(Entrypoint::Compute(workgroup_size.get(0).unwrap_or(&default).clone(),
                            workgroup_size.get(1).unwrap_or(&default).clone(),
                            workgroup_size.get(2).unwrap_or(&default).clone()));
                            let t = data.take();
                            assert!(t.ty == TokenType::Special(Special::RoundBracketClose));
                            let t = data.take();
                            assert!(t.ty == TokenType::Special(Special::SquareBracketClose));
                            span = span.expand(&t.span);
                        },
                        "push" => {
                            assert!(data.take().ty == TokenType::Special(Special::RoundBracketOpen));
                            let offset = parse_u32_or_constant(data);
                            attribute = Attribute::Push(offset);
                            let t = data.take();
                            assert!(t.ty == TokenType::Special(Special::RoundBracketClose));
                            let t = data.take();
                            assert!(t.ty == TokenType::Special(Special::SquareBracketClose));
                            span = span.expand(&t.span);
                        }
                        _ => {
                            panic!("Custom attributes are currently not supported")
                        }
                    }
                    attrs.push((attribute, span));
                } else {
                    panic!("Identifier expected");
                }
            }
            _ => {
                panic!("Invalid token: {:#?}", t);
            }
        }
    }
}


fn parse_visibility(data: &mut ParserData) -> Option<Visibility> {
    let t = data.peek();
    match t.ty {
        TokenType::Keyword(Keyword::Pub) => {
            data.take();
            Some(Visibility::Pub)
        },
        TokenType::Keyword(Keyword::Package) => {
            data.take();
            Some(Visibility::Pack)
        },
        TokenType::Keyword(Keyword::Private) => {
            data.take();
            Some(Visibility::Priv)
        },
        _ => None,
    }
}


fn parse_import(data: &mut ParserData) -> Import {
    assert!(data.take().ty == TokenType::Keyword(Keyword::Use));
    let path = parse_item_path(data, false);
    assert!(data.take().ty == TokenType::Special(Special::Semicolon));
    return Import { path };
}


fn parse_module_file(data: &mut ParserData) -> Module {
    let t = data.take();
    assert_eq!(t.ty, TokenType::Start);
    let start = t.span.clone();
    let m = parse_module(data, start);
    let t = data.take();
    assert_eq!(t.ty, TokenType::End);
    m
}

fn parse_struct(data: &mut ParserData, vis: Option<(Visibility, SourceSpan)>, attrs: Vec<(Attribute, SourceSpan)>) -> Structure {
    let t = data.take();
    assert!(t.ty == TokenType::Keyword(Keyword::Struct));
    let start = t.span.clone();
    let ident = parse_ident(data);
    let mut args = vec![];
    let mut fields = vec![];
    let mut t = data.peek();
    match t.ty {
        TokenType::Special(Special::AngleBracketOpen) => {
            data.take();
            args = parse_delimited(data, parse_generic_arg_def, TokenType::Special(Special::Comma), TokenType::Special(Special::AngleBracketClose));
            assert!(data.take().ty == TokenType::Special(Special::AngleBracketClose));
            t = data.take();
        }
        _ => {}
    }
    
    match t.ty {
        TokenType::Keyword(Keyword::Where) => {
            todo!("generics bounds")
        }
        _ => {}
    }
    
    
    assert!(data.take().ty == TokenType::Special(Special::CurlyBracketOpen));
    fields = parse_delimited(data, parse_struct_field, TokenType::Special(Special::Comma), TokenType::Special(Special::CurlyBracketClose));
    let t = data.take();
    assert!(t.ty == TokenType::Special(Special::CurlyBracketClose));
    let span = start.expand(&t.span);
    
    return Structure {
        attrs,
        ident,
        args,
        fields,
        span,
    };
}

fn parse_struct_field(data: &mut ParserData) -> StructField {
    let vis = if let Some(v) = parse_visibility(data) {
        v
    } else {
        Visibility::Priv
    };
    let def = parse_ident_type(data);
    StructField {
        vis,
        ident: def.0,
        ty: def.1,
    }
}

fn parse_impl(data: &mut ParserData, vis: Option<(Visibility, SourceSpan)>, attrs: Vec<(Attribute, SourceSpan)>) -> Implementation {
    todo!()
}

fn parse_trait(data: &mut ParserData, vis: Option<(Visibility, SourceSpan)>, attrs: Vec<(Attribute, SourceSpan)>) -> Trait {
    todo!()
}


fn parse_u32_or_constant(data: &mut ParserData) -> ValueOrConstant<u32> {
    let t = data.peek();
    match &t.ty {
        TokenType::Int(i) => {
            let v = ValueOrConstant::Value((*i).try_into().expect("Immediate integer too big"), t.span.clone());
            data.take();
            return v;
        },
        _ => {}
    }
    return ValueOrConstant::Constant(parse_item_path(data, false));
}


fn parse_function(data: &mut ParserData, vis: Option<(Visibility, SourceSpan)>, attrs: Vec<(Attribute, SourceSpan)>) -> Function {
    let t = data.take();
    assert_eq!(t.ty, TokenType::Keyword(Keyword::Fn));
    let fn_ = t.span.clone();
    let mut uni  = None;
    let mut safety = Safety::Safe;
    
    let mut generics = vec![];
    
    for a in &attrs {
        match &a.0 {
            Attribute::Entrypoint(Entrypoint::Compute(value_or_constant, value_or_constant1, value_or_constant2)) => {
                uni = Some((Uniformity::Uniform, fn_.clone()));
            },
            _ => {}
        }
    }
    
    
    
    if data.peek().ty == TokenType::Special(Special::AngleBracketOpen) {
        data.take();
        generics = parse_delimited(data, parse_generic_arg_def, TokenType::Special(Special::Comma), TokenType::Special(Special::AngleBracketClose));
        let t = data.take();
        assert!(t.ty == TokenType::Special(Special::AngleBracketClose));
    }
    
    let t = data.peek();
    if t.ty == TokenType::Keyword(Keyword::Unsafe) {
        data.take();
        safety = Safety::Unsafe;
    }
    
    let puni = parse_uniformity(data);
    
    if puni.is_some() {
        if uni.is_some() && puni.as_ref().unwrap().0 != uni.as_ref().unwrap().0 {
            panic!("Function uniformity already specified by attribute");
        }
        uni = puni;
    } else {
        if uni.is_none() {
            panic!("missing function uniformity")
        }
    }
    let uni = uni.unwrap();
    
    
    let t = data.take();
    
    let ident = match &t.ty {
        TokenType::Ident(id) => {
            (id.clone(), t.span.clone())
        },
        _ => panic!("Expected identifier")
    };
    
    let t = data.take();
    assert_eq!(t.ty, TokenType::Special(Special::RoundBracketOpen));
    let params = parse_delimited(data, parse_ident_type, TokenType::Special(Special::Comma), TokenType::Special(Special::RoundBracketClose));
    data.take();
    let t = data.peek();
    let mut ret = Type::Unit(fn_.clone());
    match &t.ty {
        TokenType::Special(s) => {
            match s {
                Special::ThinArrow => {
                    data.take();
                    ret = parse_type(data);
                },
                Special::CurlyBracketOpen => {},
                _ => {
                    panic!("Expected ':' or '{{'");
                }
            }
        },
        _ => {
            panic!("Expected ':' or '{{'");
        }
    }
    let block = parse_block(data);
    return Function {
        generic_args: generics,
        attrs,
        vis,
        safety,
        uni,
        fn_,
        ident,
        params,
        ret,
        block,
    }
}

fn parse_generic_arg_def(data: &mut ParserData) -> GenericArgDefinition {
    let t = data.take();
    match &t.ty {
        TokenType::Keyword(kw) => {
            match kw {
                Keyword::Const => {
                    let t = data.take();
                    match &t.ty {
                        TokenType::Ident(ident) => {
                            return GenericArgDefinition::Expr(ident.clone(), t.span.clone());
                        }
                        _ => panic!("Expected generic argument")
                    }
                },
                Keyword::Uni => {
                    let t = data.take();
                    match &t.ty {
                        TokenType::Uniformity(ident) => {
                            return GenericArgDefinition::Uni(ident.clone(), t.span.clone());
                        }
                        _ => panic!("Expected generic argument")
                    }
                },
                _ => panic!("Expected generic argument")
            }
        },
        TokenType::Ident(ident) => {
            return GenericArgDefinition::Type(ident.clone(), t.span.clone())
        },
        _ => panic!("Expected generic argument")
    }
}

fn parse_block(data: &mut ParserData) -> Block {
    let mut statements= vec![];
    let mut value: Option<Expression> = None;
    let t = data.take();
    if t.ty != TokenType::Special(Special::CurlyBracketOpen) {
        panic!("Expected '{{'");
    }
    let start = t.span.clone();
    let span;
    loop {
        let t = data.peek();
        match &t.ty {
            TokenType::Special(Special::CurlyBracketClose) => {
                span = start.expand(&data.take().span);
                break;
            },
            _ => {
                let (s, semi) = parse_statement(data);
                if ! semi {
                    match s {
                        Statement::Expression(e) => {
                            value = Some(e);
                        },
                        _ => unreachable!(),
                    }
                    let t= data.take();
                    assert!(t.ty == TokenType::Special(Special::CurlyBracketClose));
                    span = start.expand(&data.take().span);
                    break;
                }
                statements.push(s);
            }
        }
    }
    return Block {
        statements,
        value,
        span,
    };
}

fn parse_statement(data: &mut ParserData) -> (Statement, bool) {
    let t = data.peek();
    match &t.ty {
        TokenType::Keyword(keyword) => {
            (match keyword {
                Keyword::Return => {
                    data.take();
                    let t = data.peek();
                    let e = match &t.ty {
                        TokenType::Special(Special::Semicolon) => {
                            None
                        },
                        _ => {
                            Some(parse_expr(data))
                        }
                    };
                    let s = Statement::Return(e);
                    assert!(data.take().ty == TokenType::Special(Special::Semicolon), "Expected ';'");
                    s
                },
                Keyword::Let => {
                    let span = data.take().span.clone();
                    let m = parse_mutability(data);
                    let ident = parse_ident(data).0;
                    let mut ty = None;
                    let mut init = None;
                    let t = data.peek();
                    if t.ty == TokenType::Special(Special::Colon) {
                        data.take();
                        ty = Some(parse_type(data));
                    }
                    let t = data.peek();
                    if t.ty == TokenType::Special(Special::Equals) {
                        data.take();
                        init = Some(parse_expr(data));
                    }
                    let t = data.take();
                    assert!(t.ty == TokenType::Special(Special::Semicolon), "Expected ';', got {:?}", t.ty);
                    let l = Statement::Let(Let::Single(span.expand(&t.span), m, ident, ty, init));
                    l
                },
                Keyword::Break => {
                    Statement::Break(data.take().span.clone())
                },
                Keyword::Continue => {
                    Statement::Continue(data.take().span.clone())
                },
                _ => {
                    panic!("invalid keyword use");
                }
            }, true)
        },
        _ => {
            let e = parse_expr(data);
            let semicolon = data.peek().ty == TokenType::Special(Special::Semicolon);
            if semicolon {
                data.take();
            }
            (Statement::Expression(e), semicolon)
        }
    }
}


fn parse_ident(data: &mut ParserData) -> (Ident, SourceSpan) {
    let t = data.take();
    match &t.ty {
        TokenType::Ident(id) => {
            return (id.clone(), t.span.clone())
        },
        _ => {
            panic!("Expected identifier at {:?}, got {:#?}", t.span, t);
        }
    }
}

fn parse_mutability(data: &mut ParserData) -> Mutability {
    let t = data.peek();
    match &t.ty {
        TokenType::Keyword(Keyword::Mut) => {
            data.take();
            Mutability::Mutable
        },
        _ => {
            Mutability::Immutable
        }
    }
}

fn parse_ident_type(data: &mut ParserData) -> ((Ident, SourceSpan), Type) {
    //let t = data.take();
    let id = parse_ident(data);
    let t = data.take();
    match t.ty {
        TokenType::Special(Special::Colon) => {},
        _ => {
            panic!("Expected colon");
        }
    }
    return (id, parse_type(data));
}


fn parse_uniformity(data: &mut ParserData) -> Option<(Uniformity, SourceSpan)> {
    let t = data.peek();
    match &t.ty {
        TokenType::Keyword(kw) => {
            match kw {
                Keyword::Uni => {
                    let t = data.take();
                    return Some((Uniformity::Uniform, t.span.clone()));
                },
                Keyword::SUni => {
                    let t = data.take();
                    return Some((Uniformity::SubUniform, t.span.clone()));
                },
                Keyword::Nuni => {
                    let t = data.take();
                    return Some((Uniformity::NonUniform, t.span.clone()));
                },
                _ => {}
            }
        },
        TokenType::Uniformity(ident) => {
            let ident = ident.clone();
            let t = data.take();
            return Some((Uniformity::Generic(ident.clone()), t.span.clone()));
        }
        _ => {}
    }
    return None;
}


fn parse_storage_class(data: &mut ParserData) -> Option<StorageClass> {
    let t = data.peek();
    let r = match &t.ty {
        TokenType::Keyword(kw) => {
            match kw {
                Keyword::Uniform => Some(StorageClass::Uniform),
                Keyword::Storage => Some(StorageClass::Storage),
                Keyword::Private => Some(StorageClass::Private),
                Keyword::PhysicalStorage => Some(StorageClass::PhysicalStorage),
                Keyword::Function => Some(StorageClass::Function),
                _ => None
            }
        },
        _ => None
    };
    if r.is_some() {
        data.take();
    }
    return r;
}

fn parse_type(data: &mut ParserData) -> Type {
    let uni = parse_uniformity(data);
    let t = data.peek();
    match &t.ty {
        TokenType::Keyword(Keyword::SelfType) => {
            let t = data.take();
            return Type::SelfType(t.span.clone());
        },
        TokenType::Special(Special::Star) => {
            let start = data.take().span.clone().start;
            let t = data.take();
            let mutable = match &t.ty {
                TokenType::Keyword(Keyword::Const) => Mutability::Immutable,
                TokenType::Keyword(Keyword::Mut) => Mutability::Mutable,
                _ => {
                    panic!("Expected 'const' or 'mut'");
                }
            };
            let storage = parse_storage_class(data);
            let ty = parse_type(data).into();
            return Type::Pointer(Pointer {
                uni,
                storage,
                mutable,
                ty,
                start,
            });
        },
        TokenType::Special(Special::And) => {
            let start = data.take().span.clone().start;
            let t = data.peek();
            let mutable = match &t.ty {
                TokenType::Keyword(Keyword::Mut) => {
                    data.take();
                    Mutability::Mutable
                },
                _ => {
                    Mutability::Immutable
                }
            };
            let storage = parse_storage_class(data);
            let ty = parse_type(data).into();
            return Type::Reference(Reference {
                uni,
                storage,
                mutable,
                ty,
                start,
            });
        },
        TokenType::Special(Special::RoundBracketOpen) => {
            let start = data.take().span.clone();
            let t2 = data.take();
            assert!(t2.ty == TokenType::Special(Special::RoundBracketClose));
            return Type::Unit(start.expand(&t2.span));
        }
        _ => {
            return Type::Item(uni, parse_item_path(data, false));
        }
    }
}


fn parse_generic_arg(data: &mut ParserData) -> GenericArg {
    let t = data.peek();
    match &t.ty {
        TokenType::Keyword(Keyword::Const) => {
            data.take();
            return GenericArg::Expr(parse_expr(data));
        },
        // special case for not requiring const for an int literal or minus.
        TokenType::Int(i) => {
            let i = *i;
            let t = data.take();
            return GenericArg::Expr(Expression::Int(t.span.clone(), i));
        },
        TokenType::Special(Special::Minus) => {
            return GenericArg::Expr(parse_expr(data));
        }
        _ => {}
    }
    return GenericArg::Type(parse_type(data));
}


fn parse_item_path(data: &mut ParserData, in_expr: bool) -> ItemPath {
    let mut p = ItemPath {
        segments: vec![],
        global: false,
    };
    let t = data.peek();
    let mut start = None;
    match &t.ty {
        TokenType::Special(Special::DoubleColon) => {
            start = Some(t.span.clone());
            data.take();
            p.global = true;
        },
        TokenType::Ident(_) => {},
        _ => {
            panic!("Invalid token: {:#?}", t);
        }
    }
    loop {
        let t = data.peek();
        let id = match &t.ty {
            TokenType::Ident(id) => {
                let id = id.clone();
                let t = data.take();
                let span = if p.segments.is_empty() && start.is_some() {
                    start.as_ref().unwrap().clone().expand(&t.span)
                } else {
                    t.span.clone()
                };
                (id, span)
            }
            _ => {
                if p.segments.is_empty() {
                    panic!("Invalid token")
                } else {
                    return p;
                }
            }
        };
        let t = data.peek();
        match t.ty {
            TokenType::Special(Special::DoubleColon) => {
                data.take();
                let t = data.peek();
                match t.ty {
                    TokenType::Special(Special::AngleBracketOpen) => {
                        data.take();
                        let args = parse_delimited(data, |data| parse_generic_arg(data), TokenType::Special(Special::Comma), TokenType::Special(Special::AngleBracketClose));
                        let t = data.take();
                        assert_eq!(t.ty, TokenType::Special(Special::AngleBracketClose));
                        p.segments.push((id.0, id.1, args));
                    },
                    TokenType::Ident(_) => {
                        p.segments.push((id.0, id.1, vec![]));
                    },
                    _ => {
                        p.segments.push((id.0, id.1, vec![]));
                        return p;
                    }
                }
            },
            TokenType::Special(Special::AngleBracketOpen) => {
                if in_expr {
                    p.segments.push((id.0, id.1, vec![]));
                    return p;
                } else {
                    data.take();
                    let args = parse_delimited(data, |data| parse_generic_arg(data), TokenType::Special(Special::Comma), TokenType::Special(Special::AngleBracketClose));
                    let t = data.take();
                    assert_eq!(t.ty, TokenType::Special(Special::AngleBracketClose));
                    p.segments.push((id.0, id.1, args));
                }
            },
            _ => {
                p.segments.push((id.0, id.1, vec![]));
                return p;
            }
        }
    }
}


fn parse_delimited<P, T>(data: &mut ParserData, parser: P, delimiter: TokenType, end: TokenType) -> Vec<T> where P: Fn(&mut ParserData) -> T {
    let mut v = vec![];
    loop {
        let t = data.peek();
        if t.ty == end {
            return v;
        }
        v.push(parser(data));
        let t = data.peek();
        match (t.ty == end, t.ty == delimiter) {
            (true, _x) => {
                return v;
            },
            (_x, true) => {
                data.take();
            },
            _ => {
                panic!("Expected '{:?}' or '{:?}'", delimiter, end);
            }
        }
    }
}






static PREFIX_OPS: LazyLock<HashMap<TokenType, (UnOp, u16)>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(TokenType::Special(Special::Exclamation), (UnOp::Not, 10));
    m.insert(TokenType::Special(Special::Minus), (UnOp::Neg, 104));
    return m;
});


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Infix {
    Bin(BinOp),
    Property,
    Index,
}

static INFIX_OPS: LazyLock<HashMap<TokenType, (Infix, (u16, u16))>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    fn pair(i: u16) -> (u16, u16) {
        (i, i+1)
    }
    
    m.insert(TokenType::Special(Special::Plus), (Infix::Bin(BinOp::Add), pair(100)));
    m.insert(TokenType::Special(Special::Minus), (Infix::Bin(BinOp::Sub), pair(100)));
    m.insert(TokenType::Special(Special::Star), (Infix::Bin(BinOp::Mul), pair(102)));
    m.insert(TokenType::Special(Special::Slash), (Infix::Bin(BinOp::Div), pair(102)));
    m.insert(TokenType::Special(Special::Dot), (Infix::Property, pair(998)));
    m.insert(TokenType::Special(Special::SquareBracketOpen), (Infix::Index, pair(996)));
    m.insert(TokenType::Special(Special::Equals), (Infix::Bin(BinOp::Assign), (2, 1)));
    return m;
});


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Postfix {
    Call,
}

static POSTFIX_OPS: LazyLock<HashMap<TokenType, (Postfix, u16)>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(TokenType::Special(Special::RoundBracketOpen), (Postfix::Call, 900));
    return m;
});


fn parse_expr(data: &mut ParserData) -> Expression {
    fn pratt(data: &mut ParserData, min_bp: u16) -> Expression {
        let t = data.peek();
        let mut lhs;
        match &t.ty {
            TokenType::Keyword(keyword) => {
                todo!("if, loop, etc...")
            },
            TokenType::Ident(ident) => {
                lhs = Expression::Item(parse_item_path(data, true));
            },
            TokenType::Special(special) => {
                match special {
                    Special::RoundBracketOpen => {
                        let start = data.take().span.clone();
                        let t = data.peek();
                        if t.ty == TokenType::Special(Special::RoundBracketClose) {
                            lhs = Expression::Unit(start.expand(&t.span));
                            data.take();
                        } else {
                            lhs = pratt(data, 0);
                            let t = data.take();
                            match t.ty {
                                TokenType::Special(Special::RoundBracketClose) => {},
                                TokenType::Special(Special::Comma) => {
                                    let mut exprs = vec![lhs];
                                    let mut comma = true;
                                    let end;
                                    loop {
                                        let t = data.peek();
                                        if t.ty == TokenType::Special(Special::RoundBracketClose) {
                                            end = data.take().span.clone();
                                            break;
                                        }
                                        if t.ty == TokenType::Special(Special::Comma) {
                                            data.take();
                                            if comma {
                                                panic!("Expected ')' or expression");
                                            } else {
                                                comma = true;
                                            }
                                            continue;
                                        }
                                        exprs.push(pratt(data, 0));
                                    }
                                    lhs = Expression::Tuple(start.expand(&end), exprs);
                                },
                                _ => {
                                    panic!("Expected comma or closing bracket");
                                }
                            }
                        }
                    }
                    _ => {
                        let ops = t.span.clone();
                        let op = PREFIX_OPS.get(&t.ty).expect(&format!("unrecognized prefix operator: {:#?}", t.ty));
                        data.take();
                        let rhs = pratt(data, op.1);
                        lhs = Expression::UnOp(ops, op.0, Box::new(rhs));
                    }
                }
            },
            TokenType::Int(i) => {
                lhs = Expression::Int(t.span.clone(), *i);
                data.take();
            },
            TokenType::Float(f) => {
                lhs = Expression::Float(t.span.clone(), *f);
                data.take();
            },
            TokenType::String(s) => {
                todo!("string");
                data.take();
            },
            TokenType::Char(c) => {
                todo!("char");
                data.take();
            },
            TokenType::DocComment(_) => {
                data.take();
                lhs = pratt(data, min_bp);
            }
            _ => {
                panic!("Expected operand, got {:#?}", t.ty);
            }
        }
        
        loop {
            let opt = data.peek();
            
            let op_post = POSTFIX_OPS.get(&opt.ty);
            let op_in = INFIX_OPS.get(&opt.ty);
            
            
            if op_post.is_none() && op_in.is_none() {
                break;
            }
            
            if let Some((op, bp)) = op_post {
                if *bp < min_bp {
                    break;
                }
                let opt = data.take();
                match *op {
                    Postfix::Call => {
                        let args = parse_delimited(data, 
                            parse_expr, 
                            TokenType::Special(Special::Comma), 
                            TokenType::Special(Special::RoundBracketClose));
                        assert!(data.take().ty == TokenType::Special(Special::RoundBracketClose));
                        lhs = Expression::Call(Box::new(lhs), args);
                    }
                }
                //lhs = Expression::UnOp(opt.span.clone(), *op, Box::new(lhs));
                continue;
            }
            if let Some((op, (lbp, rbp))) = op_in {
                if *lbp < min_bp {
                    break;
                }
                let opt = data.take();
                match *op {
                    Infix::Bin(op) => {
                        let rhs = pratt(data, *rbp);
                        lhs = Expression::BinOp(Box::new(lhs), op, Box::new(rhs));
                    }
                    Infix::Property => {
                        let t = data.take();
                        match &t.ty {
                            TokenType::Ident(id) => {
                                lhs = Expression::Property(Box::new(lhs), id.clone(), t.span.clone());
                            },
                            _ => {
                                panic!("Expected identifier for property access");
                            }
                        }
                    },
                    Infix::Index => {
                        let rhs = parse_expr(data);
                        assert!(data.take().ty == TokenType::Special(Special::SquareBracketClose));
                        lhs = Expression::Index(Box::new(lhs), Box::new(rhs));
                    }
                }
                continue;
            }
            break;
        }
        return lhs;
    }
    return pratt(data, 0);
}


#[cfg(test)]
mod test {
    use std::{fs::{File, OpenOptions}, io::{stdout, Write}, thread::sleep, time::Duration};

    use crate::{ast::tokenizer::tokenize, mid::Scope, passes::run_passes};

    use super::*;
    
    #[test]
    fn test_item_path() {
        let file: Rc<PathBuf> = PathBuf::new().into();
        let tokens = tokenize("::a::b<c, d>", file.clone());
        let mut data = ParserData { tokens: &tokens, index: 1 };
        assert_eq!(parse_item_path(&mut data, false), ItemPath {
            global: true,
            segments: vec![
                ("a".try_into().unwrap(), SourceSpan {
                    file: file.clone(),
                    start: SourcePos { line: 1, character: 1 },
                    end: SourcePos { line: 1, character: 4 }
                }, vec![]),
                ("b".try_into().unwrap(), SourceSpan {
                    file: file.clone(),
                    start: SourcePos { line: 1, character: 6 },
                    end: SourcePos { line: 1, character: 7 }
                }, vec![
                    GenericArg::Type(Type::Item(None, ItemPath {
                        global: false,
                        segments: vec![
                            ("c".try_into().unwrap(), SourceSpan {
                                file: file.clone(),
                                start: SourcePos { line: 1, character: 8 },
                                end: SourcePos { line: 1, character: 9 }
                            }, vec![])
                        ]
                    })),
                    GenericArg::Type(Type::Item(None, ItemPath {
                        global: false,
                        segments: vec![
                            ("d".try_into().unwrap(), SourceSpan {
                                file: file.clone(),
                                start: SourcePos { line: 1, character: 11 },
                                end: SourcePos { line: 1, character: 12 }
                            }, vec![])
                        ]
                    })),
                ]),
            ]
        });
    }
    
    
    #[test]
    fn test_module() {
        let file: Rc<PathBuf> = PathBuf::new().into();
        let tokens = tokenize(r"
        fn test(a: u8) -> u16 {
            return a + 1;
        }
        ", file.clone());
        let mut data = ParserData { tokens: &tokens, index: 0 };
        println!("{:#?}", parse_module_file(&mut data));
    }
    
    #[test]
    fn test_token() {
        let file: Rc<PathBuf> = PathBuf::new().into();
        let tokens = tokenize(r"
        :: ..
        ", file.clone());
        println!("{:#?}", tokens);
        
        
        
    }
    
    #[test]
    fn test_expr() {
        let file: Rc<PathBuf> = PathBuf::new().into();
        let tokens = tokenize(r"
        1 + - 2 * 3
        ", file.clone());
        let mut data = ParserData { tokens: &tokens, index: 1 };
        println!("{}", parse_expr(&mut data));
        
        let tokens = tokenize(r"
        1 = 2 = 3
        ", file.clone());
        let mut data = ParserData { tokens: &tokens, index: 1 };
        println!("{}", parse_expr(&mut data));
        
        
    }
    
    #[test]
    fn test_simple_compute() {
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
        //println!("{:#?}", tokens);
        let mut data = ParserData { tokens: &tokens, index: 0 };
        let m = parse_module_file(&mut data);
        //println!("{:#?}", m);
        let mut root = Scope::root();
        root.items.insert(Ident { str: "test".to_string() }, crate::mid::ModuleItem::Module(Scope::from_ast(m)));
        let md = run_passes(&mut root);
        
        let ptokens = tokenize("::test::add", file.clone());
        let mut datap = ParserData { tokens: &ptokens, index: 1 };
        println!("{:#?}", root.lookup_path(&parse_item_path(&mut datap, false)));
        //println!("{:#?}", root.scopes[&Ident { str: "test".to_string() }]);
        //let mut f = OpenOptions::new().create(true).write(true).open("test-data.txt").unwrap();
        //writeln!(f, "{:#?}", root.scopes[&Ident { str: "test".to_string() }]);
        //println!("{:#?}", md);
        
    }
}




