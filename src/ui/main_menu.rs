use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    text::Line,
    widgets::{Block, BorderType, Clear, List, ListItem},
};

use crate::app::App;
use crate::theme;

pub(super) fn render_main_menu(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    let popup_area = super::centered_rect(40, 25, area);

    frame.render_widget(Clear, popup_area);

    let block = Block::bordered()
        .title("syres")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(theme::title_border());

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let item_count = 2u16;
    let top_pad = inner.height.saturating_sub(item_count) / 2;

    let chunks = Layout::vertical([
        Constraint::Length(top_pad),
        Constraint::Length(item_count),
        Constraint::Min(0),
    ])
    .split(inner);

    let items: Vec<ListItem> = ["Create a booking", "Manage bookings"]
        .iter()
        .map(|&s| ListItem::new(Line::from(s).alignment(Alignment::Center)))
        .collect();

    let list = List::new(items).highlight_style(theme::selected_item());

    frame.render_stateful_widget(list, chunks[1], &mut app.main_menu_list_state);
}
