mod integration_test_application;
mod send_mock_events;

pub use self::{integration_test_application::IntegrationTestApplication, send_mock_events::SendMockEvents};
use crate::{game_data::CustomGameData, GameStateEvent};

use crate::TestEvent;
use amethyst::Trans;

#[allow(dead_code)]
pub fn quit_during_tests() -> Trans<CustomGameData<'static, 'static>, GameStateEvent> {
    if cfg!(test) {
        Trans::Quit
    } else {
        Trans::None
    }
}

pub fn handle_test_event(test_event: &TestEvent) -> Trans<CustomGameData<'static, 'static>, GameStateEvent> {
    match test_event {
        TestEvent::Quit => Trans::Quit,
        TestEvent::Pop => Trans::Pop,
        TestEvent::Panic => panic!("test panic"),
    }
}
