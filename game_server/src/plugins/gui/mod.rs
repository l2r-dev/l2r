use avian3d::prelude::*;
// use bevy_mod_picking::{DefaultPickingPlugins, PickableBundle};
// use bevy_transform_gizmo::{GizmoPickSource, GizmoTransformable, TransformGizmoPlugin};
use bevy::winit::{WakeUp, WinitPlugin};
use bevy::{
    ecs::{
        query::{QueryData, QueryFilter},
        relationship::Relationship,
        system::SystemParam,
    },
    log,
    prelude::*,
    window::{ExitCondition, Window, WindowCloseRequested, WindowPlugin},
};
use bevy_egui::EguiPrimaryContextPass;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use bevy_rich_text3d::{Text3d, Text3dPlugin, Text3dStyling, TextAtlas};
use game_core::{character::Character, stats::ColliderCapsuleSize};
use l2r_core::{assets::CustomAssetPlugin, plugins::custom_hierarchy::DespawnChildOf};
use std::{num::NonZero, sync::atomic::Ordering};
use ui::{
    camera_controller::{CameraControl, CameraControlPlugin},
    show_ui_system,
    state::UiState,
};

mod ui;

pub struct ServerUIPlugin;
impl Plugin for ServerUIPlugin {
    fn build(&self, app: &mut App) {
        let mut winit_plugin = WinitPlugin::<WakeUp>::default();
        winit_plugin.run_on_any_thread = true;
        app.add_plugins(
            DefaultPlugins
                .set(CustomAssetPlugin::custom())
                .set(winit_plugin)
                .set(WindowPlugin {
                    primary_window: Some({
                        let mut window = Window {
                            title: "L2r Gameserver".to_string(),
                            ..Default::default()
                        };
                        window.set_minimized(true);
                        window
                    }),
                    exit_condition: ExitCondition::DontExit,
                    ..default()
                }),
        )
        .register_type::<Option<Handle<Image>>>()
        .register_type::<AlphaMode>()
        .add_plugins(CameraControlPlugin)
        .add_plugins(DefaultInspectorConfigPlugin)
        .add_plugins(bevy_egui::EguiPlugin::default())
        .add_plugins(PhysicsDiagnosticsPlugin)
        .add_plugins(PhysicsDebugPlugin::default())
        .add_plugins(Text3dPlugin {
            load_system_fonts: true,
            ..Default::default()
        })
        // .add_plugins(DefaultPickingPlugins)
        // .add_plugins(TransformGizmoPlugin::default())
        .insert_resource(UiState::new())
        .add_systems(Startup, setup_camera)
        .add_systems(EguiPrimaryContextPass, show_ui_system)
        .add_systems(
            Update,
            (
                attach_collider_gizmo_to_characters,
                attach_floating_text_to_entities,
                update_floating_text_positions,
                handle_window_close_request,
            ),
        );
    }
}

fn setup_camera(mut commands: Commands) {
    let camera_translation = Vec3::new(27257.0, -3723.0, 10866.0);
    let camera_transform = Transform::from_translation(camera_translation);
    commands.spawn((
        Camera3d::default(),
        Projection::from(PerspectiveProjection {
            far: 1000.0,
            ..PerspectiveProjection::default()
        }),
        CameraControl::default(),
        camera_transform,
    ));
}

fn attach_collider_gizmo_to_characters(
    mut commands: Commands,
    query: Query<Entity, (With<Character>, Without<DebugRender>)>,
) {
    for entity in query.iter() {
        log::info!("Attaching collider gizmo to character {:?}", entity);
        commands
            .entity(entity)
            .try_insert((DebugRender::default().with_collider_color(Color::srgb(1.0, 0.0, 0.0)),));
    }
}

/// Component to mark text entities that are floating above other entities
#[derive(Component, Debug)]
#[require(Name::new("FloatingText".to_string()))]
pub struct FloatingText;

#[derive(Component, Debug)]
pub struct HasFloatingText;

/// QueryData for entities that can have floating text attached
#[derive(QueryData)]
struct FloatingTextTargetData<'a> {
    entity: Entity,
    name: &'a Name,
    collider: Option<&'a Collider>,
    transform: &'a Transform,
}

/// QueryFilter to exclude entities that already have floating text
#[derive(QueryFilter)]
struct FloatingTextTargetFilter {
    without_floating_text: Without<FloatingText>,
    without_has_floating_text: Without<HasFloatingText>,
}

