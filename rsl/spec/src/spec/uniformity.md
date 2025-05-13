# Uniformity

Uniformity exists at the workgroup and subgroup level. Where something isn't uniform on the workgroup or subgroup level, it is said to be non-uniform. If something is uniform at the workgroup level, it is also uniform at the subgroup level.


## Value Uniformity

Value uniformity provides the basis of control flow uniformity. Most inputs to a shader are workgroup uniform, like the uniform and storage buffers and push constants. Some values like the global invocation ID are non-uniform. Since each shader invocation usually computes a different result to the other ones (at least it cannot be statically known), complete value uniformity isn't desired.

Whenever an operation produces a value, unless otherwise noted its uniformity is the least uniform of all its operands. E.g. by indexing a uniform storage buffer array with a non-uniform value (e.g. the global invocation id), the resulting value is non-uniform.

A notable exception are references and pointers: Loading from pointers results in the uniformity specified in the pointee type. E.g. dereferencing a non-uniform pointer to a uniform value will result in a uniform value, despite the only operand to the operation being non-uniform.

Uniformity is encoded in the type system: A type specifies its uniformity, and only values that are at least as uniform as specified can be stored there. Due to inference this should be mostly negligible, but function signatures have to specify their type uniformity (or use generics in the future).


## Control Flow Uniformity


Control flow starts as workgroup uniform upon starting the entrypoint, but can diverge by branching on subgroup or non-uniform values. In such cases, uniformity is restored when exiting the control flow structure that caused it to diverge. An exception to that are return statements. Without the maximal reconvergence capability, returning inside non-uniform control flow is not allowed, however the compiler may hoist the return out of the non-uniform region in the future. The maximal recovergence capability ensures that uniformity is restored to the most uniform possible at any point in the program, including using function returns as a reconvergence point.

Because some intrinsics require some degree of uniformity, like sampling textures and subgroup operations, functions can define the needed amount of control flow uniformity to be  allowed to be called. This ensures the compiler can put the blame on the caller code instead of generating a uniformity violation error for intrinsics inside the function.


