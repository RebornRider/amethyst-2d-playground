mod integration_test_application;
mod raw_file_loader_source;
mod send_mock_events;

pub use self::{
    integration_test_application::IntegrationTestApplication, raw_file_loader_source::RawFileLoaderSource,
    send_mock_events::ConditionBarrierResult, send_mock_events::SendMockEvents,
};
use crate::{game_data::CustomGameData, GameStateEvent};

use crate::TestEvent;
use amethyst::Trans;

pub fn handle_test_event(test_event: &TestEvent) -> Trans<CustomGameData<'static, 'static>, GameStateEvent> {
    match test_event {
        TestEvent::Quit => Trans::Quit,
        TestEvent::Pop => Trans::Pop,
        TestEvent::Panic => panic!("test panic"),
    }
}
