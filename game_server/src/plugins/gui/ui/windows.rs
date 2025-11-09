use super::{
    camera_controller::CameraControl,
    state::{InspectorSelection, SearchQueries},
};
use crate::plugins::gui::ui::camera_controller::CameraControlSettings;
use bevy::{
    ecs::{query::QueryFilter, system::SystemIdMarker},
    prelude::*,
};
use bevy_asset::ReflectAsset;
use bevy_inspector_egui::bevy_inspector::{
    self, Filter,
    hierarchy::{Hierarchy, SelectedEntities},
    ui_for_entities_shared_components, ui_for_entity_with_children,
};
use bevy_reflect::TypeRegistry;
use l2r_core::plugins::custom_hierarchy::{DespawnChildOf, DespawnChildren};
use spatial::NavigationDirection;
use strum::EnumIter;

#[derive(Clone, Copy, Debug, EnumIter, Eq, Hash, PartialEq)]
pub enum EguiWindow {
    GameView,
    Hierarchy,
    Resources,
    Assets,
    Inspector,
}

pub struct L2rTabViewer<'a> {
    pub world: &'a mut World,
    pub selected_entities: &'a mut SelectedEntities,
    pub selection: &'a mut InspectorSelection,
    pub viewport_rect: &'a mut egui::Rect,
    pub search_queries: &'a mut SearchQueries,
}

#[derive(QueryFilter)]
struct HierarchyFilter {
    no_system_id: Without<SystemIdMarker>,
}

impl egui_dock::TabViewer for L2rTabViewer<'_> {
    type Tab = EguiWindow;

    fn ui(&mut self, ui: &mut egui_dock::egui::Ui, window: &mut Self::Tab) {
        let type_registry = self.world.resource::<AppTypeRegistry>().0.clone();
        let type_registry = type_registry.read();

        match window {
            EguiWindow::GameView => {
                *self.viewport_rect = ui.clip_rect();

                let mut query = self
                    .world
                    .query_filtered::<&Transform, With<CameraControl>>();
                if let Some(transform) = query.iter(self.world).next() {
                    let rotation = transform.rotation;

                    let direction = NavigationDirection::from(rotation);
                    // log::debug!("Direction: {:?}", direction);

                    let compass_text = direction.compass_line();

                    ui.vertical_centered(|ui| {
                        ui.label(compass_text);
                    });
                }

                // Add help label in bottom-right corner
                let rect = ui.available_rect_before_wrap();
                let help_size = egui::Vec2::new(200.0, 120.0);
                let help_pos = egui::Pos2::new(
                    rect.max.x - help_size.x - 10.0, // 10px margin from right edge
                    rect.max.y - help_size.y - 50.0, // 50px margin from bottom edge
                );

                let help_rect = egui::Rect::from_min_size(help_pos, help_size);
                ui.scope_builder(egui::UiBuilder::new().max_rect(help_rect), |ui| {
                    ui.group(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new("🎮 Flight Controls").strong());
                            ui.separator();
                            ui.label("Alt + Click: Enable flight mode");
                            ui.label("Escape: Disable flight mode");
                            ui.separator();
                            ui.label("WASD: Move horizontally");
                            ui.label("Space: Move up");
                            ui.label("Ctrl: Move down");
                            ui.label("Shift: Faster movement");
                        });
                    });
                });
            }
            EguiWindow::Hierarchy => {
                let selected = hierarchy_ui_custom(
                    self.world,
                    ui,
                    self.selected_entities,
                    self.search_queries,
                );
                if selected {
                    *self.selection = InspectorSelection::Entities;
                }
            }
            EguiWindow::Resources => {
                select_resource(ui, &type_registry, self.selection, self.search_queries)
            }
            EguiWindow::Assets => select_asset(
                ui,
                &type_registry,
                self.world,
                self.selection,
                self.search_queries,
            ),
            EguiWindow::Inspector => inspect_selection(
                ui,
                &type_registry,
                self.world,
                self.selection,
                self.selected_entities,
                self.search_queries,
            ),
        }
    }

    fn title(&mut self, window: &mut Self::Tab) -> egui_dock::egui::WidgetText {
        format!("{window:?}").into()
    }

    fn clear_background(&self, window: &Self::Tab) -> bool {
        !matches!(window, EguiWindow::GameView)
    }
}

