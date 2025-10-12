---@class PlaySound
local PlaySound = {}

-- Sends a PlaySound packet with the specified sound file
---@param entity Entity The entity to play the sound for
---@param sound_file string The name of the sound file to play
function PlaySound.send(entity, sound_file)
    local play_sound_packet = construct(types.PlaySound, {
        _unknown1 = 0,
        sound_file = sound_file,
        _unknown3 = 0,
        _unknown4 = 0,
        _unknown5 = 0,
        _unknown6 = 0,
        _unknown7 = 0,
        _unknown8 = 0
    })

    local gs_packet = construct(types.GameServerPacket, { variant = "PlaySound", _1 = play_sound_packet })
    local broadcastScope = construct(types.BroadcastScope, { variant = "KnownAndSelf" })
    GameServerPacket.broadcast({ entity, gs_packet, broadcastScope })
end

return PlaySound
