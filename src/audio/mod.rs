mod audio_utils;

cfg_if::cfg_if! {
    if #[cfg(not(test))] {
        pub use self::audio_utils::set_sink_volume;
    }
}

pub use self::audio_utils::{initialise_audio, load_audio_track, play_bounce, Music, Sounds};
