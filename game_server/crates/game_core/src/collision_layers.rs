use avian3d::prelude::*;

/// Collision layers for the game
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Layer {
    /// Characters (players, NPCs)
    Character = 0,
    /// Static environment (walls, doors, destructible objects)
    Environment = 1,
    /// Sensors/Triggers (trigger zones)
    Sensor = 2,
    /// Items on the ground
    Item = 3,
}

impl Layer {
    /// Сollision layers for a character
    pub fn character() -> CollisionLayers {
        CollisionLayers::new([Self::Character], [Self::Character, Self::Environment])
    }

    /// Сollision layers for static environment (walls, closed doors, solid objects)
    /// Blocks characters
    pub fn environment_solid() -> CollisionLayers {
        CollisionLayers::new([Self::Environment], [Self::Character])
    }

    /// Сollision layers for passable environment (open doors, destroyed walls)
    /// Doesn't block movement but can still be detected
    pub fn environment_passable() -> CollisionLayers {
        CollisionLayers::new([Self::Environment], LayerMask::NONE)
    }

    /// Сollision layers for sensors (triggers)
    pub fn sensor() -> CollisionLayers {
        CollisionLayers::new([Self::Sensor], LayerMask::ALL)
    }

    /// Сollision layers for items
    pub fn item() -> CollisionLayers {
        CollisionLayers::new([Self::Item], [Self::Character])
    }

    /// Mask for encounters detection (characters, items, environment objects)
    /// Used by spatial queries to find entities that should be visible to players
    pub fn encounters_mask() -> LayerMask {
        LayerMask::from([Self::Character, Self::Item, Self::Environment])
    }
}

impl From<Layer> for LayerMask {
    fn from(layer: Layer) -> Self {
        LayerMask::from(1u32 << layer as u32)
    }
}
