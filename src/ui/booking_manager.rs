use chrono::NaiveTime;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, List, ListItem, Paragraph},
};

use crate::app::App;
use crate::theme;

fn to_12h(time_str: &str) -> String {
    NaiveTime::parse_from_str(time_str, "%H:%M:%S")
        .or_else(|_| NaiveTime::parse_from_str(time_str, "%H:%M"))
        .map(|t| t.format("%-I:%M %p").to_string())
        .unwrap_or_else(|_| time_str.to_string())
}

pub(super) fn render_booking_manager(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    let popup_area = super::centered_rect(70, 70, area);

    frame.render_widget(Clear, popup_area);

    let outer_block = Block::bordered()
        .title("Manage Bookings")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(theme::title_border());

    let inner_area = outer_block.inner(popup_area);
    frame.render_widget(outer_block, popup_area);

    let chunks = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(inner_area);

    // Build list items from bookings
    let items: Vec<ListItem> = app
        .user_bookings
        .iter()
        .map(|b| {
            let title = b["title"]
                .as_str()
                .filter(|s| !s.is_empty())
                .unwrap_or("No title");
            ListItem::new(title.to_string())
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::bordered()
                .title("Bookings")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded)
                .border_style(theme::focused_border()),
        )
        .highlight_style(theme::selected_item())
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, chunks[0], &mut app.user_booking_list_state);

    // Details panel
    let detail_lines = if let Some(selected) = app.user_booking_list_state.selected() {
        if let Some(booking) = app.user_bookings.get(selected) {
            let title = booking["title"]
                .as_str()
                .filter(|s| !s.is_empty())
                .unwrap_or("No title");

            let start = booking["start"].as_str().unwrap_or("—");
            let end = booking["end"].as_str().unwrap_or("—");

            let (date_str, start_time) = start.split_once('T').unwrap_or((start, ""));
            let (_, end_time) = end.split_once('T').unwrap_or((end, ""));

            // Look up space names
            let spaces_text = if let Some(spaces) = booking["spaces"].as_array() {
                let names: Vec<&str> = spaces
                    .iter()
                    .filter_map(|s| {
                        let id = s
                            .as_str()
                            .map(String::from)
                            .or_else(|| s.as_i64().map(|n| n.to_string()));
                        id.and_then(|id| app.venue_space_ids.get(&id).map(|s| s.as_str()))
                    })
                    .collect();
                if names.is_empty() {
                    "Unknown".to_string()
                } else {
                    names.join(", ")
                }
            } else {
                "Unknown".to_string()
            };

            vec![
                Line::from(vec![
                    Span::styled("Title: ", theme::dim_text()),
                    Span::styled(title, theme::body_text()),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Date:  ", theme::dim_text()),
                    Span::styled(date_str, theme::body_text()),
                ]),
                Line::from(vec![
                    Span::styled("Time:  ", theme::dim_text()),
                    Span::styled(format!("{} — {}", to_12h(start_time), to_12h(end_time)), theme::body_text()),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Space: ", theme::dim_text()),
                    Span::styled(spaces_text, theme::body_text()),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "Enter to cancel · Esc to go back",
                    theme::key_hint(),
                )),
            ]
        } else {
            vec![Line::from(Span::styled(
                "No booking selected",
                theme::dim_text(),
            ))]
        }
    } else if app.user_bookings.is_empty() {
        vec![Line::from(Span::styled(
            "No bookings found",
            theme::dim_text(),
        ))]
    } else {
        vec![Line::from(Span::styled(
            "Select a booking",
            theme::dim_text(),
        ))]
    };

    let details = Paragraph::new(detail_lines).block(
        Block::bordered()
            .title("Details")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .border_style(theme::unfocused_border()),
    );

    frame.render_widget(details, chunks[1]);

    if app.confirm_delete {
        let title = app
            .user_booking_list_state
            .selected()
            .and_then(|i| app.user_bookings.get(i))
            .and_then(|b| b["title"].as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("this booking");

        let confirm_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("Cancel \"{}\"?", title),
                theme::body_text(),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Enter to confirm · Esc to go back",
                theme::key_hint(),
            )),
        ];

        let confirm_area = super::centered_rect(40, 20, chunks[1]);
        frame.render_widget(Clear, confirm_area);
        let confirm_popup = Paragraph::new(confirm_text)
            .alignment(Alignment::Center)
            .block(
                Block::bordered()
                    .title("Confirm")
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Rounded)
                    .border_style(theme::title_border()),
            );
        frame.render_widget(confirm_popup, confirm_area);
    }
}
