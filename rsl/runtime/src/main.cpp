#include "main.hpp"


#include <exception>
#include <iostream>
#include <vector>
#include <algorithm>

#define STUB(name) vk ## name = (PFN_vk ## name) throwStub;


namespace rsl {
    
    
    static void throwStub(void) {
        throw std::runtime_error("Tried to call unloaded Vulkan function");
    }
    
    
    
    
    Context Context::init() {
        using namespace std;
        if (volkInitialize() != VK_SUCCESS) {
            throw std::runtime_error("Could not load Vulkan");
        }
        // stub out instance functions so the vulkan profiles library is happy
        STUB(GetDeviceProcAddr);
        STUB(EnumerateDeviceExtensionProperties);
        STUB(GetPhysicalDeviceFeatures2);
        STUB(GetPhysicalDeviceProperties2);
        STUB(GetPhysicalDeviceFormatProperties2);
        STUB(GetPhysicalDeviceQueueFamilyProperties2);
        STUB(CreateDevice);
        VkResult res;
        

        VkApplicationInfo ainfo {
            .sType = VK_STRUCTURE_TYPE_APPLICATION_INFO,
            .pNext = nullptr,
            .pApplicationName = nullptr,
            .applicationVersion = 0,
            .pEngineName = nullptr,
            .engineVersion = 0,
            .apiVersion = VP_RSL_MIN_MIN_API_VERSION,
        };
        VpCapabilities caps;
        VpCapabilitiesCreateInfo icaps {
            .flags = VpCapabilitiesCreateFlagBits::VP_PROFILE_CREATE_STATIC_BIT,
            .apiVersion = VK_API_VERSION_1_4,
            .pVulkanFunctions = nullptr,
        };
        if ((res = vpCreateCapabilities(&icaps, nullptr, &caps)) != VK_SUCCESS) {
            vk::detail::throwResultException(vk::Result(res), "Could not create capabilities");
        }
        VpProfileProperties pprops {
            .profileName = VP_RSL_MIN_NAME,
            .specVersion = VP_RSL_MIN_SPEC_VERSION,
        };
        
        
        VkBool32 profileSupported;
        if ((res = vpGetInstanceProfileSupport(caps, nullptr, &pprops, &profileSupported)) != VK_SUCCESS) {
            vk::detail::throwResultException(vk::Result(res), "Could not check profile");
        }
        
        if (profileSupported != VK_TRUE) {
            throw runtime_error("Minimum profile not supported");
        }
        
        
        
        VkInstanceCreateInfo iinfo {
            .pApplicationInfo = &ainfo,
        };
        
        
        VpInstanceCreateInfo vpiinfo {
            .pCreateInfo = &iinfo,
            .enabledFullProfileCount = 1,
            .pEnabledFullProfiles = &pprops,
        };
        
        VkInstance rinst;
        if ((res = vpCreateInstance(caps, &vpiinfo, nullptr, &rinst))) {
            vk::detail::throwResultException(vk::Result(res), "Could not create instance");
        }
        
        volkLoadInstance(rinst);
        //asm volatile(""::: "memory");
        vk::raii::Context c;
        vk::raii::Instance inst(c, rinst);
        
        // Reset caps so the newly loaded function pointers get used.
        caps = {};
        
        if ((res = vpCreateCapabilities(&icaps, nullptr, &caps)) != VK_SUCCESS) {
            vk::detail::throwResultException(vk::Result(res), "Could not create capabilities");
        }
        
        
        
        vector<vk::raii::PhysicalDevice> validDevices;
        
        for (auto d : inst.enumeratePhysicalDevices()) {
            VkBool32 supported;
            if (vpGetPhysicalDeviceProfileSupport(caps, (vk::Instance) inst, (vk::PhysicalDevice) d, &pprops, &supported) != VK_SUCCESS) {
                vk::detail::throwResultException(vk::Result(res), "Could not check profile");
            }
            if (supported == VK_TRUE) {
                validDevices.push_back(d);
                cout << "found device" << endl;
            }
        }
        
        if (validDevices.empty()) {
            throw runtime_error("No devices matching profile found");
        }
        
        auto pd = validDevices[0];
        
        
        auto qprops = pd.getQueueFamilyProperties();
        
        int mainQueue = -1;
        
        int index = 0;
        for (auto& p : qprops) {
            if (p.queueFlags & vk::QueueFlagBits::eCompute && p.queueFlags & vk::QueueFlagBits::eGraphics && p.queueFlags & vk::QueueFlagBits::eTransfer) {
                mainQueue = index;
                break;
            }
            index++;
        }
        
        if (mainQueue == -1) {
            throw runtime_error("No main queue found");
        }
        
        
        vk::DeviceQueueCreateInfo qinfo {
            .queueCount = 1,
            .queueFamilyIndex = (uint32_t) mainQueue,
        };
        
        vk::DeviceCreateInfo vdinfo {
            .queueCreateInfoCount = 1,
            .pQueueCreateInfos = &qinfo,
        };
        
        auto dinfo = VpDeviceCreateInfo {
            .enabledFullProfileCount = 1,
            .pEnabledFullProfiles = &pprops,
            .pCreateInfo = vdinfo,
        };
        
        VkDevice d;
        if (vpCreateDevice(caps, vk::PhysicalDevice(pd), &dinfo, nullptr, &d) != VK_SUCCESS) {
            throw runtime_error("Could not create device");
        }
        volkLoadDevice(d);
        vk::raii::Device rd = vk::raii::Device(pd, vk::Device(d));
        
        auto q = rd.getQueue(mainQueue, 0);
        
        auto mprops = pd.getMemoryProperties();
        
        int main_device_heap = -1;
        
        for (int i = 0; i < mprops.memoryHeapCount; i++) {
            auto& h = mprops.memoryHeaps[i];
            if (h.flags & vk::MemoryHeapFlagBits::eDeviceLocal) {
                if (main_device_heap == -1 || h.size > mprops.memoryHeaps[main_device_heap].size) {
                    main_device_heap = i;
                }
            }
        }
        
        if (main_device_heap == -1) {
            throw runtime_error("Could not find device local heap");
        }
        
        int mem_device_buffer = -1;
        int mem_host_buffer = -1;
        int mem_device_visible_buffer = -1;
        
        int mem_image_color = -1;
        int mem_image_depth = -1;
        // TODO: image memory types
        
        VkMemoryRequirements2 ireqs_color = {
            .sType = VK_STRUCTURE_TYPE_MEMORY_REQUIREMENTS_2,
            .pNext = nullptr,
        };
        VkMemoryRequirements2 ireqs_depth = {
            .sType = VK_STRUCTURE_TYPE_MEMORY_REQUIREMENTS_2,
            .pNext = nullptr,
        };
        
        VkMemoryRequirements2 breqs_device = {
            .sType = VK_STRUCTURE_TYPE_MEMORY_REQUIREMENTS_2,
            .pNext = nullptr,
        };
        VkMemoryRequirements2 breqs_host = {
            .sType = VK_STRUCTURE_TYPE_MEMORY_REQUIREMENTS_2,
            .pNext = nullptr,
        };
        // buffer config for the main shader memory
        VkBufferCreateInfo binfo_device = {
            .sType = VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO,
            .sharingMode = VK_SHARING_MODE_EXCLUSIVE,
            .pQueueFamilyIndices = nullptr,
            .queueFamilyIndexCount = 0,
            .size = 1024,
            .usage = 
                VK_BUFFER_USAGE_SHADER_DEVICE_ADDRESS_BIT |
                VK_BUFFER_USAGE_STORAGE_BUFFER_BIT |
                VK_BUFFER_USAGE_INDEX_BUFFER_BIT |
                VK_BUFFER_USAGE_INDIRECT_BUFFER_BIT |
                VK_BUFFER_USAGE_TRANSFER_DST_BIT |
                VK_BUFFER_USAGE_TRANSFER_SRC_BIT,
            .flags = 0,
            .pNext = nullptr,
        };
        // buffer config for host buffers only used for transferring data
        VkBufferCreateInfo binfo_host_only = {
            .sType = VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO,
            .sharingMode = VK_SHARING_MODE_EXCLUSIVE,
            .pQueueFamilyIndices = nullptr,
            .queueFamilyIndexCount = 0,
            .size = 1024,
            .usage = 
                VK_BUFFER_USAGE_TRANSFER_DST_BIT |
                VK_BUFFER_USAGE_TRANSFER_SRC_BIT,
            .flags = 0,
            .pNext = nullptr,
        };
        
        VkDeviceBufferMemoryRequirements bminfo = {
            .sType = VK_STRUCTURE_TYPE_DEVICE_BUFFER_MEMORY_REQUIREMENTS,
            .pCreateInfo = &binfo_device,
            .pNext = nullptr,
        };
        
        vkGetDeviceBufferMemoryRequirements(*rd, &bminfo, &breqs_device);
        bminfo.pCreateInfo = &binfo_host_only;
        vkGetDeviceBufferMemoryRequirements(*rd, &bminfo, &breqs_host);
        
        
        for (int i = 0; i < mprops.memoryTypeCount; i++) {
            auto& t = mprops.memoryTypes[i];
            auto flags = t.propertyFlags;
            bool allows_main_memory = (breqs_device.memoryRequirements.memoryTypeBits >> i) & 1;
            bool allows_host_memory = (breqs_host.memoryRequirements.memoryTypeBits >> i) & 1;
            
            
            // select first fitting host memory type
            if (mem_host_buffer == -1 && allows_host_memory &&
                flags & vk::MemoryPropertyFlagBits::eHostCoherent &&
                flags & vk::MemoryPropertyFlagBits::eHostCached &&
                ! (flags & vk::MemoryPropertyFlagBits::eDeviceLocal)) {
                mem_host_buffer = i;
            }
            
            if (mem_device_buffer == -1 && allows_main_memory &&
                flags & vk::MemoryPropertyFlagBits::eDeviceLocal &&
                t.heapIndex == main_device_heap) {
                mem_device_buffer = i;
            }
            
            if (mem_device_visible_buffer == -1 && allows_main_memory &&
                flags & vk::MemoryPropertyFlagBits::eDeviceLocal &&
                flags & vk::MemoryPropertyFlagBits::eHostCoherent) {
                mem_device_visible_buffer = i;
            }
        }
        
        
        bool has_rebar = false;
        
        if (mem_device_buffer == -1 || mem_host_buffer == -1) {
            throw runtime_error("unable to find memory types");
        }
        
        
        // rebar means the whole device local memory is accessible
        if (mem_device_visible_buffer != -1 &&
            mprops.memoryTypes[mem_device_visible_buffer].heapIndex == mprops.memoryTypes[mem_device_buffer].heapIndex) {
            has_rebar = true;
        }
        
        
        cout << "main memory: " << mem_device_buffer << endl;
        cout << "host memory: " << mem_host_buffer << endl;
        cout << "visible memory: " << mem_device_visible_buffer << endl;
        cout << "rebar: " << has_rebar << endl;
        
        
        
        Context rc = Context(std::move(c), std::move(inst), std::move(rd), std::move(q));
        rc.mem_device_buffer = mem_device_buffer;
        rc.mem_host_buffer = mem_host_buffer;
        rc.mem_device_visible_buffer = mem_device_visible_buffer;
        rc.main_qf = mainQueue;
        return rc;
    }
    
}







