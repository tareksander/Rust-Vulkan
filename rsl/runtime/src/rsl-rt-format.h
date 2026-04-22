#pragma once


#include <stdint.h>




const char RSL_MAGIC[4] = {'W', 'R', 'P', '\0'};

const uint32_t rsl_abi_version = 0;


/** @file
* File layout:
* - Header (4 byte aligned)
* - SPIR-V blob
* - compute entrypoints
* - graphics pipelines
* - indirect execution sets
* - flow shaders
* - names section (containing null-terminated strings)
*/


/// Offset into the names section.
typedef uint32_t rsl_blob_name;


typedef struct {
    /// Used to detect file type and endianness.
    char magic[4];
    
    /// File layout version. The runtime can only load a fully matching version.
    uint32_t abi_revision;
    
    /// The minimum language version, with 16 bits per patch, minor and major version, starting from the LSB.
    /// Beta versions have the MSB set and require and exact version match. Additionally, beta and release versions aren't compatible.
    uint64_t min_runtime;
    
    /// Length of the spir-v block that comes directly after the header (which means the header should be 4 byte aligned).
    uint32_t spirv_length;
    
    
    
    /// Number of compute shader entrypoints after the spir-v blob.
    uint32_t compute_shaders;
    
    /// Number of graphics pipelines after the compute shader entrypoints.
    uint32_t gfx_pipelines;
    
    /// Number of indirect execution set lists after the graphics pipelines.
    uint32_t ies_shaders;
    
    /// Number of flow shader metadata lists after the indirect execution sets.
    uint32_t flow_shaders;
    
    
    /// Length of the names section in bytes.
    uint32_t names_length;
    
    
    
    
    
    
} rsl_binary_header;





typedef struct {
    /// Name of the entrypoint in spir-v.
    rsl_blob_name name;
} rsl_compute_entrypoint;



typedef struct {
    /// Type of pipeline.
    rsl_pipeline_type ty;
    union {
        /// Name of the fragment shader in spir-v.
        rsl_blob_name fragment;
    };
    union {
        /// Name of the vertex shader in spir-v.
        rsl_blob_name vertex;
        struct {
            /// Name of the mesh shader in spir-v.
            rsl_blob_name mesh;
            /// Name of the task shader in spir-v.
            rsl_blob_name task;
        };
        
    };
} rsl_gfx_pipeline;


typedef enum {
    RSL_PIPELINE_VERTEX = 0,
    RSL_PIPELINE_MESH = 1,
    /// Currently invalid
    RSL_PIPELINE_RT = 2,
    
    
    /// Max future value of pipeline types, invalid value itself
    RSL_PIPELINE_MAX = 255,
} rsl_pipeline_type;




typedef struct {
    rsl_ies_type ty;
    /// Number of following names.
    uint32_t count;
} rsl_ies;



typedef enum {
    /// IES of compute shaders
    RSL_IES_COMPUTE = 0,
    
    
    /// Max future value of pipeline types, invalid value itself
    RSL_PIPELINE_MAX = 255,
} rsl_ies_type;


typedef struct {
    /// Name of the flow shader when invoking it in the runtime, as well as the prefix of compute shader part names specific to this flow shader.
    rsl_blob_name name;
    /// Number of following execution blocks.
    uint32_t execution_block_count;
} rsl_flow_shader;


typedef struct {
    rsl_execution_type ty;
    uint32_t count;
} rsl_execution_block;

typedef enum {
    /// Defines a list of dispatches running in parallel.
    RSL_EXECUTION_COMPUTE = 0,
    /// Defines a render pass with draw calls.
    RSL_EXECUTION_RENDER = 1,
    /// Defines a single-threaded dispatch name.
    RSL_EXECUTION_FLOW_PART = 2,
    
    /// Max future value of execution types, invalid value itself
    RSL_EXECUTION_MAX = 255,
} rsl_execution_type;


typedef struct {
    rsl_blob_name name;
} rsl_execution_compute;




typedef struct {
    
} rsl_execution_render;



typedef struct {
    /// The name of the compute entrypoint is the flow shader name, and underscore and then the index as a decimal number.
    uint32_t index;
} rsl_execution_flow_part;








