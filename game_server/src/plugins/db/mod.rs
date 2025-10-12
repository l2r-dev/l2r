use ::scripting::{
    bindings::{InteropError, ReflectReference, WorldAccessGuard},
    prelude::ScriptValue,
};
use bevy::prelude::*;
use game_core::{
    character::{
        self, CharacterRepository,
        skills::{CharacterSkillsRepository, SkillPK},
    },
    items::{self, ItemsRepository},
    object_id::ObjectId,
    shortcut::{
        self,
        model::{CharacterShortcutsRepository, ShortcutPK},
    },
    skills,
    stats::SubClassVariant,
};
use l2r_core::db::{
    DbConnection, DbRepositoryPlugin, PostgresPlugin, RedisConfig, RedisPlugin, RepositoryManager,
};
use sea_orm::ConnectOptions;
use state::GameServerStateSystems;
use std::time::Duration;
use strum::{AsRefStr, Display, EnumDiscriminants, EnumString};

pub mod migrations;

mod scripting;

pub struct GameDbPlugin;
impl GameDbPlugin {
    fn pause_if_no_postgres_connection(
        connection: Res<DbConnection>,
        state: Res<State<GameServerStateSystems>>,
        mut next_state: ResMut<NextState<GameServerStateSystems>>,
    ) {
        if connection.disconnected() {
            if *state.get() == GameServerStateSystems::Run {
                next_state.set(GameServerStateSystems::Pause);
            }
        } else if *state.get() == GameServerStateSystems::Pause {
            next_state.set(GameServerStateSystems::Run);
        }
    }
}
impl Plugin for GameDbPlugin {
    fn build(&self, app: &mut App) {
        let (database_url, redis_url) = {
            let config = app.world().resource::<config::Config>();
            (
                config.general().database_url.0.clone(),
                config.general().redis_url.0.clone(),
            )
        };

        app.add_plugins(RedisPlugin::new(RedisConfig {
            redis_url,
            timeout: Duration::from_secs(5),
        }));

        let mut connect_options = ConnectOptions::new(database_url);
        connect_options
            .min_connections(1)
            .max_connections(10)
            .connect_timeout(std::time::Duration::from_secs(5));

        app.add_plugins(PostgresPlugin {
            connection: None,
            config: Some(connect_options.into()),
        });
        app.add_plugins(DbGameRepositoryPlugin);

        app.add_plugins(migrations::GameServerMigrationPlugin);

        app.add_systems(Update, Self::pause_if_no_postgres_connection);

        app.add_plugins(scripting::DatabaseScriptingPlugin);
    }
}

#[derive(Clone, Copy, Debug, EnumDiscriminants)]
#[strum_discriminants(name(GameRepoName))]
#[strum_discriminants(derive(AsRefStr, Display, EnumString, Hash))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
pub enum GameRepoKey {
    Character(ObjectId),
    CharacterSkills(SkillPK),
    CharacterShortcuts(ShortcutPK),
    Items(ObjectId),
}

#[derive(Clone)]
pub enum GameRepoModel {
    Character(character::model::Model),
    CharacterSkills(character::skills::Model),
    CharacterShortcuts(shortcut::model::Model),
    Items(items::model::Model),
}

impl From<&GameRepoModel> for GameRepoName {
    fn from(model: &GameRepoModel) -> Self {
        match model {
            GameRepoModel::Character(_) => GameRepoName::Character,
            GameRepoModel::CharacterSkills(_) => GameRepoName::CharacterSkills,
            GameRepoModel::CharacterShortcuts(_) => GameRepoName::CharacterShortcuts,
            GameRepoModel::Items(_) => GameRepoName::Items,
        }
    }
}

impl From<GameRepoModel> for GameRepoName {
    fn from(model: GameRepoModel) -> Self {
        (&model).into()
    }
}

impl<'a> TryFrom<(&ReflectReference, WorldAccessGuard<'a>)> for GameRepoModel {
    type Error = InteropError;

    fn try_from(
        (model_ref, world_guard): (&ReflectReference, WorldAccessGuard<'a>),
    ) -> Result<Self, Self::Error> {
        // Try to downcast to each supported model type
        if let Ok(model) = model_ref.downcast::<character::model::Model>(world_guard.clone()) {
            Ok(GameRepoModel::Character(model))
        } else if let Ok(model) =
            model_ref.downcast::<character::skills::Model>(world_guard.clone())
        {
            Ok(GameRepoModel::CharacterSkills(model))
        } else if let Ok(model) = model_ref.downcast::<items::model::Model>(world_guard.clone()) {
            Ok(GameRepoModel::Items(model))
        } else if let Ok(model) =
            model_ref.downcast::<game_core::shortcut::model::Model>(world_guard.clone())
        {
            Ok(GameRepoModel::CharacterShortcuts(model))
        } else {
            Err(InteropError::string_type_mismatch(
                "one of: Character, CharacterSkills, Items, CharacterShortcuts".to_string(),
                None,
            )
            .with_context("Failed to downcast model to any known repository type"))
        }
    }
}

impl TryFrom<(&GameRepoName, &ScriptValue)> for GameRepoKey {
    type Error = InteropError;

