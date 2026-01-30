# Syntax



The syntax is specified in a sort of BFN. Keywords are used without quotes, Words starting with a capital letter are generally a language construct produced by a rule (Expect Ident, which is an identifier token). Special characters are quoted when they have to appear as tokens. `|` means "or", spaces between symbols means "must be followed by", with `|` having lower precedence. `[]` means optional, `{}` means any number of times.



## Common utility definitions

`Visibility = pub | priv | pack`

Visibility: Public (exported), private (only this module and submodules) and package (only this shader library) respectively.


`Uniformity = uni | suni | nuni`

Uniformity: Workgroup/Draw, subgroup, and non-uniform respectively.


`DelimitedList<T> = [T] {"," T}`

A delimited list of T, for easier representation of the syntax.



## Module

`Module = {StructDefinition | FunctionDefinition | StaticDefinition | ModuleDefinition | ModuleInline}`

This is the top-level language construct.

`ModuleDefinition = mod Ident ";"`


`ModuleInline = mod Ident "{" Module "}"`



## Function definitions

`FunctionDefinition = [Visibility] fn ["<" DelimitedList<GenericArgDefinition> ">"] [Uniformity] Ident "(" DelimitedList<Ident ":" Type> ")" ["->" Type] Block`

The notable deviation from standard programming languages is the optional function uniformity requirement. A function can only be called if the uniformity requirement is met. The default is `nuni`, which means a function can be called from non-uniform control flow, as well as subgroup and workgroup uniform control flow.
`suni` denotes subgroup uniform or workgroup uniform control flow, while `uni` functions can only be called from workgroup uniform control flow.

The full function contract also includes the type uniformity. For more information, see the subchapter about uniformity.

As a shorthand, if the uniformity is omitted anywhere in a function signature, the function is treated as a generic function with an unnamed uniformity parameter that is used in place of all the omitted uniformities. That is `fn foo(a: u32, b: u32) -> u32` is equivalent to `fn<#U> #U foo(a: #U u32, b: #U u32) -> #U u32`. For entrypoints, the uniformity defaults to uniform instead.




