//! Background music — prioritizes the original bundled `resources/track.mp3`.

use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use rodio::Source;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicTrack {
    Title,
    Battle,
    Victory,
    Pokedex,
}

impl MusicTrack {
    /// Ordered candidates. Always ends with the original `resources/track.mp3`.
    pub fn candidates(self) -> Vec<PathBuf> {
        let mut out = Vec::new();
        match self {
            Self::Title | Self::Pokedex => {
                out.push(PathBuf::from("resources/track.mp3"));
                out.push(PathBuf::from("resources/audio/track.mp3"));
            }
            Self::Battle => {
                // Optional battle clip; fall back to main theme (original behaviour)
                out.push(PathBuf::from("resources/audio/battle.mp3"));
                out.push(PathBuf::from("resources/track.mp3"));
                out.push(PathBuf::from("resources/audio/track.mp3"));
            }
            Self::Victory => {
                out.push(PathBuf::from("resources/audio/victory.mp3"));
                out.push(PathBuf::from("resources/track.mp3"));
                out.push(PathBuf::from("resources/audio/track.mp3"));
            }
        }
        out
    }
}

pub struct AudioManager {
    _stream: rodio::OutputStream,
    sink: rodio::Sink,
    pub enabled: bool,
    pub volume: f32,
    current: Option<MusicTrack>,
}

impl AudioManager {
    pub fn try_start() -> Option<Self> {
        let (stream, stream_handle) = match rodio::OutputStream::try_default() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Warning: no audio device ({e}); continuing without music.");
                return None;
            }
        };
        let sink = match rodio::Sink::try_new(&stream_handle) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Warning: audio sink failed ({e}); continuing without music.");
                return None;
            }
        };
        sink.set_volume(0.65);
        let mut mgr = Self {
            _stream: stream,
            sink,
            enabled: true,
            volume: 0.65,
            current: None,
        };
        mgr.play(MusicTrack::Title);
        Some(mgr)
    }

    pub fn set_enabled(&mut self, on: bool) {
        self.enabled = on;
        if on {
            self.sink.play();
        } else {
            self.sink.pause();
        }
    }

    pub fn toggle(&mut self) {
        self.set_enabled(!self.enabled);
    }

    pub fn set_volume(&mut self, vol: f32) {
        self.volume = vol.clamp(0.0, 1.0);
        self.sink.set_volume(self.volume);
    }

    pub fn play(&mut self, track: MusicTrack) {
        // Don't restart if same track is still playing
        if self.current == Some(track) && !self.sink.empty() {
            return;
        }
        // Title/pokedex/battle all may resolve to the same original file — avoid restarting
        if matches!(track, MusicTrack::Title | MusicTrack::Pokedex | MusicTrack::Battle)
            && matches!(self.current, Some(MusicTrack::Title | MusicTrack::Pokedex | MusicTrack::Battle))
            && !self.sink.empty()
        {
            self.current = Some(track);
            return;
        }

        self.current = Some(track);
        self.sink.stop();

        let path = resolve_track(track);
        let Some(path) = path else {
            eprintln!("Warning: no music file found (expected resources/track.mp3)");
            return;
        };
        let file = match File::open(&path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Warning: could not open {:?}: {e}", path);
                return;
            }
        };
        let source = match rodio::Decoder::new(BufReader::new(file)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Warning: could not decode {:?}: {e}", path);
                return;
            }
        };

        // Loop the original theme like classic single-track BGM
        self.sink.append(source.repeat_infinite());
        if !self.enabled {
            self.sink.pause();
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

fn resolve_track(track: MusicTrack) -> Option<PathBuf> {
    track.candidates().into_iter().find(|c| c.exists())
}

pub fn audio_assets_present() -> bool {
    Path::new("resources/track.mp3").exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_track_is_first_for_title() {
        let c = MusicTrack::Title.candidates();
        assert_eq!(c[0], PathBuf::from("resources/track.mp3"));
    }
}
