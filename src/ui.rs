use ratatui::{
    style::Stylize,
    symbols::Marker,
    widgets::{Axis, Chart, Dataset, GraphType},
    Frame,
};

use crate::app::App;

pub fn render(app: &mut App, frame: &mut Frame) {
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

    let area = frame.size();
    frame.render_widget(
        Chart::new(vec![dataset])
            .red()
            .x_axis(x_axis)
            .y_axis(y_axis),
        area,
    );
}
