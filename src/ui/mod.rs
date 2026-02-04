use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

use crate::app::{App, ViewState};
use crate::backdrop;

mod login;
mod main_menu;
mod location;
mod booking_manager;
mod booking;
mod confirmation;

pub fn render(app: &mut App, frame: &mut Frame) {
    // Permanent café backdrop behind all views
    let area = frame.area();
    backdrop::render_backdrop(frame, area);

    match app.current_view {
        ViewState::Login => login::render_login(app, frame),
        ViewState::MainMenu => main_menu::render_main_menu(app, frame),
        ViewState::BookingManager => booking_manager::render_booking_manager(app, frame),
        ViewState::LocationSelection => location::render_location_selection(app, frame),
        ViewState::BookingForm => booking::render_booking_form(app, frame),
        ViewState::Confirmation => confirmation::render_confirmation(app, frame),
    }
}

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
