# Syntax

## Common utility definitions

`Visibility = pub | priv | pack`

Visibility: Public (exported), private (only this module and submodules) and package (only this shader library) respectively.


`Uniformity = uni | suni | nonu`

Uniformity: Workgroup, subgroup, and non-uniform respectively.


`DelimitedList<T> = [T] {"," T}`

A delimited list of T, for easier representation of the syntax.



## Module Files

`ModuleFile = {StructDefinition, FunctionDefinition, StaticDefinition, Module}`

This is the top-level language construct.





## Function definitions

`FunctionDefinition = [Visibility] fn [Uniformity] Ident "(" DelimitedList<Ident ":" Type> ")" ["->" Type] Block`

The notable deviation from standard programming languages is the optional function uniformity requirement. A function can only be called if the uniformity requirement is met. The default is `nonu`, which means a function can be called from non-uniform control flow, as well as subgroup and workgroup uniform control flow.
`suni` denotes subgroup uniform or workgroup uniform control flow, while `uni` functions can only be called from workgroup uniform control flow.

The full function contract also includes the type uniformity. For more information, see the subchapter about uniformity.











