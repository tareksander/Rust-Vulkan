
module hash_map;

import world;

/// A simple HashMap implementation that can be placed in a GPU buffer and used in shaders.
// The struct layout has to be kept in sync with world.rs.
public struct GPUHashMap {
    /// Current size in elements
    uint32_t size;
    /// Maximum possible size in elements with the current allocation.
    uint32_t allocated;
    /// Slot size.
    uint16_t slot_size;
    uint16_t _reserved1;
    uint32_t _reserved2;

    static void* lookup(GPUHashMap* map, EntityID id) {
        uint32_t slot = id.id % map.size;
        uint8_t* data = (uint8_t*)map;
        data = &data[sizeof(GPUHashMap)];
        var i = slot;
        do {
            let meta = (SlotMeta*)&data[map.slot_size * (i + 1) - sizeof(SlotMeta)];
            if (meta.id == id) {
                return (void*)&data[map.slot_size * i];
            }
            i = i + 1;
            i %= map.size;
        } while (i != slot);
        return nullptr;
    }

    static uint32_t len(GPUHashMap* map) {
        return map.size;
    }

    static void* get(GPUHashMap* map, uint32_t slot) {
        if (slot >= map.size) {
            return nullptr;
        }
        uint8_t* data = (uint8_t*)map;
        data = &data[sizeof(GPUHashMap)];
        let meta = (SlotMeta*)&data[map.slot_size * (slot + 1) - sizeof(SlotMeta)];
        if (meta.flags & SlotFlags::Used) {
            return (void*)&data[map.slot_size * slot];
        }
        return nullptr;
    }
    
}

[Flags]
enum SlotFlags: uint8_t {
    Used
}

struct SlotMeta {
    SlotFlags flags;
    EntityID id;
}















