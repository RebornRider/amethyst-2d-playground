#[cfg(not(test))]
use crate::audio::set_sink_volume;
use crate::{audio::initialise_audio, game_data::CustomGameData};

use crate::{
    states::{util::delete_hierarchy, GameplayState},
    GameStateEvent,
};
use amethyst::{
    assets::{Completion, ProgressCounter},
    ecs::prelude::*,
    input::{is_close_requested, is_key_down, is_mouse_button_down},
    prelude::*,
    ui::UiCreator,
    winit::{MouseButton, VirtualKeyCode},
};

#[derive(Default)]
pub struct WelcomeScreen {
    ui_handle: Option<Entity>,
    load_progress: Option<ProgressCounter>,
}

impl<'a, 'b> State<CustomGameData<'static, 'static>, GameStateEvent> for WelcomeScreen {
    fn on_start(&mut self, data: StateData<'_, CustomGameData<'_, '_>>) {
        data.world.insert(GameplayState::Paused);

        let mut progress = ProgressCounter::default();
        self.ui_handle = Some(
            data.world
                .exec(|mut creator: UiCreator<'_>| creator.create("ui/welcome.ron", &mut progress)),
        );
        initialise_audio(data.world, &mut progress);
        self.load_progress = Some(progress);

        #[cfg(not(test))]
        set_sink_volume(data.world, 0.2);
    }

    fn on_stop(&mut self, data: StateData<'_, CustomGameData<'_, '_>>) {
        if let Some(handler) = self.ui_handle {
            delete_hierarchy(handler, data.world).expect("Failed to remove WelcomeScreen");
        }
        self.ui_handle = None;
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, CustomGameData<'_, '_>>,
        event: GameStateEvent,
    ) -> Trans<CustomGameData<'static, 'static>, GameStateEvent> {
        data.data.update(data.world, true);
        match event {
            GameStateEvent::Window(event) => {
                if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                    log::info!("[Trans::Quit] Quitting Application!");
                    Trans::Quit
                } else if is_mouse_button_down(&event, MouseButton::Left) {
                    if self
                        .load_progress
                        .as_ref()
                        .map_or(Completion::Complete, ProgressCounter::complete)
                        != Completion::Loading
                        || cfg!(test)
                    {
                        log::info!("[Trans::Switch] Switching to MainMenu!");
                        Trans::Switch(Box::new(crate::states::MainMenu::default()))
                    } else {
                        Trans::None
                    }
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

    use crate::test_harness::SendMockEvents;
    use amethyst::{
        core::shrev::EventChannel,
        input::{InputEvent, StringBindings},
        ui::{UiEvent, UiEventType},
        winit,
        winit::*,
    };

    #[test]
    fn left_mouse_button() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = crate::test_harness::IntegrationTestApplication::pong_base()
            .with_state(|| {
                SendMockEvents::test_state(|_world| Box::new(WelcomeScreen::default()))
                    .with_wait(1.0)
                    .with_step(|world| unsafe {
                        let event = Event::WindowEvent {
                            window_id: WindowId::dummy(),
                            event: WindowEvent::MouseInput {
                                device_id: DeviceId::dummy(),
                                state: ElementState::Pressed,
                                button: MouseButton::Left,
                                modifiers: winit::ModifiersState::default(),
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
            .with_state(|| {
                SendMockEvents::test_state(|_world| Box::new(WelcomeScreen::default()))
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
    fn escape_key() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = crate::test_harness::IntegrationTestApplication::pong_base()
            .with_state(|| {
                SendMockEvents::test_state(|_world| Box::new(WelcomeScreen::default()))
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
    fn window_event() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = crate::test_harness::IntegrationTestApplication::pong_base()
            .with_state(|| {
                SendMockEvents::test_state(|_world| Box::new(WelcomeScreen::default()))
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
    fn unhandled_ui_event() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = crate::test_harness::IntegrationTestApplication::pong_base()
            .with_state(|| {
                SendMockEvents::test_state(|_world| Box::new(WelcomeScreen::default()))
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

    #[test]
    fn unhandled_input_event() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = crate::test_harness::IntegrationTestApplication::pong_base()
            .with_state(|| {
                SendMockEvents::test_state(|_world| Box::new(WelcomeScreen::default()))
                    .with_step(|world| {
                        let event = InputEvent::<StringBindings>::CursorMoved {
                            delta_x: 0.0,
                            delta_y: 0.0,
                        };

                        let mut events: Write<EventChannel<InputEvent<StringBindings>>> = world.system_data();
                        events.single_write(event);
                    })
                    .end_test()
            })
            .run();
        assert!(test_result.is_ok());
    }
}
