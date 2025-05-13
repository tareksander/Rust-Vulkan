# Shader Interface

This document defines the ABI between the Shaders on the GPU and the CPU. Scalar block layout is assumed.

Shaders get a uniform buffer in set 0 binding 0 with the following content:
- 16 bytes reserved and with undefined contents.
- An array of Query structs, with the length dictated by the shader
- An array of Buffer structs with the specified length of all Query structs combined

The Query struct consists of the following:
- u32: The total number of entities in this query
- u16: the number of Buffer structs
- 2 bytes reserved and with undefined contents

The Buffer struct consists of the following:
- u32: Number of entities in the buffer
- u16: Size of an element in the archetype data area
- 2 bytes reserved
- u64: The physical storage buffer pointer to the archetype storage area
- A list of u16 component offsets, with the number of components dictated by the shader. Optional components that are not present have all bits set.

A shader invocation then uses its global invocation id in the x direction to determine which entities from which queries to operate on.
When the ID overflows the entities for Query 1, add an index for Query 2, etc:

````
q2_entity = id / q1_entities
q3_entity = q2_entity / q2_entities
...
q1_entity = id % q1_entities
q2_entity = q2_entity % q2_entities
...
````


3, 2 ,2

9

q2 = 9 / 3 = 3
q3 = 3 / 2 = 1

q1 = 9 % 3 = 0
q2 = 3 % 2 = 1




Example:

Component A and B with a u32 each, 3 Entities with A and 2 with B:

Querying for A and B to get all combinations of A and B:

````Rust
struct UB {
    _unused: [u8; 16],
    // Query 1
    q1_entities: u32, // = 3
    q1_buffers: u16, // = 1
    _unused2: u16,
    q1_b1_entities: u32, // = 3
    q1_b1_size: u16, // = 4
    _reserved3: u16,
    q1_b1_p: u64,
    q1_b1_c1: u16, // = 0
    
    q2_entities: u32, // = 2
    q2_buffers: u16, // = 1
    _unused2: u16,
    q2_b1_entities: u32, // = 2
    q2_b1_size: u16, // = 4
    _reserved3: u16,
    q2_b1_p: u64,
    q2_b1_c1: u16, // = 0
}
````













