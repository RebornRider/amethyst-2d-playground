use crate::{audio::Sounds, Ball, ScoreBoard};
use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source},
    core::{transform::Transform, SystemDesc},
    derive::SystemDesc,
    ecs::prelude::{Entity, Join, Read, ReadExpect, System, SystemData, Write, WriteStorage},
    ui::UiText,
    utils::fps_counter::FpsCounter,
};
/// This system is responsible for checking if a ball has moved into a left or
/// a right edge. Points are distributed to the player on the other side, and
/// the ball is reset.
#[derive(SystemDesc)]
pub struct WinnerSystem;

impl<'s> System<'s> for WinnerSystem {
    type SystemData = (
        WriteStorage<'s, Ball>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, UiText>,
        Write<'s, ScoreBoard>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        ReadExpect<'s, ScoreText>,
        Option<Read<'s, Output>>,
        Read<'s, FpsCounter>,
    );

    fn run(
        &mut self,
        (mut balls,
            mut transforms,
            mut text,
            mut score_board,
            storage,
            sounds,
            score_text,
            audio_output,
            fps_counter): Self::SystemData,
    ) {
        if let Some(text) = text.get_mut(score_text.fps_display) {
            let fps = fps_counter.sampled_fps();
            text.text = format!("FPS: {:.*}", 2, fps);
        }

        for (ball, transform) in (&mut balls, &mut transforms).join() {
            use crate::ARENA_WIDTH;

            let ball_x = transform.translation().x;

            let did_hit = if ball_x <= ball.radius {
                // Right player scored on the left side.
                // We top the score at 999 to avoid text overlap.
                score_board.score_right = (score_board.score_right + 1).min(999);
                if let Some(text) = text.get_mut(score_text.p2_score) {
                    text.text = score_board.score_right.to_string();
                }
                true
            } else if ball_x >= ARENA_WIDTH - ball.radius {
                // Left player scored on the right side.
                // We top the score at 999 to avoid text overlap.
                score_board.score_left = (score_board.score_left + 1).min(999);
                if let Some(text) = text.get_mut(score_text.p1_score) {
                    text.text = score_board.score_left.to_string();
                }
                true
            } else {
                false
            };

            if did_hit {
                // Reset the ball.
                ball.velocity[0] = -ball.velocity[0];
                transform.set_translation_x(ARENA_WIDTH / 2.0);

                // Play audio.
                if let Some(ref output) = audio_output {
                    if let Some(sound) = storage.get(&sounds.score_sfx) {
                        if cfg!(not(test)) {
                            output.play_once(sound, 1.0);
                        }
                    }
                }
            }
        }
    }
}

/// Stores the entities that are displaying the player score with `UiText`.
pub struct ScoreText {
    pub p1_score: Entity,
    pub p2_score: Entity,
    pub fps_display: Entity,
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        audio::initialise_audio,
        states::{initialise_ball, initialise_score, load_sprite_sheet},
        ScoreBoard,
    };
    use amethyst::{assets::ProgressCounter, ecs::prelude::WorldExt, prelude::Builder, ui::UiCreator};
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn reset_ball_on_hitting_left_side() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = crate::test_harness::IntegrationTestApplication::pong_base()
            .with_setup(|world| {
                let mut progress = ProgressCounter::default();
                initialise_audio(world, &mut progress);
                world.insert(ScoreBoard::new());

                let ui_root = Some(world.exec(|mut creator: UiCreator<'_>| creator.create("ui/hud.ron", ())));
                if let Some(ui_root) = ui_root {
                    initialise_score(world, ui_root);
                }

                // Initialize ball
                let root_entity = Some(world.create_entity().with(Transform::default()).build());
                let sprite_sheet_handle = Some(load_sprite_sheet(world));
                if let Some(root_entity) = root_entity {
                    if let Some(sprite_sheet) = sprite_sheet_handle {
                        initialise_ball(
                            world,
                            root_entity,
                            sprite_sheet,
                            crate::BALL_RADIUS,
                            [-10.0, 0.0],
                            Some([0.0, 0.0]),
                        );
                    }
                }
            })
            .with_system_single(WinnerSystem, "", &[])
            .with_assertion(|world| {
                let balls = world.read_storage::<Ball>();
                let transforms = world.read_storage::<Transform>();
                assert_eq!(1, balls.count());

                for (ball, transform) in (&balls, &transforms).join() {
                    assert_approx_eq!(ball.velocity[0], 10.0);
                    assert_approx_eq!(transform.translation().x, crate::ARENA_WIDTH / 2.0);
                }
            })
            .run();
        assert!(test_result.is_ok());
    }

    #[test]
    fn reset_ball_on_hitting_right_side() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = crate::test_harness::IntegrationTestApplication::pong_base()
            .with_setup(|world| {
                let mut progress = ProgressCounter::default();
                initialise_audio(world, &mut progress);
                world.insert(ScoreBoard::new());

                let ui_root = Some(world.exec(|mut creator: UiCreator<'_>| creator.create("ui/hud.ron", ())));
                if let Some(ui_root) = ui_root {
                    initialise_score(world, ui_root);
                }

                // Initialize ball
                let root_entity = Some(world.create_entity().with(Transform::default()).build());
                let sprite_sheet_handle = Some(load_sprite_sheet(world));
                if let Some(root_entity) = root_entity {
                    if let Some(sprite_sheet) = sprite_sheet_handle {
                        initialise_ball(
                            world,
                            root_entity,
                            sprite_sheet,
                            crate::BALL_RADIUS,
                            [10.0, 0.0],
                            Some([crate::ARENA_WIDTH, 0.0]),
                        );
                    }
                }
            })
            .with_system_single(WinnerSystem, "", &[])
            .with_assertion(|world| {
                let balls = world.read_storage::<Ball>();
                let transforms = world.read_storage::<Transform>();
                assert_eq!(1, balls.count());

                for (ball, transform) in (&balls, &transforms).join() {
                    assert_approx_eq!(ball.velocity[0], -10.0);
                    assert_approx_eq!(transform.translation().x, crate::ARENA_WIDTH / 2.0);
                }
            })
            .run();
        assert!(test_result.is_ok());
    }
}
