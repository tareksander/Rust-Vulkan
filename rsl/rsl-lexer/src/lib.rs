use std::{cell::{Cell, RefCell}, collections::HashMap};

use ariadne::{Color, Label, Report};
use unicode_segmentation::UnicodeSegmentation;

use rsl_data::internal::{tokens::{Keyword, Special, TokenType}, SourceSpan, StringTable};






pub fn tokenize<'a>(code: &'a str, file: usize, strings: &StringTable) -> Result<(Vec<TokenType>, Vec<SourceSpan>), Report<'a, SourceSpan>> {
    let p = Cell::new(0);
    let pr = &p;
    let chars = code.graphemes(true).collect::<Vec<&str>>();
    let c = chars.as_slice();
    let gobble_whitespace = || {
        loop {
            let index = pr.get();
            if let Some(c) = c.get(index) {
                if *c == "\r" || *c == "\n" || *c == "\r\n" {
                    pr.set(pr.get()+1);
                    continue;
                }
                if c.len() != 1 {
                    break;
                }
                if c.chars().next().unwrap().is_ascii_whitespace() {
                    pr.set(pr.get()+1);
                    continue;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    };
    
    let tokens: RefCell<Vec<TokenType>> = Vec::with_capacity(code.len()/4).into();
    let spans: RefCell<Vec<SourceSpan>> = Vec::with_capacity(code.len()/4).into();
    tokens.borrow_mut().push(TokenType::Start);
    spans.borrow_mut().push(SourceSpan { file, start: 0, end: 0 });
    
    
    let mut keywords = HashMap::new();
    {
        use Keyword::*;
        keywords.insert("fn", Fn);
        keywords.insert("pub", Pub);
        keywords.insert("package", Package);
        keywords.insert("return", Return);
        keywords.insert("use", Use);
        keywords.insert("impl", Impl);
        keywords.insert("for", For);
        keywords.insert("loop", Loop);
        keywords.insert("in", In);
        keywords.insert("mut", Mut);
        keywords.insert("struct", Struct);
        keywords.insert("super", Super);
        keywords.insert("const", Const);
        keywords.insert("self", SelfValue);
        keywords.insert("Self", SelfType);
        keywords.insert("mod", Mod);
        keywords.insert("let", Let);
        keywords.insert("break", Break);
        keywords.insert("continue", Continue);
        keywords.insert("uni", Uni);
        keywords.insert("suni", SUni);
        keywords.insert("nuni", Nuni);
        keywords.insert("Storage", Storage);
        keywords.insert("PhysicalStorage", PhysicalStorage);
        keywords.insert("Uniform", Uniform);
        keywords.insert("Workgroup", Workgroup);
        keywords.insert("Function", Function);
        keywords.insert("Private", Private);
        keywords.insert("static", Static);
        keywords.insert("trait", Trait);
        keywords.insert("unsafe", Unsafe);
        keywords.insert("where", Where);
    }
    
    let mut special = HashMap::new();
    {
        use Special::*;
        special.insert(';', Semicolon);
        special.insert(':', Colon);
        special.insert('[', SquareBracketOpen);
        special.insert(']', SquareBracketClose);
        special.insert('(', RoundBracketOpen);
        special.insert(')', RoundBracketClose);
        special.insert('{', CurlyBracketOpen);
        special.insert('}', CurlyBracketClose);
        special.insert('+', Plus);
        special.insert('-', Minus);
        special.insert('*', Star);
        special.insert('/', Slash);
        special.insert('%', Percent);
        special.insert('^', Caret);
        special.insert('!', Exclamation);
        special.insert('~', Tilde);
        special.insert('#', Hash);
        special.insert('.', Dot);
        special.insert(',', Comma);
        special.insert('<', AngleBracketOpen);
        special.insert('>', AngleBracketClose);
        special.insert('|', Bar);
        special.insert('&', And);
        special.insert('=', Equals);
    }
    
    let mut special_double = HashMap::new();
    {
        use Special::*;
        special_double.insert(':', DoubleColon);
        special_double.insert('.', DoubleDot);
        special_double.insert('=', DoubleEquals);
        special_double.insert('&', DoubleAnd);
        special_double.insert('|', DoubleBar);
    }
    
    
    let check_keyword_ident = || {
        let start = pr.get();
        let mut index = start;
        let mut s = String::with_capacity(10);
        loop {
            if let Some(c) = c.get(index as usize) {
                index += 1;
                if s.len() == 0 {
                    if ! c.chars().next().unwrap().is_ascii_alphabetic() && c.chars().next().unwrap() != '_' {
                        index -= 1;
                        break;
                    }
                } else {
                    if ! c.chars().next().unwrap().is_ascii_alphanumeric() && c.chars().next().unwrap() != '_' {
                        index -= 1;
                        break;
                    }
                }
                s.push_str(c);
            } else {
                break;
            }
        }
        let s = s.as_str();
        pr.set(index);
        if s.len() == 0 {
            return;
        }
        
        let span = SourceSpan {
            file,
            start,
            end: pr.get(),
        };
        spans.borrow_mut().push(span);
        if let Some(k) = keywords.get(s) {
            tokens.borrow_mut().push(TokenType::Keyword(*k));
        } else {
            tokens.borrow_mut().push(TokenType::Ident(strings.insert_get(s)));
        }
    };
    
    let check_special = || {
        let start = pr.get();
        let mut index = start;
        if let Some(gr) = c.get(index as usize) {
            index += 1;
            if gr.len() != 1 || ! special.contains_key(&gr.chars().next().unwrap()) {
                index -= 1;
                return;
            }
            let sc = gr.chars().next().unwrap();
            if special_double.contains_key(&sc) {
                if let Some(c) = c.get(index as usize) {
                    if c.len() == 1 && c.chars().next().unwrap() == sc {
                        index += 1;
                        pr.borrow_mut().character += 1;
                        let span = SourceSpan {
                            file: file.clone(),
                            start,
                            end: pr.get(),
                        };
                        pr.borrow_mut().index = index;
                        spans.borrow_mut().push(span);
                        tokens.borrow_mut().push(TokenType::Special(special_double[&sc]));
                        return;
                    }
                }
            }
            if sc == '-' || sc == '=' {
                if let Some(c) = c.get(index as usize) {
                    if c.len() == 1 && c.chars().next().unwrap() == '>' {
                        index += 1;
                        pr.borrow_mut().character += 1;
                        let span = SourceSpan {
                            file: file.clone(),
                            start,
                            end: pr.get(),
                        };
                        pr.borrow_mut().index = index;
                        spans.borrow_mut().push(span);
                        tokens.borrow_mut().push(TokenType::Special(match sc {
                            '-' => Special::ThinArrow,
                            '=' => Special::ThickArrow,
                            _ => {
                                unreachable!()
                            }
                        }));
                        return;
                    }
                }
            }
            let span = SourceSpan {
                file: file.clone(),
                start,
                end: pr.get(),
            };
            pr.borrow_mut().index = index;
            spans.borrow_mut().push(span);
            tokens.borrow_mut().push(TokenType::Special(special[&sc]));
        }
    };
    
    
    let check_comment = || {
        if let Some(s) = c.get(pr.borrow().index as usize.. (pr.borrow().index + 2) as usize) {
            let s = s.join("");
            if s == "//" {
                let mut p = pr.borrow_mut();
                p.character = 1;
                p.line += 1;
                p.index += 2;
                loop {
                    if let Some(c) = c.get(p.index as usize) {
                        p.index += 1;
                        if c.contains("\n") {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }
    };
    
    let check_float = |whole: f64, start: SourcePos| {
        let mut s = String::with_capacity(10);
        let mut index = pr.borrow().index;
        loop {
            if let Some(c) = c.get(index as usize) {
                index += 1;
                pr.borrow_mut().character += 1;
                if c.len() != 1 || ! c.chars().next().unwrap().is_ascii_digit() {
                    index -= 1;
                    pr.borrow_mut().character -= 1;
                    break;
                }
                s.push_str(c);
            } else {
                break;
            }
        }
        let mut fraction = u128::from_str_radix(s.as_str(), 10).unwrap() as f64;
        fraction = fraction / (fraction.log10().floor() + 1.0);
        pr.borrow_mut().index = index;
        let span = SourceSpan {
            file: file.clone(),
            start,
            end: pr.get(),
        };
        spans.borrow_mut().push(span);
        tokens.borrow_mut().push(TokenType::Float(whole + fraction));
    };
    
    let check_int_float = || {
        let start = pr.get();
        let mut s = String::with_capacity(10);
        let mut index = start;
        #[derive(Debug, PartialEq, Eq)]
        enum LiteralType {
            Dec, Hex, Bin
        }
        let mut ty = LiteralType::Dec;
        'l: loop {
            if let Some(c) = c.get(index as usize) {
                index += 1;
                pr.borrow_mut().character += 1;
                if c.len() != 1 {
                    index -= 1;
                    pr.borrow_mut().character -= 1;
                    break;
                }
                let cc = c.chars().next().unwrap();
                'b : {
                    if ! cc.is_ascii_digit() {
                        if s == "0" && ty == LiteralType::Dec {
                            match cc {
                                'x' => {
                                    s.clear();
                                    ty = LiteralType::Hex;
                                    println!("hex");
                                    break 'b;
                                },
                                'b' => {
                                    s.clear();
                                    ty = LiteralType::Bin;
                                    break 'b;
                                },
                                _ => {}
                            }
                        }
                        if cc == '.' {
                            if ty != LiteralType::Dec {
                                let p = *pr.borrow();
                                let span = SourceSpan {
                                    file,
                                    start,
                                    end: p
                                };
                                return Err(Report::build(ariadne::ReportKind::Error, span)
                                .with_label(Label::new(span).with_color(Color::Red).with_message("This float literal"))
                                .with_message("Float literals are only allowed in decimal notation").finish());
                            }
                            pr.borrow_mut().index = index;
                            check_float(u128::from_str_radix(s.as_str(), 10).unwrap() as f64, start);
                            return Ok(());
                        }
                        index -= 1;
                        pr.borrow_mut().character -= 1;
                        break 'l;
                    } else {
                        s.push_str(c);
                    }
                }
            } else {
                break;
            }
        }
        pr.borrow_mut().index = index;
        if s.len() == 0 {
            return Ok(());
        }
        let span = SourceSpan {
            file: file.clone(),
            start,
            end: pr.get(),
        };
        let radix = match ty {
            LiteralType::Dec => 10,
            LiteralType::Hex => 16,
            LiteralType::Bin => 2,
        };
        spans.borrow_mut().push(span);
        tokens.borrow_mut().push(TokenType::Int(u128::from_str_radix(s.as_str(), radix).unwrap()));
        return Ok(());
    };
    
    
    
    
    let check_lifetime = || {
        let start = pr.get();
        let mut s = String::with_capacity(10);
        let mut index = start;
        if let Some(c) = c.get(index as usize) {
            if c.len() != 1 || ! (c.chars().next().unwrap() == '\'') {
                return;
            }
            index += 1;
            pr.borrow_mut().character += 1;
        } else {
            return;
        }
        loop {
            if let Some(c) = c.get(index as usize) {
                if c.len() != 1 || (s.len() == 0 && ! c.chars().next().unwrap().is_ascii_alphabetic() ||
                                    s.len() != 0 && c.chars().next().unwrap().is_ascii_alphanumeric()) {
                    if s.len() == 0 {
                        panic!("Unterminated lifetime");
                    } else {
                        break;
                    }
                }
                s.push_str(c);
                index += 1;
                pr.borrow_mut().character += 1;
            } else {
                return;
            }
        }
        let s = s.as_str();
        pr.borrow_mut().index = index;
        let span = SourceSpan {
            file: file.clone(),
            start,
            end: pr.get(),
        };
        spans.borrow_mut().push(span);
        tokens.borrow_mut().push(TokenType::Lifetime(strings.insert_get(s)));
    };
    
    let check_uniformity = || {
        let start = pr.get();
        let mut s = String::with_capacity(10);
        let mut index = start;
        if let Some(c) = c.get(index as usize) {
            if c.len() != 1 || ! (c.chars().next().unwrap() == '~') {
                return;
            }
            index += 1;
            pr.borrow_mut().character += 1;
        } else {
            return;
        }
        loop {
            if let Some(c) = c.get(index as usize) {
                if c.len() != 1 || (s.len() == 0 && ! c.chars().next().unwrap().is_ascii_alphabetic() ||
                                    s.len() != 0 && c.chars().next().unwrap().is_ascii_alphanumeric()) {
                    if s.len() == 0 {
                        panic!("Unterminated uniformity");
                    } else {
                        break;
                    }
                }
                s.push_str(c);
                index += 1;
                pr.borrow_mut().character += 1;
            } else {
                return;
            }
        }
        let s = s.as_str();
        pr.borrow_mut().index = index;
        let span = SourceSpan {
            file: file.clone(),
            start,
            end: pr.get(),
        };
        spans.borrow_mut().push(span);
        tokens.borrow_mut().push(TokenType::Lifetime(strings.insert_get(s)));
    };
    
    loop {
        let start = p.borrow().index;
        gobble_whitespace();
        
        // TODO strings
        // make sure to increment the index by the number of bytes read for unicode chars in strings.
        
        check_keyword_ident();
        check_special();
        
        
        check_lifetime();
        check_uniformity();
        
        check_int_float()?;
        
        check_comment();
        // TODO check doc comment
        if start == p.borrow().index {
            let pb = p.borrow();
            panic!("Unable to match a character at index {}, line {}, position {}", pb.index, pb.line, pb.character);
        }
        if p.borrow().index as usize >= chars.len() {
            spans.borrow_mut().push(SourceSpan { file, start: *p.borrow(), end: *p.borrow() });
            tokens.borrow_mut().push(TokenType::End);
            break;
        }
    }
    return Ok((tokens.take(), spans.take()));
}


#[cfg(test)]
mod test {
    use ariadne::FnCache;
    use rsl_data::internal::{tokens, ReportSourceCache, Sources};

    use super::*;
    
    #[test]
    fn test_tokenize() {
        let strings = StringTable::new();
        let code = "pub fn foo() 0x1.2";
        let cache = ReportSourceCache::new(&Sources {
            source_files: vec![],
            source_strings: vec![code.to_string()]
        });
        let res = tokenize(code, 0, &strings);
        match res {
            Ok((tokens, spans)) => {
                let c2 = code;
                println!("{:#?}", tokens[0]);
            },
            Err(r) => {
                println!("Report:");
                r.print(cache).unwrap();
            },
        }
        
    }
    
    
    
    
    
}

