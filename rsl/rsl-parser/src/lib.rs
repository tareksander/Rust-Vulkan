use std::{collections::HashMap, f32::consts::E, sync::LazyLock};

use ariadne::{Color, Label, Report, ReportBuilder, ReportKind};
use rsl_data::internal::{Attribute, CompilerData, InternedString, Mutability, ShaderType, SourceSpan, StorageClass, StringTable, Uniformity, Visibility, ast::{BinOp, Block, Expression, FunctionDefinition, GenericArg, ItemPath, ItemPathSegment, ModuleData, Statement, TokenRange, Type, UnOp}, tokens::{Keyword, Special, Token}};



type ParserResult<T> = Result<T, ()>;

struct ParserData<'a> {
    file: usize,
    tokens: &'a[Token],
    spans: &'a[SourceSpan],
    index: usize,
    strings: &'a StringTable,
    errors: Vec<Report<'static, SourceSpan>>,
}



impl<'a> ParserData<'a> {
    fn take(&mut self) -> &Token {
        // Can't use peek here due to the borrow checker.
        loop {
            match self.tokens[self.index] {
                Token::DocComment(_) => {
                    self.index += 1;
                    continue;
                },
                _ => {
                    let t = &self.tokens[self.index];
                    self.index += 1;
                    return t;
                }
            }
        }
    }
    
    fn peek(&mut self) -> &Token {
        loop {
            match self.tokens[self.index] {
                Token::DocComment(_) => {
                    self.index += 1;
                    continue;
                },
                _ => {
                    let t = &self.tokens[self.index];
                    return t;
                }
            }
        }
    }
    
    fn take_with_comment(&mut self) -> &Token {
        let t = &self.tokens[self.index];
        self.index += 1;
        t
    }
    
    fn peek_with_comment(&self) -> &Token {
        &self.tokens[self.index as usize]
    }
    
    fn take_ident(&mut self) -> ParserResult<(InternedString, TokenRange)> {
        let i = self.index;
        let span = self.spans[i];
        let t = &self.tokens[self.index];
        self.index += 1;
        match t {
            Token::Ident(s) => Ok((*s, TokenRange::point(self.file, i))),
            _ => {
                self.errors.push(Report::build(ReportKind::Error, span)
            .with_message("Expected identifier.")
            .with_label(Label::new(span).with_message("Here"))
            .finish());
                return Err(());
            }
        }
    }
    
    fn skip_to(&mut self, t: &Token) {
        loop {
            let ct = &self.tokens[self.index];
            if ct == &Token::End || ct == t {
                return;
            }
            self.index += 1;
        }
    }
    
    fn skip_after(&mut self, t: &Token) {
        self.skip_to(t);
        if &self.tokens[self.index] != &Token::End {
            self.index += 1;
        }
    }
    
}



pub fn parse_file(tokens: &[Token], spans: &[SourceSpan], file: usize, attrs: Vec<Attribute>, strings: &StringTable) -> ModuleData {
    let mut data = ParserData {
        file,
        tokens,
        spans,
        index: 0,
        strings,
        errors: vec![],
    };
    parse_module_file(&mut data, attrs)
}

fn parse_module_file(data: &mut ParserData, mut attrs: Vec<Attribute>) -> ModuleData {
    assert!(*data.take() == Token::Start);
    let m = parse_module(data, &mut attrs, true);
    assert!(*data.take() == Token::End);
    return m;
}


