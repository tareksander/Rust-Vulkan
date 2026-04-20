


mod passes;














#[cfg(test)]
mod tests {
    use std::{fs, hint::black_box, path::PathBuf, time::Instant};

    use rsl_data::internal::{ir::{GlobalItem, SymbolTable}, *};
    use rsl_lexer::tokenize;
    use rsl_parser::parse_file;
    use rsl_type_checker::type_checking;

    use crate::passes::{emit_spirv::emit_spirv, logical_pointer_specialization::logical_pointer_specialization};

    use super::*;

    #[test]
    fn basic() -> Result<(), ()> {
        let strings = StringTable::new();
        let code_template = r"
        #[compute]
        fn test(a: *const f32, b: *const f32, c: *mut f32, f: f32)
        {
            c[globalInvocationID.x] = a[globalInvocationID.x] * b[globalInvocationID.x] + testu(&a[globalInvocationID.x]);
        }
        
        fn testu(a: *const f32) -> f32 {
            *a * 2.0
        }
        ";
        
        
        let mut code = String::new();
        const N: usize = 1;
        for i in 0..N {
            code += &code_template.replace("test", &("test".to_string() + &i.to_string()));
        }
        let code = black_box(code);
        let mut cache = ReportSourceCache::new(&Sources {
            source_files: vec![PathBuf::from("test.rsl")],
            source_strings: vec![code.to_string()]
        });
        let t0 = Instant::now();
        let res = tokenize(&code, 0, &strings);
        match res {
            Ok((tokens, spans)) => {
                let spans = spans.iter().map(|r| SourceSpan {
                    file: 0,
                    start: r.start,
                    end: r.end,
                }).collect::<Vec<_>>();
                
                let (m, e) = parse_file(&tokens, &spans, 0, vec![], &strings);
                if ! e.is_empty() {
                    e.iter().for_each(|e| e.eprint(&mut cache).unwrap());
                    return Err(());
                }
                let m = SymbolTable::from_module(m, &strings);
                
                let core = SymbolTable::core(&strings);
                let mut toplevel = SymbolTable::new();
                toplevel.insert_module(core, strings.insert_get("core"));
                toplevel.insert_module(m, strings.insert_get("test"));
                
                toplevel.resolve_paths(&strings);
                //toplevel.eval_constexprs();
                
                for s in toplevel.iter() {
                    let name = toplevel.get_name(s);
                    println!("{}", strings.lookup(name));
                }
                
                
                //println!("{:#?}", toplevel);
                
                let t1 = Instant::now();
                //println!("Time: {} ms", (t1- t0).as_millis());
                for i in 0..N {
                    type_checking(&toplevel, &strings, match &toplevel.lookup(&strings.insert_get(&(format!("::test::test{}", i)))).unwrap().1 {
                        GlobalItem::Function(function) => function,
                        _ => panic!()
                    });
                    type_checking(&toplevel, &strings, match &toplevel.lookup(&strings.insert_get(&(format!("::test::test{}u", i)))).unwrap().1 {
                        GlobalItem::Function(function) => function,
                        _ => panic!()
                    });
                }
                
                logical_pointer_specialization(&mut toplevel, &strings);
                //println!("{:#?}", toplevel);
                fs::write("test.spv", bytemuck::cast_slice(emit_spirv(&mut toplevel, &strings).as_slice())).unwrap();
                
                let t2 = Instant::now();
                println!("Time: {} ms", (t2- t1).as_millis());
            },
            Err(r) => {
                r.print(cache).unwrap();
                return Err(());
            },
        }
        return Ok(());
    }
}
