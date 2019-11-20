use crate::{
    audio::{play_bounce, Sounds},
    Ball, Paddle, Side,
};
use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source},
    core::{transform::Transform, SystemDesc},
    derive::SystemDesc,
    ecs::prelude::{Join, Read, ReadExpect, ReadStorage, System, SystemData, World, WriteStorage},
};

/// This system is responsible for detecting collisions between balls and
/// paddles, as well as balls and the top and bottom edges of the arena.
#[derive(SystemDesc)]
pub struct BounceSystem;

impl<'s> System<'s> for BounceSystem {
    type SystemData = (
        WriteStorage<'s, Ball>,
        ReadStorage<'s, Paddle>,
        ReadStorage<'s, Transform>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        Option<Read<'s, Output>>,
    );

    fn run(&mut self, (mut balls, paddles, transforms, storage, sounds, audio_output): Self::SystemData) {
        // Check whether a ball collided, and bounce off accordingly.
        //
        // We also check for the velocity of the ball every time, to prevent multiple collisions
        // from occurring.
        for (ball, transform) in (&mut balls, &transforms).join() {
            use crate::ARENA_HEIGHT;

            let ball_x = transform.translation().x;
            let ball_y = transform.translation().y;

            // Bounce at the top or the bottom of the arena.
            if (ball_y <= ball.radius && ball.velocity[1] < 0.0) || (ball_y >= ARENA_HEIGHT - ball.radius && ball.velocity[1] > 0.0) {
                ball.velocity[1] = -ball.velocity[1];
                play_bounce(&*sounds, &storage, audio_output.as_ref().map(std::ops::Deref::deref));
            }

            // Bounce at the paddles.
            for (paddle, paddle_transform) in (&paddles, &transforms).join() {
                let paddle_x = paddle_transform.translation().x - (paddle.width * 0.5);
                let paddle_y = paddle_transform.translation().y - (paddle.height * 0.5);

                // To determine whether the ball has collided with a paddle, we create a larger
                // rectangle around the current one, by subtracting the ball radius from the
                // lowest coordinates, and adding the ball radius to the highest ones. The ball
                // is then within the paddle if its centre is within the larger wrapper
                // rectangle.
                if point_in_rect(
                    ball_x,
                    ball_y,
                    paddle_x - ball.radius,
                    paddle_y - ball.radius,
                    paddle_x + (paddle.width + ball.radius),
                    paddle_y + (paddle.height + ball.radius),
                ) && ((paddle.side == Side::Left && ball.velocity[0] < 0.0) || (paddle.side == Side::Right && ball.velocity[0] > 0.0))
                {
                    ball.velocity[0] = -ball.velocity[0];
                    play_bounce(&*sounds, &storage, audio_output.as_ref().map(std::ops::Deref::deref));
                }
            }
        }
    }
}

