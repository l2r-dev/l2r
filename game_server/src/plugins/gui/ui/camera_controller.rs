use bevy::{
    ecs::system::SystemParam,
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow, Window},
};

#[derive(Reflect, Resource)]
#[reflect(Resource)]
pub struct CameraControlSettings {
    pub active_camera: Option<Entity>,
    pub active_window: Option<Entity>,
    pub base_speed: f32,
    pub alt_speed: f32,
    pub sensitivity: f32,
    pub attachment_offset: Vec3,
}

impl Default for CameraControlSettings {
    fn default() -> Self {
        let base_speed = 1000.0;
        Self {
            active_camera: None,
            active_window: None,
            base_speed,
            alt_speed: base_speed * 20.0,
            sensitivity: 0.001,
            attachment_offset: Vec3::new(0.0, 150.0, 300.0), // Y = height above, Z = distance behind
        }
    }
}

#[derive(Component, Default, Reflect)]
pub struct CameraControl {
    pub attached_to: Option<Entity>,
}

pub struct CameraControlPlugin;
impl Plugin for CameraControlPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CameraControlSettings>();
        app.register_type::<CameraControl>();

        app.init_resource::<CameraControlSettings>();
        app.add_systems(PostStartup, camera_init);
        app.add_systems(Update, (camera_update, camera_attachment_update).chain());
    }
}

fn camera_init(
    cameras: Query<Entity, With<CameraControl>>,
    mut settings: ResMut<CameraControlSettings>,
) {
    use bevy::ecs::query::QuerySingleError;

    if settings.active_camera.is_none() {
        settings.active_camera = match cameras.single() {
            Ok(a) => Some(a),
            Err(QuerySingleError::NoEntities(_)) => {
                warn!("Failed to find a CameraControl; Active camera will remain unset.");
                None
            }
            Err(QuerySingleError::MultipleEntities(_)) => {
                warn!("Found more than one CameraControl; Active camera will remain unset.");
                None
            }
        };
    }
}

#[derive(SystemParam)]
pub struct InputResources<'w, 's> {
    time: Res<'w, Time>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    buttons: Res<'w, ButtonInput<MouseButton>>,
    motion: EventReader<'w, 's, MouseMotion>,
}

fn camera_update(
    mut input: InputResources,
    mut settings: ResMut<CameraControlSettings>,
    mut windows: Query<(&mut Window, Option<&PrimaryWindow>)>,
    mut camera_query: Query<(&mut Transform, &mut CameraControl)>,
    mut focus: Local<bool>,
) {
    let Some(camera_id) = settings.active_camera else {
        input.motion.clear();
        return;
    };

    let Ok((mut camera_transform, mut camera_control)) = camera_query.get_mut(camera_id) else {
        error!("Failed to find camera for active camera entity ({camera_id:?})");
        settings.active_camera = None;
        input.motion.clear();
        return;
    };

    let mut window = match settings.active_window {
        Some(active) => {
            let Ok((window, _)) = windows.get_mut(active) else {
                error!("Failed to find active window ({active:?})");
                settings.active_window = None;
                input.motion.clear();
                return;
            };
            window
        }
        None => {
            let Some((window, _)) = windows.iter_mut().find(|(_, primary)| primary.is_some())
            else {
                return;
            };
            window
        }
    };

    update_focus(&input.keys, &input.buttons, &mut focus, &mut window);

    // If camera is attached, detach it when user tries to move it
    if camera_control.attached_to.is_some() && *focus {
        let has_movement_input = input.keys.pressed(KeyCode::KeyW)
            || input.keys.pressed(KeyCode::KeyS)
            || input.keys.pressed(KeyCode::KeyA)
            || input.keys.pressed(KeyCode::KeyD)
            || input.keys.pressed(KeyCode::Space)
            || input.keys.pressed(KeyCode::ControlLeft);

        let has_mouse_input = input.motion.read().count() > 0;

        if has_movement_input || has_mouse_input {
            camera_control.attached_to = None;
            info!("Camera detached due to user input");
        }
    }

    // Only allow manual control if camera is not attached
    if camera_control.attached_to.is_none() && *focus {
        handle_rotation(&mut input.motion, &mut camera_transform, &settings);
        handle_translation(&input.keys, &input.time, &mut camera_transform, &settings);
    }

    input.motion.clear();
}

