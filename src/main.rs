use anyhow::Result;
use clap::Parser;
use crossterm::event::KeyCode;
use ezboard::{app::App, event::{Event, EventStream}, tui::Tui};
use ratatui::{backend::CrosstermBackend, Terminal};

use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    path: Option<PathBuf>,

    /// Render interval in milliseconds
    #[clap(long, default_value = "100")]
    render_interval: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    let backend = CrosstermBackend::new(std::io::stdout());
    let terminal = Terminal::new(backend)?;

    let event_stream = EventStream::new(args.render_interval, &args.path).await;
    
    let mut app = App::new();
    let mut tui = Tui::new(terminal, event_stream);
    tui.init()?;
    
    while app.running {
        let event = tui.event_stream.next().await;

        match event {
            Event::Tick => tui.draw(&mut app)?,
            Event::LineRead(line) => app.process_line(&line),
            Event::End => {
                app.quit();
                break;
            },
            Event::Key(key) => if key.code == KeyCode::Char('q') || key.code == KeyCode::Char('Q'){
                app.quit();
                break;
            }
        }
    }

    tui.exit()?;

    std::io::copy(&mut std::io::stdin(),&mut std::io::stdout())?;
    Ok(())
}
