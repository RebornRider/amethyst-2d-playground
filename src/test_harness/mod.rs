mod integration_test_application;
mod send_mock_events;

pub use self::{integration_test_application::IntegrationTestApplication, send_mock_events::SendMockEvents};
use crate::{game_data::CustomGameData, GameStateEvent};

use amethyst::Trans;

pub fn quit_during_tests() -> Trans<CustomGameData<'static, 'static>, GameStateEvent> {
    if cfg!(test) {
        Trans::Quit
    } else {
        Trans::None
    }
}