// A point is in a box when its coordinates are smaller or equal than the top
// right and larger or equal than the bottom left.
fn point_in_rect(x: f32, y: f32, left: f32, bottom: f32, right: f32, top: f32) -> bool {
    x >= left && x <= right && y >= bottom && y <= top
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        audio::initialise_audio,
        setup_loader_for_test,
        states::{initialise_ball, initialise_paddles, load_sprite_sheet},
    };
    use amethyst::{
        assets::AssetStorage,
        audio::AudioBundle,
        core::{Parent, TransformBundle},
        ecs::prelude::WorldExt,
        prelude::Builder,
        renderer::{SpriteRender, SpriteSheet, Texture},
    };
    use amethyst_test::AmethystApplication;
    use test_case::test_case;

    #[test_case(0.0, 0.0, 0.0, 0.0, 0.0, 0.0 => true)]
    #[test_case(0.0, 0.0, -1.0, -1.0, 1.0, 1.0 => true)]
    #[test_case(1.0, 0.0, -1.0, -1.0, 1.0, 1.0 => true)]
    #[test_case(0.0, 1.0, -1.0, -1.0, 1.0, 1.0 => true)]
    #[test_case(-10.0, 0.0, -10.0, -10.0, 10.0, 10.0 => true)]
    #[test_case(0.0, -10.0, -10.0, -10.0, 10.0, 10.0 => true)]
    #[test_case(2.0, 0.0, -1.0, -1.0, 1.0, 1.0 => false)]
    #[test_case(0.0, 2.0, -1.0, -1.0, 1.0, 1.0 => false)]
    #[test_case(-20.0, 0.0, -10.0, -10.0, 10.0, 10.0 => false)]
    #[test_case(0.0, -20.0, -10.0, -10.0, 10.0, 10.0 => false)]
    fn test_point_in_rect(x: f32, y: f32, left: f32, bottom: f32, right: f32, top: f32) -> bool {
        point_in_rect(x, y, left, bottom, right, top)
    }

    #[test]
    fn basic_bounce_system_setup() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = AmethystApplication::blank()
            .with_bundle(TransformBundle::new())
            .with_bundle(AudioBundle::default())
            .with_setup(|world| {
                setup_loader_for_test(world);
                world.insert(AssetStorage::<Source>::default());
                initialise_audio(world);

                let tex_storage = AssetStorage::<Texture>::default();
                let ss_storage = AssetStorage::<SpriteSheet>::default();
                world.insert(tex_storage);
                world.insert(ss_storage);
                world.register::<Transform>();
                world.register::<Parent>();
                world.register::<SpriteRender>();
                world.register::<Paddle>();
                world.register::<Ball>();

                // Initialize paddles and ball
                let root_entity = Some(world.create_entity().with(Transform::default()).build());
                let sprite_sheet_handle = Some(load_sprite_sheet(world));
                if let Some(root_entity) = root_entity {
                    if let Some(sprite_sheet) = sprite_sheet_handle.clone() {
                        initialise_paddles(world, root_entity, sprite_sheet);
                    }
                    if let Some(sprite_sheet) = sprite_sheet_handle.clone() {
                        use crate::{BALL_RADIUS, BALL_VELOCITY_X, BALL_VELOCITY_Y};
                        initialise_ball(world, root_entity, sprite_sheet, BALL_RADIUS, [BALL_VELOCITY_X, BALL_VELOCITY_Y], None);
                    }
                }
            })
            .with_system_single(BounceSystem, "collision_system", &[])
            .run();
        assert!(test_result.is_ok());
    }

    #[test]
    fn paddle_reflect() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = AmethystApplication::blank()
            .with_bundle(TransformBundle::new())
            .with_bundle(AudioBundle::default())
            .with_setup(|world| {
                setup_loader_for_test(world);
                world.insert(AssetStorage::<Source>::default());
                initialise_audio(world);

                let tex_storage = AssetStorage::<Texture>::default();
                let ss_storage = AssetStorage::<SpriteSheet>::default();
                world.insert(tex_storage);
                world.insert(ss_storage);
                world.register::<Transform>();
                world.register::<Parent>();
                world.register::<SpriteRender>();
                world.register::<Paddle>();
                world.register::<Ball>();

                // Initialize paddles and ball
                let root_entity = Some(world.create_entity().with(Transform::default()).build());
                let sprite_sheet_handle = Some(load_sprite_sheet(world));
                if let Some(root_entity) = root_entity {
                    if let Some(sprite_sheet) = sprite_sheet_handle.clone() {
                        initialise_paddles(world, root_entity, sprite_sheet);
                    }
                    if let Some(sprite_sheet) = sprite_sheet_handle.clone() {
                        use crate::{ARENA_HEIGHT, BALL_RADIUS, BALL_VELOCITY_Y};
                        initialise_ball(world, root_entity, sprite_sheet, BALL_RADIUS, [-1.0, BALL_VELOCITY_Y], Some([0.0, ARENA_HEIGHT / 2.0]));
                    }
                }
            })
            .with_system_single(BounceSystem, "collision_system", &[])
            .with_assertion(|world| {
                let balls = world.read_storage::<Ball>();
                let transforms = world.read_storage::<Transform>();
                assert_eq!(1, balls.count());

                for (ball, _transform) in (&balls, &transforms).join() {
                    assert_eq!(ball.velocity[0], 1.0);
                }
            })
            .run();
        assert!(test_result.is_ok());
    }

    #[test]
    fn no_paddle_reflect_if_going_into_direction_of_paddle() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = AmethystApplication::blank()
            .with_bundle(TransformBundle::new())
            .with_bundle(AudioBundle::default())
            .with_setup(|world| {
                setup_loader_for_test(world);
                world.insert(AssetStorage::<Source>::default());
                initialise_audio(world);

                let tex_storage = AssetStorage::<Texture>::default();
                let ss_storage = AssetStorage::<SpriteSheet>::default();
                world.insert(tex_storage);
                world.insert(ss_storage);
                world.register::<Transform>();
                world.register::<Parent>();
                world.register::<SpriteRender>();
                world.register::<Paddle>();
                world.register::<Ball>();

                // Initialize paddles and ball
                let root_entity = Some(world.create_entity().with(Transform::default()).build());
                let sprite_sheet_handle = Some(load_sprite_sheet(world));
                if let Some(root_entity) = root_entity {
                    if let Some(sprite_sheet) = sprite_sheet_handle.clone() {
                        initialise_paddles(world, root_entity, sprite_sheet);
                    }
                    if let Some(sprite_sheet) = sprite_sheet_handle.clone() {
                        use crate::{ARENA_HEIGHT, BALL_RADIUS, BALL_VELOCITY_Y};
                        initialise_ball(world, root_entity, sprite_sheet, BALL_RADIUS, [1.0, BALL_VELOCITY_Y], Some([0.0, ARENA_HEIGHT / 2.0]));
                    }
                }
            })
            .with_system_single(BounceSystem, "collision_system", &[])
            .with_assertion(|world| {
                let balls = world.read_storage::<Ball>();
                let transforms = world.read_storage::<Transform>();
                assert_eq!(1, balls.count());

                for (ball, _transform) in (&balls, &transforms).join() {
                    assert_eq!(ball.velocity[0], 1.0);
                }
            })
            .run();
        assert!(test_result.is_ok());
    }
}
