# RSL - Rusty Shading Language

A shading language for modern[^1] Vulkan with Rust-like syntax.

🚧 This project is in early development and not fit for general usage yet. If you do want to try it out, expect bugs and breakage in future versions 🚧



````Rust
use ::globalInvocationId;

struct PushConstants {
    a: *const PhysicalStorage uni u32,
    b: *const PhysicalStorage uni u32,
    c: *mut PhysicalStorage nuni u32,
}

#[push(0)]
static PUSH: PushConstants;

#[compute(1, 1, 1)]
fn unsafe add() {
    let i = globalInvocationId.x;
    PUSH.C[i] = PUSH.A[i] + PUSH.B[i];
}
````





## Features

- [x] Mostly Rust-like syntax
    - [x] If and loops as expressions
    - [ ] Enums
        - [ ] Numeric
        - [ ] With data
    - [x] Tuples
    - [ ] Destructuring Tuples
        - [ ] Enums with values
        - [ ] Match expressions
        - [ ] Destructuring structs enums
- [ ] Capability system
    - [ ] Profile for Vulkan 1.3 + desktop baseline
    - [ ] Roadmap profiles
    - [ ] 1.4 profile
    - [ ] Profile for Android 16
- [ ] Vulkan buffer binding & compute shaders
    - [ ] Scalar block layout
    - [ ] Extended alignment
    - [ ] Basic alignment
- [ ] Vertex & fragment shaders
- [ ] Image & sampler binding
    - [ ] Builtins
- [ ] Push constants
- [ ] Specialization constants
- [ ] WGSL backend?
    - [ ] Uniformity analysis
- [ ] Rust backend
- [ ] Generator for Rust and C struct definitions
    - [ ] Automatic binding assignment with generated info for consuming the SPIR-V module
        - [ ] Bounds for automatic assignment, e.g. by the selected Vulkan profile
- [ ] Documentation
    - [ ] Tutorial
        - [ ] Compute shader
        - [ ] Vertex & fragment shader
    - [ ] Specification


## Detailed Roadmap

- [ ] Parsing
    - [x] Structs
        - [ ] Impl blocks and associated functions
    - [x] Functions
    - [ ] Statements
        - [x] let
        - [ ] Destructuring let
    - [ ] Expressions
        - [x] Algebra
        - [ ] If
            - [ ] If let
        - [ ] Match
        - [x] Tuples
        - [x] Assignment
        - [ ] loops
            - [ ] loop
            - [ ] while
            - [ ] do-while
            - [ ] for (numeric)
    - [ ] Enums
    - [ ] Storage classes for pointers: Storage, PhysicalStorage, Uniform, Workgroup, Function, Private
        - [ ] static pointers have a default storage class of PhysicalStorage (which is also the only allowed one)
        - [ ] static pointers use base alignment by default. Only really important with vec3.
            - [ ] For custom types, alignment is specified via the repr attribute
    - [ ] Uniformity for variables: Uniform, dynamically uniform, non-uniform
    - [ ] Attributes
- [ ] Builtin attributes
    - [ ] repr
        - [ ] std430/base
        - [ ] std140/extended
        - [ ] scalar
        - [x] rsl
    - [ ] set
    - [ ] binding
    - [ ] stage
        - [x] all (default)
        - [ ] compute
        - [ ] vertex
        - [ ] fragment
    - [ ] entry
        - [ ] compute
        - [ ] vertex
        - [ ] fragment
    - [ ] builtin variables
        - [ ] global & local invocation ID for compute
        - [ ] base instance
        - [ ] base vertex
        - [ ] clip distance
        - [ ] cull distance
        - [ ] device index
        - [ ] draw index
        - [ ] frag coord
        - [ ] frag depth
        - [ ] front facing
        - [ ] helper invocation
        - [ ] instance id
        - [ ] invocation id
        - [ ] instance index
        - [ ] layer
        - [ ] local invocation index
        - [ ] num subgroups
        - [ ] num workgroups
        - [ ] position
        - [ ] primitive id
        - [ ] sample id
        - [ ] sample mask
        - [ ] sample position
        - [ ] subgroup id
        - [ ] subgroup local invocation id
        - [ ] subgroup size
        - [ ] vertex index
        - [ ] view index
        - [ ] viewport index
        - [ ] workgroup id
        - [ ] workgroup size
- [ ] Checking
    - [/] Type checking
        - [ ] Constraint checking
            - [ ] Constraints on buffer types
    - [ ] Borrow checking
- [ ] Desugaring
    - [ ] while and to-while to loop
    - [ ] destructuring let to multiple lets with the tuple cached in a temporary variable
    - [ ] enums are lowered to a tag and an array of the widest type they contain, which is reinterpreted on an if let or match
        - [ ] in the storage buffer storage class, enums use an array of u8, the pointer to which is cast to the correct type on access
    - [ ] References as parameters or return values are desugared into pointers
- [ ] Codegen
    - [ ] storage buffer
    - [ ] uniform buffer
    - [ ] Compute
    - [ ] Expressions
    - [ ] Return
    - [ ] If
        - [ ] If-expression with phy
    - [ ] loops
    - [ ] storage images
    - [ ] sampled images & samplers
    - [ ] vertex & fragment






## Setup

A correctly installed [Vulkan SDK][vksdk] is needed, with all programs accessible via the `PATH` environment variable. As the language is not yet strictly defined, you should pin a compiler version to avoid breaking on language changes.





### Why not `rust-gpu`?

`rust-gpu` aims to compile the full Rust language to SPIR-V. However, many algorithms could benefit from a better implementation on a SIMT architecture anyways, so full source compatibility shouldn't be needed.



### Why not [WGSL][wgsl]?

[WGSL][wgsl] is mostly rust-like, but a completely safe and limited subset of shading languages. With enough extensions to cover all of SPIR-V, you'd have a whole different language than standard WGSL anyways.


### Why not [Slang][slang]?

[Slang][slang] is not purpose-build for SPIR-V and e.g. only supports pointers in a limited way. It is also an amalgamation of GLSL, HLSL and custom syntax and currently not fully documented.











[^1]: Features that are both in Android Baseline Profile 2022 and in Vulkan Desktop Baseline Profile 2022 are required.


[slang]: https://shader-slang.org/
[wgsl]: https://gpuweb.github.io/gpuweb/wgsl/
[vksdk]: https://vulkan.lunarg.com/sdk/home


