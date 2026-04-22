
# This Readme doesn't represent the current state of the project, but a work-in-progress readme for a future, more feature-full version


# Warpfield

Fearless GPU programming

A shading language for modern[^1] Vulkan with Rust-like syntax.

🚧 This project is in early development and not fit for general usage yet. If you do want to try it out, expect bugs and breakage in future versions 🚧

````Rust
pub compute fn add(
    a: InvocationBuffer<u32>,
    b: InvocationBuffer<u32>,
    c: InvocationBuffer<u32>,
) {
    *c = *a + *b;
}
````


## Roadmap

- [ ] 0.1: MVP
    - [ ] Compute shaders
    - [ ] Images & samplers
        - Using a bindless scheme that should map well to descriptor indexing and descriptor heaps
    - [ ] Work graphs by dispatching from a compute shader
        - More like "work trees" for now, figure out a graph structure later
- [ ] 0.2: Shading
    - [ ] Vertex shaders
    - [ ] Fragment shaders
    - [ ] Draw calls and render passes from compute shaders
- [ ] 0.4: Borrow checking & lifetimes
    - [ ] Some borrow checker implementation, probably more simplistic than Rust
    - [ ] Data flow analysis based on borrows to run dispatches and draws in parallel without synchronization in between when possible
- [ ] 0.5: Capability enforcement & profiles
    - [ ] Capabilities will be correctly enforced
    - [ ] Profiles for common capability sets
    - [ ] Custom profiles via toml files
- [ ] 0.6: Polishing the language
    - [ ] Look for any deficiencies
    - [ ] Fastmath controls
- [ ] 0.7: Extensions
    - [ ] Add more SPIR-V extensions
    - [ ] Cooperative matrices
    - [ ] Ray tracing
    - [ ] Mesh shaders
    - [ ] Shader Clock
- [ ] 0.8
    - [ ] Struct definition builder for C
    - [ ] Standard library
        - [ ] optimized prefix sum
        
- [ ] 1.0
    - [ ] Stabilize the compiler API
    - [ ] Rust Macro API
    - [ ] Test tool
    - [ ] SPIR-V interpreter for catching memory model violations and UB
- [ ] 1.1
    - [ ] Python bindings
    - [ ] Better iGPU support
        - [ ] DGC not required, instead the shared memory with relatively low latency between CPU and GPU can be used to encode command buffers from the CPU
        - [ ] Use host image copy and eliminate potential redundant copys (because host and device memory are expected to be the same, with a large heap that is device local and host visible)


## Setup

A correctly installed [Vulkan SDK][vksdk] is needed, with all programs accessible via the `PATH` environment variable. As the language is not yet strictly defined, you should pin a compiler version to avoid breaking on language changes.



<!-- TODO: Rethink stance on rust-gpu and slang, maybe point out the native work and render graph features of Warpfield as a reason to differentiate -->

### Why not `rust-gpu`?

`rust-gpu` aims to compile the full Rust language to SPIR-V. However, many algorithms could benefit from a better implementation on a SIMT architecture anyways, so full source compatibility shouldn't be needed. As a side effect, you can integrate GPU concepts like uniformity deeper into the compiler to get better error messages and inlay hints for performance guides to detect divergent control flow.


### Why not [WGSL][wgsl]?

[WGSL][wgsl] is mostly Rust-like, but a completely safe and limited subset of shading languages. With enough extensions to cover all of SPIR-V, you'd have a whole different language than standard WGSL anyways.


### Why not [Slang][slang]?

[Slang][slang] is not purpose-build for SPIR-V and e.g. only supports pointers in a limited way. It is also an amalgamation of GLSL, HLSL and custom syntax and currently not fully documented.









[^1]: The target during initial development will be Vulkan 1.4 + Roadmap 2024 + descriptor indexing, with the runtime optimized for dedicated desktop GPUs. Advanced features will always require device-generated-commands, a CPU roundtrip or a future cross-vendor work graphs extension.


[slang]: https://shader-slang.org/
[wgsl]: https://gpuweb.github.io/gpuweb/wgsl/
[vksdk]: https://vulkan.lunarg.com/sdk/home




