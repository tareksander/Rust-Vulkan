# Warpfield

🚧 This project is in early development and not fit for general usage yet. If you do want to try it out, expect bugs and breakage in future versions 🚧

A work-in-progress compute and shading language for modern Vulkan (last ~5 years of AMD and Nvidia desktop cards with updated drivers).

### Trying it out

Prerequisites:

- A correctly configured and up-to-date Vulkan SDK
- The Rust toolchain (also up-to-date stable branch)
- A C/C++ toolchain
- CMake (at least 3.31.6)
- Python 3.x (whatever version the Vulkan SDK profile library scripts need, tested with 3.13)
- Git

#### Step 1

Clone this repo, since the crates aren't worth to put on crates.io yet.

#### Step 2

Edit the test source code in `rsl/rsl-spirv/lib.rs` to your liking.

#### Step 3

Run the test (and pray the compiler works).

#### Step 4

Run `spirv-val rsl/rsl-spirv/test.spv` and see if it complains. If yes, tough luck, file a bug if you want.


#### Step 5

Adjust the runtime test (`rsl/runtime/src/main-test.cpp`) according to your compute shader's parameters.

Notably:
- Adjust the buffer and memory size allocation, which currently assumes 3 buffers of at most 4 byte wide data.
- Write your data at the correct offsets in the mapped memory
- If you change the output, adjust the readback and print code accordingly
- If you add more buffers put their addresses in the `mpbuffer` buffer
- If you add scalar parameters, those go in the `mpbuffer` as well


#### Step 6

Run the `rt-test` CMake target of the runtime and hopefully watch the output of your correctly working compute shader.

#### Step 7

Give feedback! It's appreciated, since one person can only try out so much.

