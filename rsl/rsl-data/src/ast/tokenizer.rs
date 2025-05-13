use std::{cell::{Cell, RefCell}, collections::HashMap, fs, path::PathBuf, rc::Rc};

use unicode_segmentation::UnicodeSegmentation;

use crate::{ast::token::*, SourcePos, SourceSpan};



/// Tokenizes a file.
pub fn tokenize_file(file: PathBuf) -> Vec<Token> {
    tokenize(&fs::read_to_string(&file).unwrap(), Rc::new(file))
}



/// Tokenizes a code string, assuming the file it's from is `file`.
pub fn tokenize(code: &str, file: Rc<PathBuf>) -> Vec<Token> {
    let p = RefCell::new(SourcePos {
        line: 1,
        character: 1,
    });
    let i: Cell<u32> = Cell::new(0);
    let pr = &p;
    let chars = code.graphemes(true).collect::<Vec<&str>>();
    let c = chars.as_slice();
    let gobble_whitespace = || {
        loop {
            if let Some(c) = c.get(i.get() as usize) {
                if *c == "\r" || *c == "\n" || *c == "\r\n" {
                    let mut p = pr.borrow_mut();
                    p.line += 1;
                    p.character = 1;
                    i.set(i.get() + 1);
                    continue;
                }
                if c.len() != 1 {
                    break;
                }
                if c.chars().next().unwrap().is_ascii_whitespace() {
                    i.set(i.get() + 1);
                    pr.borrow_mut().character += 1;
                    continue;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    };
    
    let tokens: RefCell<Vec<Token>> = Vec::with_capacity(100).into();
    tokens.borrow_mut().push(Token { span: SourceSpan { file: file.clone(), start: p.borrow().clone(), end: p.borrow().clone() }, ty: TokenType::Start });
    
    let mut keywords = HashMap::new();
    {
        use Keyword::*;
        keywords.insert("fn", Fn);
        keywords.insert("pub", Pub);
        keywords.insert("package", Package);
        keywords.insert("return", Return);
        keywords.insert("use", Use);
        keywords.insert("type", Type);
        keywords.insert("impl", Impl);
        keywords.insert("for", For);
        keywords.insert("loop", Loop);
        keywords.insert("in", In);
        keywords.insert("mut", Mut);
        //keywords.insert("mix", Mix);
        //keywords.insert("mixin", Mixin);
        keywords.insert("struct", Struct);
        keywords.insert("super", Super);
        keywords.insert("const", Const);
        keywords.insert("self", SelfValue);
        keywords.insert("Self", SelfType);
        //keywords.insert("void", Void);
        keywords.insert("mod", Mod);
        keywords.insert("let", Let);
        keywords.insert("break", Break);
        keywords.insert("continue", Continue);
        keywords.insert("uni", Uni);
        keywords.insert("dyn", Dyn);
        keywords.insert("suni", SUni);
        keywords.insert("nuni", Nuni);
        keywords.insert("Storage", Storage);
        keywords.insert("PhysicalStorage", PhysicalStorage);
        keywords.insert("Uniform", Uniform);
        keywords.insert("Workgroup", Workgroup);
        keywords.insert("Function", Function);
        keywords.insert("Private", Private);
        keywords.insert("Push", Push);
        keywords.insert("memory", Memory);
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
        let start = pr.borrow().clone();
        let mut s = String::with_capacity(10);
        let mut index = i.get();
        loop {
            if let Some(c) = c.get(index as usize) {
                index += 1;
                pr.borrow_mut().character += 1;
                if s.len() == 0 {
                    if ! c.chars().next().unwrap().is_ascii_alphabetic() && c.chars().next().unwrap() != '_' {
                        index -= 1;
                        pr.borrow_mut().character -= 1;
                        break;
                    }
                } else {
                    if ! c.chars().next().unwrap().is_ascii_alphanumeric() && c.chars().next().unwrap() != '_' {
                        index -= 1;
                        pr.borrow_mut().character -= 1;
                        break;
                    }
                }
                s.push_str(c);
            } else {
                break;
            }
        }
        i.set(index);
        if s.len() == 0 {
            return;
        }
        let span = SourceSpan {
            file: file.clone(),
            start,
            end: pr.borrow().clone(),
        };
        if let Some(k) = keywords.get(s.as_str()) {
            tokens.borrow_mut().push(Token { span, ty: TokenType::Keyword(*k) });
        } else {
            tokens.borrow_mut().push(Token { span, ty: TokenType::Ident(s.as_str().try_into().unwrap()) });
        }
    };
    
    let check_special = || {
        let start = pr.borrow().clone();
        let mut index = i.get();
        if let Some(gr) = c.get(index as usize) {
            index += 1;
            pr.borrow_mut().character += 1;
            if gr.len() != 1 || ! special.contains_key(&gr.chars().next().unwrap()) {
                index -= 1;
                pr.borrow_mut().character -= 1;
                i.set(index);
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
                            end: pr.borrow().clone(),
                        };
                        i.set(index);
                        tokens.borrow_mut().push(Token { span, ty: TokenType::Special(special_double[&sc]) });
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
                            end: pr.borrow().clone(),
                        };
                        i.set(index);
                        tokens.borrow_mut().push(Token { span, ty: TokenType::Special(match sc {
                            '-' => Special::ThinArrow,
                            '=' => Special::ThickArrow,
                            _ => {
                                unreachable!()
                            }
                        }) });
                        return;
                    }
                }
            }
            let span = SourceSpan {
                file: file.clone(),
                start,
                end: pr.borrow().clone(),
            };
            i.set(index);
            tokens.borrow_mut().push(Token { span, ty: TokenType::Special(special[&sc]) });
        }
    };
    
    
    let check_comment = || {
        if let Some(s) = c.get(i.get() as usize.. (i.get() + 2) as usize) {
            let s = s.join("");
            if s == "//" {
                let mut p = pr.borrow_mut();
                p.character = 1;
                p.line += 1;
                i.set(i.get() + 2);
                loop {
                    if let Some(c) = c.get(i.get() as usize) {
                        i.set(i.get() + 1);
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
        let mut index = i.get();
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
        i.set(index);
        let span = SourceSpan {
            file: file.clone(),
            start,
            end: pr.borrow().clone(),
        };
        tokens.borrow_mut().push(Token {span, ty: TokenType::Float(whole + fraction)});
    };
    
    let check_int_float = || {
        let start = pr.borrow().clone();
        let mut s = String::with_capacity(10);
        let mut index = i.get();
        #[derive(PartialEq, Eq)]
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
                                    break 'b;
                                },
                                'b' => {
                                    s.clear();
                                    ty = LiteralType::Bin;
                                    break 'b;
                                },
                                '.' => {
                                    if ty != LiteralType::Dec {
                                        panic!("Float literals are only allowed in decimal notation");
                                    }
                                    i.set(index);
                                    check_float(u128::from_str_radix(s.as_str(), 10).unwrap() as f64, start);
                                    return;
                                }
                                _ => {}
                            }
                        }
                        index -= 1;
                        pr.borrow_mut().character -= 1;
                        break 'l;
                    }
                }
                s.push_str(c);
            } else {
                break;
            }
        }
        i.set(index);
        if s.len() == 0 {
            return;
        }
        let span = SourceSpan {
            file: file.clone(),
            start,
            end: pr.borrow().clone(),
        };
        let radix = match ty {
            LiteralType::Dec => 10,
            LiteralType::Hex => 16,
            LiteralType::Bin => 2,
        };
        tokens.borrow_mut().push(Token {span, ty: TokenType::Int(u128::from_str_radix(s.as_str(), radix).unwrap())});
    };
    
    
    
    
    let check_lifetime = || {
        let start = pr.borrow().clone();
        let mut s = String::with_capacity(10);
        let mut index = i.get();
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
        i.set(index);
        let span = SourceSpan {
            file: file.clone(),
            start,
            end: pr.borrow().clone(),
        };
        tokens.borrow_mut().push(Token {span, ty: TokenType::Lifetime(s.as_str().try_into().unwrap())});
    };
    
    let check_uniformity = || {
        let start = pr.borrow().clone();
        let mut s = String::with_capacity(10);
        let mut index = i.get();
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
        i.set(index);
        let span = SourceSpan {
            file: file.clone(),
            start,
            end: pr.borrow().clone(),
        };
        tokens.borrow_mut().push(Token {span, ty: TokenType::Lifetime(s.as_str().try_into().unwrap())});
    };
    
    loop {
        let start = i.get();
        gobble_whitespace();
        
        
        
        check_keyword_ident();
        check_special();
        
        
        check_lifetime();
        check_uniformity();
        
        check_int_float();
        
        check_comment();
        // TODO check doc comment
        if start == i.get() {
            let pb = p.borrow();
            panic!("Unable to match a character at index {}, line {}, position {}", i.get(), pb.line, pb.character);
        }
        if i.get() as usize >= chars.len() {
            tokens.borrow_mut().push(Token { span: SourceSpan { file: file.clone(), start: p.borrow().clone(), end: p.borrow().clone() }, ty: TokenType::End });
            break;
        }
    }
    return tokens.take();
}


