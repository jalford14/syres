use chrono::{Local, TimeDelta};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Stylize,
    text::{Line, Masked, Span, Text},
    widgets::{Block, BorderType, Clear, List, ListItem, Paragraph, Tabs},
};

use crate::app::{App, BookingFocus, LoginField, LoginMode, ViewState};
use crate::backdrop;
use crate::map_ui;
use crate::skedda;
use crate::theme;

pub fn render(app: &mut App, frame: &mut Frame) {
    // Permanent café backdrop behind all views
    let area = frame.area();
    backdrop::render_backdrop(frame, area);

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
        .border_style(theme::title_border());

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
        .border_style(theme::unfocused_border());
    let tabs = Tabs::new(tab_titles)
        .block(tab_block)
        .select(mode_index)
        .style(theme::tab_inactive())
        .highlight_style(theme::tab_highlight())
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
            .style(theme::error_text())
            .alignment(Alignment::Center);
        frame.render_widget(error_msg, chunks[5]);
    }

    // -- Help text --
    let key_style = theme::key_hint();
    let desc_style = theme::dim_text();
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
        theme::focused_label()
    } else {
        theme::unfocused_label()
    };
    frame.render_widget(
        Paragraph::new("  Email").style(email_label_style),
        chunks[0],
    );

    // Email input
    let email_border_style = if email_focused {
        theme::focused_border()
    } else {
        theme::unfocused_border()
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
        .style(theme::body_text());
    frame.render_widget(email_input, chunks[1]);

    // Password label
    let password_focused = app.login_field_focus == LoginField::Password;
    let password_label_style = if password_focused {
        theme::focused_label()
    } else {
        theme::unfocused_label()
    };
    frame.render_widget(
        Paragraph::new("  Password").style(password_label_style),
        chunks[3],
    );

    // Password input (masked)
    let password_border_style = if password_focused {
        theme::focused_border()
    } else {
        theme::unfocused_border()
    };
    let password_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(password_border_style);
    let password_input = if password_focused {
        let masked_val = Masked::new(app.password_input.as_str(), '\u{2022}')
            .value()
            .to_string();
        Paragraph::new(format!("{masked_val}\u{2588}"))
    } else {
        Paragraph::new(Text::from(Masked::new(
            app.password_input.as_str(),
            '\u{2022}',
        )))
    };
    let password_input = password_input
        .block(password_block)
        .style(theme::body_text());
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
        theme::focused_label()
    } else {
        theme::unfocused_label()
    };
    frame.render_widget(
        Paragraph::new("  X-Skedda-ApplicationCookie").style(cookie_label_style),
        chunks[0],
    );

    // Cookie input
    let cookie_border_style = if cookie_focused {
        theme::focused_border()
    } else {
        theme::unfocused_border()
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
        Paragraph::new(format!("{masked_val}  (cookie set)"))
    };
    let cookie_input = cookie_input
        .block(cookie_block)
        .style(theme::body_text());
    frame.render_widget(cookie_input, chunks[1]);

    let instructions = vec![
        Line::from(Span::styled(
            "  To get your session cookie:",
            theme::dim_text(),
        )),
        Line::from(Span::styled(
            "  1. Log in at switchyards.skedda.com with Oauth",
            theme::dim_text(),
        )),
        Line::from(Span::styled(
            "  2. Open Dev Tools \u{2192} Application \u{2192} Cookies",
            theme::dim_text(),
        )),
        Line::from(Span::styled(
            "  3. Copy the X-Skedda-ApplicationCookie value",
            theme::dim_text(),
        )),
    ];
    frame.render_widget(Paragraph::new(instructions), chunks[3]);
}

