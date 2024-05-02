use core::f64;

use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::utils::RingBuffer;

type Entry = (f64, f64);

pub struct AppState {
    pub data: Vec<Entry>,
    pub min_val: f64,
    pub max_val: f64,
    pub passthrough: bool,
    pub ema_factor: f64,
    pub linebuf: RingBuffer<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            data: vec![],
            min_val: f64::INFINITY,
            max_val: f64::NEG_INFINITY,
            passthrough: false,
            ema_factor: 1.0,
            linebuf: RingBuffer::new(10),
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, new_val: f64) -> Result<()> {
        let new_val = match self.data.last() {
            Some((_, val)) => self.ema_factor * new_val + (1.0 - self.ema_factor) * val,
            None => new_val,
        };

        self.min_val = f64::min(self.min_val, new_val);
        self.max_val = f64::max(self.max_val, new_val);

        let new_t = self.max_t();
        self.data.push((new_t, new_val));

        Ok(())
    }

    pub fn max_t(&self) -> f64 {
        self.data.len() as f64
    }
}

/// Application
/// Contains all app state and logic
/// Shouldn't contain any gui related functions and state
pub struct App {
    /// Is the application running?
    pub running: bool,
    ///
    pub state: AppState,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            state: AppState::default(),
        }
    }
}

impl App {
    pub fn new(line_buffer_length: usize, ema_factor: f64) -> Self {
        App {
            running: true,
            state: AppState {
                linebuf: RingBuffer::new(line_buffer_length),
                ema_factor,
                ..Default::default()
            },
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn insert(&mut self, new_val: f64) -> Result<()> {
        self.state.insert(new_val)?;

        Ok(())
    }

    pub fn process_line(&mut self, line: &str) {
        static PATTERN: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"(?i)(\bloss\b|\berror\b|\bcost\b).*?([0-9]+(?:\.[0-9]+)?(?:e-?[0-9]+)?)")
                .unwrap()
        });

        self.state.linebuf.add(line.to_owned());

        let Some(cap) = PATTERN.captures(&line) else {
            return;
        };
        let Some(val) = cap.get(2) else { return };

        let Ok(val) = val.as_str().parse() else {
            return;
        };

        let _ = self.insert(val);
    }
}
