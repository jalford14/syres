use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, BorderType, Clear, List, Paragraph},
};

use crate::app::{App, LoginField, ViewState};
use crate::skedda::AvailableSlot;

/// Renders the user interface.
pub fn render(app: &mut App, frame: &mut Frame) {
    match app.current_view {
        ViewState::Login => render_login(app, frame),
        ViewState::LocationSelection => render_location_selection(app, frame),
        ViewState::BookingForm => render_booking_form(app, frame),
        ViewState::Confirmation => render_confirmation(app, frame),
    }
}

fn render_login(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    let popup_area = centered_rect(50, 50, area);

    frame.render_widget(Clear, popup_area);

    let block = Block::bordered()
        .title("syres - Login to Skedda")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // spacer
            Constraint::Length(1), // username label
            Constraint::Length(1), // username input
            Constraint::Length(1), // spacer
            Constraint::Length(1), // password label
            Constraint::Length(1), // password input
            Constraint::Length(1), // spacer
            Constraint::Length(2), // error message
            Constraint::Min(1),   // help text
        ])
        .split(inner);

    // Username label
    let username_style = if app.login_field_focus == LoginField::Username {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    let username_label = Paragraph::new("Username (email):").style(username_style);
    frame.render_widget(username_label, chunks[1]);

    // Username input
    let username_display = if app.login_field_focus == LoginField::Username {
        format!("{}|", app.username_input)
    } else {
        app.username_input.clone()
    };
    let username_input = Paragraph::new(username_display)
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(username_input, chunks[2]);

    // Password label
    let password_style = if app.login_field_focus == LoginField::Password {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    let password_label = Paragraph::new("Password:").style(password_style);
    frame.render_widget(password_label, chunks[4]);

    // Password input (masked)
    let masked: String = "*".repeat(app.password_input.len());
    let password_display = if app.login_field_focus == LoginField::Password {
        format!("{}|", masked)
    } else {
        masked
    };
    let password_input = Paragraph::new(password_display)
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(password_input, chunks[5]);

    // Error message
    if let Some(ref err) = app.auth_error {
        let error_msg = Paragraph::new(err.as_str())
            .style(Style::default().fg(Color::Red));
        frame.render_widget(error_msg, chunks[7]);
    }

    // Help text
    let help = Paragraph::new("Tab to switch fields | Enter to login | Esc to quit")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[8]);
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

fn format_slot(s: &AvailableSlot) -> String {
    let t1 = s.start.split('T').nth(1).unwrap_or(&s.start);
    let t2 = s.end.split('T').nth(1).unwrap_or(&s.end);
    format!("{} - {}", t1, t2)
}

fn render_booking_form(app: &mut App, frame: &mut Frame) {
    let spaces_list = List::new(
        app.selected_location_space_ids
            .iter()
            .map(|space_id| {
                app.venue_space_ids
                    .get(space_id)
                    .cloned()
                    .unwrap_or_else(|| space_id.clone())
            })
            .collect::<Vec<_>>(),
    )
    .block(
        Block::default()
            .title("Spaces")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded),
    )
    .highlight_style(Color::Yellow)
    .highlight_symbol(">> ");

    let space_name = app
        .selected_space_id
        .as_ref()
        .and_then(|id| app.venue_space_ids.get(id))
        .cloned()
        .unwrap_or_else(|| app.selected_space_id.clone().unwrap_or_default());
    let avail_title = if app.availability_date.is_empty() {
        "Available times".to_string()
    } else {
        format!("Available: {} on {}", space_name, app.availability_date)
    };
    let slot_lines: Vec<_> = if app.available_slots.is_empty() {
        vec!["No availability data".to_string()]
    } else {
        app.available_slots.iter().map(format_slot).collect()
    };
    let availability_list = List::new(slot_lines)
        .block(
            Block::default()
                .title(avail_title)
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded),
        );

    let area = frame.area();
    let popup_area = centered_rect(80, 70, area);

    frame.render_widget(Clear, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Min(5),
            Constraint::Min(5),
        ])
        .split(popup_area);

    let title = format!(
        "Booking Form - {}",
        app.selected_location
            .as_ref()
            .unwrap_or(&"Unknown".to_string())
    );

    let block = Block::bordered()
        .title(title)
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);

    let content = vec![
        Line::from(""),
        Line::from("Select a space. Available times (for the first space) are shown below."),
        Line::from(""),
        Line::from("Press Enter to confirm booking"),
        Line::from("Press Esc to go back"),
    ];

    let paragraph = Paragraph::new(Text::from(content))
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, chunks[0]);
    frame.render_stateful_widget(spaces_list, chunks[1], &mut app.list_state);
    frame.render_widget(availability_list, chunks[2]);
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
        Line::from(format!(
            "Your booking at {} has been confirmed!",
            app.selected_location
                .as_ref()
                .unwrap_or(&"Unknown".to_string())
        )),
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
