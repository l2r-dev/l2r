use bevy::prelude::*;
use bevy_ecs::system::{SystemParam, SystemState};
use game_core::{
    items::{self, ItemsDataQuery, PaperDoll, WeaponKind},
    network::packets::server::{GameServerPacket, SystemMessage},
    stats::*,
};
use scripting::{
    bindings::{FunctionCallContext, InteropError},
    prelude::{NamespaceBuilder, ScriptValue},
};
use spatial::TransformRelativeDirection;

pub struct ShieldResultPlugin;
impl Plugin for ShieldResultPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ShieldResult>();
        NamespaceBuilder::<ShieldResult>::new(app.world_mut())
            .register("calculate", script_calculate);
    }
}

#[derive(SystemParam)]
pub struct CalcShieldQuery<'w, 's> {
    pub transforms: Query<'w, 's, Ref<'static, Transform>>,
    pub paper_dolls: Query<'w, 's, Ref<'static, PaperDoll>>,
    pub defence_stats: Query<'w, 's, Ref<'static, DefenceStats>>,
    pub items_data_query: ItemsDataQuery<'w>,
}

pub fn calculate_shield_result(
    attacker: Entity,
    target: Entity,
    query: &CalcShieldQuery,
) -> ShieldResult {
    calculate_shield_result_inner(attacker, target, query).unwrap_or(ShieldResult::Failed)
}

#[inline]
fn calculate_shield_result_inner(
    attacker: Entity,
    target: Entity,
    query: &CalcShieldQuery,
) -> Option<ShieldResult> {
    let target_paper_doll = query.paper_dolls.get(target).ok()?;
    let unique_item = target_paper_doll.get(items::DollSlot::LeftHand)?;
    let item_info = query
        .items_data_query
        .get_item_info(unique_item.item().id())
        .ok()?;

    if !item_info.kind().shield() {
        return None;
    }

    let target_defence_stats = query.defence_stats.get(target).ok()?;

    let shield_angle = target_defence_stats.get(DefenceStat::ShieldAngle);
    let shield_rate = target_defence_stats.get(DefenceStat::ShieldRate);

    if shield_angle <= 0.0 || shield_rate <= 0.0 {
        return None;
    }

    let target_transform = query.transforms.get(target).ok()?;

    let attacker_transform = query.transforms.get(attacker).ok()?;

    let is_within_angle =
        attacker_transform.is_within_angle_relative_to(target_transform.as_ref(), shield_angle);

    if !is_within_angle {
        return None;
    }

    let adjusted_shield_rate = calculate_adjusted_shield_rate(shield_rate, item_info.kind());

    // TODO: Make this 10 (perfect block chance) configurable via Res<Config>
    let result = calculate_block_result(adjusted_shield_rate, 10);

    Some(result)
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
    attacker_entity: Entity,
    target_entity: Entity,
    mut commands: Commands,
) {
    match result {
        ShieldResult::Succeed => {
            commands.trigger_targets(
                GameServerPacket::from(SystemMessage::new_empty(
                    system_messages::Id::YourShieldDefenceHasSucceeded,
                )),
                target_entity,
            );
            commands.trigger_targets(
                GameServerPacket::from(SystemMessage::new_empty(
                    system_messages::Id::TheAttackHasBeenBlocked,
                )),
                attacker_entity,
            );
        }

        ShieldResult::PerfectBlock => {
            commands.trigger_targets(
                GameServerPacket::from(SystemMessage::new_empty(
                    system_messages::Id::YourExcellentShieldDefenceWasASuccess,
                )),
                target_entity,
            );
            commands.trigger_targets(
                GameServerPacket::from(SystemMessage::new_empty(
                    system_messages::Id::TheAttackHasBeenBlocked,
                )),
                attacker_entity,
            );
        }
        _ => (),
    }
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
        let mut shield_state: SystemState<CalcShieldQuery> = SystemState::new(world);
        let shield_query = shield_state.get(world);

        let shield_result = calculate_shield_result(attacker_entity, target_entity, &shield_query);
        let mut commands = world.commands();
        send_shield_result_system_message(
            shield_result,
            attacker_entity,
            target_entity,
            commands.reborrow(),
        );

        ScriptValue::Integer(shield_result.into())
    })
}
