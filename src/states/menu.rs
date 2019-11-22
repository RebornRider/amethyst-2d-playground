use amethyst::{
    ecs::prelude::Entity,
    input::{is_close_requested, is_key_down},
    prelude::*,
    ui::{UiCreator, UiEvent, UiEventType, UiFinder},
    winit::VirtualKeyCode,
};

use crate::game_data::CustomGameData;
use crate::states::{util::delete_hierarchy, CreditsScreen, Pong, WelcomeScreen};
use crate::GameStateEvent;

const BUTTON_START: &str = "start";
const BUTTON_LOAD: &str = "load";
const BUTTON_OPTIONS: &str = "options";
const BUTTON_CREDITS: &str = "credits";

#[derive(Default, Debug)]
pub struct MainMenu {
    ui_root: Option<Entity>,
    button_start: Option<Entity>,
    button_load: Option<Entity>,
    button_options: Option<Entity>,
    button_credits: Option<Entity>,
}

impl<'a, 'b> State<CustomGameData<'static, 'static>, GameStateEvent> for MainMenu {
    fn on_start(&mut self, data: StateData<'_, CustomGameData<'_, '_>>) {
        // create UI from prefab and save the reference.
        let world = data.world;

        self.ui_root = Some(world.exec(|mut creator: UiCreator<'_>| creator.create("ui/menu.ron", ())));
    }

    fn on_stop(&mut self, data: StateData<'_, CustomGameData<'_, '_>>) {
        // after destroying the current UI, invalidate references as well (makes things cleaner)
        if let Some(entity) = self.ui_root {
            delete_hierarchy(entity, data.world).expect("Failed to remove MainMenu");
        }
        self.ui_root = None;
        self.button_start = None;
        self.button_load = None;
        self.button_options = None;
        self.button_credits = None;
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
                } else if is_key_down(&event, VirtualKeyCode::Escape) {
                    log::info!("[Trans::Switch] Switching back to WelcomeScreen!");
                    Trans::Switch(Box::new(WelcomeScreen::default()))
                } else {
                    Trans::None
                }
            }
            GameStateEvent::Ui(UiEvent {
                event_type: UiEventType::Click,
                target,
            }) => {
                if Some(target) == self.button_credits {
                    log::info!("[Trans::Switch] Switching to CreditsScreen!");
                    return Trans::Switch(Box::new(CreditsScreen::default()));
                }
                if Some(target) == self.button_start {
                    log::info!("[Trans::Switch] Switching to Pong!");
                    return Trans::Switch(Box::new(Pong::default()));
                }
                if Some(target) == self.button_load || Some(target) == self.button_options {
                    log::info!("This Buttons functionality is not yet implemented!");
                }

                Trans::None
            }
            _ => Trans::None,
        }
    }

    fn update(
        &mut self,
        data: StateData<'_, CustomGameData<'_, '_>>,
    ) -> Trans<CustomGameData<'static, 'static>, GameStateEvent> {
        data.data.update(data.world, true);
        // only search for buttons if they have not been found yet
        let StateData { world, .. } = data;

        if self.button_start.is_none()
            || self.button_load.is_none()
            || self.button_options.is_none()
            || self.button_credits.is_none()
        {
            world.exec(|ui_finder: UiFinder<'_>| {
                self.button_start = ui_finder.find(BUTTON_START);
                self.button_load = ui_finder.find(BUTTON_LOAD);
                self.button_options = ui_finder.find(BUTTON_OPTIONS);
                self.button_credits = ui_finder.find(BUTTON_CREDITS);
            });
        }

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
    use amethyst::core::transform::TransformBundle;

    #[test]
    fn test_main_menu_state() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = crate::test_harness::IntegrationTestApplication::blank()
            .with_bundle(TransformBundle::new())
            .with_setup(|world| {
                setup_loader_for_test(world);
            })
            .with_state(MainMenu::default)
            .run();
        assert!(test_result.is_ok());
    }
}
