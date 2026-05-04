
#include "main.hpp"

#include <iostream>
#include <vector>
#include <fstream>
#include <filesystem>




int main() {
    
    const int size = 128;
    auto c = rsl::Context::init();
    auto buffer = c.device.createBuffer(vk::BufferCreateInfo {
        .sharingMode = vk::SharingMode::eConcurrent,
        .usage = vk::BufferUsageFlagBits::eShaderDeviceAddress | vk::BufferUsageFlagBits::eTransferDst,
        .size = size * 4 * 3 + 3 * 8 + 4,
    });
    auto mem = c.device.allocateMemory(vk::MemoryAllocateInfo {
        .memoryTypeIndex = c.mem_device_visible_buffer,
        .allocationSize = size * 4 * 3 + 3 * 8 + 4,
    });
    buffer.bindMemory(mem, 0);
    auto baddress = c.device.getBufferAddress(vk::BufferDeviceAddressInfo {
        .buffer = buffer
    });
    
    
    auto mapped_memory = (float*) c.device.mapMemory2(vk::MemoryMapInfo {
        .memory = mem,
        .offset = 0,
        .size = VK_WHOLE_SIZE,
    });
    
    for (int i = 0; i < size; i++) {
        mapped_memory[i*2] = (float)i - 0.5;
        mapped_memory[i*2 + 1] = i % 2 == 0 ? 0.2 : -0.2;
    }
    
    auto mpbuffer = (uint64_t*) (((char*) mapped_memory) + size * 4 * 3);
    auto mpbufferf = (float*) (((char*) mapped_memory) + size * 4 * 3 + 3 * 8);
    *mpbufferf = -1000.0;
    
    mpbuffer[0] = baddress;
    
    //mpbuffer[1] = baddress + size * 4;
    
    mpbuffer[1] = baddress + size * 4 * 2;
    
    
    std::vector<uint32_t> shader_code;
    std::ifstream shader_file {"../rsl/rsl-spirv/test.spv", std::ios::binary | std::ios::ate};
    std::cout << std::filesystem::current_path() << std::endl;
    if (shader_file.bad()) {
        throw std::runtime_error("Unable to load shader file");
    }
    shader_code.resize(shader_file.tellg()/4);
    shader_file.seekg(0, std::ios::beg);
    shader_file.read((char*) shader_code.data(), shader_code.size()*4);
    if (shader_file.bad()) {
        throw std::runtime_error("Unable to load shader file");
    }
    std::ofstream("test.spv", std::ios::binary).write((char*)shader_code.data(), shader_code.size() * 4);
    auto prange = vk::PushConstantRange {
        .offset = 0,
        .size = 8,
        .stageFlags = vk::ShaderStageFlagBits::eCompute,
    };
    auto l = c.device.createPipelineLayout(vk::PipelineLayoutCreateInfo {
        .pushConstantRangeCount = 1,
        .pPushConstantRanges = &prange,
    });
    
    auto mod = vk::ShaderModuleCreateInfo {
        .codeSize = shader_code.size()*4,
        .pCode = shader_code.data(),
    };
    
    auto shader = c.device.createComputePipeline(nullptr, vk::ComputePipelineCreateInfo {
        .layout = l,
        .stage = vk::PipelineShaderStageCreateInfo {
            .pName = "test",
            .stage = vk::ShaderStageFlagBits::eCompute,
            .pNext = &mod,
        }
    });
    
    auto cpool = c.device.createCommandPool(vk::CommandPoolCreateInfo {
        .queueFamilyIndex = c.main_qf,
    });
    
    auto cb = std::move(c.device.allocateCommandBuffers(vk::CommandBufferAllocateInfo {
        .commandBufferCount = 1,
        .commandPool = cpool,
        .level = vk::CommandBufferLevel::ePrimary,
    })[0]);
    
    cb.begin(vk::CommandBufferBeginInfo {});
    
    
    cb.bindPipeline(vk::PipelineBindPoint::eCompute, shader);
    uint64_t push_data[1];
    push_data[0] = baddress + size * 4 * 3;
    
    cb.pushConstants<uint64_t>(l, vk::ShaderStageFlagBits::eCompute, 0, vk::ArrayProxy(1, push_data));
    
    cb.dispatch(size / 32, 1, 1);
    
    cb.end();
    c.mainQueue.submit(vk::SubmitInfo {
        .commandBufferCount = 1,
        .pCommandBuffers = &*cb,
    });
    
    c.device.waitIdle();
    
    std::cout << "context created" << std::endl;
    //std::cout.precision(2);
    for (int i = 0; i < size; i++) {
        std:: cout << mapped_memory[size * 2 + i] << std::endl;
    }
    
    
}


