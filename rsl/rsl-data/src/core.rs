use std::fmt::format;

use crate::mid::Scope;





pub fn generate_core() -> Scope {
    let mut source = "#![internal_attributes]\n".to_owned();
    let generate_arith_trait = |name: &str| {
        let upper_name = name[0..1].to_ascii_uppercase() + &name[1..];
        format(format_args!("#[lang({name})]\npub trait {upper_name}<Rhs, ~lhs, ~rhs> {{\n    type Output;\n    fn nonu {name}(~lhs self, rhs: ~rhs Rhs) -> Self::Output;\n}}"))
    };
    // Only impls among primitives themselves needed, coercions can handle the rest.
    // for coercion of ints to floats, f32 has 24 mantissa bits, f64 has 53 bits, f16 has 11 bits.
    // Only coercions that fit in the float's mantissa are allowed, other casts have to be explicit with "as", as they can loose precision.
    let generate_arith_impl = |trait_name: &str, type_name: &str| {
        let upper_trait_name = trait_name[0..1].to_ascii_uppercase() + &trait_name[1..];
        format(format_args!("impl {upper_trait_name}<{type_name}> for {type_name} {{\n    type Output = {type_name};\n    fn nonu {trait_name}(self, rhs: {type_name}) -> Self::Output;\n}}"))
    };
    let generate_vec = |n: u32| {
        format(format_args!("#[lang(vec{n})]\npub struct vec{n}<T> {{\n    data: [T; {n}]\n}}"))
    };
    
    source += &generate_arith_trait("add");
    source += &generate_arith_trait("sub");
    source += &generate_arith_trait("mul");
    source += &generate_arith_trait("div");
    source += &generate_arith_trait("mod");
    
    source += &generate_vec(2);
    source += &generate_vec(3);
    source += &generate_vec(4);
    
    
    
    
    
    todo!()
}














