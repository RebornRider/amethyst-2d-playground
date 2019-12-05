use crate::{
    game_data::CustomGameData,
    states::{util::delete_hierarchy, MainMenu},
    GameStateEvent,
};
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
            GameStateEvent::Test(test_event) => crate::test_harness::handle_test_event(&test_event),
            _ => Trans::None,
        }
    }

    fn update(
        &mut self,
        data: StateData<'_, CustomGameData<'_, '_>>,
    ) -> Trans<CustomGameData<'static, 'static>, GameStateEvent> {
        data.data.update(data.world, true);
        Trans::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::initialise_audio;
    use crate::test_harness::{ConditionBarrierResult, SendMockEvents};
    use amethyst::{
        assets::ProgressCounter,
        core::shrev::EventChannel,
        ecs::prelude::*,
        ui::{UiEvent, UiEventType},
        winit,
        winit::*,
    };
    use std::time::Duration;

    #[test]
    fn test_credits_state() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = crate::test_harness::IntegrationTestApplication::pong_base()
            .with_state(|| {
                SendMockEvents::test_state(|_world| Box::new(CreditsScreen::default()))
                    .with_wait(1.0)
                    .end_test()
            })
            .run();
        assert!(test_result.is_ok());
    }

    #[test]
    fn is_close_requested() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = crate::test_harness::IntegrationTestApplication::pong_base()
            .with_setup(|world| {
                let mut progress = ProgressCounter::default();
                initialise_audio(world, &mut progress);
            })
            .with_state(|| {
                SendMockEvents::test_state(|_world| Box::new(CreditsScreen::default()))
                    .with_step(|world| unsafe {
                        let event = Event::WindowEvent {
                            window_id: WindowId::dummy(),
                            event: WindowEvent::CloseRequested,
                        };
                        let mut events: Write<EventChannel<Event>> = world.system_data();
                        events.single_write(event);
                    })
                    .end_test()
            })
            .run();
        assert!(test_result.is_ok());
    }

    #[test]
    fn escape_key() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = crate::test_harness::IntegrationTestApplication::pong_base()
            .with_setup(|world| {
                let mut progress = ProgressCounter::default();
                initialise_audio(world, &mut progress);
            })
            .with_state(|| {
                SendMockEvents::test_state(|_world| Box::new(CreditsScreen::default()))
                    .with_step(|world| unsafe {
                        let event = Event::WindowEvent {
                            window_id: WindowId::dummy(),
                            event: WindowEvent::KeyboardInput {
                                device_id: DeviceId::dummy(),
                                input: KeyboardInput {
                                    scancode: 0,
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    modifiers: winit::ModifiersState::default(),
                                },
                            },
                        };
                        let mut events: Write<EventChannel<Event>> = world.system_data();
                        events.single_write(event);
                    })
                    .end_test()
            })
            .run();
        assert!(test_result.is_ok());
    }

    #[test]
    fn unhandled_window_event() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = crate::test_harness::IntegrationTestApplication::pong_base()
            .with_setup(|world| {
                let mut progress = ProgressCounter::default();
                initialise_audio(world, &mut progress);
            })
            .with_state(|| {
                SendMockEvents::test_state(|_world| Box::new(CreditsScreen::default()))
                    .with_step(|world| unsafe {
                        let event = Event::WindowEvent {
                            window_id: WindowId::dummy(),
                            event: WindowEvent::HoveredFileCancelled,
                        };
                        let mut events: Write<EventChannel<Event>> = world.system_data();
                        events.single_write(event);
                    })
                    .end_test()
            })
            .run();
        assert!(test_result.is_ok());
    }

    #[test]
    fn unhandled_ui_event() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = crate::test_harness::IntegrationTestApplication::pong_base()
            .with_setup(|world| {
                let mut progress = ProgressCounter::default();
                initialise_audio(world, &mut progress);
            })
            .with_state(|| {
                SendMockEvents::test_state(|_world| Box::new(CreditsScreen::default()))
                    .with_step(|world| {
                        let event = UiEvent::new(UiEventType::ValueChange, world.create_entity().build());
                        let mut events: Write<EventChannel<UiEvent>> = world.system_data();
                        events.single_write(event);
                    })
                    .end_test()
            })
            .run();
        assert!(test_result.is_ok());
    }
}