fn render_location_selection(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    let popup_area = centered_rect(50, 60, area);

    frame.render_widget(Clear, popup_area);

    let locations_list = List::new(app.locations.clone())
        .block(
            Block::bordered()
                .title("Locations")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(theme::selected_item())
        .highlight_symbol(">> ");

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

fn render_booking_form(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    let popup_area = centered_rect(90, 85, area);

    frame.render_widget(Clear, popup_area);

    // Vertical: title | panels | status
    let outer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // title
            Constraint::Min(8),    // panels
            Constraint::Length(3), // status bar
        ])
        .split(popup_area);

    // -- Title bar --
    render_booking_title(app, frame, outer_chunks[0]);

    // Horizontal split for three panels
    let panel_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(40),
            Constraint::Percentage(35),
        ])
        .split(outer_chunks[1]);

    // -- Left panel: Spaces --
    render_spaces_panel(app, frame, panel_chunks[0]);

    // -- Middle panel: Date picker or Time Slots --
    if app.booking_focus == BookingFocus::TimeSlots {
        render_timeslots_panel(app, frame, panel_chunks[1]);
    } else {
        render_date_picker_panel(app, frame, panel_chunks[1]);
    }

    // -- Right panel: Map --
    map_ui::render_map_panel(app, frame, panel_chunks[2]);

    // -- Status bar --
    render_booking_status(app, frame, outer_chunks[2]);
}

fn render_booking_title(app: &App, frame: &mut Frame, area: Rect) {
    let location = app.selected_location.as_deref().unwrap_or("Unknown");
    let title = if app.availability_date.is_empty() {
        format!(" Booking - {} ", location)
    } else {
        format!(" Booking - {} - {} ", location, app.availability_date)
    };
    let block = Block::bordered()
        .title(title)
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(theme::title_border());
    frame.render_widget(block, area);
}

fn render_spaces_panel(app: &mut App, frame: &mut Frame, area: Rect) {
    let focused = app.booking_focus == BookingFocus::Spaces;
    let border_style = if focused { theme::focused_border() } else { theme::unfocused_border() };

    let items: Vec<ListItem> = app
        .selected_location_space_ids
        .iter()
        .map(|space_id| {
            let name = app
                .venue_space_ids
                .get(space_id)
                .cloned()
                .unwrap_or_else(|| space_id.clone());
            ListItem::new(name)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::bordered()
                .title(" Spaces ")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded)
                .border_style(border_style),
        )
        .highlight_style(theme::selected_item())
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut app.spaces_list_state);
}

fn render_date_picker_panel(app: &mut App, frame: &mut Frame, area: Rect) {
    let focused = app.booking_focus == BookingFocus::DateSelection;
    let border_style = if focused {
        theme::focused_border()
    } else {
        theme::unfocused_border()
    };

    let today = Local::now().date_naive();

    let items: Vec<ListItem> = app
        .week_dates
        .iter()
        .map(|date| {
            let date_str = date.format("%Y-%m-%d").to_string();
            let date_label = if *date == today {
                format!("Today, {}", date.format("%b %-d"))
            } else {
                date.format("%a, %b %-d").to_string()
            };

            let availability_info =
                if let Some(slots) = app.week_availability.get(&date_str) {
                    if slots.is_empty() {
                        "fully booked".to_string()
                    } else {
                        let total_min = skedda::available_minutes(slots);
                        if total_min >= 60 && total_min % 60 == 0 {
                            format!("{}h available", total_min / 60)
                        } else if total_min >= 60 {
                            format!("{}h {}m available", total_min / 60, total_min % 60)
                        } else {
                            format!("{}m available", total_min)
                        }
                    }
                } else {
                    String::new()
                };

            let line = format!("  {:<16} {}", date_label, availability_info);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::bordered()
                .title(" Select Date ")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded)
                .border_style(border_style),
        )
        .highlight_style(theme::selected_item())
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut app.date_list_state);
}

