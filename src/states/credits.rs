use amethyst::{
    ecs::prelude::Entity,
    input::{is_close_requested, is_key_down, is_mouse_button_down},
    prelude::*,
    ui::UiCreator,
    winit::{MouseButton, VirtualKeyCode},
};

use crate::states::{util::delete_hierarchy, MainMenu};

// A simple 'Screen' State, only capable of loading/showing the prefab ui and registering simple
// UI interactions (pressing escape or clicking anywhere).

#[derive(Debug, Default)]
pub struct CreditsScreen {
    ui_handle: Option<Entity>,
}

impl SimpleState for CreditsScreen {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        self.ui_handle = Some(world.exec(|mut creator: UiCreator<'_>| creator.create("ui/credits.ron", ())));
    }

    fn on_stop(&mut self, data: StateData<GameData>) {
        if let Some(handler) = self.ui_handle {
            delete_hierarchy(handler, data.world).expect("Failed to remove CreditScreen");
        }
        self.ui_handle = None;
    }

    fn handle_event(&mut self, _: StateData<'_, GameData<'_, '_>>, event: StateEvent) -> SimpleTrans {
        match event {
            StateEvent::Window(event) => {
                if is_close_requested(&event) {
                    log::info!("[Trans::Quit] Quitting Application!");
                    Trans::Quit
                } else if is_key_down(&event, VirtualKeyCode::Escape) || is_mouse_button_down(&event, MouseButton::Left) {
                    log::info!("[Trans::Switch] Switching to MainMenu!");
                    Trans::Switch(Box::new(MainMenu::default()))
                } else {
                    Trans::None
                }
            }
            StateEvent::Ui(..) | StateEvent::Input(..) => Trans::None,
        }
    }

    fn update(&mut self, _data: &mut StateData<GameData>) -> SimpleTrans {
        if cfg!(test) {
            Trans::Quit
        } else {
            Trans::None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup_loader_for_test;
    use amethyst::audio::AudioBundle;
    use amethyst::core::transform::TransformBundle;
    use amethyst_test::AmethystApplication;

    #[test]
    fn test_credits_state() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = AmethystApplication::blank()
            .with_bundle(TransformBundle::new())
            .with_bundle(AudioBundle::default())
            .with_setup(|world| {
                setup_loader_for_test(world);
            })
            .with_state(CreditsScreen::default)
            .run();
        assert!(test_result.is_ok());
    }
}
