use amethyst::{
    assets::{AssetStorage, Loader},
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
pub fn load_audio_track(loader: &Loader, world: &World, file: &str) -> SourceHandle {
    loader.load(file, OggFormat, (), &world.read_resource())
}

/// Initialise audio in the world. This includes the background track and the
/// sound effects.
pub fn initialise_audio(world: &mut World) {
    use crate::{AUDIO_BOUNCE, AUDIO_MUSIC, AUDIO_SCORE};

    let (sound_effects, music) = {
        let loader = world.read_resource::<Loader>();

        let music = AUDIO_MUSIC
            .iter()
            .map(|file| load_audio_track(&loader, world, file))
            .collect::<Vec<_>>()
            .into_iter()
            .cycle();
        let music = Music { music };

        let sound = Sounds {
            bounce_sfx: load_audio_track(&loader, world, AUDIO_BOUNCE),
            score_sfx: load_audio_track(&loader, world, AUDIO_SCORE),
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
            #[cfg(not(test))]
            output.play_once(sound, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup_loader_for_test;
    use amethyst::audio::AudioBundle;
    use amethyst::core::transform::TransformBundle;
    use amethyst_test::AmethystApplication;

    #[test]
    fn test_initialise_audio() {
        amethyst::start_logger(amethyst::LoggerConfig::default());
        let test_result = AmethystApplication::blank()
            .with_bundle(TransformBundle::new())
            .with_bundle(AudioBundle::default())
            .with_setup(|world| {
                setup_loader_for_test(world);
                world.insert(AssetStorage::<Source>::default());
                initialise_audio(world);
            })
            .with_assertion(|world| {
                world.read_resource::<Music>();
                world.read_resource::<Sounds>();
            })
            .run();
        assert!(test_result.is_ok());
    }
}
