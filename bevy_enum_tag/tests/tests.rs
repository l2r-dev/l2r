#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    #[derive(Component, Default)]
    struct TestComponent;

    #[derive(Component, Default)]
    struct TestComponent2;

    #[derive(EnumComponentTag)]
    enum TestEnum {
        #[require(TestComponent)]
        Variant1,
        #[require(TestComponent, TestComponent2)]
        Variant2,
        Variant3,
    }

    fn spawn_test_enum(mut commands: Commands) {
        commands.spawn(TestEnum::Variant1);
        commands.spawn(TestEnum::Variant2);
    }

    use bevy_enum_tag::EnumComponentTag;
    use test_enum::{Variant1, Variant2};

    fn check_enum_tags(
        query1: Query<&TestEnum, With<Variant1>>,
        query2: Query<&TestEnum, With<Variant2>>,
    ) {
        assert!(!query1.is_empty());
        assert!(!query2.is_empty());
    }

    fn check_test_component_is_with_variant1(
        query: Query<Entity, (With<TestComponent>, With<Variant1>)>,
    ) {
        assert!(!query.is_empty());
    }

    fn remove_variant1(mut commands: Commands, query: Query<Entity, With<Variant1>>) {
        query.iter().for_each(|entity| {
            commands.entity(entity).remove::<TestEnum>();
        });
    }

    fn check_variant1_removed(
        query: Query<Entity, With<Variant1>>,
        query2: Query<Entity, With<TestComponent>>,
    ) {
        assert!(query.is_empty());
        assert!(!query2.is_empty());
    }

    fn remove_variant_2(mut commands: Commands, query: Query<Entity, With<Variant2>>) {
        query.iter().for_each(|entity| {
            commands.entity(entity).remove::<Variant2>();
        })
    }

    fn check_variant2_not_removed(query: Query<&TestEnum>) {
        assert!(!query.is_empty());
    }

    fn insert_tag_should_auto_remove(mut commands: Commands) {
        commands.spawn(TestEnum::Variant3);
    }

    fn check_tag_auto_removed(query: Query<Entity, With<test_enum::Variant3>>) {
        assert!(query.is_empty());
    }

    #[test]
    fn test_enum_tags() {
        let mut app = App::new();
        app.add_systems(
            Update,
            (
                spawn_test_enum,
                check_enum_tags,
                check_test_component_is_with_variant1,
                remove_variant1,
                check_variant1_removed,
                remove_variant_2,
                check_variant2_not_removed,
                insert_tag_should_auto_remove,
                check_tag_auto_removed,
            )
                .chain(),
        );
        app.update();
    }
}