fn parse_module(data: &mut ParserData, attrs: &mut Vec<Attribute>, toplevel: bool) -> ModuleData {
    let mut visibility = None;
    let mut m = ModuleData {
        attrs: attrs.clone(),
        structs: vec![],
        traits: vec![],
        functions: vec![],
        statics: vec![],
        consts: vec![],
        struct_impls: vec![],
        trait_impls: vec![],
        inline_modules: vec![],
        outline_modules: vec![],
        span: TokenRange::point(data.file, data.index),
    };
    let mut attrs = vec![];
    loop {
        match *data.peek() {
            Token::Keyword(kw) => {
                match kw {
                    Keyword::Pub => {
                        if visibility.is_some() {
                            break;
                        }
                        visibility = Some((Visibility::Pub, TokenRange::point(data.file, data.index)));
                        data.take();
                    },
                    Keyword::Package => {
                        if visibility.is_some() {
                            break;
                        }
                        visibility = Some((Visibility::Pub, TokenRange::point(data.file, data.index)));
                        data.take();
                    },
                    Keyword::Struct => {
                        todo!()
                    },
                    Keyword::Unsafe => {
                        todo!()
                    },
                    Keyword::Fn => {
                        match parse_function(data, visibility, None, None, None, &attrs) {
                            Ok(f) => {
                                m.functions.push(f);
                            },
                            Err(_) => {}
                        }
                        visibility = None;
                    },
                    _=> {
                        break;
                    }
                }
            }
            Token::Special(Special::CurlyBracketClose) => {
                if toplevel {
                    break;
                } else {
                    return m;
                }
            },
            Token::End => {
                if ! toplevel {
                    break;
                } else {
                    return m;
                }
            },
            _ => {
                break;
            }
        }
    }
    data.errors.push(Report::build(ReportKind::Error, data.spans[data.index])
        .with_message("Expected module item, found invalid token")
        .with_label(Label::new(data.spans[data.index])
            .with_message("This token")
            .with_color(Color::Red))
        .finish());
    return m;
}













fn parse_function(data: &mut ParserData, vis: Option<(Visibility, TokenRange)>, unsafe_token: Option<TokenRange>, shader_type: Option<(ShaderType, TokenRange)>, uni: Option<(Uniformity, TokenRange)>, attrs: &Vec<Attribute>) -> ParserResult<FunctionDefinition> {
    let fn_token = TokenRange::point(data.file, data.index);
    assert!(*data.take() == Token::Keyword(Keyword::Fn));
    let generics = vec![];
    let generics_constraints = vec![];
    let block;
    let params;
    let mut ret = Type::Unit;
    let ident = data.take_ident()?;
    if *data.peek() == Token::Special(Special::AngleBracketOpen) {
        data.take();
        todo!()
    }
    if *data.take() == Token::Special(Special::RoundBracketOpen) {
        params = parse_delimited(data, parse_ident_type, Token::Special(Special::Comma), Token::Special(Special::RoundBracketClose))?;
        if *data.take() != Token::Special(Special::RoundBracketClose) {
            data.errors.push(Report::build(ReportKind::Error, data.spans[data.index-1])
            .with_message("Expected ')', found invalid token")
            .with_label(Label::new(data.spans[data.index-1])
                .with_message("This token")
                .with_color(Color::Red))
            .finish());
            return Err(());
        }
    } else {
        data.errors.push(Report::build(ReportKind::Error, data.spans[data.index-1])
            .with_message("Expected '(', found invalid token")
            .with_label(Label::new(data.spans[data.index-1])
                .with_message("This token")
                .with_color(Color::Red))
            .finish());
        return Err(());
    }
    if *data.peek() == Token::Special(Special::ThinArrow) {
        data.take();
        ret = parse_type(data, None)?;
    }
    block = parse_block(data)?;
    return Ok(FunctionDefinition {
        attrs: attrs.clone(),
        visibility: vis,
        unsafe_token,
        shader_type,
        uniformity: uni,
        fn_token,
        ident: ident.0,
        ident_token: ident.1,
        generics,
        generics_constraints,
        params,
        block,
        ret,
    })
}

fn parse_ident_type(data: &mut ParserData) -> ParserResult<(InternedString, TokenRange, Type)> {
    let t = data.take_ident()?;
    if *data.take() != Token::Special(Special::Colon) {
        data.errors.push(Report::build(ReportKind::Error, data.spans[data.index-1])
        .with_message("Expected colon, found invalid token")
        .with_label(Label::new(data.spans[data.index-1])
            .with_message("This token")
            .with_color(Color::Red))
        .finish());
        return Err(());
    }
    return Ok((t.0, t.1, parse_type(data, None)?));
}



