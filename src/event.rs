use std::{path::PathBuf, pin::Pin, time::Duration};

use crossterm::event::{Event as CrosstermEvent, KeyEvent};
use futures::{FutureExt, StreamExt};
use tokio::{fs::File, io::{stdin, AsyncBufRead, AsyncBufReadExt, BufReader, Lines}, sync::mpsc};
use anyhow::Result;

/// Terminal events
#[derive(Debug, Clone)]
pub enum Event {
    /// Terminall tick
    Tick,
    /// Key press
    Key(KeyEvent),
    /// Input read and processed
    LineRead(String),
    ///
    End,
}

/// Combines terminal events, tick rate and io events in a single event stream.
#[allow(dead_code)]
pub struct EventStream {
    /// Event sender channel
    sender: mpsc::UnboundedSender<Event>,
    /// Event receiver channel
    receiver: mpsc::UnboundedReceiver<Event>,
    /// Event handler thread
    handler: tokio::task::JoinHandle<()>,
    /// Input stream
    input_stream: Lines<Pin<Box<dyn AsyncBufRead>>>
}

impl EventStream {
    /// Constructs a new instance of [`EventStream`].
    pub async fn new(tick_rate: u64, input_file_path: &Option<PathBuf>) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::unbounded_channel();
        let _sender = sender.clone();

        let input_file_path = input_file_path.clone();
        let input_stream = get_input_stream(&input_file_path).await.unwrap().lines();

        // Seperate thread for tick and key input events
        let handler = tokio::spawn(async move {
            let mut event_reader = crossterm::event::EventStream::new();
            let mut tick = tokio::time::interval(tick_rate);

            // main event loop
            // waits for the `next` event and sends it to whoever consumes the event stream
            loop {
                let tick_delay = tick.tick();
                let crossterm_event = event_reader.next().fuse();
                tokio::select! {
                    _ = _sender.closed() => {
                        break;
                    }
                    _ = tick_delay => {
                        _sender.send(Event::Tick).unwrap();
                    }
                    Some(Ok(evt)) = crossterm_event => {
                        match evt {
                            CrosstermEvent::Key(key) => {
                                if key.kind == crossterm::event::KeyEventKind::Press {
                                    _sender.send(Event::Key(key)).unwrap();
                                }
                            },
                            _ => (),
                        }
                    },
                };
            }
        });
        Self {
            sender,
            receiver,
            handler,
            input_stream,
        }
    }

    pub async fn next(&mut self) -> Event {
        // file io is done in main thread
        tokio::select! {
            Some(evt) = self.receiver.recv() => {
                evt
            },
            line = self.input_stream.next_line() => {
                match line {
                    Ok(Some(line)) => Event::LineRead(line),
                    _ => Event::End
                }
            }
        }
    }
}

async fn get_input_stream(path: &Option<PathBuf>) -> Result<Pin<Box<dyn AsyncBufRead>>> {
    let reader: Pin<Box<dyn AsyncBufRead>> = match path {
        Some(path) => {
            let f = File::open(path).await?;
            Box::pin(BufReader::new(f))
        }
        None => Box::pin(BufReader::new(stdin())),
    };

    Ok(reader)
}
