---@class PathFinding Helper module for pathfinding operations
local PathFinding = {}

---Requests visibility check and pathfinding if needed
---@param entity Entity The entity that needs to move
---@param start_pos Vec3 Starting position {x: number, y: number, z: number}
---@param target_pos Vec3 Target position {x: number, y: number, z: number}
function PathFinding.request_visibility_check(entity, start_pos, target_pos)
    local request_data = {
        entity = entity,
        start = start_pos,
        target = target_pos
    }

    RegionGeoData.request_visibility_check(request_data)
end

return PathFinding
