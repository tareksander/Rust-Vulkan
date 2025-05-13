# Capabilities

Many language features depend on optional SPIR-V extensions or aren't available in WGSL. These features are gated behind capabilities, which are specified at compile time, either individually or via profiles.



### Pointers

This capability implies storage buffer pointers.

The SPIR-V capabilities `VariablePointers` and `VariablePointersStorageBuffer` are required. 

Without this capability, pointers are only allowed in function signatures and as function parameters.


Capability name: `pointers`


### 8/16 Bit Storage

6 Capabilities allowing 8/16 bit types in storage buffers, push constants and uniform buffers respectively.

Capability name: `storage8`, `storage16`, `uniform8`, `uniform16`, `push8`, `push16`


### 8/16 Bit Native

3 Capabilities denoting whether native support for 8 and 16 bit types is possible. If not, these types can still be used and the compiler will widen the types to the next bigger supported type. For integers, this means that for each assignment (and when explicitly cast), the upper bits are erased. Floating point types just get added precision, which should not negatively impact logic.

Capability name: `i8`, `i16`, `f16`

# 64 Bit Types

2 capabilities denoting support for 64 bit integers and floats.

Capability name: `i64`, `f64`


### Subgroup Reconvergence

Corresponds to `SPV_KHR_subgroup_uniform_control_flow`. Allows divergent invocations in a subgroup to reconverge, as long as the block was entered subgroup uniformly.

Capability name: `subgroup_reconvergence`

### Maximal Reconvergence

Corresponds to `SPV_KHR_maximal_reconvergence`. Reconverges invocations as a developer might expect, taking every opportunity to reconverge.

Capability name: `maximal_reconvergence`

### Desktop Baseline

Corresponds to the capability set of the Vulkan desktop baseline profiles.

Capability names: `dbXXXX`

### Roadmap Profile

Corresponds to the capability set of the Vulkan roadmap profiles.

Capability names: `rXXXX`

### Android Baseline

Corresponds to the capability set of the Vulkan Android baseline profiles.

Capability names: `abXXXX`

### Vulkan Version

Corresponds to the capability set of the Vulkan version's required features.

Capability names: `vX.X`

### Android Version

Corresponds to the capability set of the Vulkan Android profiles.

Capability names: `aXX`




### TODO

VK_KHR_compute_shader_derivatives

VK_KHR_cooperative_matrix

VK_KHR_shader_clock

VK_KHR_shader_bfloat16

VK_KHR_shader_quad_control

VK_KHR_workgroup_memory_explicit_layout

VK_EXT_mesh_shader

VK_EXT_shader_atomic_float

VK_EXT_shader_atomic_float2

VK_KHR_shader_atomic_int64

Promoted:

Check which ones are supported unconditionally in ABP22 and Desktop baseline 22 and subsequently don`t need an extra capability (except for WGSL support).

VK_KHR_relaxed_block_layout

VK_KHR_shader_draw_parameters

VK_KHR_shader_expect_assume

VK_KHR_shader_float_controls

VK_KHR_shader_float_controls2

VK_KHR_shader_integer_dot_product

VK_KHR_shader_subgroup_extended_types

VK_KHR_shader_subgroup_rotate

VK_KHR_uniform_buffer_standard_layout

VK_EXT_scalar_block_layout

VK_EXT_shader_subgroup_ballot

VK_EXT_shader_subgroup_vote

VK_EXT_subgroup_size_control
