pub mod model;

use bevy::prelude::*;
use l2r_core::db::RepositoryManager;

pub struct AccountsPlugin;

impl Plugin for AccountsPlugin {
    fn build(&self, app: &mut App) {
        let repo = model::AccountsRepository::new("accounts");

        app.world_mut()
            .resource_mut::<RepositoryManager>()
            .register(repo);
    }
}
