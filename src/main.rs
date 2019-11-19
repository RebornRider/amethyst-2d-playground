//! Pong
mod audio;
mod states;
mod systems;

use amethyst::{
    assets::HotReloadBundle,
    audio::{AudioBundle, DjSystemDesc},
    core::{frame_limiter::FrameRateLimitStrategy, transform::TransformBundle},
    ecs::{Component, DenseVecStorage},
    error::Error,
    input::{InputBundle, StringBindings},
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    ui::{RenderUi, UiBundle},
    utils::{application_root_dir, fps_counter::FpsCounterBundle},
};

use crate::{audio::Music, systems::UiEventHandlerSystemDesc};
extern crate dunce;
use std::{path, time::Duration};

const ARENA_HEIGHT: f32 = 90.0;
const ARENA_WIDTH: f32 = 160.0;
const PADDLE_HEIGHT: f32 = 16.0;
const PADDLE_WIDTH: f32 = 4.0;
const PADDLE_VELOCITY: f32 = 75.0;

const BALL_VELOCITY_X: f32 = 75.0;
const BALL_VELOCITY_Y: f32 = 50.0;
const BALL_RADIUS: f32 = 2.0;

const AUDIO_MUSIC: &[&str] = &[
    "audio/Computer_Music_All-Stars_-_Wheres_My_Jetpack.ogg",
    "audio/Computer_Music_All-Stars_-_Albatross_v2.ogg",
];
const AUDIO_BOUNCE: &str = "audio/bounce.ogg";
const AUDIO_SCORE: &str = "audio/score.ogg";

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(amethyst::LoggerConfig::default());
    let (display_config_path, key_bindings_path, assets_dir) = initialize_paths()?;
    let mut game = build_game(display_config_path, key_bindings_path, assets_dir)?;
    game.run();
    Ok(())
}

fn initialize_paths() -> Result<(path::PathBuf, path::PathBuf, path::PathBuf), Error> {
    let app_root = path::PathBuf::from(
        dunce::canonicalize(application_root_dir()?)?
            .to_str()
            .unwrap_or_default()
            .replace(r"\target\debug\deps", "")
            .replace(r"\target\debug", "")
            .replace(r"\target\release\deps", "")
            .replace(r"\target\release", ""),
    );
    let display_config_path = app_root.join("resources/config/display.ron");
    let key_bindings_path = {
        if cfg!(feature = "sdl_controller") {
            app_root.join("resources/config/input_controller.ron")
        } else {
            app_root.join("resources/config/input.ron")
        }
    };
    let assets_dir = app_root.join("resources/");
    Ok((display_config_path, key_bindings_path, assets_dir))
}

fn build_game(
    display_config_path: path::PathBuf,
    key_bindings_path: path::PathBuf,
    assets_dir: path::PathBuf,
) -> Result<CoreApplication<'static, GameData<'static, 'static>>, Error> {
    let game_data = build_game_data(display_config_path, key_bindings_path)?;
    let game = Application::build(assets_dir, states::WelcomeScreen::default())?
        .with_frame_limit(FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)), 144)
        .build(game_data)?;
    Ok(game)
}

fn build_game_data(display_config_path: path::PathBuf, key_bindings_path: path::PathBuf) -> Result<GameDataBuilder<'static, 'static>, Error> {
    use log::warn;
    if key_bindings_path.as_path().exists() == false || key_bindings_path.as_path().is_file() == false {
        let path = key_bindings_path.into_os_string();
        warn!("{:?} does not exist", path);
        return Err(Error::from_string("bad key_bindings_path"));
    }

    if display_config_path.as_path().exists() == false || display_config_path.as_path().is_file() == false {
        let path = display_config_path.into_os_string();
        warn!("{:?} does not exist", path);
        return Err(Error::from_string("bad display_config_path"));
    }

    GameDataBuilder::default()
    // Add the transform bundle which handles tracking entity positions
    .with_bundle(TransformBundle::new())?
    .with_bundle(HotReloadBundle::default())?
    .with_bundle(InputBundle::<StringBindings>::new().with_bindings_from_file(key_bindings_path)?)?
    .with_bundle(AudioBundle::default())?
    .with_bundle(FpsCounterBundle::default())?
    .with_system_desc(
        DjSystemDesc::new(|music: &mut Music| music.music.next()),
        "dj_system",
        &[],
    )
    .with_system_desc(UiEventHandlerSystemDesc::default(), "ui_event_handler", &[])
    .with_bundle(UiBundle::<StringBindings>::new())?
    .with_bundle(
        RenderingBundle::<DefaultBackend>::new()
            // The RenderToWindow plugin provides all the scaffolding for opening a window and
            // drawing on it
            .with_plugin(RenderToWindow::from_config_path(display_config_path).with_clear([0.34, 0.36, 0.52, 1.0]))
            .with_plugin(RenderFlat2D::default())
            .with_plugin(RenderUi::default()),
    )
}

pub struct Ball {
    pub velocity: [f32; 2],
    pub radius: f32,
}

impl Component for Ball {
    type Storage = DenseVecStorage<Self>;
}

#[derive(PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

pub struct Paddle {
    pub velocity: f32,
    pub side: Side,
    pub width: f32,
    pub height: f32,
}

impl Paddle {
    pub const fn new(side: Side) -> Self {
        Self {
            velocity: 1.0,
            side,
            width: 1.0,
            height: 1.0,
        }
    }
}

impl Component for Paddle {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Default)]
pub struct ScoreBoard {
    score_left: i32,
    score_right: i32,
}

impl ScoreBoard {
    pub const fn new() -> Self {
        Self { score_left: 0, score_right: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_board_initialisation() {
        let scoreboard = ScoreBoard::new();
        assert_eq!(scoreboard.score_left, 0);
        assert_eq!(scoreboard.score_right, 0);
        assert_eq!(scoreboard.score_right, scoreboard.score_left);
    }

    #[test]
    fn validate_paths_are_not_garbage() -> amethyst::Result<()> {
        let (mut display_config_path, mut key_bindings_path, mut assets_dir) = initialize_paths()?;
        assert!(display_config_path.is_absolute());
        assert!(display_config_path.pop());

        assert!(key_bindings_path.is_absolute());
        assert!(key_bindings_path.pop());

        assert!(assets_dir.is_absolute());
        assert!(assets_dir.pop());
        Ok(())
    }

    #[test]
    fn validate_game_data_builder() -> amethyst::Result<()> {
        let (display_config_path, key_bindings_path, _) = initialize_paths()?;
        use log::info;

        {
            let path = key_bindings_path.clone().into_os_string();
            info!("validate_game_data_builder - key_bindings_path:  {:?}", path);
        }

        {
            let path = display_config_path.clone().into_os_string();
            info!("validate_game_data_builder - display_config_path:  {:?}", path);
        }

        let _game_data = build_game_data(display_config_path, key_bindings_path)?;
        Ok(())
    }
}
