module world;

public struct EntityID {
    public uint32_t id;
    public uint32_t extra;
    
    
    
    
    
}

public bool operator ==(EntityID a, EntityID b) {
    return a.id == b.id && ((a.extra & 0xffff) == (b.extra & 0xffff));
}

public struct WorldData {
    
    
    
    
}

[vk::binding(0, 0)]
ConstantBuffer<WorldData, ScalarDataLayout> world;












