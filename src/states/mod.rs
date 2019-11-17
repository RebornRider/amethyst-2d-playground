mod credits;
mod menu;
mod pause;
mod pong;
mod util;
mod welcome;

pub use self::{credits::CreditsScreen, menu::MainMenu, pause::PauseMenuState, pong::Pong, util::delete_hierarchy, welcome::WelcomeScreen};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameplayState {
    Running,
    Paused,
}

impl std::default::Default for GameplayState {
    fn default() -> Self {
        Self::Paused
    }
}
