use crate::{
    states::{delete_hierarchy, GameplayState, PauseMenuState},
    systems::ScoreText,
    Ball, Paddle, Side, ARENA_HEIGHT, ARENA_WIDTH,
};
use amethyst::{
    assets::{AssetStorage, Handle, Loader},
    core::{timing::Time, transform::Transform, Parent},
    ecs::{
        prelude::{Entity, World, WorldExt},
        Dispatcher, DispatcherBuilder,
    },
    input::{is_close_requested, is_key_down},
    prelude::*,
    renderer::{Camera, ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture},
    ui::{Anchor, TtfFormat, UiCreator, UiText, UiTransform},
    winit::VirtualKeyCode,
};

#[derive(Default)]
pub struct Pong<'a, 'b> {
    ball_spawn_timer: Option<f32>,
    sprite_sheet_handle: Option<Handle<SpriteSheet>>,
    dispatcher: Option<Dispatcher<'a, 'b>>,
    root_entity: Option<Entity>,
    ui_root: Option<Entity>,
}

impl<'a, 'b> SimpleState for Pong<'a, 'b> {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        *data.world.write_resource::<GameplayState>() = GameplayState::Running;

        self.initialize_gameplay_dispatcher(data.world);

        // Wait one second before spawning the ball.
        self.ball_spawn_timer.replace(1.0);

        // Initialize roots
        self.root_entity = Some(data.world.create_entity().with(Transform::default()).build());
        self.ui_root = Some(data.world.exec(|mut creator: UiCreator<'_>| creator.create("ui/hud.ron", ())));

        // Load the spritesheet necessary to render the graphics.
        // `spritesheet` is the layout of the sprites on the image;
        // `texture` is the pixel data.
        self.sprite_sheet_handle.replace(load_sprite_sheet(data.world));
        if let Some(root_entity) = self.root_entity {
            if let Some(sprite_sheet_handle) = self.sprite_sheet_handle.clone() {
                initialise_paddles(data.world, root_entity, sprite_sheet_handle);
                initialise_camera(data.world, root_entity);
            }
        }
        if let Some(ui_root) = self.ui_root {
            initialise_score(data.world, ui_root);
        }
    }

    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        if let Some(entity) = self.root_entity.take() {
            delete_hierarchy(entity, data.world).expect("Failed to remove Pong entities");
        }

        if let Some(entity) = self.ui_root.take() {
            delete_hierarchy(entity, data.world).expect("Failed to remove Pong HUD entities");
        }

        if let Some(dispatcher) = self.dispatcher.take() {
            dispatcher.dispose(data.world);
        }

        self.sprite_sheet_handle = None;
        self.ball_spawn_timer = None;
    }

    fn on_pause(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        *data.world.write_resource::<GameplayState>() = GameplayState::Paused;
    }

    fn on_resume(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        *data.world.write_resource::<GameplayState>() = GameplayState::Running;
    }

    fn handle_event(&mut self, _: StateData<'_, GameData<'_, '_>>, event: StateEvent) -> SimpleTrans {
        match event {
            StateEvent::Window(event) => {
                if is_close_requested(&event) {
                    log::info!("[Trans::Quit] Quitting Application!");
                    Trans::Quit
                } else if is_key_down(&event, VirtualKeyCode::Escape) {
                    log::info!("[Trans::Push] Pausing Game!");
                    Trans::Push(Box::new(PauseMenuState::default()))
                } else {
                    Trans::None
                }
            }
            _ => Trans::None,
        }
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if let Some(dispatcher) = self.dispatcher.as_mut() {
            dispatcher.dispatch(data.world);
        }

        if *data.world.read_resource::<GameplayState>() == GameplayState::Paused {
            return Trans::None;
        }

        if let Some(mut timer) = self.ball_spawn_timer.take() {
            // If the timer isn't expired yet, substract the time that passed since last update.
            {
                let time = data.world.fetch::<Time>();
                timer -= time.delta_seconds();
            }
            if timer <= 0.0 {
                if let Some(root_entity) = self.root_entity {
                    // When timer expire, spawn the ball
                    let sprite_sheet_clone = self.sprite_sheet_handle.clone();
                    if let Some(sprite_sheet) = sprite_sheet_clone {
                        use crate::{BALL_RADIUS, BALL_VELOCITY_X, BALL_VELOCITY_Y};
                        initialise_ball(data.world, root_entity, sprite_sheet, BALL_RADIUS, [BALL_VELOCITY_X, BALL_VELOCITY_Y], None);
                        #[cfg(test)]
                        return Trans::Quit;
                    }
                }
            } else {
                // If timer is not expired yet, put it back onto the state.
                self.ball_spawn_timer.replace(timer);
            }
        }

        Trans::None
    }
}

