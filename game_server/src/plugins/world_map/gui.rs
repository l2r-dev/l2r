use bevy::prelude::*;
use game_core::custom_hierarchy::DespawnChildOf;
use map::{Region, WorldMap, block::Block};
use spatial::GameVec3;

pub struct WorldMapGui;
impl Plugin for WorldMapGui {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_block_gizmos);
    }
}

#[derive(Component)]
struct BlockGizmos;

fn update_block_gizmos(
    mut commands: Commands,
    mut gizmo_assets: ResMut<Assets<GizmoAsset>>,
    world_map: Res<WorldMap>,
    all_regions: Query<&Region>,
    changed_regions: Query<Entity, Changed<Region>>,
    existing_gizmos: Query<Entity, With<BlockGizmos>>,
) {
    if changed_regions.is_empty() {
        return;
    }

    for entity in existing_gizmos.iter() {
        commands.entity(entity).despawn();
    }

    let active_regions = world_map.active_regions();
    for active_region in active_regions {
        if let Ok(region) = all_regions.get(active_region) {
            let block_centers = region.block_centers();
            let mut gizmo = GizmoAsset::new();
            if let Some(block_centers) = block_centers {
                for block_center in block_centers {
                    let color = Color::srgba(0.0, 1.0, 0.0, 1.0); // Green
                    add_square_to_gizmo(&mut gizmo, *block_center, color);
                }
            }
            commands.spawn((
                DespawnChildOf(active_region),
                Gizmo {
                    handle: gizmo_assets.add(gizmo),
                    line_config: GizmoLineConfig {
                        width: 1.0,
                        ..default()
                    },
                    ..default()
                },
                BlockGizmos,
            ));
        }
    }
}

fn add_square_to_gizmo(gizmo: &mut GizmoAsset, block_center: GameVec3, color: Color) {
    let half_size = Block::SIZE_X as f32 / 2.0;

    // Define the corners of the square
    let bottom_left = Vec3::new(
        block_center.x as f32 - half_size,
        block_center.z as f32,
        block_center.y as f32 - half_size,
    );
    let bottom_right = Vec3::new(
        block_center.x as f32 + half_size,
        block_center.z as f32,
        block_center.y as f32 - half_size,
    );
    let top_left = Vec3::new(
        block_center.x as f32 - half_size,
        block_center.z as f32,
        block_center.y as f32 + half_size,
    );
    let top_right = Vec3::new(
        block_center.x as f32 + half_size,
        block_center.z as f32,
        block_center.y as f32 + half_size,
    );

    gizmo.line(bottom_left, bottom_right, color);
    gizmo.line(bottom_right, top_right, color);
    gizmo.line(top_right, top_left, color);
    gizmo.line(top_left, bottom_left, color);
}
