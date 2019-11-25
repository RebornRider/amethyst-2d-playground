use amethyst::{
    assets::{AssetStorage, Loader, ProgressCounter},
    audio::{output::Output, OggFormat, Source, SourceHandle},
    ecs::{World, WorldExt},
};

use std::{iter::Cycle, vec::IntoIter};

pub struct Sounds {
    pub score_sfx: SourceHandle,
    pub bounce_sfx: SourceHandle,
}

pub struct Music {
    pub music: Cycle<IntoIter<SourceHandle>>,
}

/// Loads an ogg audio track.
pub fn load_audio_track(loader: &Loader, world: &World, file: &str, progress: &mut ProgressCounter) -> SourceHandle {
    loader.load(file, OggFormat, progress, &world.read_resource())
}

/// Initialise audio in the world. This includes the background track and the
/// sound effects.
pub fn initialise_audio(world: &mut World, progress: &mut ProgressCounter) {
    use crate::{AUDIO_BOUNCE, AUDIO_MUSIC, AUDIO_SCORE};

    let (sound_effects, music) = {
        let loader = world.read_resource::<Loader>();

        let music = AUDIO_MUSIC
            .iter()
            .map(|file| load_audio_track(&loader, world, file, progress))
            .collect::<Vec<_>>()
            .into_iter()
            .cycle();
        let music = Music { music };

        let sound = Sounds {
            bounce_sfx: load_audio_track(&loader, world, AUDIO_BOUNCE, progress),
            score_sfx: load_audio_track(&loader, world, AUDIO_SCORE, progress),
        };

        (sound, music)
    };

    // Add sound effects to the world. We have to do this in another scope because
    // world won't let us insert new resources as long as `Loader` is borrowed.
    world.insert(sound_effects);
    world.insert(music);
}

#[cfg(not(test))]
pub fn set_sink_volume(world: &mut World, volume: f32) {
    use amethyst::audio::AudioSink;
    let mut sink = world.write_resource::<AudioSink>();
    sink.set_volume(volume);
}

/// Plays the bounce sound when a ball hits a side or a paddle.
pub fn play_bounce(sounds: &Sounds, storage: &AssetStorage<Source>, output: Option<&Output>) {
    if let Some(output) = output.as_ref() {
        if let Some(sound) = storage.get(&sounds.bounce_sfx) {
            if cfg!(not(test)) {
                output.play_once(sound, 1.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_harness::IntegrationTestApplication;

    #[test]
    fn test_initialise_audio() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = IntegrationTestApplication::pong_base()
            .with_setup(|world| {
                let mut progress = ProgressCounter::default();
                initialise_audio(world, &mut progress);
            })
            .with_assertion(|world| {
                world.read_resource::<Music>();
                world.read_resource::<Sounds>();
            })
            .run();
        assert!(test_result.is_ok());
    }
}
