use anyhow::{Context, Result};
use clap::Parser;
use tokio::{
    fs::File,
    io::{stdin, AsyncBufRead, AsyncBufReadExt, BufReader},
};

use std::{path::PathBuf, pin::Pin};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    path: Option<PathBuf>,
}

async fn get_input_stream(Cli { path, .. }: &Cli) -> Result<Pin<Box<dyn AsyncBufRead>>> {
    let reader: Pin<Box<dyn AsyncBufRead>> = match path {
        Some(path) => {
            let f = File::open(path).await?;
            Box::pin(BufReader::new(f))
        }
        None => Box::pin(BufReader::new(stdin())),
    };

    Ok(reader)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    let reader = get_input_stream(&args).await?;

    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        println!("{}", line);
    }

    Ok(())
}
