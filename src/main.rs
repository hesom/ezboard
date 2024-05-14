use anyhow::Result;
use clap::Parser;
use crossterm::event::KeyCode;
use ezboard::{
    app::App,
    event::{Event, EventStream},
    tui::Tui,
};
use ratatui::{backend::CrosstermBackend, Terminal};

use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    path: Option<PathBuf>,

    /// Render interval in milliseconds
    #[clap(long, default_value = "100")]
    render_interval: u64,

    /// Max number of buffered lines in passthrough mode
    #[clap(long, default_value = "100")]
    line_buffer_length: usize,

    /// Exponential moving average smoothing factor between 0 (constant) and 1 (no smoothing)
    #[clap(long, short, default_value = "1.0", value_parser = ranged_float)]
    ema_factor: f64,
}

fn ranged_float(s: &str) -> Result<f64, String> {
    let f: f64 = s.parse().map_err(|_| format!("`{s}` isn't a number"))?;
    if f < 0.0 || f > 1.0 {
        return Err("Only numbers between 0 and 1 are supported".into());
    }

    Ok(f)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    let backend = CrosstermBackend::new(std::io::stdout());
    let terminal = Terminal::new(backend)?;

    let event_stream = EventStream::new(args.render_interval, &args.path).await;

    let mut app = App::new(args.line_buffer_length, args.ema_factor);
    let mut tui = Tui::new(terminal, event_stream);
    tui.init()?;

    while app.running {
        let event = tui.event_stream.next().await;

        match event {
            Event::Tick => tui.draw(&mut app)?,
            Event::LineRead(line) => app.process_line(&line),
            Event::Key(key) => match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    app.quit();
                    break;
                }
                _ => app.handle_keypress(key.code),
            },
            _ => (),
        }
    }

    tui.exit()?;

    std::io::copy(&mut std::io::stdin(), &mut std::io::stdout())?;
    Ok(())
}
