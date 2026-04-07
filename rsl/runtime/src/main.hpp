#pragma once


#define VULKAN_HPP_CPP_VERSION 23
#define VULKAN_HPP_NO_CONSTRUCTORS 1
#define VK_NO_PROTOTYPES 1
#define VP_USE_OBJECT 1
#include <vulkan/vulkan.hpp>
#include <volk/volk.h>
#include <vulkan/vulkan_raii.hpp>
#include "vulkan_profiles.hpp"



namespace rsl {
    
    class Context {
        public:
        
        static Context init();
        
        
        
        
        
        Context(vk::raii::Context&& c, vk::raii::Instance&& i, vk::raii::Device&& d, vk::raii::Queue&& q) : raiiContext{std::move(c)}, instance{std::move(i)}, device{std::move(d)}, mainQueue{std::move(q)} {};
        vk::raii::Context raiiContext;
        vk::raii::Instance instance;
        vk::raii::Device device;
        vk::raii::Queue mainQueue;
        
        uint32_t mem_device_buffer;
        uint32_t mem_host_buffer;
        uint32_t mem_device_visible_buffer;
        
        uint32_t main_qf;
        
        
        
        
        
    };
    
    
    
}