fn parse_type(data: &mut ParserData, uni: Option<(Uniformity, TokenRange)>) -> ParserResult<Type> {
    match *data.peek() {
        Token::Special(Special::Star) => {
            let start = TokenRange::point(data.file, data.index);
            data.take();
            let mutability;
            match *data.peek() {
                Token::Keyword(Keyword::Const) => {
                    mutability = Mutability::Immutable;
                }
                Token::Keyword(Keyword::Mut) => {
                    mutability = Mutability::Mutable;
                }
                _ => {
                    data.errors.push(Report::build(ReportKind::Error, data.spans[data.index])
                        .with_message("Expected const or mut, found invalid token")
                        .with_label(Label::new(data.spans[data.index-1])
                            .with_message("This token")
                            .with_color(Color::Red))
                        .finish());
                    return Err(());
                }
            }
            data.take();
            return Ok(Type::Pointer { star_token: start, uni, mutability, ty: Box::new(parse_type(data, None)?) });
        },
        Token::Special(Special::And) => {
            let start = TokenRange::point(data.file, data.index);
            data.take();
            todo!("reference")
            
            
        },
        Token::Ident(s) => {
            let s = s.get(data.strings);
            match s.as_str() {
                "dispatch" => {
                    let t = TokenRange::point(data.file, data.index);
                    data.take();
                    return parse_type(data, Some((Uniformity::Dispatch, t)));
                },
                "workgroup" => {
                    let t = TokenRange::point(data.file, data.index);
                    data.take();
                    return parse_type(data, Some((Uniformity::Workgroup, t)));
                },
                "subgroup" => {
                    let t = TokenRange::point(data.file, data.index);
                    data.take();
                    return parse_type(data, Some((Uniformity::Subgroup, t)));
                },
                "invocation" => {
                    let t = TokenRange::point(data.file, data.index);
                    data.take();
                    return parse_type(data, Some((Uniformity::Invocation, t)));
                },
                _ => {
                    return Ok(Type::Path(parse_item_path(data, false)?, uni));
                }
            }
        },
        _ => {}
    }
    data.errors.push(Report::build(ReportKind::Error, data.spans[data.index])
        .with_message("Expected type, found invalid token")
        .with_label(Label::new(data.spans[data.index])
            .with_message("This token")
            .with_color(Color::Red))
        .finish());
    return Err(());
}


fn parse_generic_arg(data: &mut ParserData) -> ParserResult<GenericArg> {
    match *data.peek() {
        Token::Special(Special::Star) => {
            return Ok(GenericArg::Type(parse_type(data, None)?));
        },
        Token::Special(Special::And) => {
            return Ok(GenericArg::Type(parse_type(data, None)?));
        },
        Token::Ident(_) => {
            return Ok(GenericArg::Type(parse_type(data, None)?));
        },
        
        _ => {}
    }
    data.errors.push(Report::build(ReportKind::Error, data.spans[data.index])
        .with_message("Expected generic argument, found invalid token")
        .with_label(Label::new(data.spans[data.index])
            .with_message("This token")
            .with_color(Color::Red))
        .finish());
    return Err(());
}




fn parse_item_path(data: &mut ParserData, in_expr: bool) -> ParserResult<ItemPath> {
    let global = if *data.peek() == Token::Special(Special::DoubleColon) {
        data.take();
        true
    } else {
        false
    };
    let mut segments: Vec<ItemPathSegment> = vec![];
    loop {
        if segments.len() != 0 && *data.peek() == Token::Special(Special::AngleBracketOpen) {
            let i = segments.len()-1;
            segments[i].generic_args = parse_delimited(data, parse_generic_arg,
                Token::Special(Special::Comma),
            Token::Special(Special::AngleBracketClose))?;
        }
        let t = data.take_ident()?;
        segments.push(ItemPathSegment {
            ident: t.0,
            ident_token: t.1,
            generic_args: vec![],
        });
        if ! in_expr && *data.peek() == Token::Special(Special::AngleBracketOpen) {
            let i = segments.len()-1;
            segments[i].generic_args = parse_delimited(data, parse_generic_arg,
                Token::Special(Special::Comma),
            Token::Special(Special::AngleBracketClose))?;
        }
        
        if *data.peek() == Token::Special(Special::DoubleColon) {
            data.take();
        } else {
            break;
        }
    }
    return Ok(ItemPath { segments, global });
}



