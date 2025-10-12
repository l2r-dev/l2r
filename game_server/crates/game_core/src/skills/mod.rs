use bevy::prelude::*;

mod id;
mod kind;
mod level;
mod list;
mod reuse_timer_system;
mod reuse_timers;
mod skill;
mod tree;

pub use id::*;
pub use kind::*;
pub use level::*;
pub use list::*;
pub use reuse_timer_system::*;
pub use reuse_timers::*;
pub use skill::*;
pub use tree::*;

pub struct SkillsComponentsPlugin;
impl Plugin for SkillsComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Level>()
            .register_type::<Kind>()
            .register_type::<Skill>()
            .register_type::<SkillList>()
            .register_type::<SkillTreesHandlers>()
            .register_type::<SkillReuseTimers>();

        l2r_core::register_optional_types!(app, Id);
    }
}