impl<'a, 'b> Pong<'a, 'b> {
    fn initialize_gameplay_dispatcher(&mut self, world: &mut World) {
        if self.dispatcher.is_none() {
            use crate::systems::PongBundle;
            use amethyst::core::bundle::SystemBundle;

            let mut dispatcher_builder = DispatcherBuilder::new();
            let pong_bundle = PongBundle {};

            pong_bundle.build(world, &mut dispatcher_builder).expect("couldn't add pong bundle to dispatcher");

            // Build and setup the `Dispatcher`.
            let mut dispatcher = dispatcher_builder.build();
            dispatcher.setup(world);
            self.dispatcher = Some(dispatcher);
        }
    }
}

pub fn load_sprite_sheet(world: &mut World) -> Handle<SpriteSheet> {
    // Load the sprite sheet necessary to render the graphics.
    // The texture is the pixel data
    // `sprite_sheet` is the layout of the sprites on the image
    // `texture_handle` is a cloneable reference to the texture

    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load("texture/pong_spritesheet.png", ImageFormat::default(), (), &texture_storage)
    };

    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        "texture/pong_spritesheet.ron",    // Here we load the associated ron file
        SpriteSheetFormat(texture_handle), // We pass it the texture we want it to use
        (),
        &sprite_sheet_store,
    )
}

/// Initialise the camera.
pub fn initialise_camera(world: &mut World, parent: Entity) {
    // Setup camera in a way that our screen covers whole arena and (0, 0) is in the bottom left.
    let mut transform = Transform::default();
    transform.set_translation_xyz(ARENA_WIDTH * 0.5, ARENA_HEIGHT * 0.5, 1.0);

    world
        .create_entity()
        .with(Camera::standard_2d(ARENA_WIDTH, ARENA_HEIGHT))
        .with(Parent { entity: parent })
        .with(transform)
        .build();
}

/// Initialises one paddle on the left, and one paddle on the right.
pub fn initialise_paddles(world: &mut World, parent: Entity, sprite_sheet_handle: Handle<SpriteSheet>) {
    use crate::{PADDLE_HEIGHT, PADDLE_VELOCITY, PADDLE_WIDTH};

    let mut left_transform = Transform::default();
    let mut right_transform = Transform::default();

    // Correctly position the paddles.
    let y = ARENA_HEIGHT / 2.0;
    left_transform.set_translation_xyz(PADDLE_WIDTH * 0.5, y, 0.0);
    right_transform.set_translation_xyz(ARENA_WIDTH - PADDLE_WIDTH * 0.5, y, 0.0);

    // Assign the sprites for the paddles
    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: 0, // paddle is the first sprite in the sprite_sheet
    };

    // Create a left plank entity.
    world
        .create_entity()
        .with(sprite_render.clone())
        .with(Paddle {
            velocity: PADDLE_VELOCITY,
            side: Side::Left,
            width: PADDLE_WIDTH,
            height: PADDLE_HEIGHT,
        })
        .with(left_transform)
        .with(Parent { entity: parent })
        .build();

    // Create right plank entity.
    world
        .create_entity()
        .with(sprite_render)
        .with(Paddle {
            velocity: PADDLE_VELOCITY,
            side: Side::Right,
            width: PADDLE_WIDTH,
            height: PADDLE_HEIGHT,
        })
        .with(right_transform)
        .with(Parent { entity: parent })
        .build();
}

