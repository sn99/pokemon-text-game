/* MIT — sn99 */
mod app;

use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{self, Event};

use app::{draw, handle_key, App, AudioManager, MusicTrack};

fn run(
    mut terminal: ratatui::DefaultTerminal,
    mut app: App,
    audio: &mut Option<AudioManager>,
) -> Result<()> {
    loop {
        app.tick();
        if let Some(ref mut a) = audio {
            if let Some(v) = app.pending_audio_volume.take() {
                a.set_volume(v);
            }
            if let Some(en) = app.pending_audio_enabled.take() {
                a.set_enabled(en);
            }
        }

        terminal.draw(|frame| draw(&mut app, frame))?;

        if event::poll(Duration::from_millis(80))? {
            if let Event::Key(key) = event::read()? {
                handle_key(&mut app, key);
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;

    // Start original track.mp3 BGM (same as v1 behaviour)
    let mut audio = AudioManager::try_start();

    let terminal = ratatui::init();
    let app = App::new();
    if let Some(ref mut a) = audio {
        a.set_enabled(app.save.settings.music_enabled);
        a.set_volume(app.save.settings.music_volume);
        let _ = MusicTrack::Title; // keep import used for API surface
    }
    let result = run(terminal, app, &mut audio);
    ratatui::restore();
    result
}
