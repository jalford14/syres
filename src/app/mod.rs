use std::collections::HashMap;

use chrono::{Local, NaiveDate, NaiveTime, TimeDelta};
use serde_json::Value;
use crate::credentials::{self, Credentials};
use crate::event::EventHandler;
use crate::maps::{self, FloorMap};
use crate::skedda::{self, AvailableSlot, Skedda, TimeIncrement};
use crate::ui;

use ratatui::{
    DefaultTerminal,
    widgets::{ListItem, ListState},
};

mod input;

const MAIN_MENU_ITEMS: [&str; 2] = [
    "Create a booking",
    "Manage bookings",
];

pub(crate) const LOCATIONS: [&str; 13] = [
    "Adair Park",
    "Avondale Estates",
    "Buckhead",
    "Cabbagetown",
    "Chamblee",
    "Decatur",
    "Downtown",
    "Midtown",
    "Old Fourth Ward",
    "Roswell",
    "Virginia-Highland",
    "Westside",
    "Toco Hills",
];

#[derive(Debug, Clone, PartialEq)]
pub enum ViewState {
    Login,
    MainMenu,
    LocationSelection,
    BookingManager,
    BookingForm,
    Confirmation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoginMode {
    EmailPassword,
    SessionCookie,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoginField {
    Username,
    Password,
    Cookie,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BookingFocus {
    Spaces,
    DateSelection,
    TimeSlots,
    TitleInput,
}

/// Application.
pub struct App<'a> {
    pub running: bool,
    //Main menu
    pub main_menu_items: Vec<ListItem<'a>>,
    pub main_menu_list_state: ListState,
    // Locations
    pub locations: Vec<ListItem<'a>>,
    pub events: EventHandler,
    pub location_list_state: ListState,
    pub spaces_list_state: ListState,
    pub timeslots_list_state: ListState,
    pub current_view: ViewState,
    pub selected_location: Option<String>,
    pub venue_space_ids: HashMap<String, String>,
    pub selected_location_space_ids: Vec<String>,
    pub available_slots: Vec<AvailableSlot>,
    pub selected_space_id: Option<String>,
    pub availability_date: String,
    // Manage bookings
    pub user_bookings: Vec<Value>,
    pub user_booking_list_state: ListState,
    pub confirm_delete: bool,
    // Booking title
    pub booking_title: String,
    // Time slot selection
    pub time_increments: Vec<TimeIncrement>,
    pub booking_focus: BookingFocus,
    pub selection_duration: usize,
    pub week_dates: Vec<NaiveDate>,
    pub week_availability: HashMap<String, Vec<AvailableSlot>>,
    pub date_list_state: ListState,
    pub booking_error: Option<String>,
    // Login fields
    pub login_mode: LoginMode,
    pub username_input: String,
    pub password_input: String,
    pub cookie_input: String,
    pub login_field_focus: LoginField,
    pub auth_error: Option<String>,
    // Persistent authenticated client
    pub skedda: Option<Skedda>,
    // Map data
    pub floor_maps: Vec<FloorMap>,
    pub floor_map_index: Option<usize>,
}

impl Default for App<'_> {
    fn default() -> Self {
        Self {
            running: true,
            main_menu_items: MAIN_MENU_ITEMS
                .iter()
                .map(|&s| ListItem::new(s.to_string()))
                .collect(),
            main_menu_list_state: ListState::default().with_selected(Some(0)),
            user_bookings: Vec::new(),
            user_booking_list_state: ListState::default().with_selected(Some(0)),
            confirm_delete: false,
            booking_title: String::new(),
            locations: LOCATIONS
                .iter()
                .map(|&s| ListItem::new(s.to_string()))
                .collect(),
            events: EventHandler::new(),
            location_list_state: ListState::default().with_selected(Some(0)),
            spaces_list_state: ListState::default(),
            timeslots_list_state: ListState::default(),
            current_view: ViewState::Login,
            selected_location: None,
            selected_location_space_ids: Vec::new(),
            venue_space_ids: HashMap::new(),
            available_slots: Vec::new(),
            selected_space_id: None,
            availability_date: String::new(),
            time_increments: Vec::new(),
            booking_focus: BookingFocus::Spaces,
            selection_duration: 4,
            week_dates: Vec::new(),
            week_availability: HashMap::new(),
            date_list_state: ListState::default(),
            booking_error: None,
            login_mode: LoginMode::EmailPassword,
            username_input: String::new(),
            password_input: String::new(),
            cookie_input: String::new(),
            login_field_focus: LoginField::Username,
            auth_error: None,
            skedda: None,
            floor_maps: Vec::new(),
            floor_map_index: None,
        }
    }
}

impl App<'_> {
    pub fn new() -> Self {
        let mut app = Self::default();
        app.floor_maps = maps::load_all_maps();

        if let Ok(Some(creds)) = credentials::load_credentials() {
            match creds {
                Credentials::Password { username, password } => {
                    if let Ok(mut skedda) = Skedda::new() {
                        if skedda.authenticate(&username, &password).is_ok() {
                            app.skedda = Some(skedda);
                            app.current_view = ViewState::MainMenu;
                        } else {
                            let _ = credentials::clear_credentials();
                            app.current_view = ViewState::Login;
                        }
                    }
                }
                Credentials::Cookie { cookie } => {
                    if let Ok(mut skedda) = Skedda::new() {
                        if skedda.authenticate_with_cookie(&cookie).is_ok() {
                            app.skedda = Some(skedda);
                            app.current_view = ViewState::MainMenu;
                        } else {
                            let _ = credentials::clear_credentials();
                            app.current_view = ViewState::Login;
                        }
                    }
                }
            }
        }

        app
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    pub fn render(&mut self, frame: &mut ratatui::Frame) {
        ui::render(self, frame);
    }

    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.running = false;
    }