fn render_timeslots_panel(app: &mut App, frame: &mut Frame, area: Rect) {
    let focused = app.booking_focus == BookingFocus::TimeSlots;
    let border_style = if focused { theme::focused_border() } else { theme::unfocused_border() };

    let cursor = app.timeslots_list_state.selected().unwrap_or(0);
    let duration = app.selection_duration;

    // Build the title showing the selected range
    let title = if let Some((start, end)) = app.selected_time_range() {
        let total_min = app.selection_duration * 15;
        let display = if total_min >= 60 && total_min % 60 == 0 {
            format!("{}h", total_min / 60)
        } else if total_min >= 60 {
            format!("{}h {}m", total_min / 60, total_min % 60)
        } else {
            format!("{total_min}m")
        };
        format!(
            " {} - {} ({}) ",
            start.format("%I:%M %p"),
            end.format("%I:%M %p"),
            display
        )
    } else {
        " Time Slots ".to_string()
    };

    // Build lines for each time increment
    let lines: Vec<Line> = if app.time_increments.is_empty() {
        vec![Line::from(Span::styled(
            "  No available times",
            theme::dim_text(),
        ))]
    } else {
        let mut result = Vec::new();
        let mut prev_block: Option<usize> = None;

        for (i, inc) in app.time_increments.iter().enumerate() {
            // Insert separator between blocks
            if let Some(pb) = prev_block {
                if inc.block_index != pb {
                    result.push(Line::from(Span::styled(
                        "  ---- booked ----",
                        theme::dim_text(),
                    )));
                }
            }
            prev_block = Some(inc.block_index);

            let end_time = inc.time + TimeDelta::minutes(15);
            let label = format!(
                "  {} - {}",
                inc.time.format("%I:%M %p"),
                end_time.format("%I:%M %p")
            );

            let cursor_block = app
                .time_increments
                .get(cursor)
                .map(|t| t.block_index);
            let in_selection = i >= cursor
                && i < cursor + duration
                && Some(inc.block_index) == cursor_block;

            let is_cursor_line = i == cursor;

            let style = if in_selection && focused {
                if is_cursor_line {
                    theme::time_selection_cursor()
                } else {
                    theme::time_selection()
                }
            } else if is_cursor_line && focused {
                theme::cursor_line()
            } else {
                theme::body_text()
            };

            result.push(Line::from(Span::styled(label, style)));
        }
        result
    };

    // Calculate scroll offset to keep cursor visible
    let visible_height = area.height.saturating_sub(2) as usize; // minus borders
    let total_lines = lines.len();
    // Account for separator lines when computing scroll position
    let separator_count_before_cursor: usize = {
        let mut count = 0;
        let mut prev: Option<usize> = None;
        for (i, inc) in app.time_increments.iter().enumerate() {
            if let Some(pb) = prev {
                if inc.block_index != pb {
                    count += 1;
                }
            }
            prev = Some(inc.block_index);
            if i == cursor {
                break;
            }
        }
        count
    };
    let visual_cursor = cursor + separator_count_before_cursor;
    let scroll_offset = if visible_height > 0 && visual_cursor >= visible_height {
        visual_cursor.saturating_sub(visible_height / 2)
    } else {
        0
    };
    let scroll_offset = scroll_offset.min(total_lines.saturating_sub(visible_height));

    let block = Block::bordered()
        .title(title)
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(border_style);

    let paragraph = Paragraph::new(lines)
        .block(block)
        .scroll((scroll_offset as u16, 0));

    frame.render_widget(paragraph, area);
}

fn render_booking_status(app: &App, frame: &mut Frame, area: Rect) {
    let key_style = theme::key_hint();
    let desc_style = theme::dim_text();

    let mut lines = Vec::new();

    // Show error if present
    if let Some(ref err) = app.booking_error {
        lines.push(Line::from(Span::styled(
            err.as_str(),
            theme::error_text(),
        )));
    }

    // Help text based on focus
    let help = match app.booking_focus {
        BookingFocus::Spaces => Line::from(vec![
            Span::styled("j/k", key_style),
            Span::styled(" navigate  ", desc_style),
            Span::styled("Tab/Enter", key_style),
            Span::styled(" select date  ", desc_style),
            Span::styled("Esc", key_style),
            Span::styled(" back", desc_style),
        ]),
        BookingFocus::DateSelection => Line::from(vec![
            Span::styled("j/k", key_style),
            Span::styled(" navigate  ", desc_style),
            Span::styled("Enter", key_style),
            Span::styled(" select  ", desc_style),
            Span::styled("Tab/Esc", key_style),
            Span::styled(" spaces", desc_style),
        ]),
        BookingFocus::TimeSlots => Line::from(vec![
            Span::styled("j/k", key_style),
            Span::styled(" move  ", desc_style),
            Span::styled("h/l", key_style),
            Span::styled(" duration  ", desc_style),
            Span::styled("Enter", key_style),
            Span::styled(" book  ", desc_style),
            Span::styled("Tab/Esc", key_style),
            Span::styled(" dates", desc_style),
        ]),
    };
    lines.push(help);

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(theme::unfocused_border());

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

fn render_confirmation(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    let popup_area = centered_rect(50, 30, area);

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
