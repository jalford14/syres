use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Stylize},
    widgets::{Block, BorderType, List, ListState, Paragraph},
};

use crate::app::App;

/// Renders the user interface.
pub fn render(app: &mut App, frame: &mut Frame) {
    let locations_list = List::new(app.locations.iter().map(|s| s.to_string()))
        .block(
            Block::default()
                .title("Locations")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(Color::Yellow)
        .highlight_symbol(">> ");

    let block = Block::bordered()
        .title("syres")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);

    let text = format!("Make a booking at Switchyards");

    let paragraph = Paragraph::new(text)
        .block(block)
        .fg(Color::Blue)
        .bg(Color::Black)
        .centered();

    frame.render_widget(paragraph, frame.area());
    frame.render_stateful_widget(locations_list, frame.area(), &mut app.list_state);
}
