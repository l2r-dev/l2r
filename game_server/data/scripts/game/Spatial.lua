---@class Spatial
local Spatial = {}

-- Calculates the 2D distance between two 3D points (ignoring Y axis)
---@param from table The starting position (with x, y, z coordinates)
---@param to table The ending position (with x, y, z coordinates)
---@return number The flat distance between the two points
function Spatial.flat_distance(from, to)
    local dx = to.x - from.x
    local dz = to.z - from.z

    return math.sqrt(dx * dx + dz * dz)
end

-- Converts a quaternion to degrees for game rotation
---@param quat table The quaternion to convert
---@return number The rotation in degrees (0-360), adjusted for game coordinate system
function Spatial.quat_to_degrees(quat)
    local euler_rot = construct(types.EulerRot, { variant = "YXZ" })
    local euler = quat:to_euler(euler_rot)

    local yaw = euler[1]
    local degrees = math.deg(yaw) % 360

    -- Need substract from 270 because on server 0 is North, but in game 0 is East
    return (270 - degrees) % 360
end

-- Calculates the direction angle from one entity to another
---@param from_entity Entity The entity to calculate direction from
---@param to_entity Entity The entity to calculate direction to
---@return number The direction angle in degrees (0-360), or 0 if either entity lacks a Transform
function Spatial.calculate_direction(from_entity, to_entity)
    local from_transform = world.get_component(from_entity, types.Transform)
    local to_transform = world.get_component(to_entity, types.Transform)

    if not from_transform or not to_transform then
        return 0
    end

    -- Calculate direction vector
    local dx = to_transform.translation.x - from_transform.translation.x
    local dz = to_transform.translation.z - from_transform.translation.z

    -- Convert to angle in degrees
    return math.deg(math.atan(dz, dx)) % 360
end

return Spatial
