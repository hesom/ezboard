use anyhow::Result;
use clap::Parser;
use ezboard::event::{Event, EventStream};

use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    path: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    let mut event_stream = EventStream::new(1000, &args.path).await;
    
    loop {
        let event = event_stream.next().await;

        match event {
            Event::Tick => println!("TICK"),
            Event::LineRead(line) => println!("{}", line),
            Event::End => break,
            _ => ()
        }
    }

    Ok(())
}
