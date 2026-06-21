/// Keeps rodio's output stream alive for the whole process. Dropping the stream
/// stops playback, so this must outlive the game loop.
pub struct BackgroundMusic {
    _stream: rodio::OutputStream,
    _sink: rodio::Sink,
}

impl BackgroundMusic {
    /// Start background track if possible. Returns `None` so the game can run without audio.
    pub fn try_start() -> Option<Self> {
        use std::fs::File;
        use std::io::BufReader;

        let (stream, stream_handle) = match rodio::OutputStream::try_default() {
            Ok(pair) => pair,
            Err(e) => {
                eprintln!("Warning: no audio output device ({e}); continuing without music.");
                return None;
            }
        };

        let sink = match rodio::Sink::try_new(&stream_handle) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Warning: could not open audio sink ({e}); continuing without music.");
                return None;
            }
        };

        let song_file = match File::open("resources/track.mp3") {
            Ok(f) => f,
            Err(e) => {
                eprintln!(
                    "Warning: could not open resources/track.mp3 ({e}); continuing without music."
                );
                return None;
            }
        };

        let source = match rodio::Decoder::new(BufReader::new(song_file)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Warning: could not decode track.mp3 ({e}); continuing without music.");
                return None;
            }
        };

        sink.append(source);
        Some(Self {
            _stream: stream,
            _sink: sink,
        })
    }
}
