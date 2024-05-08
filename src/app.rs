use core::f64;

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

    pub fn insert(&mut self, new_val: f64) {
        let new_val = match self.data.last() {
            Some((_, val)) => self.ema_factor * new_val + (1.0 - self.ema_factor) * val,
            None => new_val,
        };

        self.min_val = f64::min(self.min_val, new_val);
        self.max_val = f64::max(self.max_val, new_val);

        let new_t = self.max_t();
        self.data.push((new_t, new_val));
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

    pub fn insert(&mut self, new_val: f64) {
        self.state.insert(new_val);
    }

    pub fn process_line(&mut self, line: &str) {
        static PATTERN: Lazy<Regex> = Lazy::new(|| {
            Regex::new(
                r"(?i)(\b\w*?(?:loss|error|cost)\b)[\s--\n]*:?[\s--\n]*([0-9]+(?:\.[0-9]+)?(?:e-?[0-9]+)?)",
            )
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

        self.insert(val);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_vec(app: &mut App, test_lines: Vec<(&str, f64, f64)>) {
        for (line, t_exp, val_exp) in test_lines {
            app.process_line(line);
            let (t, val) = app.state.data.last().expect("No value added");
            assert_eq!(*val, val_exp, "Wrong loss value for line {}", line);
            assert_eq!(*t, t_exp, "Wrong t value for line {}", line);
        }
    }

    #[test]
    fn simple_parse() {
        let mut app = App::new(5, 1.0);

        let test_lines = vec![
            ("loss 1.0", 0.0, 1.0),
            ("loss 2", 1.0, 2.0),
            ("Loss 3.1", 2.0, 3.1),
            ("Loss 4", 3.0, 4.0),
        ];

        test_vec(&mut app, test_lines);
    }

    #[test]
    fn skip_lines() {
        let mut app = App::new(5, 1.0);

        let test_lines = vec![
            ("loss 1.0", 0.0, 1.0),
            ("empty 2.0", 0.0, 1.0),
            ("loss 3.0", 1.0, 3.0),
        ];

        test_vec(&mut app, test_lines)
    }

    #[test]
    fn hard_parse() {
        let mut app = App::new(5, 1.0);

        let test_lines = vec![
            ("loss 0.0", 0.0, 0.0),
            ("loss 1.0, acc 2.0", 1.0, 1.0),
            ("MainLoss 2.0, AuxLoss 3.0, acc 2.0", 2.0, 2.0),
            ("loss Loss loss Loss acc 4.0", 2.0, 2.0),
            ("Loss loss loss loss 120.0", 3.0, 120.0),
        ];

        test_vec(&mut app, test_lines);
    }

    #[test]
    fn whitespace() {
        let mut app = App::new(5, 1.0);

        let test_lines = vec![
            ("loss\t1.0", 0.0, 1.0),
            ("loss\n2.0", 0.0, 1.0),
            ("loss    3.0", 1.0, 3.0),
            ("loss:\t4.0", 2.0, 4.0),
        ];

        test_vec(&mut app, test_lines);
    }

    #[test]
    fn identifiers() {
        let mut app = App::new(5, 1.0);

        let test_lines = vec![
            ("loss 1.0", 0.0, 1.0),
            ("cost 2.0", 1.0, 2.0),
            ("error 3.0", 2.0, 3.0),
            ("maincost 4.0", 3.0, 4.0),
        ];

        test_vec(&mut app, test_lines);
    }
}
