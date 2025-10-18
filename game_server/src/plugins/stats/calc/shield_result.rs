use bevy::prelude::*;
use game_core::{
    items::{self, PaperDoll, WeaponKind},
    network::packets::server::{GameServerPacket, SystemMessage},
    stats::*,
};
use scripting::{
    bindings::{FunctionCallContext, InteropError},
    prelude::{NamespaceBuilder, ScriptValue},
};
use spatial::{Degrees, DirectionDegrees};
use system_messages::Id;

pub struct ShieldResultPlugin;
impl Plugin for ShieldResultPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ShieldResult>();
        NamespaceBuilder::<ShieldResult>::new(app.world_mut())
            .register("calculate", script_calculate);
    }
}

pub fn calculate_shield_result(
    attacker_entity: EntityRef,
    target_entity: EntityRef,
    world: &World,
) -> ShieldResult {
    calculate_shield_result_inner(attacker_entity, target_entity, world)
        .unwrap_or(ShieldResult::Failed)
}

#[inline]
fn calculate_shield_result_inner(
    attacker_entity: EntityRef,
    target_entity: EntityRef,
    world: &World,
) -> Option<ShieldResult> {
    let target_paper_doll = target_entity.get::<PaperDoll>()?;
    let items_data_table = world.get_resource::<items::ItemsDataTable>()?;
    let items_data_assets = world.get_resource::<Assets<items::ItemsInfo>>()?;
    let unique_item = target_paper_doll.get(items::DollSlot::LeftHand)?;
    let item_info = items_data_table
        .get_item_info(unique_item.item().id(), items_data_assets)
        .ok()?;

    if !item_info.kind().shield() {
        return None;
    }

    let target_defence_stats = target_entity.get::<DefenceStats>()?;

    let shield_angle = target_defence_stats.get(DefenceStat::ShieldAngle);
    let shield_rate = target_defence_stats.get(DefenceStat::ShieldRate);

    if shield_angle <= 0.0 || shield_rate <= 0.0 {
        return None;
    }

    let target_transform = target_entity.get::<Transform>()?;

    let attacker_transform = attacker_entity.get::<Transform>()?;

    let is_within_angle = check_shield_angle(attacker_transform, target_transform, shield_angle);

    if !is_within_angle {
        return None;
    }

    let adjusted_shield_rate = calculate_adjusted_shield_rate(shield_rate, item_info.kind());

    // TODO: Make this 10 (perfect block chance) configurable via Res<Config>
    let result = calculate_block_result(adjusted_shield_rate, 10);

    Some(result)
}

fn check_shield_angle(
    attacker_transform: &Transform,
    target_transform: &Transform,
    shield_angle: f32,
) -> bool {
    let target_degrees = *Degrees::from(target_transform.rotation);
    let direction_to_target = attacker_transform
        .translation
        .direction_degrees(&target_transform.translation);

    let mut angle_diff = (direction_to_target - target_degrees).abs() % 360.0;
    if angle_diff > 180.0 {
        angle_diff = 360.0 - angle_diff;
    }

    angle_diff <= (shield_angle / 2.0)
}

#[inline]
fn calculate_adjusted_shield_rate(base_rate: f32, attacker_weapon_kind: &items::Kind) -> f32 {
    let bonus_rate_weapon_types = [WeaponKind::Bow, WeaponKind::Crossbow];

    if let items::Kind::Weapon(weapon) = attacker_weapon_kind
        && bonus_rate_weapon_types.contains(&weapon.kind)
    {
        base_rate * 1.3
    } else {
        base_rate
    }
}

#[inline]
fn calculate_block_result(shield_rate: f32, perfect_block_chance: u8) -> ShieldResult {
    use rand::Rng;
    let roll = rand::thread_rng().gen_range(0..100);

    if roll < perfect_block_chance {
        ShieldResult::PerfectBlock
    } else if roll < shield_rate as u8 {
        ShieldResult::Succeed
    } else {
        ShieldResult::Failed
    }
}

pub fn send_shield_result_system_message(
    result: ShieldResult,
    entity: Entity,
    mut commands: Commands,
) {
    let message = match result {
        ShieldResult::Succeed => SystemMessage::new_empty(Id::YourShieldDefenceHasSucceeded),
        ShieldResult::PerfectBlock => {
            SystemMessage::new_empty(Id::YourExcellentShieldDefenceWasASuccess)
        }
        _ => return,
    };
    commands.trigger_targets(GameServerPacket::from(message), entity);
}

fn script_calculate(
    ctx: FunctionCallContext,
    script_value: ScriptValue,
) -> Result<ScriptValue, InteropError> {
    let world_guard = ctx.world()?;

    let (attacker_entity, target_entity) = match script_value {
        // it must be a list of two ReflectReferences
        ScriptValue::List(list) if list.len() == 2 => {
            let attacker_entity = list
                .first()
                .ok_or_else(|| InteropError::argument_count_mismatch(2, 1))?;
            let target_entity = list
                .get(1)
                .ok_or_else(|| InteropError::argument_count_mismatch(2, 1))?;
            if let (
                ScriptValue::Reference(attacker_entity),
                ScriptValue::Reference(target_entity),
            ) = (attacker_entity, target_entity)
            {
                let attacker_entity = attacker_entity.downcast::<Entity>(world_guard.clone())?;
                let target_entity = target_entity.downcast::<Entity>(world_guard.clone())?;
                (attacker_entity, target_entity)
            } else {
                return Err(InteropError::type_mismatch(
                    std::any::TypeId::of::<Entity>(),
                    None,
                ));
            }
        }
        _ => return Err(InteropError::argument_count_mismatch(2, 1)),
    };

    world_guard.with_global_access(|world| {
        let attacker_entity = world.get_entity(attacker_entity);
        let target_entity = world.get_entity(target_entity);
        if let (Ok(attacker_entity), Ok(target_entity)) = (attacker_entity, target_entity) {
            let shield_result = calculate_shield_result(attacker_entity, target_entity, world);
            let target_id = target_entity.id();
            let mut commands = world.commands();
            send_shield_result_system_message(shield_result, target_id, commands.reborrow());

            ScriptValue::Integer(shield_result.into())
        } else {
            ScriptValue::Integer(0)
        }
    })
}
