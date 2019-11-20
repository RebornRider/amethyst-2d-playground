use amethyst::{
    ecs::prelude::Entity,
    input::{is_close_requested, is_key_down, is_mouse_button_down},
    prelude::*,
    ui::UiCreator,
    winit::{MouseButton, VirtualKeyCode},
};

use crate::{
    audio::{initialise_audio, set_sink_volume},
    setup_loader_for_test,
    states::{util::delete_hierarchy, GameplayState},
};

#[derive(Default, Debug)]
pub struct WelcomeScreen {
    ui_handle: Option<Entity>,
}

impl SimpleState for WelcomeScreen {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let mut world = data.world;

        world.insert(GameplayState::Paused);
        self.ui_handle = Some(world.exec(|mut creator: UiCreator<'_>| creator.create("ui/welcome.ron", ())));

        initialise_audio(world);
        #[cfg(not(test))]
        set_sink_volume(world, 0.2);
    }

    fn on_stop(&mut self, data: StateData<GameData>) {
        if let Some(handler) = self.ui_handle {
            delete_hierarchy(handler, data.world).expect("Failed to remove WelcomeScreen");
        }
        self.ui_handle = None;
    }

    fn handle_event(&mut self, _: StateData<'_, GameData<'_, '_>>, event: StateEvent) -> SimpleTrans {
        match &event {
            StateEvent::Window(event) => {
                if is_close_requested(event) || is_key_down(event, VirtualKeyCode::Escape) {
                    log::info!("[Trans::Quit] Quitting Application!");
                    Trans::Quit
                } else if is_mouse_button_down(event, MouseButton::Left) {
                    log::info!("[Trans::Switch] Switching to MainMenu!");
                    Trans::Switch(Box::new(crate::states::MainMenu::default()))
                } else {
                    Trans::None
                }
            }
            _ => Trans::None,
        }
    }

    fn update(&mut self, _data: &mut StateData<GameData>) -> SimpleTrans {
        #[cfg(test)]
        return Trans::Quit;
        Trans::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::initialize_paths;
    use amethyst::assets::{Directory, Loader};
    use amethyst::audio::AudioBundle;
    use amethyst::core::transform::TransformBundle;
    use amethyst_test::AmethystApplication;

    #[test]
    fn test_welcome_screen() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = AmethystApplication::blank()
            .with_bundle(TransformBundle::new())
            .with_bundle(AudioBundle::default())
            .with_setup(|world| {
                setup_loader_for_test(world);
            })
            .with_state(|| WelcomeScreen::default())
            .run();
        assert!(test_result.is_ok());
    }
}
