use std::{cell::{Cell, RefCell}, collections::HashMap, ops::Range};

use ariadne::{Color, Label, Report};
use unicode_segmentation::UnicodeSegmentation;

use rsl_data::internal::{tokens::{Keyword, Special, Token}, SourceSpan, StringTable};






pub fn tokenize<'a>(code: &'a str, file: usize, strings: &StringTable) -> Result<(Vec<Token>, Vec<Range<usize>>), Report<'a, SourceSpan>> {
    let i = Cell::new(0);
    let ascii_char_at = |i: usize| {
        let c = code.as_bytes().get(i);
        if let Some(c) = c {
            let c = *c as char;
            if c.is_ascii() {
                return Ok(Some(c));
            } else {
                let span = SourceSpan {
                    file,
                    start: i,
                    end: i,
                };
                let uc = code[i..].graphemes(true).next().unwrap();
                return Err(Report::build(ariadne::ReportKind::Error, span)
                    .with_message(format!("Unexpected Unicode character: {}", uc))
                    .with_label(Label::new(span).with_message("Here").with_color(Color::Red))
                    .with_note("Unicode characters are not allowed outside of comments, doc comments, string and char literals.").finish());
            }
        } else {
            return Ok(None);
        }
    };
    let gobble_whitespace = || {
        loop {
            if let Some(c) = ascii_char_at(i.get())? {
                if c.is_ascii_whitespace() {
                    i.set(i.get()+1);
                    continue;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        return Ok(())
    };
    
    let tokens: RefCell<Vec<Token>> = Vec::with_capacity(code.len()/4).into();
    let spans: RefCell<Vec<SourceSpan>> = Vec::with_capacity(code.len()/4).into();
    tokens.borrow_mut().push(Token::Start);
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
        keywords.insert("type", Type);
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
    
    let must_ident = || {
        let start = i.get();
        loop {
            if let Some(c) = ascii_char_at(i.get())? {
                if i.get() == start {
                    if c.is_ascii_alphabetic() || c == '_' {
                        i.set(i.get()+1);
                    } else {
                        break;
                    }
                } else {
                    if c.is_ascii_alphanumeric() || c == '_' {
                        i.set(i.get()+1);
                    } else {
                        break;
                    }
                }
            } else {
                break;
            }
        }
        if start == i.get() {
            let span = SourceSpan { file, start, end: start };
            return Err(Report::build(ariadne::ReportKind::Error, span)
                .with_message("Expected identifier")
                .with_label(Label::new(span).with_message("Here").with_color(Color::Red)).finish());
        }
        let s  = &code[start..i.get()];
        return Ok(s);
    };
    
    let check_keyword_ident = || {
        let start = i.get();
        loop {
            if let Some(c) = ascii_char_at(i.get())? {
                if i.get() == start {
                    if c.is_ascii_alphabetic() || c == '_' {
                        i.set(i.get()+1);
                    } else {
                        break;
                    }
                } else {
                    if c.is_ascii_alphanumeric() || c == '_' {
                        i.set(i.get()+1);
                    } else {
                        break;
                    }
                }
            } else {
                break;
            }
        }
        if start == i.get() {
            return Ok(());
        }
        let s  = &code[start..i.get()];
        let span = SourceSpan {
            file,
            start,
            end: i.get()
        };
        spans.borrow_mut().push(span);
        if let Some(kw) = keywords.get(s) {
            tokens.borrow_mut().push(Token::Keyword(*kw));
        } else {
            tokens.borrow_mut().push(Token::Ident(strings.insert_get(s)));
        }
        return Ok(());
    };
    
    let check_generic_uniformity = || {
        let start = i.get();
        if let Some(c) = ascii_char_at(i.get())? {
            if c == '#' {
                let s = must_ident()?;
                spans.borrow_mut().push(SourceSpan { file, start, end: i.get() });
                tokens.borrow_mut().push(Token::Uniformity(strings.insert_get(s)));
            }
        }
        return Ok(());
    };
    
    let check_comment = || {
        if let Some(c) = ascii_char_at(i.get())? {
            if c == '/' {
                if let Some(c) = ascii_char_at(i.get()+1)? {
                    if c == '/' {
                        i.set(i.get()+2);
                        for g in code[i.get()..].graphemes(true) {
                            i.set(i.get()+g.len());
                            if g.contains('\n') {
                                break;
                            }
                        }
                    }
                }
            }
        }
        return Ok(());
    };
    
    let check_doc_comment = || {
        if let Some(c) = ascii_char_at(i.get())? {
            if c == '/' {
                if let Some(c) = ascii_char_at(i.get())? {
                    if c == '/' {
                        if let Some(c) = ascii_char_at(i.get())? {
                            if c == '/' {
                                i.set(i.get()+3);
                                let start = i.get();
                                for g in code[i.get()..].graphemes(true) {
                                    let end = i.get();
                                    i.set(i.get()+g.len());
                                    if g.contains('\n') {
                                        spans.borrow_mut().push(SourceSpan { file, start, end });
                                        tokens.borrow_mut().push(Token::DocComment(code[start..end].to_string()));
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        return Ok(());
    };
    
    
    let check_special = || {
        let start = i.get();
        'b: {
            if let Some(c) = ascii_char_at(i.get())? {
                if c == '=' || c == '-' {
                    if let Some(c2) = ascii_char_at(i.get()+1)? {
                        if c2 == '>' {
                            i.set(i.get()+2);
                            spans.borrow_mut().push(SourceSpan { file, start, end: i.get() });
                            if c == '=' {
                                tokens.borrow_mut().push(Token::Special(Special::ThickArrow));
                            }
                            if c == '-' {
                                tokens.borrow_mut().push(Token::Special(Special::ThinArrow));
                            }
                            break 'b;
                        }
                    }
                }
                if special_double.contains_key(&c) {
                    if let Some(c2) = ascii_char_at(i.get()+1)? {
                        if c2 == c {
                            i.set(i.get()+2);
                            spans.borrow_mut().push(SourceSpan { file, start, end: i.get() });
                            tokens.borrow_mut().push(Token::Special(special_double[&c]));
                            break 'b;
                        }
                    }
                }
                if special.contains_key(&c) {
                    i.set(i.get()+1);
                    spans.borrow_mut().push(SourceSpan { file, start, end: i.get() });
                    tokens.borrow_mut().push(Token::Special(special[&c]));
                }
            }
        }
        return Ok(());
    };
    
    
    let check_float = |whole: f64, start: usize| {
        let fraction_start = i.get();
        loop {
            if let Some(c) = ascii_char_at(i.get())? {
                if c.is_ascii_digit() {
                    i.set(i.get()+1);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        let fs = &code[fraction_start..i.get()];
        let span = SourceSpan { file, start, end: i.get() };
        let mut number = whole;
        if fs.len() != 0 {
            if let Ok(fraction) = u128::from_str_radix(fs, 10) {
                number += fraction as f64 / 10.0_f64.powf(fs.len() as f64);
            } else {
                return Err(Report::build(ariadne::ReportKind::Error, span)
                .with_message(format!("Float fractional part too big"))
                .with_label(Label::new(span).with_message("Here").with_color(Color::Red)).finish());
            }
        }
        spans.borrow_mut().push(span);
        tokens.borrow_mut().push(Token::Float(number));
        return Ok(());
    };
    
    
    let check_int_float = || {
        let start = i.get();
        let mut number_start = start;
        #[derive(Debug, PartialEq, Eq)]
        enum LiteralType {
            Dec, Hex, Bin
        }
        let mut ty = LiteralType::Dec;
        loop {
            if let Some(c) = ascii_char_at(i.get())? {
                i.set(i.get()+1);
                if start+1 == i.get() && c == '0' && ty == LiteralType::Dec {
                    match ascii_char_at(i.get())? {
                        Some('x') => {
                            i.set(i.get()+1);
                            number_start = i.get();
                            ty = LiteralType::Hex;
                            continue;
                        },
                        Some('b') => {
                            i.set(i.get()+1);
                            number_start = i.get();
                            ty = LiteralType::Bin;
                            continue;
                        },
                        _ => {}
                    }
                } else {
                    if c.is_ascii_hexdigit() {
                        let span1 = SourceSpan { file, start, end: i.get() };
                        let span2 = SourceSpan { file, start: i.get(), end: i.get() };
                        if ty == LiteralType::Bin {
                            if c != '0' && c != '1' {
                                return Err(Report::build(ariadne::ReportKind::Error, span1)
                                    .with_message(format!("Invalid digit in binary int literal"))
                                    .with_label(Label::new(span2).with_message("Here").with_color(Color::Red)).finish());
                            }
                        }
                        if ty == LiteralType::Dec {
                            if ! c.is_ascii_digit() {
                                return Err(Report::build(ariadne::ReportKind::Error, span1)
                                    .with_message(format!("Invalid digit in decimal int literal"))
                                    .with_label(Label::new(span2).with_message("Here").with_color(Color::Red)).finish());
                            }
                        }
                    } else {
                        if c == '.' {
                            if ty != LiteralType::Dec {
                                let span = SourceSpan { file, start, end: i.get() };
                                return Err(Report::build(ariadne::ReportKind::Error, span)
                                    .with_message(format!("Float literals are only supported in decimal notation"))
                                    .with_label(Label::new(span).with_message("Non-decimal float literal").with_color(Color::Red)).finish());
                            }
                            match u128::from_str_radix(&code[number_start..(i.get()-1)], 10) {
                                Ok(whole) => {
                                    check_float(whole as f64, start)?;
                                    return Ok(());
                                },
                                Err(_) => {
                                    let span = SourceSpan { file, start, end: i.get()-1 };
                                    return Err(Report::build(ariadne::ReportKind::Error, span)
                                    .with_message(format!("Float whole part too big"))
                                    .with_label(Label::new(span).with_message("Here").with_color(Color::Red)).finish());
                                },
                            }
                        }
                        i.set(i.get()-1);
                        break;
                    }
                }
            } else {
                break;
            }
        }
        if start == i.get() {
            return Ok(());
        }
        let radix = match ty {
            LiteralType::Dec => 10,
            LiteralType::Hex => 16,
            LiteralType::Bin => 2,
        };
        let span = SourceSpan { file, start, end: i.get() };
        match u128::from_str_radix(&code[number_start..i.get()], radix) {
            Ok(number) => {
                spans.borrow_mut().push(span);
                tokens.borrow_mut().push(Token::Int(number));
            },
            Err(_) => {
                let span = SourceSpan { file, start, end: i.get()-1 };
                return Err(Report::build(ariadne::ReportKind::Error, span)
                .with_message(format!("Int literal too big"))
                .with_label(Label::new(span).with_message("Here").with_color(Color::Red)).finish());
            },
        }
        return Ok(());
    };
    
    
    
    loop {
        let start = i.get();
        
        gobble_whitespace()?;
        check_keyword_ident()?;
        check_generic_uniformity()?;
        check_comment()?;
        check_special()?;
        check_int_float()?;
        check_doc_comment()?;
        
        let span = SourceSpan { file, start: i.get(), end: i.get() };
        if i.get() >= code.len() {
            spans.borrow_mut().push(span);
            tokens.borrow_mut().push(Token::End);
            break;
        }
        // if nothing matched, we have an unknown character
        if start == i.get() {
            let c = code[i.get()..].graphemes(true).next().unwrap();
            let mut r = Report::build(ariadne::ReportKind::Error, span)
            .with_message(format!("Unexpected character: {}", c))
            .with_label(Label::new(span).with_message("Here").with_color(Color::Red));
            if ! c.chars().next().unwrap().is_ascii() {
                r.add_note("Unicode characters are not allowed outside of comments, doc comments, string and char literals.");
            }
            return Err(r.finish());
        }
    }
    return Ok((tokens.take(), spans.take().iter().map(|s| s.start..s.end).collect()));
}


#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use rsl_data::internal::{ReportSourceCache, Sources};

    use super::*;
    
    #[test]
    fn test_tokenize() -> Result<(), ()> {
        let strings = StringTable::new();
        let code = "pub fn foo() 1. /";
        let cache = ReportSourceCache::new(&Sources {
            source_files: vec![PathBuf::from("test.rsl")],
            source_strings: vec![code.to_string()]
        });
        let res = tokenize(code, 0, &strings);
        match res {
            Ok((tokens, _)) => {
                println!("{:#?}", tokens);
                println!("{:#?}", strings);
            },
            Err(r) => {
                r.print(cache).unwrap();
                return Err(());
            },
        }
        return Ok(());
    }
    
    
    
    
    
}

