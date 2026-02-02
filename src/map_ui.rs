use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{
        Block, BorderType, Paragraph,
        canvas::{Canvas, Rectangle as CanvasRect},
    },
};

use crate::app::App;

/// Render the map canvas into the given area, highlighting the rectangle
/// whose `space_id` matches `app.selected_space_id`.
pub fn render_map_panel(app: &App, frame: &mut Frame, area: Rect) {
    if let Some(map) = app.current_floor_map() {
        let vb_w = map.view_box_width;
        let vb_h = map.view_box_height;
        let rectangles = map.rectangles.clone();
        let venue_space_ids = app.venue_space_ids.clone();
        let selected_space_id = app.selected_space_id.clone();

        let canvas = Canvas::default()
            .block(
                Block::bordered()
                    .title(" Map ")
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .x_bounds([0.0, vb_w])
            .y_bounds([0.0, vb_h])
            .marker(Marker::Braille)
            .paint(move |ctx| {
                for rect in rectangles.iter() {
                    let is_selected = selected_space_id
                        .as_ref()
                        .map(|id| id == &rect.space_id)
                        .unwrap_or(false);

                    // SVG y-down to Canvas y-up
                    let canvas_y = vb_h - rect.y - rect.h;

                    let color = if is_selected {
                        Color::Yellow
                    } else {
                        Color::Cyan
                    };

                    ctx.draw(&CanvasRect {
                        x: rect.x,
                        y: canvas_y,
                        width: rect.w,
                        height: rect.h,
                        color,
                    });

                    // Label above the rectangle
                    let label_x = rect.x + rect.w / 2.0;
                    let label_y = canvas_y + rect.h + (vb_h * 0.015);

                    let space_name = venue_space_ids
                        .get(&rect.space_id)
                        .cloned()
                        .unwrap_or_else(|| rect.space_id.clone());
                    let short = shorten_space_name(&space_name);

                    let style = if is_selected {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    ctx.print(label_x, label_y, Line::from(Span::styled(short, style)));
                }
            });

        frame.render_widget(canvas, area);
    } else {
        let msg = Paragraph::new("No map data available")
            .alignment(Alignment::Center)
            .block(
                Block::bordered()
                    .title(" Map ")
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
        frame.render_widget(msg, area);
    }
}

fn shorten_space_name(name: &str) -> String {
    if let Some(idx) = name.find("Club Room") {
        let suffix = &name[idx + "Club Room".len()..];
        return format!("CR{}", suffix.trim_start_matches(' '));
    }
    if let Some(idx) = name.find("Phone Booth") {
        let suffix = &name[idx + "Phone Booth".len()..];
        return format!("PB{}", suffix.trim_start_matches(' '));
    }
    name.split_whitespace()
        .last()
        .unwrap_or(name)
        .to_string()
}
