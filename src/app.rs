use core::f64;
use std::collections::HashMap;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::utils::RingBuffer;

type Entry = (f64, f64);

pub struct Timeseries {
    data_points: Vec<Entry>,
    min_val: f64,
    max_val: f64,
}

impl Timeseries {
    pub fn len(&self) -> usize {
        self.data_points.len()
    }

    pub fn get_min(&self) -> f64 {
        self.min_val
    }

    pub fn get_max(&self) -> f64 {
        self.max_val
    }

    pub fn get_data(&self) -> &Vec<Entry> {
        &self.data_points
    }
}

impl Default for Timeseries {
    fn default() -> Self {
        Self {
            data_points: Vec::new(),
            min_val: f64::INFINITY,
            max_val: f64::NEG_INFINITY,
        }
    }
}

pub struct AppState {
    pub data: HashMap<String, Timeseries>,
    pub passthrough: bool,
    pub ema_factor: f64,
    pub linebuf: RingBuffer<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            data: HashMap::new(),
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

    fn insert(&mut self, key: &str, new_val: f64) {
        let new_t = self.max_t(key);
        let entry = self.data.entry(key.into()).or_default();

        let new_val = match entry.data_points.last() {
            Some((_, val)) => self.ema_factor * new_val + (1.0 - self.ema_factor) * val,
            None => new_val,
        };

        entry.min_val = f64::min(entry.min_val, new_val);
        entry.max_val = f64::max(entry.max_val, new_val);

        entry.data_points.push((new_t, new_val));
    }

    pub fn max_t(&self, key: &str) -> f64 {
        let Some(data) = self.data.get(key.into()) else {
            return 0.0;
        };
        data.len() as f64
    }
}

/// Application
/// Contains all app state and logic
/// Shouldn't contain any gui related functions and state
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// Contains the whole state of the app (buffered lines, parsed values, etc)
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

    fn insert(&mut self, key: &str, new_val: f64) {
        self.state.insert(key, new_val);
    }

    pub fn process_line(&mut self, line: &str) {
        static PATTERN: Lazy<Regex> = Lazy::new(|| {
            Regex::new(
                r"(?i)(\b\w*?(?:loss|error|cost|acc)\b)[\s--\n]*:?[\s--\n]*(-?[0-9]+(?:\.[0-9]+)?(?:e-?[0-9]+)?)",
            )
            .unwrap()
        });

        self.state.linebuf.add(line.to_owned());

        for (_, [key, val]) in PATTERN.captures_iter(&line).map(|c| c.extract()) {
            let Ok(val) = val.parse() else { return };
            self.insert(key, val);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_vec(app: &mut App, test_lines: Vec<(&str, &str, f64, f64)>) {
        for (line, key, t_exp, val_exp) in test_lines {
            app.process_line(line);
            let (t, val) = app
                .state
                .data
                .get(key)
                .expect(&format!("No entry for key {}!\nLine was {}", key, line))
                .data_points
                .last()
                .expect("No value added");
            assert_eq!(*val, val_exp, "Wrong loss value for line {}", line);
            assert_eq!(*t, t_exp, "Wrong t value for line {}", line);
        }
    }

    #[test]
    fn simple_parse() {
        let mut app = App::new(5, 1.0);

        let test_lines = vec![
            ("loss 1.0", "loss", 0.0, 1.0),
            ("loss 2", "loss", 1.0, 2.0),
            ("Loss 3.1", "Loss", 0.0, 3.1),
            ("Loss 4", "Loss", 1.0, 4.0),
        ];

        test_vec(&mut app, test_lines);
    }

    #[test]
    fn skip_lines() {
        let mut app = App::new(5, 1.0);

        let test_lines = vec![
            ("loss 1.0", "loss", 0.0, 1.0),
            ("empty 2.0", "loss", 0.0, 1.0),
            ("loss 3.0", "loss", 1.0, 3.0),
        ];

        test_vec(&mut app, test_lines)
    }

    #[test]
    fn hard_parse() {
        let mut app = App::new(5, 1.0);

        let test_lines = vec![
            ("loss 0.0", "loss", 0.0, 0.0),
            ("loss 1.0, acc 2.0", "loss", 1.0, 1.0),
            ("MainLoss 2.0, AuxLoss 3.0, acc 2.0", "MainLoss", 0.0, 2.0),
            ("loss Loss loss Loss acc 4.0", "acc", 2.0, 4.0),
            ("Loss loss loss loss 120.0", "loss", 2.0, 120.0),
        ];

        test_vec(&mut app, test_lines);
    }

    #[test]
    fn multi_vals_per_line() {
        let mut app = App::new(5, 1.0);

        app.process_line("loss 0.0, acc 2.0, mainloss 3.0");
        app.process_line("loss 5.0, acc 2.0, loss 4.0");
        assert_eq!(
            app.state
                .data
                .get("loss")
                .expect("Key not in data")
                .data_points[0]
                .1,
            0.0
        );

        assert_eq!(
            app.state
                .data
                .get("acc")
                .expect("Key not in data")
                .data_points[0]
                .1,
            2.0
        );

        assert_eq!(
            app.state
                .data
                .get("mainloss")
                .expect("Key not in data")
                .data_points[0]
                .1,
            3.0
        );

        assert_eq!(
            app.state
                .data
                .get("loss")
                .expect("Key not in data")
                .data_points[1]
                .1,
            5.0
        );

        assert_eq!(
            app.state
                .data
                .get("acc")
                .expect("Key not in data")
                .data_points[1]
                .1,
            2.0
        );

        assert_eq!(
            app.state
                .data
                .get("loss")
                .expect("Key not in data")
                .data_points[2]
                .1,
            4.0
        );
    }

    #[test]
    fn whitespace() {
        let mut app = App::new(5, 1.0);

        let test_lines = vec![
            ("loss\t1.0", "loss", 0.0, 1.0),
            ("loss\n2.0", "loss", 0.0, 1.0),
            ("loss    3.0", "loss", 1.0, 3.0),
            ("loss:\t4.0", "loss", 2.0, 4.0),
        ];

        test_vec(&mut app, test_lines);
    }

    #[test]
    fn identifiers() {
        let mut app = App::new(5, 1.0);

        let test_lines = vec![
            ("loss 1.0", "loss", 0.0, 1.0),
            ("cost 2.0", "cost", 0.0, 2.0),
            ("error 3.0", "error", 0.0, 3.0),
            ("maincost 4.0", "maincost", 0.0, 4.0),
        ];

        test_vec(&mut app, test_lines);
    }

    #[test]
    fn scientific() {
        let mut app = App::new(5, 1.0);

        let test_lines = vec![
            ("loss 1e-2", "loss", 0.0, 1e-2),
            ("cost -2e3", "cost", 0.0, -2e3),
            ("error 1.5e-10", "error", 0.0, 1.5e-10),
            ("loss 1.6e12", "loss", 1.0, 1.6e12),
        ];

        // We are testing for hard equality!
        test_vec(&mut app, test_lines);
    }

    #[test]
    fn overflow() {
        let mut app = App::new(2, 1.0);

        app.process_line("loss 1.0");
        app.process_line("cost 2.0");
        app.process_line("error 3.0");

        let s = app
            .state
            .linebuf
            .peek()
            .expect("There should be an element in the buffer");
        assert_eq!(s, "cost 2.0");
    }
}
