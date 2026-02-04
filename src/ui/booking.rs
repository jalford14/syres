use chrono::{Local, TimeDelta};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, List, ListItem, Paragraph},
};

use crate::app::{App, BookingFocus};
use crate::map_ui;
use crate::skedda;
use crate::theme;

pub(super) fn render_booking_form(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    let popup_area = super::centered_rect(90, 85, area);

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
    if app.booking_focus == BookingFocus::TimeSlots || app.booking_focus == BookingFocus::TitleInput
    {
        render_timeslots_panel(app, frame, panel_chunks[1]);
    } else {
        render_date_picker_panel(app, frame, panel_chunks[1]);
    }

    // -- Right panel: Map --
    map_ui::render_map_panel(app, frame, panel_chunks[2]);

    // -- Status bar --
    render_booking_status(app, frame, outer_chunks[2]);

    // -- Title input overlay --
    if app.booking_focus == BookingFocus::TitleInput {
        render_title_input(app, frame, area);
    }
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

    let lines: Vec<Line> = if app.time_increments.is_empty() {
        vec![Line::from(Span::styled(
            "  No available times",
            theme::dim_text(),
        ))]
    } else {
        let mut result = Vec::new();
        let mut prev_block: Option<usize> = None;

        for (i, inc) in app.time_increments.iter().enumerate() {
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

    let visible_height = area.height.saturating_sub(2) as usize;
    let total_lines = lines.len();
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

fn render_title_input(app: &App, frame: &mut Frame, area: Rect) {
    let dialog = super::centered_rect(50, 20, area);
    frame.render_widget(Clear, dialog);

    let block = Block::bordered()
        .title(" Booking Title ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(theme::focused_border());

    let inner = block.inner(dialog);
    frame.render_widget(block, dialog);

    let inner_chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(inner);

    let prompt = Paragraph::new(Line::from(Span::styled(
        "What should this booking be called?",
        theme::dim_text(),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(prompt, inner_chunks[0]);

    let display = format!("{}_", app.booking_title);
    let input = Paragraph::new(Line::from(Span::styled(display, theme::body_text())))
        .alignment(Alignment::Center);
    frame.render_widget(input, inner_chunks[2]);
}

fn render_booking_status(app: &App, frame: &mut Frame, area: Rect) {
    let key_style = theme::key_hint();
    let desc_style = theme::dim_text();

    let mut lines = Vec::new();

    if let Some(ref err) = app.booking_error {
        lines.push(Line::from(Span::styled(
            err.as_str(),
            theme::error_text(),
        )));
    }

    let help = match app.booking_focus {
        BookingFocus::Spaces => Line::from(vec![
            Span::styled("\u{2191}/\u{2193}", key_style),
            Span::styled(" navigate  ", desc_style),
            Span::styled("Tab/Enter", key_style),
            Span::styled(" select date  ", desc_style),
            Span::styled("Esc", key_style),
            Span::styled(" back", desc_style),
        ]),
        BookingFocus::DateSelection => Line::from(vec![
            Span::styled("\u{2191}/\u{2193}", key_style),
            Span::styled(" navigate  ", desc_style),
            Span::styled("Enter", key_style),
            Span::styled(" select  ", desc_style),
            Span::styled("Tab/Esc", key_style),
            Span::styled(" spaces", desc_style),
        ]),
        BookingFocus::TimeSlots => Line::from(vec![
            Span::styled("\u{2191}/\u{2193}", key_style),
            Span::styled(" move  ", desc_style),
            Span::styled("\u{2190}/\u{2192}", key_style),
            Span::styled(" duration  ", desc_style),
            Span::styled("Enter", key_style),
            Span::styled(" continue  ", desc_style),
            Span::styled("Tab/Esc", key_style),
            Span::styled(" dates", desc_style),
        ]),
        BookingFocus::TitleInput => Line::from(vec![
            Span::styled("Enter", key_style),
            Span::styled(" book  ", desc_style),
            Span::styled("Esc", key_style),
            Span::styled(" back", desc_style),
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
