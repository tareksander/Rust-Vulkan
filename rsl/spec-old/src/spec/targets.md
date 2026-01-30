# Backends


This chapter defines the interaction between the language and the compilation target for the backends.


## SPIR-V

This is the primary target with support for all features. This will probably be the only backend to support graphics entrypoints. If you need graphics entrypoints on the host, use a Vulkan implementation like LLVMPipe or Swiftshader.




## WGSL

This target will cross-compile to WGSL, but only supports features also possible in WGSL.



## Clang C

This target will cross-compile to Clang C code for executing the same algorithms on the CPU. It has lower priority than the other targets.



## Cuda C

There may also eventually be a Cuda C backend to support compute entrypoints via Cuda and RocM, though hopefully by that point Vulkan will have evolved enough that this backend won't be necessary.


## WebAssembly

This target will compile to WebAssembly, though it's likely to be realized via the C backend for the foreseeable future.