fn hierarchy_ui_custom(
    world: &mut World,
    ui: &mut egui::Ui,
    selected: &mut SelectedEntities,
    search_queries: &mut SearchQueries,
) -> bool {
    let search_query = search_queries.get_mut(&EguiWindow::Hierarchy).unwrap();

    ui.horizontal(|ui| {
        ui.text_edit_singleline(search_query);
    });

    let mut context_menu =
        |ui: &mut egui::Ui, entity: Entity, world: &mut World, _extra_state: &mut ()| {
            hierarchy_context_menu(ui, entity, world);
        };

    let mut hierarchy = Hierarchy {
        world,
        selected,
        context_menu: Some(&mut context_menu),
        shortcircuit_entity: None,
        extra_state: &mut (),
    };

    let children_getter = |world: &World, entity: Entity| {
        let children: Vec<Entity> = [
            world.get::<DespawnChildren>(entity).map(|dc| dc.iter()),
            world.get::<Children>(entity).map(|c| c.iter()),
        ]
        .into_iter()
        .flatten()
        .flatten()
        .collect();

        (!children.is_empty()).then_some(children)
    };

    let parent_getter = |world: &World, entity: Entity| {
        world
            .get::<ChildOf>(entity)
            .map(|c| c.parent())
            .or_else(|| world.get::<DespawnChildOf>(entity).map(|dc| dc.0))
    };

    if search_query.is_empty() {
        let filter: Filter<(Without<ChildOf>, Without<DespawnChildOf>)> = {
            Filter {
                word: "".into(),
                is_fuzzy: false,
                show_observers: false,
                get_children: Some(children_getter),
                marker: Default::default(),
            }
        };

        hierarchy.show_with_filter_and_getters::<HierarchyFilter, _, _, _>(
            ui,
            filter,
            children_getter,
            parent_getter,
        )
    } else {
        let filter: Filter<()> = {
            Filter {
                word: search_query.clone(),
                is_fuzzy: true,
                show_observers: false,
                get_children: Some(children_getter),
                marker: Default::default(),
            }
        };

        hierarchy.show_with_filter_and_getters::<HierarchyFilter, _, _, _>(
            ui,
            filter,
            children_getter,
            parent_getter,
        )
    }
}

fn select_resource(
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
    selection: &mut InspectorSelection,
    search_queries: &mut SearchQueries,
) {
    let search_query = search_queries.get_mut(&EguiWindow::Resources).unwrap();

    ui.horizontal(|ui| {
        ui.text_edit_singleline(search_query);
    });

    let mut resources: Vec<_> = type_registry
        .iter()
        .filter(|registration| registration.data::<ReflectResource>().is_some())
        .map(|registration| {
            (
                registration.type_info().type_path_table().short_path(),
                registration.type_id(),
            )
        })
        .filter(|(name, _)| {
            search_query.is_empty() || name.to_lowercase().contains(&search_query.to_lowercase())
        })
        .collect();

    resources.sort_by(|(name_a, _), (name_b, _)| name_a.cmp(name_b));

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (resource_name, type_id) in resources {
                let selected = match *selection {
                    InspectorSelection::Resource(selected, _) => selected == type_id,
                    _ => false,
                };

                if ui.selectable_label(selected, resource_name).clicked() {
                    *selection = InspectorSelection::Resource(type_id, resource_name.to_string());
                }
            }
        });
}

