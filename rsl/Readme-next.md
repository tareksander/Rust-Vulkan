

# RSL - Rusty Shading Language<img src="logo-small.png" width="5%">

<!--Fearless GPU compute.-->

A shading language for modern[^1] Vulkan with Rust-like syntax.

🚧 This project is in early development and not fit for general usage yet. If you do want to try it out, expect bugs and breakage in future versions 🚧

````Rust
use ::globalInvocationId;

#[compute(1, 1, 1)]
fn add(
    a: *const PhysicalStorage uni u32,
    b: *const PhysicalStorage uni u32,
    c: *mut PhysicalStorage nuni u32,
) {
    let i = globalInvocationId.x;
    a[i] = b[i] + c[i];
}
````


## Roadmap

- [ ] 0.1: MVP
    - [ ] Compute shaders
    - [ ] Push constants
    - [ ] Physical pointers
    - [ ] Uniform & storage buffers
    - [ ] Images & samplers
- [ ] 0.2: Shading
    - [ ] Vertex shaders
    - [ ] Fragment shaders
- [ ] 0.3: Synchronization
    - [ ] Barriers
    - [ ] Workgroup memory
    - [ ] ~~Nonprivate pointers for shader communication~~ (All pointers should be lowered as nonprivate)
        - [ ] Availability and visibility with specific loads and stores
- [ ] 0.4: Borrow checking
    - [ ] Some borrow checker implementation, probably more simplistic than Rust
- [ ] 0.5: Capability enforcement & profiles
    - [ ] Capabilities will be correctly enforced
    - [ ] Profiles for common capability sets
    - [ ] custom profiles via toml files
- [ ] 0.6: Polishing the language
    - [ ] Look for any deficiencies
    - [ ] Fastmath controls
    - [ ] Debug mode to check for some runtime UB in shaders
- [ ] 0.7: Extensions
    - [ ] Add more SPIR-V extensions
- [ ] 1.0
    - [ ] Stabilize the compiler API
    - [ ] Struct definition builder for Rust








## Setup

A correctly installed [Vulkan SDK][vksdk] is needed, with all programs accessible via the `PATH` environment variable. As the language is not yet strictly defined, you should pin a compiler version to avoid breaking on language changes.





### Why not `rust-gpu`?

`rust-gpu` aims to compile the full Rust language to SPIR-V. However, many algorithms could benefit from a better implementation on a SIMT architecture anyways, so full source compatibility shouldn't be needed.



### Why not [WGSL][wgsl]?

[WGSL][wgsl] is mostly rust-like, but a completely safe and limited subset of shading languages. With enough extensions to cover all of SPIR-V, you'd have a whole different language than standard WGSL anyways.


### Why not [Slang][slang]?

[Slang][slang] is not purpose-build for SPIR-V and e.g. only supports pointers in a limited way. It is also an amalgamation of GLSL, HLSL and custom syntax and currently not fully documented.









[^1]: Features that are both in Android Baseline Profile 2022 and in Vulkan Desktop Baseline Profile 2022 will always be required. During initial development of the language, the main target will be Desktop Baseline Profile 2024 and Android 15. The requirements may be raised to that if no use case for lower versions arises.


[slang]: https://shader-slang.org/
[wgsl]: https://gpuweb.github.io/gpuweb/wgsl/
[vksdk]: https://vulkan.lunarg.com/sdk/home