fn parse_block(data: &mut ParserData) -> ParserResult<Block> {
    let t = data.take();
    if *t != Token::Special(Special::CurlyBracketOpen) {
        data.errors.push(Report::build(ReportKind::Error, data.spans[data.index-1])
            .with_message("Expected { token, found invalid token")
            .with_label(Label::new(data.spans[data.index-1])
                .with_message("This token")
                .with_color(Color::Red))
            .finish());
        return Err(());
    }
    let mut statements = vec![];
    let mut value = None;
    while *data.peek() != Token::Special(Special::CurlyBracketClose) && *data.peek() != Token::End {
        if value.is_none() {
            match data.peek() {
                // Only statements start with a keyword, but some keywords can be both statements and expressions (e.g. if)
                Token::Keyword(keyword) => {
                    todo!()
                },
                _ => {
                    value = Some(parse_expr(data)?);
                }
            }
        }
        if let Some(v) = value {
            let is_block = match &v {
                // TODO include expressions with blocks like if that can stand as a statement without requiring a semicolon
                Expression::If { condition: _, then: _, other: _ } => true,
                
                _ => false
            };
            if *data.peek() == Token::Special(Special::Semicolon) {
                data.take();
                statements.push(Statement::Expression(v));
                value = None;
            } else {
                if *data.peek() != Token::Special(Special::CurlyBracketClose) && ! is_block {
                    data.errors.push(Report::build(ReportKind::Error, data.spans[data.index-1])
                        .with_message("Expected end of block after trailing expression")
                        .with_label(Label::new(data.spans[data.index-1])
                            .with_message("Here")
                            .with_color(Color::Red))
                        .finish());
                    data.skip_after(&Token::Special(Special::CurlyBracketClose));
                    return Err(());
                } else {
                    if is_block {
                        statements.push(Statement::Expression(v));
                        value = None;
                    } else {
                        value = Some(v);
                    }
                }
            }
        }
    }
    if *data.peek() == Token::End {
        data.errors.push(Report::build(ReportKind::Error, data.spans[data.index])
            .with_message("Expected } token, found end of file")
            .with_label(Label::new(data.spans[data.index])
                .with_message("Here")
                .with_color(Color::Red))
            .finish());
        return Err(());
    }
    data.take();
    return Ok(Block {
        statements,
        value,
        label: None,
    });
}



fn parse_delimited<P, T>(data: &mut ParserData, parser: P, delimiter: Token, end: Token) -> ParserResult<Vec<T>> where P: Fn(&mut ParserData) -> ParserResult<T> {
    let mut v = vec![];
    loop {
        let t = data.peek();
        if *t == end {
            return Ok(v);
        }
        v.push(parser(data)?);
        let t = data.peek();
        match (*t == end, *t == delimiter) {
            (true, _x) => {
                return Ok(v);
            },
            (_x, true) => {
                data.take();
            },
            _ => {
                data.errors.push(Report::build(ReportKind::Error, data.spans[data.index])
                    .with_message("Expected delimiter or list end token, found invalid token")
                    .with_label(Label::new(data.spans[data.index])
                        .with_message("This token")
                        .with_color(Color::Red))
                    .finish());
                return Err(());
            }
        }
    }
}


static PREFIX_OPS: LazyLock<HashMap<Token, (UnOp, u16)>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(Token::Special(Special::Exclamation), (UnOp::LogNot, 10));
    m.insert(Token::Special(Special::Minus), (UnOp::Negate, 104));
    m.insert(Token::Special(Special::Tilde), (UnOp::BinNot, 105));
    m.insert(Token::Special(Special::Star), (UnOp::Deref, 99));
    m.insert(Token::Special(Special::And), (UnOp::Ref, 98));
    return m;
});


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Infix {
    Bin(BinOp),
    Property,
    Index,
}