fn update_focus(
    keys: &ButtonInput<KeyCode>,
    buttons: &ButtonInput<MouseButton>,
    focus: &mut bool,
    window: &mut Window,
) {
    if keys.just_pressed(KeyCode::Escape) {
        *focus = false;
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
    } else if buttons.just_pressed(MouseButton::Left) && keys.pressed(KeyCode::AltLeft) {
        *focus = true;
        window.cursor_options.grab_mode = CursorGrabMode::Confined;
        window.cursor_options.visible = false;
    }
}

fn handle_rotation(
    motion: &mut EventReader<MouseMotion>,
    camera_transform: &mut Transform,
    settings: &CameraControlSettings,
) {
    let mouse_delta = {
        let mut total = Vec2::ZERO;
        for d in motion.read() {
            total += d.delta;
        }
        total
    };

    let mouse_x = -mouse_delta.x * settings.sensitivity;
    let mouse_y = -mouse_delta.y * settings.sensitivity;

    let mut dof = Vec3::from(camera_transform.rotation.to_euler(EulerRot::YXZ));

    dof.x += mouse_x;
    dof.y = (dof.y + mouse_y).clamp(-89f32.to_radians(), 89f32.to_radians());
    dof.z = 0f32;

    camera_transform.rotation = Quat::from_euler(EulerRot::YXZ, dof.x, dof.y, dof.z);
}

fn handle_translation(
    keys: &ButtonInput<KeyCode>,
    time: &Time,
    camera_transform: &mut Transform,
    settings: &CameraControlSettings,
) {
    let forward = if keys.pressed(KeyCode::KeyW) {
        1f32
    } else {
        0f32
    };

    let backward = if keys.pressed(KeyCode::KeyS) {
        1f32
    } else {
        0f32
    };

    let right = if keys.pressed(KeyCode::KeyD) {
        1f32
    } else {
        0f32
    };

    let left = if keys.pressed(KeyCode::KeyA) {
        1f32
    } else {
        0f32
    };

    let up = if keys.pressed(KeyCode::Space) {
        1f32
    } else {
        0f32
    };

    let down = if keys.pressed(KeyCode::ControlLeft) {
        1f32
    } else {
        0f32
    };

    let speed = if keys.pressed(KeyCode::ShiftLeft) {
        settings.alt_speed
    } else {
        settings.base_speed
    };

    let delta_axial = (forward - backward) * speed;
    let delta_lateral = (right - left) * speed;
    let delta_vertical = (up - down) * speed;

    let mut forward = *camera_transform.forward();
    forward.y = 0f32;
    forward = forward.normalize_or_zero();

    let mut right = *camera_transform.right();
    right.y = 0f32;
    let up = Vec3::Y;

    let result = forward * delta_axial + right * delta_lateral + up * delta_vertical;

    camera_transform.translation += result * time.delta_secs();
}

fn camera_attachment_update(
    mut camera_query: Query<(&mut Transform, &CameraControl)>,
    target_query: Query<&Transform, (With<Transform>, Without<CameraControl>)>,
    camera_settings: Res<CameraControlSettings>,
) {
    for (mut camera_transform, camera_control) in camera_query.iter_mut() {
        if let Some(target_entity) = camera_control.attached_to
            && let Ok(target_transform) = target_query.get(target_entity)
        {
            let target_forward = target_transform.forward();
            let target_back = -target_forward;

            // Position camera behind and above the target
            let horizontal_distance = camera_settings.attachment_offset.z;
            let vertical_offset = camera_settings.attachment_offset.y;

            // Calculate camera position: behind the target + vertical offset
            let camera_position = target_transform.translation
                + target_back * horizontal_distance
                + Vec3::Y * vertical_offset;

            camera_transform.translation = camera_position;

            let look_target = target_transform.translation + Vec3::Y * (vertical_offset * 0.3);
            camera_transform.look_at(look_target, Vec3::Y);
        }
    }
}
