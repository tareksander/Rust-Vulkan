//! 
//! 
//! 
//! 
//! 
//! 
//! 
//! 
//! 
//! 


use std::{any::TypeId, collections::HashMap, hash::{Hash, Hasher}, ops::Range};

use ash::{prelude::VkResult, vk::{self, CommandPoolCreateFlags}};
pub use hash_map::GPUHashMap;




/// The Entity ID type. The actual ID part is the first u32, the second u32 stores additional data such as the generation.
/// 
/// Currently, the lower 16 bits of the second u32 is the generation.
/// 
#[repr(packed)]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct EntityID(u32, u32);


impl EntityID {
    
    /// Constructs a new Entity ID from parts
    /// # Safety
    /// The ID - generation combination has to not have been in use for the World the ID is intended for.
    /// 
    pub unsafe fn new(id: u32, extra: u32) -> Self {
        Self(id, extra)
    }
    
    
    pub fn id(&self) -> u32 {
        self.0
    }
    
    
    pub fn generation(&self) -> u16 {
        self.1 as u16
    }
    
    
    
    
    
    
    
    
    
}


impl PartialEq for EntityID {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for EntityID {}

impl Hash for EntityID {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.0);
    }
}


mod hash_map {
    use std::ffi::c_void;

    use bitflags::bitflags;

    use super::EntityID;

    
    /// A simple HashMap implementation that can be placed in a GPU buffer and used in shaders.
    #[repr(C)]
    // The struct layout has to be kept in sync with shaders/hash-map.slang.
    pub struct GPUHashMap {
        /// Current size in elements
        size: u32,
        /// Maximum possible size in elements with the current allocation.
        allocated: u32,
        /// Slot size.
        slot_size: u16,
        _reserved1: u16,
        _reserved2: u32,
    }
    
    
    bitflags! {
        // The flags have to be kept in sync with shaders/hash-map.slang.
        #[repr(transparent)]
        struct SlotFlags: u8 {
            const USED = 1 << 0;
            
        }
    }
    
    /// Metadata at the end of a slot.
    // The struct layout has to be kept in sync with shaders/hash-map.slang.
    #[repr(C)]
    struct SlotMeta {
        flags: SlotFlags,
        id: EntityID,
    }
    
    impl SlotMeta {
        
        /// Default SlotMeta denoting a flagless empty slot.
        fn new() -> Self {
            Self {
                flags: SlotFlags::empty(),
                id: EntityID(0, 0)
            }
        }
    }
    
    impl GPUHashMap {
        
        
        /// Creates a GPUHashMap from a base address and size of the region in bytes. The function returns an error in case the memory region was too small.
        /// The minimum size is the size of the GPUHashMap structure plus 10 elements.
        /// # Safety
        /// The allocation has to at least have the size promised to the function, and has to point to valid memory.
        /// 
        pub unsafe fn new(base: *mut c_void, mut size: usize, element_size: u16, element_alignment: u16) -> Result<*mut Self, ()> {
            let s = base as *mut GPUHashMap;
            // Check if we even have enough size for the struct and subtract that from the total.
            if size <= size_of::<Self>() {
                return Err(());
            }
            size -= size_of::<Self>();
            
            // Add one to the size for internal entry bookkeeping and align again.
            let slot_size = (element_size + size_of::<SlotMeta>() as u16).next_multiple_of(element_alignment);
            
            
            // Calculate the numer of elements that fit in the allocation, rounded down.
            let allocated = (size / slot_size as usize) as u32;
            // 10 elements should be a reasonable minimum
            if allocated < 10 {
                return Err(());
            }
            
            // Write empty metadata for each slot. The elements are left uninitialized.
            for i in 1..(allocated+1) {
                // base + size_of(Self) + i * element_size - size_of(SlotMeta)
                let meta = unsafe { base.add(size_of::<Self>() + i as usize * element_size as usize)
                    .offset(-(size_of::<SlotMeta>() as isize)) } as *mut SlotMeta;
                unsafe { meta.write(SlotMeta::new()) };
            }
            
            
            unsafe { s.write(GPUHashMap {
                size: allocated,
                allocated,
                slot_size,
                _reserved1: 0,
                _reserved2: 0,
            }) };
            Ok(s)
        }
        
        
        
        
        
    }
}



pub struct AdditionHasher {
    v: u32,
}

impl Hasher for AdditionHasher {
    fn finish(&self) -> u64 {
        self.v as u64
    }

    fn write(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.v = self.v.wrapping_add(*b as u32);
        }
    }
    
    fn write_u8(&mut self, i: u8) {
        self.v = self.v.wrapping_add(i as u32);
    }
    
    fn write_u16(&mut self, i: u16) {
        self.v = self.v.wrapping_add(i as u32);
    }
    
    fn write_u32(&mut self, i: u32) {
        self.v = self.v.wrapping_add(i);
    }
    
    fn write_u64(&mut self, i: u64) {
        self.v = self.v.wrapping_add(i as u32);
        self.v = self.v.wrapping_add((i >> 32) as u32);
    }
    
    fn write_u128(&mut self, i: u128) {
        self.write_u64(i as u64);
        self.write_u64((i >> 64) as u64);
    }
    
    fn write_usize(&mut self, i: usize) {
        match size_of::<usize>() {
            4 => {
                self.write_u32(i as u32);
            },
            8 => {
                self.write_u64(i as u64);
            },
            _ => {
                self.write(&i.to_ne_bytes())
            }
        }
    }
}

