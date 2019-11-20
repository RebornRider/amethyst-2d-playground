use amethyst::{
    core::transform::ParentHierarchy,
    ecs::{
        error::WrongGeneration,
        prelude::{Entity, World, WorldExt},
    },
};

use std::iter;

/// delete the specified root entity and all of its descendents as specified
/// by the Parent component and maintained by the `ParentHierarchy` resource
// from https://github.com/amethyst/evoli src/utils/hierarchy_util.rs
pub fn delete_hierarchy(root: Entity, world: &mut World) -> Result<(), WrongGeneration> {
    let entities = {
        iter::once(root)
            .chain(world.read_resource::<ParentHierarchy>().all_children_iter(root))
            .collect::<Vec<Entity>>()
    };
    world.delete_entities(&entities)
}

#[cfg(test)]
mod tests {
    use super::*;
    use amethyst::core::transform::TransformBundle;
    use amethyst::core::Parent;
    use amethyst::prelude::Builder;
    use amethyst_test::AmethystApplication;

    #[test]
    fn test_delete_single_entity() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = AmethystApplication::blank()
            .with_bundle(TransformBundle::new())
            .with_assertion(|world| {
                let entity = world.create_entity().build();
                assert!(world.is_alive(entity));

                let result = delete_hierarchy(entity, world);

                assert!(result.is_ok());
                assert_eq!(world.is_alive(entity), false);
            })
            .run();
        assert!(test_result.is_ok());
    }

    #[test]
    fn test_delete_two_unrelated_entities() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = AmethystApplication::blank()
            .with_bundle(TransformBundle::new())
            .with_assertion(|world| {
                let entity1 = world.create_entity().build();
                assert!(world.is_alive(entity1));

                let entity2 = world.create_entity().build();
                assert!(world.is_alive(entity2));

                let result = delete_hierarchy(entity1, world);

                assert!(result.is_ok());
                assert_eq!(world.is_alive(entity1), false);
                assert_eq!(world.is_alive(entity2), true);
            })
            .run();
        assert!(test_result.is_ok());
    }

    #[test]
    fn test_delete_two_related_entities_deleting_child() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = AmethystApplication::blank()
            .with_bundle(TransformBundle::new())
            .with_assertion(|world| {
                let parent = world.create_entity().build();
                assert!(world.is_alive(parent));

                let child = world.create_entity().with(Parent { entity: parent }).build();
                assert!(world.is_alive(child));

                let result = delete_hierarchy(child, world);

                assert!(result.is_ok());
                assert_eq!(world.is_alive(parent), true);
                assert_eq!(world.is_alive(child), false);
            })
            .run();
        assert!(test_result.is_ok());
    }
}