#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn test_tokenize() {
        let p = Rc::new(PathBuf::new());
        assert_eq!(tokenize("", p.clone()), vec![Token {
            span: SourceSpan { file: p.clone(), start: SourcePos { line: 1, character: 1 }, end: SourcePos { line: 1, character: 1 } },
            ty: TokenType::Start
        },Token {
            span: SourceSpan { file: p.clone(), start: SourcePos { line: 1, character: 1 }, end: SourcePos { line: 1, character: 1 } },
            ty: TokenType::End
        }]);
        
        assert_eq!(tokenize("struct", p.clone()), vec![Token {
            span: SourceSpan { file: p.clone(), start: SourcePos { line: 1, character: 1 }, end: SourcePos { line: 1, character: 1 } },
            ty: TokenType::Start
        },Token {
            span: SourceSpan { file: p.clone(), start: SourcePos { line: 1, character: 1 }, end: SourcePos { line: 1, character: 7 } },
            ty: TokenType::Keyword(Keyword::Struct)
        },Token {
            span: SourceSpan { file: p.clone(), start: SourcePos { line: 1, character: 7 }, end: SourcePos { line: 1, character: 7 } },
            ty: TokenType::End
        }]);
        
        assert_eq!(tokenize(" \tstruct", p.clone()), vec![Token {
            span: SourceSpan { file: p.clone(), start: SourcePos { line: 1, character: 1 }, end: SourcePos { line: 1, character: 1 } },
            ty: TokenType::Start
        },Token {
            span: SourceSpan { file: p.clone(), start: SourcePos { line: 1, character: 3 }, end: SourcePos { line: 1, character: 9 } },
            ty: TokenType::Keyword(Keyword::Struct)
        },Token {
            span: SourceSpan { file: p.clone(), start: SourcePos { line: 1, character: 9 }, end: SourcePos { line: 1, character: 9 } },
            ty: TokenType::End
        }]);
        
        assert_eq!(tokenize("::", p.clone()), vec![Token {
            span: SourceSpan { file: p.clone(), start: SourcePos { line: 1, character: 1 }, end: SourcePos { line: 1, character: 1 } },
            ty: TokenType::Start
        },Token {
            span: SourceSpan { file: p.clone(), start: SourcePos { line: 1, character: 1 }, end: SourcePos { line: 1, character: 3 } },
            ty: TokenType::Special(Special::DoubleColon)
        },Token {
            span: SourceSpan { file: p.clone(), start: SourcePos { line: 1, character: 3 }, end: SourcePos { line: 1, character: 3 } },
            ty: TokenType::End
        }]);
    }
    
    
    
    
    
}

