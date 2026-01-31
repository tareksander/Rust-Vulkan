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
    use rsl_data::internal::{ir::SymbolTable, StringTable};

    use super::*;

    #[test]
    fn simple() {
        let strings = StringTable::new();
        let symbols = SymbolTable::new();
        
        
        
        
        
        
        
    }
}
