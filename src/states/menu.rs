#![allow(clippy::use_self)]

use crate::{
    game_data::CustomGameData,
    states::{util::delete_hierarchy, CreditsScreen, Pong, WelcomeScreen},
    GameStateEvent,
};
use amethyst::{
    assets::{Completion, ProgressCounter},
    ecs::prelude::Entity,
    input::{is_close_requested, is_key_down},
    prelude::*,
    ui::{UiCreator, UiEvent, UiEventType, UiFinder},
    winit::VirtualKeyCode,
};
use derivative::Derivative;

const BUTTON_START: &str = "start";
const BUTTON_LOAD: &str = "load";
const BUTTON_OPTIONS: &str = "options";
const BUTTON_CREDITS: &str = "credits";

#[derive(Derivative)]
#[derivative(Debug)]
#[derivative(Default)]
pub struct MainMenu {
    ui_root: Option<Entity>,
    button_start: Option<Entity>,
    button_load: Option<Entity>,
    button_options: Option<Entity>,
    button_credits: Option<Entity>,
    #[derivative(Debug = "ignore")]
    load_progress: Option<ProgressCounter>,
}

impl<'a, 'b> State<CustomGameData<'static, 'static>, GameStateEvent> for MainMenu {
    fn on_start(&mut self, data: StateData<'_, CustomGameData<'_, '_>>) {
        // create UI from prefab and save the reference.
        let world = data.world;
        let mut progress = ProgressCounter::default();
        self.ui_root = Some(world.exec(|mut creator: UiCreator<'_>| creator.create("ui/menu.ron", &mut progress)));
        self.load_progress = Some(progress);
    }

    fn on_stop(&mut self, data: StateData<'_, CustomGameData<'_, '_>>) {
        // after destroying the current UI, invalidate references as well (makes things cleaner)
        if let Some(entity) = self.ui_root {
            delete_hierarchy(entity, data.world).expect("Failed to remove MainMenu");
        }

        if let Some(progress) = &self.load_progress {
            if progress.complete() == Completion::Failed {
                progress.errors().iter().enumerate().for_each(|(n, e)| {
                    eprintln!("{}: error: {}", n, e.error);
                });
            }
        }

        self.ui_root = None;
        self.button_start = None;
        self.button_load = None;
        self.button_options = None;
        self.button_credits = None;
        self.load_progress = None;
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
            GameStateEvent::Test(test_event) => crate::test_harness::handle_test_event(&test_event),
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
    fn test_main_menu_state() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = crate::test_harness::IntegrationTestApplication::pong_base()
            .with_state(|| {
                SendMockEvents::test_state(|_world| Box::new(MainMenu::default()))
                    .with_wait(1.0)
                    .end_test()
            })
            .run();
        assert!(test_result.is_ok());
    }

    #[test]
    fn click_start_button() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = crate::test_harness::IntegrationTestApplication::pong_base()
            .with_setup(|world| {
                let mut progress = ProgressCounter::default();
                initialise_audio(world, &mut progress);
            })
            .with_state(|| {
                SendMockEvents::test_state(|_world| Box::new(MainMenu::default()))
                    .with_condition_barrier(
                        |world| {
                            let mut button: Option<Entity> = None;
                            world.exec(|ui_finder: UiFinder<'_>| {
                                button = ui_finder.find(BUTTON_START);
                            });
                            if button.is_some() {
                                ConditionBarrierResult::ResumeImmediately
                            } else {
                                ConditionBarrierResult::ContinueEvaluating
                            }
                        },
                        Duration::from_secs(20),
                    )
                    .with_step(|world| {
                        let mut button: Option<Entity> = None;
                        world.exec(|ui_finder: UiFinder<'_>| {
                            button = ui_finder.find(BUTTON_START);
                        });
                        let event = UiEvent::new(UiEventType::Click, button.expect("Could not find start button"));
                        let mut events: Write<EventChannel<UiEvent>> = world.system_data();
                        events.single_write(event);
                    })
                    .with_wait(1.0)
                    .end_test()
            })
            .run();
        assert!(test_result.is_ok());
    }

    #[test]
    fn click_load_button() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = crate::test_harness::IntegrationTestApplication::pong_base()
            .with_state(|| {
                SendMockEvents::test_state(|_world| Box::new(MainMenu::default()))
                    .with_condition_barrier(
                        |world| {
                            let mut button: Option<Entity> = None;
                            world.exec(|ui_finder: UiFinder<'_>| {
                                button = ui_finder.find(BUTTON_LOAD);
                            });
                            if button.is_some() {
                                ConditionBarrierResult::ResumeImmediately
                            } else {
                                ConditionBarrierResult::ContinueEvaluating
                            }
                        },
                        Duration::from_secs(20),
                    )
                    .with_step(|world| {
                        let mut button: Option<Entity> = None;
                        world.exec(|ui_finder: UiFinder<'_>| {
                            button = ui_finder.find(BUTTON_LOAD);
                        });
                        let event = UiEvent::new(UiEventType::Click, button.expect("Could not find load button"));
                        let mut events: Write<EventChannel<UiEvent>> = world.system_data();
                        events.single_write(event);
                    })
                    .with_wait(1.0)
                    .end_test()
            })
            .run();
        assert!(test_result.is_ok());
    }

    #[test]
    fn click_options_button() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = crate::test_harness::IntegrationTestApplication::pong_base()
            .with_state(|| {
                SendMockEvents::test_state(|_world| Box::new(MainMenu::default()))
                    .with_condition_barrier(
                        |world| {
                            let mut button: Option<Entity> = None;
                            world.exec(|ui_finder: UiFinder<'_>| {
                                button = ui_finder.find(BUTTON_OPTIONS);
                            });
                            if button.is_some() {
                                ConditionBarrierResult::ResumeImmediately
                            } else {
                                ConditionBarrierResult::ContinueEvaluating
                            }
                        },
                        Duration::from_secs(20),
                    )
                    .with_step(|world| {
                        let mut button: Option<Entity> = None;
                        world.exec(|ui_finder: UiFinder<'_>| {
                            button = ui_finder.find(BUTTON_OPTIONS);
                        });
                        let event = UiEvent::new(UiEventType::Click, button.expect("Could not find options button"));
                        let mut events: Write<EventChannel<UiEvent>> = world.system_data();
                        events.single_write(event);
                    })
                    .with_wait(1.0)
                    .end_test()
            })
            .run();
        assert!(test_result.is_ok());
    }

    #[test]
    fn click_credits_button() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = crate::test_harness::IntegrationTestApplication::pong_base()
            .with_state(|| {
                SendMockEvents::test_state(|_world| Box::new(MainMenu::default()))
                    .with_condition_barrier(
                        |world| {
                            let mut button: Option<Entity> = None;
                            world.exec(|ui_finder: UiFinder<'_>| {
                                button = ui_finder.find(BUTTON_CREDITS);
                            });
                            if button.is_some() {
                                ConditionBarrierResult::ResumeImmediately
                            } else {
                                ConditionBarrierResult::ContinueEvaluating
                            }
                        },
                        Duration::from_secs(20),
                    )
                    .with_step(|world| {
                        let mut button: Option<Entity> = None;
                        world.exec(|ui_finder: UiFinder<'_>| {
                            button = ui_finder.find(BUTTON_CREDITS);
                        });
                        let event = UiEvent::new(UiEventType::Click, button.expect("Could not find credits button"));
                        let mut events: Write<EventChannel<UiEvent>> = world.system_data();
                        events.single_write(event);
                    })
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
                SendMockEvents::test_state(|_world| Box::new(MainMenu::default()))
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
                SendMockEvents::test_state(|_world| Box::new(MainMenu::default()))
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
                SendMockEvents::test_state(|_world| Box::new(MainMenu::default()))
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
                SendMockEvents::test_state(|_world| Box::new(Pong::default()))
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
