use crate::game_data::CustomGameData;
use crate::states::{util::delete_hierarchy, MainMenu};
use crate::test_harness::IntegrationTestApplication;
use crate::{quit_during_tests, GameStateEvent};
use amethyst::{
    ecs::prelude::Entity,
    input::{is_close_requested, is_key_down, is_mouse_button_down},
    prelude::*,
    ui::UiCreator,
    winit::{MouseButton, VirtualKeyCode},
};

// A simple 'Screen' State, only capable of loading/showing the prefab ui and registering simple
// UI interactions (pressing escape or clicking anywhere).

#[derive(Debug, Default)]
pub struct CreditsScreen {
    ui_handle: Option<Entity>,
}

impl<'a, 'b> State<CustomGameData<'static, 'static>, GameStateEvent> for CreditsScreen {
    fn on_start(&mut self, data: StateData<'_, CustomGameData<'_, '_>>) {
        let world = data.world;

        self.ui_handle = Some(world.exec(|mut creator: UiCreator<'_>| creator.create("ui/credits.ron", ())));
    }

    fn on_stop(&mut self, data: StateData<'_, CustomGameData<'_, '_>>) {
        if let Some(handler) = self.ui_handle {
            delete_hierarchy(handler, data.world).expect("Failed to remove CreditScreen");
        }
        self.ui_handle = None;
    }

    fn handle_event(
        &mut self,
        _: StateData<'_, CustomGameData<'_, '_>>,
        event: GameStateEvent,
    ) -> Trans<CustomGameData<'static, 'static>, GameStateEvent> {
        match event {
            GameStateEvent::Window(event) => {
                if is_close_requested(&event) {
                    log::info!("[Trans::Quit] Quitting Application!");
                    Trans::Quit
                } else if is_key_down(&event, VirtualKeyCode::Escape) || is_mouse_button_down(&event, MouseButton::Left)
                {
                    log::info!("[Trans::Switch] Switching to MainMenu!");
                    Trans::Switch(Box::new(MainMenu::default()))
                } else {
                    Trans::None
                }
            }
            _ => Trans::None,
        }
    }

    fn update(
        &mut self,
        data: StateData<'_, CustomGameData<'_, '_>>,
    ) -> Trans<CustomGameData<'static, 'static>, GameStateEvent> {
        data.data.update(&data.world, true);
        quit_during_tests()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup_loader_for_test;
    use amethyst::core::transform::TransformBundle;

    #[test]
    fn test_credits_state() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = IntegrationTestApplication::blank()
            .with_bundle(TransformBundle::new())
            .with_setup(|world| {
                setup_loader_for_test(world);
            })
            .with_state(CreditsScreen::default)
            .run();
        assert!(test_result.is_ok());
    }
}
