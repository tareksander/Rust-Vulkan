//! Functions to determine compatibility of a Vulkan PhysicalDevice with the library.
//! 
//! 
//! 
//! 


use std::ptr::{self, null, null_mut};

use ash::{prelude::VkResult, vk::{self, TaggedStructure}, Instance};


/// Checks whether a [`vk::PhysicalDevice`] is compatible with the library.
/// 
/// # Safety
/// 
/// The instance is required to be a Vulkan 1.3 instance or higher in the 1.x range.
/// The device has to be a valid VkPhysicalDevice handle for the instance.
/// 
pub unsafe fn is_compatible(instance: &Instance, device: vk::PhysicalDevice) -> VkResult<bool> {
    //let extensions = unsafe { instance.enumerate_device_extension_properties(device) }?;
    let mut features2: vk::PhysicalDeviceFeatures2 = Default::default();
    unsafe { instance.get_physical_device_features2(device, &mut features2) };
    let mut features11 = vk::PhysicalDeviceVulkan11Features::default();
    let mut features12 = vk::PhysicalDeviceVulkan12Features::default();
    let mut features13 = vk::PhysicalDeviceVulkan13Features::default();
    
    let mut found11 = false;
    let mut found12 = false;
    let mut found13 = false;
    let mut base = ptr::addr_of!(features2) as *const vk::BaseInStructure;
    
    while base != null() {
        let b = unsafe { base.read() };
        if b.s_type == vk::PhysicalDeviceVulkan11Features::STRUCTURE_TYPE {
            found11 = true;
            features11 = unsafe { (base as *const vk::PhysicalDeviceVulkan11Features).read() };
        }
        if b.s_type == vk::PhysicalDeviceVulkan12Features::STRUCTURE_TYPE {
            found12 = true;
            features12 = unsafe { (base as *const vk::PhysicalDeviceVulkan12Features).read() };
        }
        if b.s_type == vk::PhysicalDeviceVulkan13Features::STRUCTURE_TYPE {
            found13 = true;
            features13 = unsafe { (base as *const vk::PhysicalDeviceVulkan13Features).read() };
        }
        base = b.p_next;
    }
    
    if found11 && found12 && found13 == false {
        return Ok(false);
    }
    let mut props2 = vk::PhysicalDeviceProperties2::default();
    unsafe { instance.get_physical_device_properties2(device, &mut props2) };
    let props = props2.properties;
    let limits = props.limits;
    
    // Only check for features not required in Vulkan 1.3
    return Ok(features2.features.shader_int16 &
        features11.storage_buffer16_bit_access &
        features11.variable_pointers_storage_buffer &
        features11.variable_pointers &
        features12.storage_buffer8_bit_access &
        features12.shader_int8 &
        features12.scalar_block_layout //&
        // features2.features.shader_int64
        == 1 &&
        vk::api_version_variant(props.api_version) == 0 &&
        vk::api_version_major(props.api_version) == 1 &&
        vk::api_version_minor(props.api_version) >= 3
    );
}



