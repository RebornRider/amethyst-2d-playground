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
}
