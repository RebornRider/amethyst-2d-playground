use amethyst::{
    ecs::prelude::Entity,
    input::{is_close_requested, is_key_down, is_mouse_button_down},
    prelude::*,
    ui::UiCreator,
    winit::{MouseButton, VirtualKeyCode},
};

use crate::audio::initialise_audio;
#[cfg(not(test))]
use crate::audio::set_sink_volume;

use crate::states::{util::delete_hierarchy, GameplayState};

#[derive(Default, Debug)]
pub struct WelcomeScreen {
    ui_handle: Option<Entity>,
}

impl SimpleState for WelcomeScreen {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        data.world.insert(GameplayState::Paused);
        self.ui_handle = Some(data.world.exec(|mut creator: UiCreator<'_>| creator.create("ui/welcome.ron", ())));

        initialise_audio(data.world);
        #[cfg(not(test))]
        set_sink_volume(data.world, 0.2);
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
        Trans::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup_loader_for_test;
    use amethyst::audio::AudioBundle;
    use amethyst::core::ecs::{Read, Write};
    use amethyst::core::shrev::EventChannel;
    use amethyst::core::transform::TransformBundle;
    use amethyst::core::EventReader;
    use amethyst::core::SystemDesc;
    use amethyst::derive::SystemDesc;
    use amethyst::ecs::prelude::System;
    use amethyst::winit::*;
    use amethyst::StateEvent;
    use amethyst::StateEventReader;
    use amethyst_test::AmethystApplication;

    pub struct SendMockEvents<T, E> {
        send_mock_events: Box<dyn Fn(&mut Write<EventChannel<Event>>) + Send>,
        next_state: Box<dyn Fn(&mut World) -> Box<dyn State<T, E>> + Send>,
    }

    impl<T, E: Send + Sync + 'static> SendMockEvents<T, E> {
        pub fn new<Fsme, Fns>(next_state: Fns, send_mock_events: Fsme) -> Self
        where
            Fsme: Fn(&mut Write<EventChannel<Event>>) + Send + 'static,
            Fns: Fn(&mut World) -> Box<dyn State<T, E>> + Send + 'static,
        {
            Self {
                send_mock_events: Box::new(send_mock_events),
                next_state: Box::new(next_state),
            }
        }
    }

    impl<T, E: Send + Sync + 'static> State<T, E> for SendMockEvents<T, E> {
        fn update(&mut self, data: StateData<'_, T>) -> Trans<T, E> {
            {
                let mut events: Write<EventChannel<Event>>;
                events = data.world.system_data();
                (self.send_mock_events)(&mut events);
            }

            Trans::Switch((self.next_state)(data.world))
        }
    }

    #[test]
    fn test_welcome_screen() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = AmethystApplication::blank()
            .with_bundle(TransformBundle::new())
            .with_bundle(AudioBundle::default())
            .with_setup(|world| {
                setup_loader_for_test(world);
            })
            .with_state(|| {
                SendMockEvents::new(
                    |_world| Box::new(WelcomeScreen::default()),
                    |events| unsafe {
                        events.single_write(Event::WindowEvent {
                            window_id: WindowId::dummy(),
                            event: WindowEvent::KeyboardInput {
                                device_id: DeviceId::dummy(),
                                input: KeyboardInput {
                                    scancode: 0,
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    modifiers: Default::default(),
                                },
                            },
                        });
                    },
                )
            })
            .run();
        assert!(test_result.is_ok());
    }
}