static INFIX_OPS: LazyLock<HashMap<Token, (Infix, (u16, u16))>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    fn pair(i: u16) -> (u16, u16) {
        (i, i+1)
    }
    
    m.insert(Token::Special(Special::Plus), (Infix::Bin(BinOp::Add), pair(100)));
    m.insert(Token::Special(Special::Minus), (Infix::Bin(BinOp::Sub), pair(100)));
    m.insert(Token::Special(Special::Star), (Infix::Bin(BinOp::Mul), pair(102)));
    m.insert(Token::Special(Special::Slash), (Infix::Bin(BinOp::Div), pair(102)));
    m.insert(Token::Special(Special::Dot), (Infix::Property, pair(998)));
    m.insert(Token::Special(Special::SquareBracketOpen), (Infix::Index, pair(996)));
    m.insert(Token::Special(Special::Equals), (Infix::Bin(BinOp::Assign), (2, 1)));
    return m;
});


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Postfix {
    Call,
}

static POSTFIX_OPS: LazyLock<HashMap<Token, (Postfix, u16)>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(Token::Special(Special::RoundBracketOpen), (Postfix::Call, 900));
    return m;
});


fn parse_expr(data: &mut ParserData) -> ParserResult<Expression> {
    fn pratt(data: &mut ParserData, min_bp: u16) -> ParserResult<Expression> {
        let t = data.peek();
        let mut lhs = match *t {
            Token::Keyword(kw) => {
                todo!("if , loop, etc.")
            },
            Token::Ident(s) => Expression::Item(parse_item_path(data, true)?),
            Token::Special(s) => {
                // TODO filter groups, tuples and references
                if let Some(op) = PREFIX_OPS.get(t) {
                    let bp = op.1;
                    let mut op = op.0;
                    data.take();
                    if s == Special::And {
                        if *data.peek() == Token::Keyword(Keyword::Mut) {
                            data.take();
                            op = UnOp::RefMut;
                        }
                    }
                    let rhs = pratt(data, bp)?;
                    Expression::Unary { e: Box::new(rhs), op: op }
                } else {
                    match s {
                        Special::DoubleColon => Expression::Item(parse_item_path(data, true)?),
                        _ => {
                            data.errors.push(Report::build(ReportKind::Error, data.spans[data.index])
                            .with_message("Expected operand, found invalid token")
                            .with_label(Label::new(data.spans[data.index])
                                .with_message("This token")
                                .with_color(Color::Red))
                            .finish());
                            return Err(())
                        }
                    }
                }
            },
            Token::Int(i) => {
                let t = data.take();
                Expression::IntLiteral(i, TokenRange::point(data.file, data.index))
            },
            Token::Float(f) => {
                let t = data.take();
                Expression::FloatLiteral(f, TokenRange::point(data.file, data.index))
            },
            Token::String(interned_string) => todo!(),
            Token::Char(_) => todo!(),
            _ => {
                data.errors.push(Report::build(ReportKind::Error, data.spans[data.index])
                .with_message("Expected operand, found invalid token")
                .with_label(Label::new(data.spans[data.index]).with_message("This token").with_color(Color::Red))
                .finish());
                return Err(());
            }
        };
        loop {
            let t = data.peek();
            let op = INFIX_OPS.get(t);
            if let Some(op) = op {
                if op.1.0 < min_bp {
                    break;
                }
                data.take();
                match op.0 {
                    Infix::Bin(bin_op) => {
                        let rhs = pratt(data, op.1.1)?;
                        lhs = Expression::Binary { lhs: Box::new(lhs), op: bin_op, rhs: Box::new(rhs) };
                    },
                    Infix::Property => {
                        let ident = data.take_ident()?;
                        lhs = Expression::Property { e: Box::new(lhs), name: ident.0, name_token: ident.1 };
                    },
                    Infix::Index => {
                        let opening = data.spans[data.index-1];
                        let rhs = pratt(data, 0)?;
                        let t = data.take();
                        if *t != Token::Special(Special::SquareBracketClose) {
                            data.errors.push(Report::build(ReportKind::Error, data.spans[data.index-1])
                                    .with_message("Expected closing square bracket, found invalid token")
                                    .with_labels(vec![
                                            Label::new(data.spans[data.index-1]).with_message("This token").with_color(Color::Red),
                                            Label::new(opening).with_message("The start of the index operation")
                                        ])
                                    .finish());
                            return Err(());
                        }
                        lhs = Expression::Binary { lhs: Box::new(lhs), op: BinOp::Index, rhs: Box::new(rhs) };
                    },
                }
            } else {
                if let Some(op) = POSTFIX_OPS.get(t) {
                    if op.1 < min_bp {
                        break;
                    }
                    data.take();
                    match op.0 {
                        Postfix::Call => {
                            todo!()
                        },
                    }
                } else {
                    break;
                }
            }
        }
        return Ok(lhs);
    }
    return pratt(data, 0)
}