fn select_asset(
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
    world: &World,
    selection: &mut InspectorSelection,
    search_queries: &mut SearchQueries,
) {
    let search_query = search_queries.get_mut(&EguiWindow::Assets).unwrap();

    ui.horizontal(|ui| {
        ui.text_edit_singleline(search_query);
    });

    let mut assets: Vec<_> = type_registry
        .iter()
        .filter_map(|registration| {
            let reflect_asset = registration.data::<ReflectAsset>()?;
            Some((
                registration.type_info().type_path_table().short_path(),
                registration.type_id(),
                reflect_asset,
            ))
        })
        .filter(|(name, _, _)| {
            search_query.is_empty() || name.to_lowercase().contains(&search_query.to_lowercase())
        })
        .collect();
    assets.sort_by(|(name_a, ..), (name_b, ..)| name_a.cmp(name_b));

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (asset_name, asset_type_id, reflect_asset) in assets {
                let handles: Vec<_> = reflect_asset.ids(world).collect();

                ui.collapsing(format!("{asset_name} ({})", handles.len()), |ui| {
                    for handle in handles {
                        let selected = match *selection {
                            InspectorSelection::Asset(_, _, selected_id) => selected_id == handle,
                            _ => false,
                        };

                        if ui
                            .selectable_label(selected, format!("{:?}", handle))
                            .clicked()
                        {
                            *selection = InspectorSelection::Asset(
                                asset_type_id,
                                asset_name.to_string(),
                                handle,
                            );
                        }
                    }
                });
            }
        });
}

fn inspect_selection(
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
    world: &mut World,
    selection: &InspectorSelection,
    selected_entities: &SelectedEntities,
    _search_queries: &mut SearchQueries,
) {
    match *selection {
        InspectorSelection::Entities => match selected_entities.as_slice() {
            &[entity] => ui_for_entity_with_children(world, entity, ui),
            entities => ui_for_entities_shared_components(world, entities, ui),
        },
        InspectorSelection::Resource(type_id, ref name) => {
            ui.label(name);
            bevy_inspector::by_type_id::ui_for_resource(world, type_id, ui, name, type_registry)
        }
        InspectorSelection::Asset(type_id, ref name, handle) => {
            ui.label(name);
            bevy_inspector::by_type_id::ui_for_asset(world, type_id, handle, ui, type_registry);
        }
    }
}

fn hierarchy_context_menu(ui: &mut egui::Ui, entity: Entity, world: &mut World) {
    let is_camera = world.get::<CameraControl>(entity).is_some();
    let has_transform = world.get::<Transform>(entity).is_some();

    if is_camera {
        let mut camera_query = world.query::<&mut CameraControl>();
        if ui.button("Reset Attachment").clicked() {
            if let Ok(mut camera_control) = camera_query.get_mut(world, entity) {
                camera_control.attached_to = None;
            }
            ui.close();
        }
    }

    // Non-camera entity with transform - show movement options
    if !is_camera && has_transform {
        if ui.button("Move Camera To").clicked() {
            move_camera_to_entity(world, entity);
            ui.close();
        }

        if ui.button("Attach Camera To").clicked() {
            attach_camera_to_entity(world, entity);
            ui.close();
        }
    }
}

fn move_camera_to_entity(world: &mut World, target_entity: Entity) {
    let target_position = if let Some(target_transform) = world.get::<Transform>(target_entity) {
        target_transform.translation
    } else {
        return;
    };

    let mut camera_query = world.query_filtered::<&mut Transform, With<CameraControl>>();
    let offset = world.resource::<CameraControlSettings>().attachment_offset;
    if let Some(mut camera_transform) = camera_query.iter_mut(world).next() {
        camera_transform.translation = target_position + offset;
        camera_transform.look_at(target_position, Vec3::Y);
    }
}

fn attach_camera_to_entity(world: &mut World, target_entity: Entity) {
    if world.get::<Transform>(target_entity).is_none() {
        warn!(
            "Entity {:?} does not have a Transform component",
            target_entity
        );
        return;
    }

    let mut camera_query = world.query_filtered::<&mut CameraControl, With<CameraControl>>();
    if let Some(mut camera_control) = camera_query.iter_mut(world).next() {
        camera_control.attached_to = Some(target_entity);
    }
}