    fn initialize_week_dates(&mut self) {
        let today = Local::now().date_naive();
        self.week_dates = (0..7)
            .map(|i| today + TimeDelta::days(i))
            .collect();
        self.date_list_state.select(Some(0));
    }

    fn recalculate_availability(&mut self) {
        if let Some(selected_idx) = self.spaces_list_state.selected() {
            if let Some(space_id) = self.selected_location_space_ids.get(selected_idx).cloned() {
                self.selected_space_id = Some(space_id.clone());
                self.week_availability.clear();

                if let Some(ref mut skedda) = self.skedda {
                    for date in &self.week_dates {
                        let date_str = date.format("%Y-%m-%d").to_string();
                        let bookings = skedda.fetch_bookings(&date_str).unwrap_or_default();
                        let slots =
                            Skedda::calculate_availability(&space_id, &date_str, &bookings);
                        self.week_availability.insert(date_str, slots);
                    }
                }

                self.available_slots.clear();
                self.time_increments.clear();
                self.timeslots_list_state.select(None);
            }
        }
    }

    fn select_date(&mut self) {
        if let Some(selected) = self.date_list_state.selected() {
            if let Some(date) = self.week_dates.get(selected) {
                let date_str = date.format("%Y-%m-%d").to_string();
                self.availability_date = date_str.clone();

                self.available_slots = self
                    .week_availability
                    .get(&date_str)
                    .cloned()
                    .unwrap_or_default();

                self.time_increments = skedda::generate_time_increments(&self.available_slots);

                if self.time_increments.is_empty() {
                    self.timeslots_list_state.select(None);
                } else {
                    self.timeslots_list_state.select(Some(0));
                    self.selection_duration = 4;
                }
                self.clamp_duration();
                self.booking_focus = BookingFocus::TimeSlots;
            }
        }
    }

    fn clamp_duration(&mut self) {
        if let Some(cursor) = self.timeslots_list_state.selected() {
            if cursor < self.time_increments.len() {
                let current_block = self.time_increments[cursor].block_index;
                let max_in_block = self.time_increments[cursor..]
                    .iter()
                    .take_while(|inc| inc.block_index == current_block)
                    .count();
                if self.selection_duration > max_in_block {
                    self.selection_duration = max_in_block;
                }
                if self.selection_duration < 1 {
                    self.selection_duration = 1;
                }
            }
        }
    }

    pub fn selected_time_range(&self) -> Option<(NaiveTime, NaiveTime)> {
        let cursor = self.timeslots_list_state.selected()?;
        let start_inc = self.time_increments.get(cursor)?;
        let end_idx = cursor + self.selection_duration - 1;
        let end_inc = self.time_increments.get(end_idx)?;
        if end_inc.block_index != start_inc.block_index {
            return None;
        }
        let start = start_inc.time;
        let end = end_inc.time + TimeDelta::minutes(15);
        Some((start, end))
    }

    fn submit_booking(&mut self) {
        self.booking_error = None;
        if let Some((start_time, end_time)) = self.selected_time_range() {
            let space_id = self.selected_space_id.clone().unwrap_or_default();
            let date = self.availability_date.clone();
            let title = if self.booking_title.is_empty() {
                "Booking"
            } else {
                &self.booking_title
            };
            if let Some(ref mut skedda) = self.skedda {
                match skedda.create_booking(&space_id, &date, &start_time, &end_time, title) {
                    Ok(()) => {
                        self.current_view = ViewState::Confirmation;
                    }
                    Err(e) => {
                        self.booking_error = Some(e.to_string());
                    }
                }
            }
        } else {
            self.booking_error = Some("Invalid time selection".to_string());
        }
    }

    pub fn current_floor_map(&self) -> Option<&FloorMap> {
        self.floor_map_index.and_then(|i| self.floor_maps.get(i))
    }
}
