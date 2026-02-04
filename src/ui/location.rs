use ratatui::{
    Frame,
    layout::Alignment,
    style::Stylize,
    widgets::{Block, BorderType, Clear, List, Paragraph},
};

use crate::app::App;
use crate::theme;

pub(super) fn render_location_selection(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    let popup_area = super::centered_rect(50, 60, area);

    frame.render_widget(Clear, popup_area);

    let locations_list = List::new(app.locations.clone())
        .block(
            Block::bordered()
                .title("Locations")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(theme::selected_item());

    let block = Block::bordered()
        .title("syres")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(theme::title_border());

    let paragraph = Paragraph::new("Make a booking at Switchyards")
        .block(block)
        .fg(theme::LAMP)
        .centered();

    frame.render_widget(paragraph, popup_area);
    frame.render_stateful_widget(locations_list, popup_area, &mut app.location_list_state);
}
