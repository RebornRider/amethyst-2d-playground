use crate::{
    states,
    systems::{BounceSystem, MoveBallsSystem, PaddleSystem, WinnerSystem},
};
use amethyst::{
    core::bundle::SystemBundle,
    ecs::prelude::{DispatcherBuilder, World},
    error::Error,
    prelude::SystemExt,
};

/// A bundle is a convenient way to initialise related resources, components and systems in a
/// world. This bundle prepares the world for a game of pong.
#[derive(Default)]
pub struct PongBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for PongBundle {
    fn build(self, _world: &mut World, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        builder.add(PaddleSystem.pausable(states::GameplayState::Running), "paddle_system", &[]);
        builder.add(MoveBallsSystem.pausable(states::GameplayState::Running), "ball_system", &[]);
        builder.add(BounceSystem.pausable(states::GameplayState::Running), "collision_system", &["paddle_system", "ball_system"]);
        builder.add(WinnerSystem.pausable(states::GameplayState::Running), "winner_system", &["paddle_system", "ball_system"]);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use amethyst_test::AmethystApplication;

    #[test]
    fn register_bundle() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        assert!(AmethystApplication::blank().with_bundle(PongBundle).run().is_ok())
    }
}
