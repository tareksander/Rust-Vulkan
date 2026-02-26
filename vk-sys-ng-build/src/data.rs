//! Data structures for a parsed Vulkan XML file.

use std::path::PathBuf;


pub struct VulkanRegistry {
    pub platforms: Vec<VKPlatform>,
    // TODO tags
    pub types: Vec<VKType>,
    
    
}





pub struct VKPlatform {
    /// Platform name identifier
    pub name: String,
    /// C macro define name guarding the definitions
    pub protect: String,
}


pub enum VKType {
    /// The C header include path
    Include(PathBuf),
}



































