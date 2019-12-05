use crate::{game_data::CustomGameData, states::MainMenu, GameStateEvent};
use amethyst::{
    ecs::Entity,
    input::{is_close_requested, is_key_down},
    prelude::*,
    shrev::EventChannel,
    ui::{UiCreator, UiEvent, UiEventType, UiFinder},
    winit::VirtualKeyCode,
    TransEvent,
};

/// Adapted, originally from amethyst/evoli `src/states/pause_menu.rs`

#[derive(Default)]
pub struct PauseMenuState {
    /// goes back to pong state
    resume_button: Option<Entity>,
    /// goes back to mainmenu state
    exit_to_main_menu_button: Option<Entity>,
    /// closes the game
    exit_button: Option<Entity>,
    /// ui hierarchy root entity
    root: Option<Entity>,
}

/// resume button prefab ID
const RESUME_BUTTON_ID: &str = "resume";
/// resume button prefab ID
const EXIT_TO_MAIN_MENU_BUTTON_ID: &str = "exit_to_main_menu";
/// resume button prefab ID
const EXIT_BUTTON_ID: &str = "exit";

// load the pause_menu.ron prefab then instantiate it
// if the "resume" button is clicked, goto MainGameState
// if the "exit_to_main_menu" button is clicked, remove the pause and main game states and go to
// MenuState. if the "exit" button is clicked, quit the program.
impl<'a, 'b> State<CustomGameData<'static, 'static>, GameStateEvent> for PauseMenuState {
    fn on_start(&mut self, data: StateData<'_, CustomGameData<'_, '_>>) {
        let world = data.world;

        self.root = Some(world.exec(|mut creator: UiCreator<'_>| creator.create("ui/pause_menu.ron", ())));
    }

    fn on_stop(&mut self, data: StateData<'_, CustomGameData<'_, '_>>) {
        if let Some(root) = self.root {
            if data.world.delete_entity(root).is_ok() {
                self.root = None;
            }
        }
        self.resume_button = None;
        self.exit_to_main_menu_button = None;
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, CustomGameData<'_, '_>>,
        event: GameStateEvent,
    ) -> Trans<CustomGameData<'static, 'static>, GameStateEvent> {
        match event {
            GameStateEvent::Window(event) => {
                if is_close_requested(&event) {
                    log::info!("[Trans::Quit] Quitting Application!");
                    Trans::Quit
                } else if is_key_down(&event, VirtualKeyCode::Escape) {
                    log::info!("[Trans::Pop] Closing Pause Menu!");
                    Trans::Pop
                } else {
                    Trans::None
                }
            }
            GameStateEvent::Ui(UiEvent {
                event_type: UiEventType::Click,
                target,
            }) => {
                if Some(target) == self.resume_button {
                    log::info!("Resuming Pong!");
                    Trans::Pop
                } else if Some(target) == self.exit_to_main_menu_button {
                    let mut state_transition_event_channel = data
                        .world
                        .write_resource::<EventChannel<TransEvent<CustomGameData<'static, 'static>, GameStateEvent>>>();

                    // this allows us to first 'Pop' this state, and then exchange whatever was
                    // below that with a new MainMenu state.
                    state_transition_event_channel.single_write(Box::new(|| Trans::Pop));
                    if cfg!(not(test)) {
                        state_transition_event_channel
                            .single_write(Box::new(|| Trans::Switch(Box::new(MainMenu::default()))));
                    }

                    log::info!("[Trans::Pop] Closing Pause Menu!");
                    log::info!("[Trans::Switch] Switching to MainMenu!");

                    Trans::None // we could also not add the pop to the channel and Pop here
                                // but like this the execution order is guaranteed (in the next
                                // versions)
                } else if Some(target) == self.exit_button {
                    Trans::Quit
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
        // once deferred creation of the root ui entity finishes, look up buttons
        if self.resume_button.is_none() || self.exit_to_main_menu_button.is_none() || self.exit_button.is_none() {
            data.world.exec(|ui_finder: UiFinder<'_>| {
                self.resume_button = ui_finder.find(RESUME_BUTTON_ID);
                self.exit_to_main_menu_button = ui_finder.find(EXIT_TO_MAIN_MENU_BUTTON_ID);
                self.exit_button = ui_finder.find(EXIT_BUTTON_ID);
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
    fn test_pause_menu_state() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = crate::test_harness::IntegrationTestApplication::pong_base()
            .with_state(|| {
                SendMockEvents::test_state(|_world| Box::new(PauseMenuState::default()))
                    .with_wait(1.0)
                    .end_test()
            })
            .run();
        assert!(test_result.is_ok());
    }

    #[test]
    fn click_exit_button() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = crate::test_harness::IntegrationTestApplication::pong_base()
            .with_state(|| {
                SendMockEvents::test_state(|_world| Box::new(PauseMenuState::default()))
                    .with_condition_barrier(
                        |world| {
                            let mut exit_button: Option<Entity> = None;
                            world.exec(|ui_finder: UiFinder<'_>| {
                                exit_button = ui_finder.find(EXIT_BUTTON_ID);
                            });
                            if exit_button.is_some() {
                                ConditionBarrierResult::ResumeImmediately
                            } else {
                                ConditionBarrierResult::ContinueEvaluating
                            }
                        },
                        Duration::from_secs(20),
                    )
                    .with_step(|world| {
                        let mut exit_button: Option<Entity> = None;
                        world.exec(|ui_finder: UiFinder<'_>| {
                            exit_button = ui_finder.find(EXIT_BUTTON_ID);
                        });
                        let event = UiEvent::new(UiEventType::Click, exit_button.expect("Could not find exit button"));
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
    fn click_exit_to_main_menu_button() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = crate::test_harness::IntegrationTestApplication::pong_base()
            .with_state(|| {
                SendMockEvents::test_state(|_world| Box::new(PauseMenuState::default()))
                    .with_condition_barrier(
                        |world| {
                            let mut exit_to_main_menu_button: Option<Entity> = None;
                            world.exec(|ui_finder: UiFinder<'_>| {
                                exit_to_main_menu_button = ui_finder.find(EXIT_TO_MAIN_MENU_BUTTON_ID);
                            });
                            if exit_to_main_menu_button.is_some() {
                                ConditionBarrierResult::ResumeImmediately
                            } else {
                                ConditionBarrierResult::ContinueEvaluating
                            }
                        },
                        Duration::from_secs(20),
                    )
                    .with_step(|world| {
                        let mut exit_to_main_menu_button: Option<Entity> = None;
                        world.exec(|ui_finder: UiFinder<'_>| {
                            exit_to_main_menu_button = ui_finder.find(EXIT_TO_MAIN_MENU_BUTTON_ID);
                        });
                        let event = UiEvent::new(
                            UiEventType::Click,
                            exit_to_main_menu_button.expect("Could not find exit to main menu button"),
                        );
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
    fn click_resume_button() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = crate::test_harness::IntegrationTestApplication::pong_base()
            .with_state(|| {
                SendMockEvents::test_state(|_world| Box::new(PauseMenuState::default()))
                    .with_condition_barrier(
                        |world| {
                            let mut resume_button: Option<Entity> = None;
                            world.exec(|ui_finder: UiFinder<'_>| {
                                resume_button = ui_finder.find(RESUME_BUTTON_ID);
                            });
                            if resume_button.is_some() {
                                ConditionBarrierResult::ResumeImmediately
                            } else {
                                ConditionBarrierResult::ContinueEvaluating
                            }
                        },
                        Duration::from_secs(20),
                    )
                    .with_step(|world| {
                        let mut resume_button: Option<Entity> = None;
                        world.exec(|ui_finder: UiFinder<'_>| {
                            resume_button = ui_finder.find(RESUME_BUTTON_ID);
                        });
                        let event =
                            UiEvent::new(UiEventType::Click, resume_button.expect("Could not find resume button"));
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
                SendMockEvents::test_state(|_world| Box::new(PauseMenuState::default()))
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
                SendMockEvents::test_state(|_world| Box::new(PauseMenuState::default()))
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
                SendMockEvents::test_state(|_world| Box::new(PauseMenuState::default()))
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

        #[test]
        fn unhandled_ui_event() {
            amethyst::start_logger(amethyst::LoggerConfig::default());
            let test_result = crate::test_harness::IntegrationTestApplication::pong_base()
                .with_setup(|world| {
                    let mut progress = ProgressCounter::default();
                    initialise_audio(world, &mut progress);
                })
                .with_state(|| {
                    SendMockEvents::test_state(|_world| Box::new(PauseMenuState::default()))
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
}
