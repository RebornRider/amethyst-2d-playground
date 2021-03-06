mod audio;
mod game_data;
mod states;
mod systems;
mod test_harness;

use crate::{audio::Music, systems::UiEventHandlerSystemDesc};
use amethyst::{
    audio::{AudioBundle, DjSystemDesc},
    core::{
        ecs::{Read, SystemData, World},
        frame_limiter::FrameRateLimitStrategy,
        shrev::{EventChannel, ReaderId},
        transform::TransformBundle,
        EventReader,
    },
    derive::EventReader,
    ecs::{Component, DenseVecStorage},
    error::Error,
    input::{BindingTypes, InputBundle, InputEvent, StringBindings},
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    ui::{RenderUi, UiBundle, UiEvent},
    utils::{application_root_dir, fps_counter::FpsCounterBundle},
    winit::Event,
};
use derivative::Derivative;
extern crate dunce;
use crate::game_data::{CustomGameData, CustomGameDataBuilder};
use sentry::integrations::panic::register_panic_handler;
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

    let sentry = sentry::init(sentry::ClientOptions::default());
    if sentry.is_enabled() {
        register_panic_handler();
    }

    let mut game = build_game()?;
    game.run();
    Ok(())
}

fn initialize_paths() -> Result<(path::PathBuf, path::PathBuf, path::PathBuf), Error> {
    let app_root = initialize_app_root()?;
    let display_config_path = app_root.join("resources/config/display.ron");
    let key_bindings_path = {
        if cfg!(feature = "sdl_controller") {
            app_root.join("resources/config/input_controller.ron")
        } else {
            app_root.join("resources/config/input.ron")
        }
    };
    let assets_dir = app_root.join("resources");
    Ok((display_config_path, key_bindings_path, assets_dir))
}

fn initialize_app_root() -> Result<path::PathBuf, Error> {
    let app_root = dunce::canonicalize(application_root_dir()?)?;
    let directory = "amethyst-2d-playground";
    if let Some(index) = app_root.to_str().unwrap_or_default().rfind("amethyst-2d-playground") {
        let mut path = app_root.to_str().unwrap_or_default().to_string();
        path.truncate(index + directory.len());
        return Ok(path::PathBuf::from(path));
    }
    Ok(app_root)
}

fn build_game(
) -> Result<CoreApplication<'static, CustomGameData<'static, 'static>, GameStateEvent, GameStateEventReader>, Error> {
    let (display_config_path, key_bindings_path, assets_dir) = initialize_paths()?;
    let game_data = build_game_data(display_config_path, key_bindings_path)?;
    let game =
        CoreApplication::<'static, CustomGameData<'static, 'static>, GameStateEvent, GameStateEventReader>::build(
            assets_dir,
            states::WelcomeScreen::default(),
        )?
        .with_frame_limit(FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)), 144)
        .build(game_data)?;
    Ok(game)
}

fn build_game_data(
    display_config_path: path::PathBuf,
    key_bindings_path: path::PathBuf,
) -> Result<CustomGameDataBuilder<'static, 'static>, Error> {
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

    let builder = if cfg!(test) {
        CustomGameDataBuilder::default()
    } else {
        CustomGameDataBuilder::default()
            .with_base_bundle(AudioBundle::default())
            .with_base(
                DjSystemDesc::new(|music: &mut Music| music.music.next()),
                "dj_system",
                &[],
            )
    };
    let builder = builder
        .with_base_bundle(TransformBundle::new())
        .with_base_bundle(InputBundle::<StringBindings>::new().with_bindings_from_file(key_bindings_path)?)
        .with_base_bundle(FpsCounterBundle::default())
        .with_base(UiEventHandlerSystemDesc::default(), "ui_event_handler", &[])
        .with_base_bundle(UiBundle::<StringBindings>::new())
        .with_base_bundle(
            RenderingBundle::<DefaultBackend>::new()
            // The RenderToWindow plugin provides all the scaffolding for opening a window and
            // drawing on it
            .with_plugin(RenderToWindow::from_config_path(display_config_path).with_clear([0.34, 0.36, 0.52, 1.0]))
            .with_plugin(RenderFlat2D::default())
            .with_plugin(RenderUi::default()),
        );
    Ok(builder)
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
        Self {
            score_left: 0,
            score_right: 0,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TestEvent {
    Quit,
    Pop,
    Panic,
}

#[derive(Debug, Derivative, EventReader)]
#[derivative(Clone(bound = ""))]
#[reader(GameStateEventReader)]
pub enum GameStateEvent<T = StringBindings>
where
    T: BindingTypes,
{
    /// Events sent by the winit window.
    Window(Event),
    /// Events sent by the ui system.
    Ui(UiEvent),
    /// Events sent by the input system.
    Input(InputEvent<T>),
    Test(TestEvent),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{panic, path::PathBuf};

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
        build_game_data(display_config_path, key_bindings_path)?;
        Ok(())
    }

    #[test]
    fn validate_game_data_builder_garbage_key_bindings_path() {
        let (_, key_bindings_path, _) = initialize_paths().expect("valid paths required");
        assert!(build_game_data(PathBuf::new(), key_bindings_path).is_err());
    }

    #[test]
    fn validate_game_data_builder_garbage_display_config_path() {
        let (display_config_path, ..) = initialize_paths().expect("valid paths required");
        assert!(build_game_data(display_config_path, PathBuf::new()).is_err());
    }
}