/// Initialises one ball in the middle-ish of the arena.
pub fn initialise_ball(world: &mut World, parent: Entity, sprite_sheet_handle: Handle<SpriteSheet>, radius: f32, velocity: [f32; 2], position: Option<[f32; 2]>) {
    // Create the translation.
    let mut local_transform = Transform::default();
    let initial_position = position.unwrap_or_else(|| [ARENA_WIDTH / 2.0, ARENA_HEIGHT / 2.0]);
    local_transform.set_translation_xyz(initial_position[0], initial_position[1], 0.0);

    // Assign the sprite for the ball
    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: 1, // ball is the second sprite on the sprite_sheet
    };

    world
        .create_entity()
        .with(sprite_render)
        .with(Ball { radius, velocity })
        .with(local_transform)
        .with(Parent { entity: parent })
        .build();
}

pub fn initialise_score(world: &mut World, parent: Entity) {
    let font = world.read_resource::<Loader>().load("font/square.ttf", TtfFormat, (), &world.read_resource());
    let p1_transform = UiTransform::new("P1".to_string(), Anchor::TopMiddle, Anchor::Middle, -50., -50., 1., 200., 50.);

    let p2_transform = UiTransform::new("P2".to_string(), Anchor::TopMiddle, Anchor::Middle, 50., -50., 1., 200., 50.);

    let fps_text_transform = UiTransform::new("FPS".to_string(), Anchor::TopLeft, Anchor::TopLeft, 0., 0., 1., 200., 50.);

    let p1_score = world
        .create_entity()
        .with(p1_transform)
        .with(UiText::new(font.clone(), "0".to_string(), [1.0, 1.0, 1.0, 1.0], 50.))
        .with(Parent { entity: parent })
        .build();

    let p2_score = world
        .create_entity()
        .with(p2_transform)
        .with(UiText::new(font.clone(), "0".to_string(), [1.0, 1.0, 1.0, 1.0], 50.))
        .with(Parent { entity: parent })
        .build();

    let fps_display = world
        .create_entity()
        .with(fps_text_transform)
        .with(UiText::new(font, "0".to_string(), [1.0, 1.0, 1.0, 1.0], 24.))
        .with(Parent { entity: parent })
        .build();
    world.insert(ScoreText { p1_score, p2_score, fps_display });
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{audio::initialise_audio, setup_loader_for_test};
    use amethyst::{
        assets::AssetStorage,
        audio::Source,
        core::{Parent, TransformBundle},
        input::StringBindings,
        renderer::{SpriteRender, SpriteSheet, Texture},
        window::ScreenDimensions,
    };
    use amethyst_test::AmethystApplication;

    #[test]
    fn test_pong_state() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = AmethystApplication::blank()
            .with_bundle(TransformBundle::new())
            .with_ui_bundles::<StringBindings>()
            .with_resource(ScreenDimensions::new(1920, 1080, 1.0))
            .with_setup(|world| {
                setup_loader_for_test(world);
                world.insert(GameplayState::Paused);
                world.insert(AssetStorage::<Source>::default());
                initialise_audio(world);

                world.insert(AssetStorage::<Texture>::default());
                world.insert(AssetStorage::<SpriteSheet>::default());
                world.register::<Transform>();
                world.register::<Parent>();
                world.register::<SpriteRender>();
                world.register::<Paddle>();
                world.register::<Ball>();
                world.register::<Camera>();
            })
            .with_state(Pong::default)
            .run();
        assert!(test_result.is_ok());
    }
}
