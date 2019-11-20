mod credits;
mod menu;
mod pause;
mod pong;
mod util;
mod welcome;

pub use self::{
    credits::CreditsScreen, menu::MainMenu, pause::PauseMenuState, pong::initialise_ball, pong::initialise_camera, pong::initialise_paddles, pong::load_sprite_sheet, pong::Pong,
    util::delete_hierarchy, welcome::WelcomeScreen,
};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_gameplay_state() {
        assert_eq!(GameplayState::default(), GameplayState::Paused);
    }
}
