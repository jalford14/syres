use ratatui::{
    Frame,
    layout::{Alignment, Rect, Layout, Direction, Constraint},
    style::{Color, Stylize},
    widgets::{Block, BorderType, List, Paragraph, Clear},
    text::{Text, Line},
};

use crate::app::{App, ViewState};

/// Renders the user interface.
pub fn render(app: &mut App, frame: &mut Frame) {
    match app.current_view {
        ViewState::LocationSelection => render_location_selection(app, frame),
        ViewState::BookingForm => render_booking_form(app, frame),
        ViewState::Confirmation => render_confirmation(app, frame),
    }
}

fn render_location_selection(app: &mut App, frame: &mut Frame) {
    let locations_list = List::new(app.locations.clone())
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

fn render_booking_form(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    
    // Create a centered popup area
    let popup_area = centered_rect(60, 40, area);
    
    // Clear the background
    frame.render_widget(Clear, popup_area);
    
    let title = format!("Booking Form - {}", app.selected_location.as_ref().unwrap_or(&"Unknown".to_string()));
    
    let block = Block::bordered()
        .title(title)
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    
    let content = vec![
        Line::from(""),
        Line::from("Booking details will go here..."),
        Line::from(""),
        Line::from("Press Enter to confirm booking"),
        Line::from("Press Esc to go back"),
    ];
    
    let paragraph = Paragraph::new(Text::from(content))
        .block(block)
        .alignment(Alignment::Center);
    
    frame.render_widget(paragraph, popup_area);
}

fn render_confirmation(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    let popup_area = centered_rect(50, 30, area);
    
    frame.render_widget(Clear, popup_area);
    
    let block = Block::bordered()
        .title("Booking Confirmed!")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    
    let content = vec![
        Line::from(""),
        Line::from(format!("Your booking at {} has been confirmed!", 
            app.selected_location.as_ref().unwrap_or(&"Unknown".to_string()))),
        Line::from(""),
        Line::from("Press Esc to return to location selection"),
    ];
    
    let paragraph = Paragraph::new(Text::from(content))
        .block(block)
        .alignment(Alignment::Center);
    
    frame.render_widget(paragraph, popup_area);
}

/// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
