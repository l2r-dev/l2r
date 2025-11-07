use bevy::prelude::*;
use config::Config;
use game_core::custom_hierarchy::DespawnChildOf;
use map::{
    Region, WorldMapQuery,
    block::{Block, Cell, GeoBlock},
};
use spatial::{GameVec3, NavigationDirection};

pub struct WorldMapGui;
impl Plugin for WorldMapGui {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_block_gizmos, update_cell_gizmos));
    }
}

const COLOR_GREEN: Color = Color::srgba(0.0, 1.0, 0.0, 1.0);
const COLOR_BLUE: Color = Color::srgba(0.0, 0.5, 1.0, 1.0);
const COLOR_RED: Color = Color::srgba(1.0, 0.0, 0.0, 1.0);

#[derive(Component)]
struct BlockGizmos;

#[derive(Component)]
struct CellGizmos;

fn update_block_gizmos(
    mut commands: Commands,
    mut gizmo_assets: ResMut<Assets<GizmoAsset>>,
    config: Res<Config>,
    map_query: WorldMapQuery,
    changed_regions: Query<Entity, Changed<Region>>,
    existing_block_gizmos: Query<Entity, With<BlockGizmos>>,
) {
    if changed_regions.is_empty() && !config.is_changed() {
        return;
    }

    if !config.gui().geodata_blocks {
        for entity in existing_block_gizmos.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    for entity in existing_block_gizmos.iter() {
        commands.entity(entity).despawn();
    }

    let active_regions = map_query.inner.world_map.active_regions();
    for active_region in active_regions {
        if let Ok(region) = map_query.inner.regions.get(active_region) {
            let block_centers = region.block_centers();
            let mut gizmo = GizmoAsset::new();
            if let Some(block_centers) = block_centers {
                for block_center in block_centers {
                    add_square_to_block_gizmo(&mut gizmo, *block_center, COLOR_GREEN);
                }
            }
            commands.spawn((
                Name::new("Block Gizmos"),
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

fn add_square_to_block_gizmo(gizmo: &mut GizmoAsset, block_center: GameVec3, color: Color) {
    let half_size = Block::SIZE_X as f32 / 2.0;

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

fn update_cell_gizmos(
    mut commands: Commands,
    mut gizmo_assets: ResMut<Assets<GizmoAsset>>,
    config: Res<Config>,
    map_query: WorldMapQuery,
    changed_regions: Query<Entity, Changed<Region>>,
    existing_cell_gizmos: Query<Entity, With<CellGizmos>>,
) {
    if changed_regions.is_empty() && !config.is_changed() {
        return;
    }

    if !config.gui().geodata_cells {
        for entity in existing_cell_gizmos.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    for entity in existing_cell_gizmos.iter() {
        commands.entity(entity).despawn();
    }

    let active_regions = map_query.inner.world_map.active_regions();
    for active_region in active_regions {
        if let Ok(region) = map_query.inner.regions.get(active_region) {
            if let Ok(geodata) = map_query.inner.region_geodata(region.id()) {
                let mut gizmo = GizmoAsset::new();

                for block_x in 0..Region::BLOCKS_X {
                    for block_y in 0..Region::BLOCKS_Y {
                        if let Some(block) = geodata.block_by_grid(block_x, block_y) {
                            let block_geo_pos = region.block_position_geo_by_grid(block_x, block_y);

                            let is_flat_block = matches!(block, map::block::Block::Flat(_));
                            if is_flat_block {
                                for offset in 0..Block::CELLS {
                                    let cell_coords = Block::cell_coordinates_by_offset(offset);
                                    let cell_geo_pos = spatial::GeoVec3::new(
                                        spatial::GeoPoint::new(
                                            block_geo_pos.point.x + cell_coords.x,
                                            block_geo_pos.point.y + cell_coords.y,
                                        ),
                                        0,
                                    );

                                    let height = block.nearest_height(cell_geo_pos);
                                    let cell_position = map::WorldMap::geo_to_game(
                                        spatial::GeoVec3::new(cell_geo_pos.point, height),
                                    );

                                    // Flat blocks are always passable (no NSWE flags stored)
                                    add_cell_square_to_gizmo(&mut gizmo, cell_position, COLOR_BLUE);
                                }
                            } else {
                                for (cell, offset) in block.cells_with_offsets() {
                                    let cell_coords = Block::cell_coordinates_by_offset(offset);
                                    let cell_geo_pos = spatial::GeoVec3::new(
                                        spatial::GeoPoint::new(
                                            block_geo_pos.point.x + cell_coords.x,
                                            block_geo_pos.point.y + cell_coords.y,
                                        ),
                                        0,
                                    );
                                    let height = cell.height();
                                    let cell_position = map::WorldMap::geo_to_game(
                                        spatial::GeoVec3::new(cell_geo_pos.point, height),
                                    );
                                    let color = if cell.is_fully_blocked() {
                                        COLOR_RED
                                    } else {
                                        COLOR_BLUE
                                    };

                                    add_cell_square_to_gizmo(&mut gizmo, cell_position, color);

                                    // Draw red lines showing blocked directions
                                    add_blocked_directions_to_gizmo(
                                        &mut gizmo,
                                        cell,
                                        cell_position,
                                    );
                                }
                            }
                        }
                    }
                }

                commands.spawn((
                    Name::new("Cell Gizmos"),
                    DespawnChildOf(active_region),
                    Gizmo {
                        handle: gizmo_assets.add(gizmo),
                        line_config: GizmoLineConfig {
                            width: 0.5,
                            ..default()
                        },
                        ..default()
                    },
                    CellGizmos,
                ));
            }
        }
    }
}

fn add_cell_square_to_gizmo(gizmo: &mut GizmoAsset, cell_position: GameVec3, color: Color) {
    let half_size = Cell::HALF_SIZE as f32;

    let bottom_left = Vec3::new(
        cell_position.x as f32 - half_size,
        cell_position.z as f32,
        cell_position.y as f32 - half_size,
    );
    let bottom_right = Vec3::new(
        cell_position.x as f32 + half_size,
        cell_position.z as f32,
        cell_position.y as f32 - half_size,
    );
    let top_left = Vec3::new(
        cell_position.x as f32 - half_size,
        cell_position.z as f32,
        cell_position.y as f32 + half_size,
    );
    let top_right = Vec3::new(
        cell_position.x as f32 + half_size,
        cell_position.z as f32,
        cell_position.y as f32 + half_size,
    );

    gizmo.line(bottom_left, bottom_right, color);
    gizmo.line(bottom_right, top_right, color);
    gizmo.line(top_right, top_left, color);
    gizmo.line(top_left, bottom_left, color);
}

const BLOCKED_DIRECTION_LINE_SIZE: f32 = Cell::HALF_SIZE as f32 * 0.8;

fn add_blocked_directions_to_gizmo(gizmo: &mut GizmoAsset, cell: &Cell, cell_position: GameVec3) {
    let center = Vec3::new(
        cell_position.x as f32,
        cell_position.z as f32,
        cell_position.y as f32,
    );

    for direction in NavigationDirection::all().iter_composite() {
        if !cell.is_passable(direction) {
            let (dx, dy) = direction.offset(1);
            let end = Vec3::new(
                center.x + dx as f32 * BLOCKED_DIRECTION_LINE_SIZE,
                center.y,
                center.z + dy as f32 * BLOCKED_DIRECTION_LINE_SIZE,
            );

            gizmo.line(center, end, COLOR_RED);
        }
    }
}
