use core::f64;

use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;

type Entry = (f64, f64);

pub struct AppState {
    pub data: Vec<Entry>,
    pub min_val: f64,
    pub max_val: f64,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            data: vec![],
            min_val: f64::INFINITY,
            max_val: f64::NEG_INFINITY,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, new_val: f64) -> Result<()> {
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
    pub fn new() -> Self {
        Self::default()
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