pub const REQUIRED_EXTENSIONS: [&str; 0] = [];
pub const REQUIRED_FEATURES:
(vk::PhysicalDeviceFeatures, vk::PhysicalDeviceVulkan11Features<'_>, vk::PhysicalDeviceVulkan12Features<'_>, vk::PhysicalDeviceVulkan13Features<'_>)
= (vk::PhysicalDeviceFeatures {
    robust_buffer_access: vk::FALSE,
    full_draw_index_uint32: vk::FALSE,
    image_cube_array: vk::FALSE,
    independent_blend: vk::FALSE,
    geometry_shader: vk::FALSE,
    tessellation_shader: vk::FALSE,
    sample_rate_shading: vk::FALSE,
    dual_src_blend: vk::FALSE,
    logic_op: vk::FALSE,
    multi_draw_indirect: vk::FALSE,
    draw_indirect_first_instance: vk::FALSE,
    depth_clamp: vk::FALSE,
    depth_bias_clamp: vk::FALSE,
    fill_mode_non_solid: vk::FALSE,
    depth_bounds: vk::FALSE,
    wide_lines: vk::FALSE,
    large_points: vk::FALSE,
    alpha_to_one: vk::FALSE,
    multi_viewport: vk::FALSE,
    sampler_anisotropy: vk::FALSE,
    texture_compression_etc2: vk::FALSE,
    texture_compression_astc_ldr: vk::FALSE,
    texture_compression_bc: vk::FALSE,
    occlusion_query_precise: vk::FALSE,
    pipeline_statistics_query: vk::FALSE,
    vertex_pipeline_stores_and_atomics: vk::FALSE,
    fragment_stores_and_atomics: vk::FALSE,
    shader_tessellation_and_geometry_point_size: vk::FALSE,
    shader_image_gather_extended: vk::FALSE,
    shader_storage_image_extended_formats: vk::FALSE,
    shader_storage_image_multisample: vk::FALSE,
    shader_storage_image_read_without_format: vk::FALSE,
    shader_storage_image_write_without_format: vk::FALSE,
    shader_uniform_buffer_array_dynamic_indexing: vk::FALSE,
    shader_sampled_image_array_dynamic_indexing: vk::FALSE,
    shader_storage_buffer_array_dynamic_indexing: vk::FALSE,
    shader_storage_image_array_dynamic_indexing: vk::FALSE,
    shader_clip_distance: vk::FALSE,
    shader_cull_distance: vk::FALSE,
    shader_float64: vk::FALSE,
    //shader_int64: vk::TRUE,
    shader_int64: vk::FALSE,
    shader_int16: vk::TRUE,
    shader_resource_residency: vk::FALSE,
    shader_resource_min_lod: vk::FALSE,
    sparse_binding: vk::FALSE,
    sparse_residency_buffer: vk::FALSE,
    sparse_residency_image2_d: vk::FALSE,
    sparse_residency_image3_d: vk::FALSE,
    sparse_residency2_samples: vk::FALSE,
    sparse_residency4_samples: vk::FALSE,
    sparse_residency8_samples: vk::FALSE,
    sparse_residency16_samples: vk::FALSE,
    sparse_residency_aliased: vk::FALSE,
    variable_multisample_rate: vk::FALSE,
    inherited_queries: vk::FALSE,
}, vk::PhysicalDeviceVulkan11Features {
    s_type: vk::PhysicalDeviceVulkan11Features::STRUCTURE_TYPE,
    p_next: null_mut(),
    storage_buffer16_bit_access: vk::TRUE,
    uniform_and_storage_buffer16_bit_access: vk::FALSE,
    storage_push_constant16: vk::FALSE,
    storage_input_output16: vk::FALSE,
    multiview: vk::FALSE,
    multiview_geometry_shader: vk::FALSE,
    multiview_tessellation_shader: vk::FALSE,
    variable_pointers_storage_buffer: vk::TRUE,
    variable_pointers: vk::TRUE,
    protected_memory: vk::FALSE,
    sampler_ycbcr_conversion: vk::FALSE,
    shader_draw_parameters: vk::FALSE,
    _marker: std::marker::PhantomData,
}, vk::PhysicalDeviceVulkan12Features {
    s_type: vk::PhysicalDeviceVulkan12Features::STRUCTURE_TYPE,
    p_next: null_mut(),
    sampler_mirror_clamp_to_edge: vk::FALSE,
    draw_indirect_count: vk::FALSE,
    storage_buffer8_bit_access: vk::TRUE,
    uniform_and_storage_buffer8_bit_access: vk::FALSE,
    storage_push_constant8: vk::FALSE,
    shader_buffer_int64_atomics: vk::FALSE,
    shader_shared_int64_atomics: vk::FALSE,
    shader_float16: vk::FALSE,
    shader_int8: vk::TRUE,
    descriptor_indexing: vk::FALSE,
    shader_input_attachment_array_dynamic_indexing: vk::FALSE,
    shader_uniform_texel_buffer_array_dynamic_indexing: vk::FALSE,
    shader_storage_texel_buffer_array_dynamic_indexing: vk::FALSE,
    shader_uniform_buffer_array_non_uniform_indexing: vk::FALSE,
    shader_sampled_image_array_non_uniform_indexing: vk::FALSE,
    shader_storage_buffer_array_non_uniform_indexing: vk::FALSE,
    shader_storage_image_array_non_uniform_indexing: vk::FALSE,
    shader_input_attachment_array_non_uniform_indexing: vk::FALSE,
    shader_uniform_texel_buffer_array_non_uniform_indexing: vk::FALSE,
    shader_storage_texel_buffer_array_non_uniform_indexing: vk::FALSE,
    descriptor_binding_uniform_buffer_update_after_bind: vk::FALSE,
    descriptor_binding_sampled_image_update_after_bind: vk::FALSE,
    descriptor_binding_storage_image_update_after_bind: vk::FALSE,
    descriptor_binding_storage_buffer_update_after_bind: vk::FALSE,
    descriptor_binding_uniform_texel_buffer_update_after_bind: vk::FALSE,
    descriptor_binding_storage_texel_buffer_update_after_bind: vk::FALSE,
    descriptor_binding_update_unused_while_pending: vk::FALSE,
    descriptor_binding_partially_bound: vk::FALSE,
    descriptor_binding_variable_descriptor_count: vk::FALSE,
    runtime_descriptor_array: vk::FALSE,
    sampler_filter_minmax: vk::FALSE,
    scalar_block_layout: vk::TRUE,
    imageless_framebuffer: vk::FALSE,
    uniform_buffer_standard_layout: vk::FALSE,
    shader_subgroup_extended_types: vk::FALSE,
    separate_depth_stencil_layouts: vk::FALSE,
    host_query_reset: vk::TRUE,
    timeline_semaphore: vk::TRUE,
    buffer_device_address: vk::TRUE,
    buffer_device_address_capture_replay: vk::FALSE,
    buffer_device_address_multi_device: vk::FALSE,
    vulkan_memory_model: vk::TRUE,
    vulkan_memory_model_device_scope: vk::FALSE,
    vulkan_memory_model_availability_visibility_chains: vk::FALSE,
    shader_output_viewport_index: vk::FALSE,
    shader_output_layer: vk::FALSE,
    subgroup_broadcast_dynamic_id: vk::TRUE,
    _marker: std::marker::PhantomData,
}, vk::PhysicalDeviceVulkan13Features {
    s_type: vk::PhysicalDeviceVulkan13Features::STRUCTURE_TYPE,
    p_next: null_mut(),
    robust_image_access: vk::FALSE,
    inline_uniform_block: vk::FALSE,
    descriptor_binding_inline_uniform_block_update_after_bind: vk::FALSE,
    pipeline_creation_cache_control: vk::FALSE,
    private_data: vk::FALSE,
    shader_demote_to_helper_invocation: vk::TRUE,
    shader_terminate_invocation: vk::TRUE,
    subgroup_size_control: vk::TRUE,
    compute_full_subgroups: vk::TRUE,
    synchronization2: vk::TRUE,
    texture_compression_astc_hdr: vk::FALSE,
    shader_zero_initialize_workgroup_memory: vk::FALSE,
    dynamic_rendering: vk::FALSE,
    shader_integer_dot_product: vk::FALSE,
    maintenance4: vk::TRUE,
    _marker: std::marker::PhantomData,
});








