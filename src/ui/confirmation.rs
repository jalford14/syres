use ratatui::{
    Frame,
    layout::Alignment,
    text::{Line, Text},
    widgets::{Block, BorderType, Clear, Paragraph},
};

use crate::app::App;
use crate::theme;

pub(super) fn render_confirmation(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    let popup_area = super::centered_rect(50, 30, area);

    frame.render_widget(Clear, popup_area);

    let block = Block::bordered()
        .title(" Booking Confirmed! ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(theme::confirmation_border());

    let space_name = app
        .selected_space_id
        .as_ref()
        .and_then(|id| app.venue_space_ids.get(id))
        .cloned()
        .unwrap_or_else(|| "Unknown".to_string());

    let time_range = if let Some((start, end)) = app.selected_time_range() {
        format!(
            "{} - {}",
            start.format("%I:%M %p"),
            end.format("%I:%M %p")
        )
    } else {
        "N/A".to_string()
    };

    let content = vec![
        Line::from(""),
        Line::from(format!(
            "Your booking at {} has been confirmed!",
            app.selected_location
                .as_deref()
                .unwrap_or("Unknown")
        )),
        Line::from(""),
        Line::from(format!("Space: {space_name}")),
        Line::from(format!("Date: {}", app.availability_date)),
        Line::from(format!("Time: {time_range}")),
        Line::from(""),
        Line::from("Press Enter or Esc to return"),
    ];

    let paragraph = Paragraph::new(Text::from(content))
        .block(block)
        .style(theme::body_text())
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, popup_area);
}