pub struct World {
    device: ash::Device,
    queue: vk::Queue,
    queue_family: u32,
    pool: vk::CommandPool,
    semaphore: vk::Semaphore,
    state: WorldState,
}


struct Archetype {
    /// Offsets of the components in the archetype struct on the GPU.
    component_offsets: Vec<u16>,
    /// Maps Component IDs to offset indices.
    component_indices: HashMap<EntityID, u16>,
}

struct WorldState {
    /// The next timeline semaphore value to wait on
    next_wait: u64,
    /// The list components and map of components to Rust types. 
    components: HashMap<TypeId, EntityID>,
    /// Stores each component's size
    component_size_alignments: HashMap<EntityID, (u16, u16)>,
}


/// A range of ID to allocate from.
pub struct IDRange {
    range: Range<u32>,
    next: u32,
}

impl IDRange {
    /// 
    /// # Safety
    /// ID ranges used in a World must not overlap or include 0.
    /// 
    pub unsafe fn new(range: Range<u32>) -> Self {
        Self {
            next: range.start,
            range,
        }
    }
    
    
}


impl World {
    
    /// Creates a new ECS world.
    /// 
    /// # Safety
    /// - The supplied device has to live at least as long as this struct.
    /// - The device has to have been created with all required extensions and capabilities and the required Vulkan version.
    /// - The physical device has to be the physical device the logical device was created from.
    /// - The queue has to allow compute work.
    /// - The queue has to belong to the device.
    /// - The queue family has to be the family index of the queue.
    /// 
    /// 
    pub unsafe fn new(physical_device: vk::PhysicalDevice, device: ash::Device, queue: vk::Queue, queue_family: u32) -> VkResult<Self> {
        let pool;
        {
            let info = vk::CommandPoolCreateInfo::default().queue_family_index(queue_family).flags(CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
            pool = unsafe { device.create_command_pool(&info, None) }?;
        }
        
        // TODO
        // create descriptor set layout for the world uniform buffer *(or just use a push constant to set the world data pointer)*
        // Select appropriate heap and memory types (select one if possible on integrated GPUs)
        // create pipeline layouts for entity removal, moving, addition
        // create the pipelines from included SPIR-V
        // VK_PIPELINE_CREATE_DISPATCH_BASE_BIT needed + VK_PIPELINE_CREATE_EARLY_RETURN_ON_FAILURE_BIT + VK_PIPELINE_CREATE_NO_PROTECTED_ACCESS_BIT
        
        
        
        
        let semaphore;
        {
            let mut tinfo = vk::SemaphoreTypeCreateInfo::default().initial_value(0).semaphore_type(vk::SemaphoreType::TIMELINE);
            let info = vk::SemaphoreCreateInfo::default().push_next(&mut tinfo);
            semaphore = unsafe { device.create_semaphore(&info, None) }?;
        }
        
        Ok(Self {
            device,
            queue,
            queue_family,
            pool,
            semaphore,
            state: WorldState {
                next_wait: 0,
                component_size_alignments: HashMap::new(),
                components: HashMap::new(),
            }
        })
    }
    
    // TODO
    // Function to create a command recorder for ECS commands (borrows the World)
    // the recorder can then be used to write into a command buffer
    
    /// Registers a new component type and returns the allocated ID. Returns an error when the range is already full.
    pub fn register_component(&mut self, range: &mut IDRange, ty: TypeId, size: u16, alignment: u16) -> Result<EntityID, ()> {
        if range.next >= range.range.end {
            return Err(());
        }
        let id = unsafe { EntityID::new(range.next, 0) };
        range.next += 1;
        self.state.components.insert(ty, id);
        self.state.component_size_alignments.insert(id, (size, alignment));
        return Ok(id);
    }
    
    
    
    pub fn encoder(&mut self) -> WorldEncoder {
        WorldEncoder::new(self)
    }
    
    
}


/// Used to encode ECS commands for a World.
pub struct WorldEncoder<'a> {
    world: &'a mut World,
    
}

impl<'a> WorldEncoder<'a> {
    fn new (world: &'a mut World) -> Self {
        Self {
            world
        }
    }
    
    
    
    
    
    
    
}





impl Drop for World {
    fn drop(&mut self) {
        todo!()
    }
}



#[cfg(test)]
mod test {
    use std::ffi::c_void;

    use super::GPUHashMap;

    
    
    #[test]
    fn test_hashmap() {
        let mut buffer: [u8; 1024] = [0; 1024];
        let map = unsafe { GPUHashMap::new(std::ptr::addr_of_mut!(buffer) as *mut c_void, 1024, 8, 4) };
        
        
    }
    
}


