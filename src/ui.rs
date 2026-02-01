use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Masked, Span, Text},
    widgets::{Block, BorderType, Clear, List, Paragraph, Tabs},
};

use crate::app::{App, LoginField, LoginMode, ViewState};
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
    let popup_area = centered_rect(60, 80, area);

    frame.render_widget(Clear, popup_area);

    let outer_block = Block::bordered()
        .title(" syres ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Blue));

    let inner = outer_block.inner(popup_area);
    frame.render_widget(outer_block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // spacer
            Constraint::Length(3), // tabs (with border)
            Constraint::Length(1), // spacer
            Constraint::Length(9), // form fields
            Constraint::Length(1), // spacer
            Constraint::Length(2), // error
            Constraint::Min(1),   // help
        ])
        .split(inner);

    // -- Mode tabs --
    let tab_titles = vec![" Email/Password ", " Session Cookie "];
    let mode_index = match app.login_mode {
        LoginMode::EmailPassword => 0,
        LoginMode::SessionCookie => 1,
    };
    let tab_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));
    let tabs = Tabs::new(tab_titles)
        .block(tab_block)
        .select(mode_index)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .divider("\u{2502}");
    frame.render_widget(tabs, chunks[1]);

    // -- Form fields --
    match app.login_mode {
        LoginMode::EmailPassword => render_email_password_form(app, frame, chunks[3]),
        LoginMode::SessionCookie => render_cookie_form(app, frame, chunks[3]),
    }

    // -- Error message --
    if let Some(ref err) = app.auth_error {
        let error_msg = Paragraph::new(err.as_str())
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        frame.render_widget(error_msg, chunks[5]);
    }

    // -- Help text --
    let key_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(Color::DarkGray);
    let help_lines = vec![
        Line::from(vec![
            Span::styled("\u{2190}/\u{2192}", key_style),
            Span::styled(" switch mode  ", desc_style),
            Span::styled("Tab", key_style),
            Span::styled(" next field  ", desc_style),
            Span::styled("Enter", key_style),
            Span::styled(" login", desc_style),
        ]),
        Line::from(vec![Span::styled("Esc to quit", desc_style)]),
    ];
    let help = Paragraph::new(help_lines).alignment(Alignment::Center);
    frame.render_widget(help, chunks[6]);
}

fn render_email_password_form(app: &App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // email label
            Constraint::Length(3), // email input
            Constraint::Length(1), // spacer
            Constraint::Length(1), // password label
            Constraint::Length(3), // password input
        ])
        .split(area);

    // Email label
    let email_focused = app.login_field_focus == LoginField::Username;
    let email_label_style = if email_focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    frame.render_widget(
        Paragraph::new("  Email").style(email_label_style),
        chunks[0],
    );

    // Email input
    let email_border_style = if email_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let email_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(email_border_style);
    let email_display = if email_focused {
        format!("{}\u{2588}", app.username_input)
    } else {
        app.username_input.clone()
    };
    let email_input = Paragraph::new(email_display)
        .block(email_block)
        .style(Style::default().fg(Color::White));
    frame.render_widget(email_input, chunks[1]);

    // Password label
    let password_focused = app.login_field_focus == LoginField::Password;
    let password_label_style = if password_focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    frame.render_widget(
        Paragraph::new("  Password").style(password_label_style),
        chunks[3],
    );

    // Password input (masked)
    let password_border_style = if password_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let password_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(password_border_style);
    let password_input = if password_focused {
        let masked_val = Masked::new(app.password_input.as_str(), '\u{2022}')
            .value()
            .to_string();
        Paragraph::new(format!("{}\u{2588}", masked_val))
    } else {
        Paragraph::new(Text::from(Masked::new(
            app.password_input.as_str(),
            '\u{2022}',
        )))
    };
    let password_input = password_input
        .block(password_block)
        .style(Style::default().fg(Color::White));
    frame.render_widget(password_input, chunks[4]);
}

fn render_cookie_form(app: &App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // cookie label
            Constraint::Length(3), // cookie input
            Constraint::Length(1), // spacer
            Constraint::Min(1),   // instructions
        ])
        .split(area);

    // Cookie label
    let cookie_focused = app.login_field_focus == LoginField::Cookie;
    let cookie_label_style = if cookie_focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    frame.render_widget(
        Paragraph::new("  X-Skedda-ApplicationCookie").style(cookie_label_style),
        chunks[0],
    );

    // Cookie input
    let cookie_border_style = if cookie_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let cookie_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(cookie_border_style);

    let cookie_input = if cookie_focused {
        Paragraph::new(format!("{}\u{2588}", app.cookie_input))
    } else if app.cookie_input.is_empty() {
        Paragraph::new("")
    } else {
        let masked_val: String = Masked::new(app.cookie_input.as_str(), '\u{2022}')
            .value()
            .chars()
            .take(8)
            .collect();
        Paragraph::new(format!("{}  (cookie set)", masked_val))
    };
    let cookie_input = cookie_input
        .block(cookie_block)
        .style(Style::default().fg(Color::White));
    frame.render_widget(cookie_input, chunks[1]);

    let instructions = vec![
        Line::from(Span::styled(
            "  To get your session cookie:",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "  1. Log in at switchyards.skedda.com with Oauth",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "  2. Open Dev Tools \u{2192} Application \u{2192} Cookies",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "  3. Copy the X-Skedda-ApplicationCookie value",
            Style::default().fg(Color::DarkGray),
        )),
    ];
    frame.render_widget(Paragraph::new(instructions), chunks[3]);
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