    fn try_from(
        (repo_name, key_value): (&GameRepoName, &ScriptValue),
    ) -> Result<Self, Self::Error> {
        match repo_name {
            GameRepoName::Character => {
                let object_id = ObjectId::try_from(key_value).map_err(|_| {
                    InteropError::value_mismatch(
                        std::any::TypeId::of::<ObjectId>(),
                        key_value.clone(),
                    )
                })?;
                Ok(GameRepoKey::Character(object_id))
            }
            GameRepoName::CharacterSkills => {
                // For CharacterSkills, we expect a list with [char_id, skill_id, sub_class]
                match key_value {
                    ScriptValue::List(list) if list.len() == 3 => {
                        let char_id = ObjectId::try_from(&list[0]).map_err(|_| {
                            InteropError::value_mismatch(
                                std::any::TypeId::of::<ObjectId>(),
                                list[0].clone(),
                            )
                            .with_context("character ID in SkillPK")
                        })?;

                        let skill_id = match &list[1] {
                            ScriptValue::Integer(id) => skills::Id::from(*id as u32),
                            other => {
                                return Err(InteropError::value_mismatch(
                                    std::any::TypeId::of::<i64>(),
                                    other.clone(),
                                )
                                .with_context("skill_id in SkillPK"));
                            }
                        };

                        let sub_class = match &list[2] {
                            ScriptValue::Integer(class) => SubClassVariant::try_from(*class as i16)
                                .map_err(|e| {
                                    InteropError::external(e)
                                        .with_context("Invalid SubClass value in SkillPK")
                                })?,
                            other => {
                                return Err(InteropError::value_mismatch(
                                    std::any::TypeId::of::<i64>(),
                                    other.clone(),
                                )
                                .with_context("sub_class in SkillPK"));
                            }
                        };

                        Ok(GameRepoKey::CharacterSkills(SkillPK {
                            char_id,
                            skill_id,
                            sub_class,
                        }))
                    }
                    ScriptValue::List(list) => Err(InteropError::length_mismatch(3, list.len())
                        .with_context(
                            "CharacterSkills requires a list of [char_id, skill_id, sub_class]",
                        )),
                    _ => Err(InteropError::string_type_mismatch(
                        "List[ObjectId, Integer, Integer]".to_string(),
                        None,
                    )
                    .with_context("CharacterSkills key")),
                }
            }
            GameRepoName::CharacterShortcuts => {
                // For CharacterShortcuts, we expect a list with [char_id, slot_id, class_variant]
                match key_value {
                    ScriptValue::List(list) if list.len() == 3 => {
                        let char_id = ObjectId::try_from(&list[0]).map_err(|_| {
                            InteropError::value_mismatch(
                                std::any::TypeId::of::<ObjectId>(),
                                list[0].clone(),
                            ).with_context("character ID in ShortcutPK")
                        })?;

                        let slot_id = match &list[1] {
                            ScriptValue::Integer(id) => {
                                game_core::shortcut::SlotId::from(*id as u32)
                            }
                            other => {
                                return Err(InteropError::value_mismatch(
                                    std::any::TypeId::of::<i64>(),
                                    other.clone(),
                                ).with_context("slot_id in ShortcutPK"));
                            }
                        };

                        let class_variant = match &list[2] {
                            ScriptValue::Integer(class) => SubClassVariant::try_from(*class as i16)
                                .map_err(|e| {
                                    InteropError::external(e)
                                        .with_context("Invalid SubClass value in ShortcutPK")
                                })?,
                            other => {
                                return Err(InteropError::value_mismatch(
                                    std::any::TypeId::of::<i64>(),
                                    other.clone(),
                                ).with_context("class_variant in ShortcutPK"));
                            }
                        };

                        Ok(GameRepoKey::CharacterShortcuts(ShortcutPK {
                            char_id,
                            slot_id,
                            class_variant,
                        }))
                    }
                    ScriptValue::List(list) => Err(InteropError::length_mismatch(3, list.len())
                        .with_context("CharacterShortcuts requires a list of [char_id, slot_id, class_variant]")),
                    _ => Err(InteropError::string_type_mismatch(
                        "List[ObjectId, Integer, Integer]".to_string(),
                        None,
                    ).with_context("CharacterShortcuts key")),
                }
            }
            GameRepoName::Items => {
                let object_id = ObjectId::try_from(key_value).map_err(|_| {
                    InteropError::value_mismatch(
                        std::any::TypeId::of::<ObjectId>(),
                        key_value.clone(),
                    )
                    .with_context("Items key")
                })?;
                Ok(GameRepoKey::Items(object_id))
            }
        }
    }
}

pub struct DbGameRepositoryPlugin;
impl Plugin for DbGameRepositoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DbRepositoryPlugin);

        app.world_mut()
            .resource_mut::<RepositoryManager>()
            .register(CharacterRepository::new(GameRepoName::Character.as_ref()))
            .register(CharacterSkillsRepository::new(
                GameRepoName::CharacterSkills.as_ref(),
            ))
            .register(ItemsRepository::new(GameRepoName::Items.as_ref()))
            .register(CharacterShortcutsRepository::new(
                GameRepoName::CharacterShortcuts.as_ref(),
            ));
    }
}
