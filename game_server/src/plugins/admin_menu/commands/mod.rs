use bevy::prelude::*;
use bevy_ecs::system::SystemParam;
use game_core::{
    account::Account, action::target::SelectedTarget, admin_menu::CommandVariants,
    custom_hierarchy::DespawnChildOf, object_id::ObjectId,
};
use sea_orm::Iterable;

mod add_skill;

mod heal;
mod kill;
mod main;
mod multisell;
mod pause;
mod resurrect;
mod set_level;
mod spawn_item;
mod spawn_npc;
mod teleport;

#[derive(SystemParam)]
pub struct AdminCommandQuery<'w, 's> {
    pub accounts: Query<'w, 's, Ref<'static, Account>>,
    pub characters: Query<
        'w,
        's,
        (
            Ref<'static, DespawnChildOf>,
            Ref<'static, ObjectId>,
            Option<Ref<'static, SelectedTarget>>,
        ),
    >,
}

impl<'w, 's> AdminCommandQuery<'w, 's> {
    /// Validates that the entity is a GM and returns the character data
    pub fn validate_gm(&self, entity: Entity) -> Result<(ObjectId, Entity)> {
        let (_, object_id, selected_target) = self
            .characters
            .get(entity)
            .map_err(|_| BevyError::from(format!("Character not found for entity {:?}", entity)))?;
        let account = self.account(entity)?;
        if !account.access().gm() {
            return Err(BevyError::from(format!(
                "Account {:?} is not GM",
                account.name()
            )));
        }
        let target = selected_target.map(|target| **target).unwrap_or(entity);
        Ok((*object_id, target))
    }
    pub fn account(&self, entity: Entity) -> Result<Ref<'_, Account>> {
        let (despawn_child_of, _, _) = self.characters.get(entity)?;
        Ok(self.accounts.get(**despawn_child_of)?)
    }
}

pub struct AdminMenuCommandsPlugin;
impl Plugin for AdminMenuCommandsPlugin {
    fn build(&self, app: &mut App) {
        for command in CommandVariants::iter() {
            match command {
                CommandVariants::Main => app.add_observer(main::handle),
                CommandVariants::Heal => app.add_observer(heal::handle),
                CommandVariants::Resurrect => app.add_observer(resurrect::handle),
                CommandVariants::SpawnItem => app.add_observer(spawn_item::handle),
                CommandVariants::SpawnNpc => app.add_observer(spawn_npc::handle),
                CommandVariants::Kill => app.add_observer(kill::handle),
                CommandVariants::Pause => app.add_observer(pause::handle),
                CommandVariants::Tp => app.add_observer(teleport::handle),
                CommandVariants::TpList => app.add_observer(teleport::handle_list),
                CommandVariants::MultiSell => app.add_observer(multisell::handle),
                CommandVariants::MultiSellList => app.add_observer(multisell::handle_list),
                CommandVariants::AddSkill => app.add_observer(add_skill::handle),
                CommandVariants::SetLevel => app.add_observer(set_level::handle),
            };
        }
    }
}
