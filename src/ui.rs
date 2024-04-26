use ratatui::{
    style::Stylize,
    symbols::Marker,
    text::Line,
    widgets::{Axis, Chart, Dataset, GraphType, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.size();

    if app.state.passthrough {
        let viewport_height = usize::max(area.as_size().height as usize - 1, 1);
        let lines: Vec<_> = app
            .state
            .linebuf
            .iter()
            .rev()
            .take(viewport_height)
            .rev()
            .cloned()
            .map(Line::from)
            .collect();
        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, area);
        return;
    }

    let dataset = Dataset::default()
        .name("Loss")
        .data(&app.state.data)
        .marker(Marker::Braille)
        .graph_type(GraphType::Line)
        .red();

    let time_step = app.state.max_t() - 1.0;
    let min_val = app.state.min_val;
    let max_val = app.state.max_val;

    let x_axis = Axis::default()
        .title("Step".red())
        .white()
        .bounds([0.0, time_step])
        .labels(vec!["0.0".into(), format!("{time_step}").into()]);

    let y_axis = Axis::default()
        .title("Loss".red())
        .white()
        .bounds([min_val, max_val])
        .labels(vec![
            format!("{min_val}").into(),
            format!("{max_val}").into(),
        ]);

    frame.render_widget(
        Chart::new(vec![dataset])
            .red()
            .x_axis(x_axis)
            .y_axis(y_axis),
        area,
    );
}
