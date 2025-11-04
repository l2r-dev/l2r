use avian3d::prelude::*;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, PhysicsLayer)]
pub enum GameLayer {
    #[default]
    Default,
    Player,
    Npc,
    /// Solid environment (walls, closed doors) - blocks movement
    Environment,
    /// Passable environment (open doors, destroyed walls) - doesn't block movement
    EnvironmentPassable,
    Sensor,
    Item,
}

impl GameLayer {
    pub fn player() -> CollisionLayers {
        CollisionLayers::new([Self::Player], [Self::Player, Self::Npc, Self::Environment])
    }

    pub fn npc() -> CollisionLayers {
        CollisionLayers::new([Self::Npc], [Self::Player, Self::Npc, Self::Environment])
    }

    /// Collision layers for static environment (walls, closed doors, solid objects)
    pub fn environment_solid() -> CollisionLayers {
        CollisionLayers::new([Self::Environment], [Self::Player, Self::Npc])
    }

    /// Collision layers for passable environment (open doors, destroyed walls)
    pub fn environment_passable() -> CollisionLayers {
        CollisionLayers::new([Self::EnvironmentPassable], [Self::Player, Self::Npc])
    }

    pub fn player_sensor() -> CollisionLayers {
        CollisionLayers::new([Self::Sensor], [Self::Player])
    }

    /// Сollision layers for items
    /// Items don't physically interact with anything, but are visible to spatial queries
    pub fn item() -> CollisionLayers {
        CollisionLayers::new([Self::Item], LayerMask::NONE)
    }

    pub fn solid_environment_mask() -> LayerMask {
        LayerMask::from([Self::Environment])
    }

    pub fn passable_environment_mask() -> LayerMask {
        LayerMask::from([Self::EnvironmentPassable])
    }

    /// Mask for spatial queries to find entities that should be visible to players
    pub fn encounters_mask() -> LayerMask {
        LayerMask::from([
            Self::Player,
            Self::Npc,
            Self::Item,
            Self::Environment,
            Self::EnvironmentPassable,
        ])
    }

    /// Mask for broadcasting packets to nearby characters
    pub fn broadcast_mask() -> LayerMask {
        LayerMask::from([Self::Player])
    }

    /// Mask for multi-target attacks to find players and NPCs
    pub fn attack_targets_mask() -> LayerMask {
        LayerMask::from([Self::Player, Self::Npc])
    }
}
