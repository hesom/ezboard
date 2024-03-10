use anyhow::{Context, Result};
use clap::Parser;
use std::{
    fs::File,
    io::{stdin, BufRead, BufReader},
    path::PathBuf,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    path: Option<PathBuf>,
}

fn get_input_stream(Cli { path, .. }: &Cli) -> Result<Box<dyn BufRead>> {
    let reader: Box<dyn BufRead> = match path {
        Some(path) => path
            .is_file()
            .then(|| {
                Box::new(BufReader::new(
                    File::open(path)
                        .context("File exists, but still error reading file")
                        .unwrap(),
                ))
            })
            .context("Path is not a file")?,
        None => Box::new(BufReader::new(stdin())),
    };

    Ok(reader)
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let reader = get_input_stream(&args)?;

    for line in reader.lines() {
        println!("{}", line?);
    }

    Ok(())
}
