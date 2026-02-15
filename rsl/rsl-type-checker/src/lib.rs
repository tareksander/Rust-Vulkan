use bitflags::bitflags;
use rsl_data::internal::{ir::{Function, Primitive, SymbolID, SymbolTable, Type}, CompilerData, Mutability, StringTable};







bitflags! {
    struct UniformitySet :u8 {
        const Uniform = 1 << 1;
        const SubgroupUniform = 1 << 2;
        const NonUniform = 1 << 3;
    }
    
    struct MutabilitySer :u8 {
        const Immutable = 1 << 1;
        const Mutable = 1 << 2;
    }
    
    struct StorageClassSet :u8 {
        const Function = 1 << 1;
        const Private = 1 << 2;
        const Workgroup = 1 << 3;
        const Storage = 1 << 4;
        const PhysicalStorage = 1 << 5;
    }
}



struct TypeID(usize);


enum TypeConstraint {
    Number,
    Float,
    Int,
    Sint,
    Uint,
    Primitive,
    Generic(SymbolID, Vec<TypeID>),
    Concrete(SymbolID),
    Implements {
        trait_id: SymbolID,
        params: Vec<TypeID>,
        outputs: Vec<TypeID>,
    }
}

// enum OrConstraints<T> {
//     Concrete(T),
//     Constraints(Vec<TypeConstraint>),
// }


// enum CheckerTypeVariant {
//     Unknown(Vec<TypeConstraint>),
//     Struct(SymbolID),
//     Primitive(Primitive),
//     Vector {
//         components: OrConstraints<u8>,
//         ty: OrConstraints<Primitive>
//     },
//     Array {
//         length: OrConstraints<u8>,
//         ty: OrConstraints<Primitive>
//     },
//     Matrix {
//         rows: OrConstraints<u8>,
//         cols: OrConstraints<u8>,
//         ty: OrConstraints<Primitive>
//     },
//     Pointer {
//         mutability: Mutability,
//         class: StorageClassSet,
//         ty: TypeID
//     }
// }

// struct CheckerType {
//     uni: Option<UniformitySet>,
//     ty: Box<CheckerTypeVariant>,
// }



pub fn type_checking(data: &CompilerData, symbols: &SymbolTable, strings: &StringTable, function: &Function) {
    let ir = function.blocks.borrow_mut();
    let mut type_vars: Vec<TypeID> = vec![];
    
    
    
}





#[cfg(test)]
mod tests {
    use std::{hint::black_box, path::PathBuf, time::Instant};

    use rsl_data::internal::SourceSpan;
    use rsl_data::internal::{ReportSourceCache, Sources, StringTable, ir::SymbolTable};
    use rsl_lexer::tokenize;
    use rsl_parser::parse_file;

    use super::*;
    use super::*;

    #[test]
    fn simple() -> Result<(), ()> {
        let strings = StringTable::new();
        let code = "#[compute] fn test(a: dispatch *const u32, b: dispatch *const u32, c: dispatch *mut u32) { c[globalInvocationID.x] = a[globalInvocationID.x] + b[globalInvocationID.x]; }";
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
                
                let t1 = Instant::now();
                let (m, e) = parse_file(&tokens, &spans, 0, vec![], &strings);
                if ! e.is_empty() {
                    e.iter().for_each(|e| e.eprint(&mut cache).unwrap());
                    return Err(());
                }
                let t2 = Instant::now();
                println!("Time: {} chars, {} ms", code.len(), (t2- t1).as_millis());
                println!("{:#?}", SymbolTable::from_module(m, &strings));
                
                let core = SymbolTable::core(&strings);
                let toplevel = SymbolTable::new();
                
                
                
            },
            Err(r) => {
                r.print(cache).unwrap();
                return Err(());
            },
        }
        return Ok(());
        
        
        
        
        
    }
}
