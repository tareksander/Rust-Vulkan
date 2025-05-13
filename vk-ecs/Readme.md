## VK-ECS

An ECS implementation on the GPU via Vulkan, working with compute shaders as systems.







### Compatibility

This library aims to be compatible with mainstream hardware in the last 5 years with up-to-date drivers if possible. Minimum requirements may raise if needed. Currently Vulkan 1.3 and some capabilities are required, which are supported in the Vulkan desktop baseline profile and on Android 15+. MoltenVK is close to Vulkan 1.3 and also supports the capabilities.










#### Why not WebGPU?

To efficiently implement an ECS, you need fine-grained synchronization, arbitrary read/write memory and pointers, all of which are currently not present in WebGPU, and probably never will be.

#### Why not Metal/D3D12?

Vulkan is the only widely-available low-level graphics API over multiple platforms, with native support on Android, Linux, Windows (with Intel, AMD or Nvidia GPUs/APUs) and the Switch. The Mesa Dozen driver enables Vulkan-over-D3D12, and there are semi-functional ports of it to UWP platforms (like the Xbox). For Apple hardware there's MoltenVK, which translates Vulkan to Metal calls.




#### License

MPL-2.0


