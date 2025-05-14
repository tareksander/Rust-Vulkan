use std::path::PathBuf;

use bitflags::bitflags;



/// The complete compiler input.
pub struct CompilerInput {
    /// The package root directories.
    pub package_roots: Vec<PathBuf>,
    /// The compiler options.
    pub options: CompilerOptions,
    
    
    
    
    
}


/// The compiler options.
pub struct CompilerOptions {
    // Sets the amount of generated debug information.
    pub debug_info_level: DebugInfoLevel,
    /// Enables debug mode. See the specification for an explanation.
    pub debug_mode: bool,
    /// Controls the logging verbosity of the compiler. Should be set to 0 when not debugging the compiler.
    pub log_level: u8,
    /// An optional file to also write the log to.
    pub log_tee: Option<PathBuf>,
    
    pub capabilities: Capabilities,
    
    
    
    
    
    
    
    
}


// Controls the amount of generated debug information.
pub enum DebugInfoLevel {
    /// No debug information, ideal for release build with minimal size.
    None,
    /// Basic debug information like names and line numbers.
    Basic,
    /// Full source code embedded into the binary.
    Full,
    
    
}


// The list of supported compiler backends.
pub enum CompilerBackend {
    /// Selects the SPIR-V backend.
    SPIRV
}





bitflags! {
    /// The list of language capabilities.
    pub struct Capabilities : u128 {
        const FLOAT16 = 1 << 0;
        const FLOAT64 = 1 << 0;
        const INT64 = 1 << 0;
        const INT16 = 1 << 0;
        const INT8 = 1 << 0;
        
        
        
        // TODO list all of the capabilities available in shaders, and some extensions
        
        
        
        
        
        
    }
}


impl Capabilities {
    
}



















