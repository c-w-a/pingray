// pingray

use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyCode};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode};
use futures::StreamExt;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::stdout;
use tokio::sync::mpsc;

mod app;
mod config;
mod probe;
mod targets;
mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    // terminal setup
    enable_raw_mode()?;
    crossterm::execute!(stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal).await;

    // teardown — always runs
    disable_raw_mode()?;
    crossterm::execute!(stdout(), LeaveAlternateScreen)?;

    result
}

async fn run(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    let cfg = config::Config::default_canada();
    let group = &targets::CANADIAN_NTP;
    let mut app = app::App::new(group);

    let (tx, mut rx) = mpsc::unbounded_channel();
    probe::spawn_probe_loop(group.targets, tx, cfg.probe_timeout_ms, cfg.probe_interval_secs);

    let mut events = EventStream::new();

    // render initial frame
    terminal.draw(|f| ui::draw(f, &app, &cfg))?;

    loop {
        tokio::select! {
            Some(results) = rx.recv() => {
                app.update(results);
                terminal.draw(|f| ui::draw(f, &app, &cfg))?;
            }
            Some(Ok(event)) = events.next() => {
                if let Event::Key(key) = event {
                    if key.code == KeyCode::Char('q') {
                        break;
                    }
                }
                terminal.draw(|f| ui::draw(f, &app, &cfg))?;
            }
        }

        if !app.running {
            break;
        }
    }

    Ok(())
}