#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rsl_data::internal::{ReportSourceCache, Sources, StringTable};
    use rsl_lexer::tokenize;

    use super::*;

    #[test]
    fn expr() -> Result<(), ()> {
        let strings = StringTable::new();
        let code = "1 + 2 * 3 / 4";
        let compare = "(1 + ((2 * 3) / 4))";
        let mut cache = ReportSourceCache::new(&Sources {
            source_files: vec![PathBuf::from("test.rsl")],
            source_strings: vec![code.to_string()]
        });
        let res = tokenize(code, 0, &strings);
        match res {
            Ok((tokens, spans)) => {
                let spans = spans.iter().map(|r| SourceSpan {
                    file: 0,
                    start: r.start,
                    end: r.end,
                }).collect::<Vec<_>>();
                let mut data = ParserData {
                    file: 0,
                    tokens: &tokens,
                    spans: &spans,
                    index: 1,
                    strings: &strings,
                    errors: vec![],
                };
                let expr = parse_expr(&mut data);
                match expr {
                    Ok(expr) => {
                        if format!("{}", expr) != compare {
                            println!("Result: {}", expr);
                            println!("expected: {}", compare);
                            return Err(());
                        }
                    },
                    Err(_) => {
                        for r in data.errors {
                            r.print(&mut cache).unwrap();
                        }
                        return Err(());
                    }
                }
            },
            Err(r) => {
                r.print(cache).unwrap();
                return Err(());
            },
        }
        return Ok(());
        
        
        
        
        
    }
    
    
    #[test]
    fn module() -> Result<(), ()> {
        let strings = StringTable::new();
        let code = "fn test(a: *const u32, b: *const u32, c: *mut u32) { c[globalInvocationID] = a[globalInvocationID] + b[globalInvocationID]; }";
        let mut cache = ReportSourceCache::new(&Sources {
            source_files: vec![PathBuf::from("test.rsl")],
            source_strings: vec![code.to_string()]
        });
        let res = tokenize(code, 0, &strings);
        match res {
            Ok((tokens, spans)) => {
                let spans = spans.iter().map(|r| SourceSpan {
                    file: 0,
                    start: r.start,
                    end: r.end,
                }).collect::<Vec<_>>();
                let mut data = ParserData {
                    file: 0,
                    tokens: &tokens,
                    spans: &spans,
                    index: 1,
                    strings: &strings,
                    errors: vec![],
                };
                let _m = parse_module(&mut data, &mut vec![], true);
                if ! data.errors.is_empty() {
                    data.errors.iter().for_each(|e| e.eprint(&mut cache).unwrap());
                    return Err(());
                }
            },
            Err(r) => {
                r.print(cache).unwrap();
                return Err(());
            },
        }
        return Ok(());
        
        
        
        
        
    }
}