/// SystemParam for querying entities that can have floating text attached
#[derive(SystemParam)]
struct FloatingTextTargetQuery<'w, 's> {
    entities: Query<'w, 's, FloatingTextTargetData<'static>, FloatingTextTargetFilter>,
}

/// QueryData for floating text entities that need position updates
#[derive(QueryData)]
#[query_data(mutable)]
struct FloatingTextUpdateData<'a> {
    transform: &'a mut Transform,
    despawn_child_of: &'a DespawnChildOf,
}

/// QueryData for parent entities (targets of floating text)
#[derive(QueryData)]
struct ParentEntityData<'a> {
    transform: &'a Transform,
    collider: Option<&'a Collider>,
}

/// QueryFilter for floating text entities
#[derive(QueryFilter)]
struct FloatingTextFilter {
    with_floating_text: With<FloatingText>,
}

/// QueryFilter for parent entities (excludes floating text)
#[derive(QueryFilter)]
struct ParentEntityFilter {
    without_floating_text: Without<FloatingText>,
}

fn attach_floating_text_to_entities(
    mut commands: Commands,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    query: FloatingTextTargetQuery,
) {
    // Create material for 3D text with transparency support
    let text_material = standard_materials.add(StandardMaterial {
        base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone_weak()),
        alpha_mode: AlphaMode::Mask(0.5),
        unlit: true,
        cull_mode: None,
        ..Default::default()
    });

    for target in query.entities.iter() {
        // Calculate text position offset based on collider or transform
        let offset = if let Some(collider) = target.collider {
            // Position above the collider
            Vec3::new(0.0, collider.height_f32() * 2.0, 0.0)
        } else {
            // Default position above the entity
            Vec3::new(0.0, 5.0, 0.0)
        };

        // Spawn the floating text entity using Text3d
        let mut transform = Transform::from_translation(target.transform.translation + offset);
        transform.scale = Vec3::splat(0.2);
        commands.spawn((
            Text3d::new(target.name.to_string()),
            Text3dStyling {
                font: "Tahoma".into(),
                size: 64.0,
                stroke: NonZero::new(5),
                color: Srgba::WHITE,
                stroke_color: Srgba::BLACK,
                layer_offset: 0.01,
                ..Default::default()
            },
            Mesh3d::default(),
            MeshMaterial3d(text_material.clone()),
            transform,
            Visibility::default(),
            FloatingText,
            DespawnChildOf(target.entity), // This ensures the text is automatically despawned when the parent entity is despawned
        ));

        // Mark the parent entity as having floating text
        commands.entity(target.entity).insert(HasFloatingText);
    }
}

fn update_floating_text_positions(
    mut floating_text_query: Query<FloatingTextUpdateData, FloatingTextFilter>,
    entities_query: Query<ParentEntityData, ParentEntityFilter>,
    camera_query: Single<Ref<Transform>, (With<Camera3d>, Without<FloatingText>)>,
) {
    let camera_transform = camera_query.as_ref();
    for mut floating_text in floating_text_query.iter_mut() {
        // Get the parent entity's transform and collider
        if let Ok(parent_entity) = entities_query.get(floating_text.despawn_child_of.get()) {
            // Update text position based on parent position and offset
            let mut offset = Vec3::new(0.0, 10.0, 0.0);

            // Update offset if parent collider height changed
            if let Some(collider) = parent_entity.collider {
                offset.y = collider.height_f32() * 2.0;
            }

            floating_text.transform.translation = parent_entity.transform.translation + offset;

            // Calculate direction from text to camera (only X and Z components for cylindrical billboarding)
            let camera_direction =
                camera_transform.translation - floating_text.transform.translation;
            let horizontal_direction =
                Vec3::new(camera_direction.x, 0.0, camera_direction.z).normalize();

            // Create a rotation that faces the camera horizontally but keeps Y axis upright
            // We want the text's forward direction (-Z) to point toward the camera horizontally
            let angle = horizontal_direction.x.atan2(horizontal_direction.z);
            floating_text.transform.rotation = Quat::from_rotation_y(angle);
        }
    }
}

fn handle_window_close_request(
    mut close_events: EventReader<WindowCloseRequested>,
    shutdown_state: Option<ResMut<crate::plugins::shutdown::ShutdownState>>,
) {
    if close_events.read().next().is_some() {
        if let Some(state) = shutdown_state {
            log::info!("Window close requested, initiating graceful shutdown...");
            state.shutdown_requested.store(true, Ordering::SeqCst);
        } else {
            log::warn!("Window close requested but ShutdownState not found!");
        }
    }
}
