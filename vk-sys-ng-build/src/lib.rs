

pub mod data;



mod parser;
mod generator;

pub use parser::VKParseError;
pub use parser::parse_registry;
pub use generator::Config;
pub use generator::generate_vk;









#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf, sync::LazyLock};

    use super::*;
    
    static SPEC_PATH: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from("../vk-sys-ng/vk-only.xml"));
    
    #[test]
    fn it_works() {
        let registry = parse_registry(&SPEC_PATH).unwrap();
        
        println!("{:#?}", registry.types["VkPhysicalDeviceFeatures2"]);
        println!("{:#?}", registry.commands["vkCmdDraw"]);
        println!("{:#?}", registry.enums["API Constants"]);
        println!("{:#?}", registry.enums["VkImageLayout"]);
        println!("{:#?}", registry.enums["VkBufferUsageFlagBits"]);
        println!("{:#?}", registry.enums["VkResult"]);
        
    }
    
    #[test]
    fn test_gen() {
        let registry = parse_registry(&SPEC_PATH).unwrap();
        fs::write("vk-test.rs", generate_vk(&Config {
            raw_bindings: false,
        }, &registry)).unwrap();
    }
    
    
}
