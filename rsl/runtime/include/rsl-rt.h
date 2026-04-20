#pragma once

/// @file 
/// Thread safety: None of the functions operating on the same context are thread-safe, but the runtime itself should not be much of a CPU bottleneck.


// If you want prototypes, import vulkan yourself before importing this file.
#define VK_NO_PROTOTYPES 1
#include <vulkan/vulkan.h>

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif



/// @brief The underlying opaque type for a loaded RSL program.
typedef struct rsl_program rsl_program;
/// @brief The underlying opaque type for a library context. Prefer to only have one global context.
typedef struct rsl_context rsl_context;

typedef uint32_t rsl_buffer;
typedef uint32_t rsl_flow_shader;

enum rsl_error {
    RSL_OK = 0,
    
    RSL_ERROR_MISC = -1,
};



typedef struct {
    
    /// The Vulkan logical device to use. Can be null, in which case the library will find an appropriate device and create the device itself.
    /// If it is not null, queues must also be specified.
    const VkDevice* device;
    
    /// Compute queues to be used by the runtime. has to be not null if and only if the device is not null.
    const VkQueue* compute_queues;
    size_t compute_queues_size;
    
    /// graphics queues to be used by the runtime. has to be not null if and only if the device is not null.
    const VkQueue* graphics_queues;
    size_t graphics_queues_size;
    
    /// transfer queues to be used by the runtime. has to be not null if and only if the device is not null.
    const VkQueue* transfer_queues;
    size_t transfer_queues_size;
    
    /// Maximum number of worker threads created by the runtime internally.
    uint8_t max_threads;
    
    
    
    
} rsl_config;




rsl_error rsl_context_init(rsl_context** context, const rsl_config* config);


rsl_error rsl_context_destroy(rsl_context* context);


rsl_error rsl_load_program(void* data, uint32_t data_length, rsl_program** program);



rsl_error rsl_get_flow_shader(rsl_program* program, char* name, rsl_flow_shader* flow_shader);

rsl_error rsl_call_flow_shader(rsl_program* program, rsl_flow_shader flow_shader, void* params);

rsl_error rsl_call_flow_shader_async(rsl_program* program, rsl_flow_shader flow_shader, void* params);

rsl_error rsl_execute();


void rsl_destroy_program(rsl_program* program);



rsl_error rsl_create_buffer(uint32_t size, rsl_buffer* buffer, void** data);

void rsl_destroy_buffer(rsl_buffer buffer);







#ifdef __cplusplus
}
#endif


