use std::collections::HashMap;

use chrono::{Local, NaiveTime, TimeDelta};
use crate::credentials::{self, Credentials};
use crate::event::{AppEvent, Event, EventHandler};
use crate::maps::{self, FloorMap};
use crate::skedda::{self, AvailableSlot, Skedda, TimeIncrement};
use crate::ui;

use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    widgets::{ListItem, ListState},
};

const LOCATIONS: [&str; 13] = [
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
    LocationSelection,
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
    TimeSlots,
}

/// Application.
pub struct App<'a> {
    pub running: bool,
    pub locations: Vec<ListItem<'a>>,
    pub events: EventHandler,
    pub location_list_state: ListState,
    pub spaces_list_state: ListState,
    pub timeslots_list_state: ListState,
    pub current_view: ViewState,
    pub selected_location: Option<String>,
    pub test_http: bool,
    pub venue_space_ids: HashMap<String, String>,
    pub selected_location_space_ids: Vec<String>,
    pub available_slots: Vec<AvailableSlot>,
    pub selected_space_id: Option<String>,
    pub availability_date: String,
    // Time slot selection
    pub time_increments: Vec<TimeIncrement>,
    pub booking_focus: BookingFocus,
    pub selection_duration: usize,
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
            test_http: false,
            selected_location_space_ids: Vec::new(),
            venue_space_ids: HashMap::new(),
            available_slots: Vec::new(),
            selected_space_id: None,
            availability_date: String::new(),
            time_increments: Vec::new(),
            booking_focus: BookingFocus::Spaces,
            selection_duration: 4,
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
                            app.current_view = ViewState::LocationSelection;
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
                            app.current_view = ViewState::LocationSelection;
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

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    pub fn handle_events(&mut self) -> color_eyre::Result<()> {
        match self.events.next()? {
            Event::Tick => self.tick(),
            Event::Crossterm(event) => match event {
                crossterm::event::Event::Key(key_event) => self.handle_key_event(key_event)?,
                crossterm::event::Event::Paste(text) => self.handle_paste(text)?,
                _ => {}
            },
            Event::App(app_event) => match app_event {
                AppEvent::Quit => self.quit()
            },
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        // Ctrl+C always quits
        if key_event.code == KeyCode::Char('c') && key_event.modifiers == KeyModifiers::CONTROL {
            self.events.send(AppEvent::Quit);
            return Ok(());
        }
        if key_event.code == KeyCode::Char('C') && key_event.modifiers == KeyModifiers::CONTROL {
            self.events.send(AppEvent::Quit);
            return Ok(());
        }

        if self.current_view == ViewState::Login {
            self.handle_login_key(key_event)?;
        } else {
            self.handle_app_key(key_event)?;
        }

        Ok(())
    }

    fn next_login_field(&self) -> LoginField {
        match self.login_mode {
            LoginMode::EmailPassword => match self.login_field_focus {
                LoginField::Username => LoginField::Password,
                LoginField::Password => LoginField::Username,
                LoginField::Cookie => LoginField::Username,
            },
            LoginMode::SessionCookie => LoginField::Cookie,
        }
    }

    fn prev_login_field(&self) -> LoginField {
        match self.login_mode {
            LoginMode::EmailPassword => match self.login_field_focus {
                LoginField::Username => LoginField::Password,
                LoginField::Password => LoginField::Username,
                LoginField::Cookie => LoginField::Password,
            },
            LoginMode::SessionCookie => LoginField::Cookie,
        }
    }

    /// Switch login mode and move focus to an appropriate field.
    fn switch_login_mode(&mut self) {
        self.login_mode = match self.login_mode {
            LoginMode::EmailPassword => LoginMode::SessionCookie,
            LoginMode::SessionCookie => LoginMode::EmailPassword,
        };
        // Move focus to the first field in the new mode
        self.login_field_focus = match self.login_mode {
            LoginMode::EmailPassword => LoginField::Username,
            LoginMode::SessionCookie => LoginField::Cookie,
        };
    }

    fn handle_login_key(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Esc => {
                self.events.send(AppEvent::Quit);
            }
            KeyCode::Left | KeyCode::Right => {
                self.switch_login_mode();
            }
            KeyCode::Tab => {
                self.login_field_focus = self.next_login_field();
            }
            KeyCode::BackTab => {
                self.login_field_focus = self.prev_login_field();
            }
            KeyCode::Char(c) => match self.login_field_focus {
                LoginField::Username => self.username_input.push(c),
                LoginField::Password => self.password_input.push(c),
                LoginField::Cookie => self.cookie_input.push(c),
            },
            KeyCode::Backspace => match self.login_field_focus {
                LoginField::Username => {
                    self.username_input.pop();
                }
                LoginField::Password => {
                    self.password_input.pop();
                }
                LoginField::Cookie => {
                    self.cookie_input.pop();
                }
            },
            KeyCode::Enter => {
                self.submit_login()?;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_paste(&mut self, text: String) -> color_eyre::Result<()> {
        if self.current_view == ViewState::Login {
            let trimmed = text.trim().to_string();
            match self.login_field_focus {
                LoginField::Username => self.username_input.push_str(&trimmed),
                LoginField::Password => self.password_input.push_str(&trimmed),
                LoginField::Cookie => self.cookie_input.push_str(&trimmed),
            }
        }
        Ok(())
    }

    fn submit_login(&mut self) -> color_eyre::Result<()> {
        self.auth_error = None;
        match self.login_mode {
            LoginMode::EmailPassword => {
                if self.username_input.is_empty() || self.password_input.is_empty() {
                    self.auth_error = Some("Email and password are required".to_string());
                    return Ok(());
                }
                match Skedda::new() {
                    Ok(mut skedda) => {
                        match skedda.authenticate(&self.username_input, &self.password_input) {
                            Ok(()) => {
                                let _ = credentials::save_password_credentials(
                                    &self.username_input,
                                    &self.password_input,
                                );
                                self.skedda = Some(skedda);
                                self.current_view = ViewState::LocationSelection;
                            }
                            Err(e) => {
                                self.auth_error = Some(e.to_string());
                            }
                        }
                    }
                    Err(e) => {
                        self.auth_error = Some(format!("Client error: {e}"));
                    }
                }
            }
            LoginMode::SessionCookie => {
                if self.cookie_input.is_empty() {
                    self.auth_error = Some("Session cookie is required".to_string());
                    return Ok(());
                }
                match Skedda::new() {
                    Ok(mut skedda) => {
                        match skedda.authenticate_with_cookie(&self.cookie_input) {
                            Ok(()) => {
                                let _ =
                                    credentials::save_cookie_credentials(&self.cookie_input);
                                self.skedda = Some(skedda);
                                self.current_view = ViewState::LocationSelection;
                            }
                            Err(e) => {
                                self.auth_error = Some(e.to_string());
                            }
                        }
                    }
                    Err(e) => {
                        self.auth_error = Some(format!("Client error: {e}"));
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_app_key(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match self.current_view {
            ViewState::LocationSelection => self.handle_location_key(key_event),
            ViewState::BookingForm => self.handle_booking_key(key_event),
            ViewState::Confirmation => self.handle_confirmation_key(key_event),
            ViewState::Login => Ok(()), // handled in handle_login_key
        }
    }

    fn handle_location_key(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.events.send(AppEvent::Quit);
            }
            KeyCode::Char('t') => {
                self.test_http = true;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let len = self.locations.len();
                let selected = self.location_list_state.selected().unwrap_or(0);
                let new = if selected == 0 {
                    len.saturating_sub(1)
                } else {
                    selected.saturating_sub(1)
                };
                self.location_list_state.select(Some(new));
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let len = self.locations.len();
                let selected = self.location_list_state.selected().unwrap_or(0);
                let new = if selected >= len.saturating_sub(1) {
                    0
                } else {
                    selected.saturating_add(1)
                };
                self.location_list_state.select(Some(new));
            }
            KeyCode::Enter => {
                if let Some(selected) = self.location_list_state.selected() {
                    if selected < self.locations.len() {
                        let location_name = LOCATIONS[selected];
                        self.selected_location = Some(location_name.to_string());
                        self.booking_focus = BookingFocus::Spaces;
                        self.selection_duration = 4;
                        self.booking_error = None;

                        if let Some(ref mut skedda) = self.skedda {
                            skedda.fetch_space_ids();
                            self.venue_space_ids = skedda.venue_space_ids.clone();
                            self.selected_location_space_ids =
                                skedda.fetch_location_space_ids(location_name);
                            self.availability_date =
                                Local::now().format("%Y-%m-%d").to_string();
                        } else {
                            self.venue_space_ids.clear();
                            self.selected_location_space_ids.clear();
                            self.available_slots = Vec::new();
                            self.selected_space_id = None;
                            self.availability_date.clear();
                        }

                        self.spaces_list_state.select(
                            if self.selected_location_space_ids.is_empty() {
                                None
                            } else {
                                Some(0)
                            },
                        );

                        // Load the floor map for this location
                        self.floor_map_index = self
                            .floor_maps
                            .iter()
                            .position(|m| m.name == location_name.replace('-', " "));

                        self.recalculate_availability();
                        self.current_view = ViewState::BookingForm;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_booking_key(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match self.booking_focus {
            BookingFocus::Spaces => match key_event.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    let len = self.selected_location_space_ids.len();
                    if len > 0 {
                        let sel = self.spaces_list_state.selected().unwrap_or(0);
                        let new = if sel == 0 { len - 1 } else { sel - 1 };
                        self.spaces_list_state.select(Some(new));
                        self.recalculate_availability();
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let len = self.selected_location_space_ids.len();
                    if len > 0 {
                        let sel = self.spaces_list_state.selected().unwrap_or(0);
                        let new = if sel >= len - 1 { 0 } else { sel + 1 };
                        self.spaces_list_state.select(Some(new));
                        self.recalculate_availability();
                    }
                }
                KeyCode::Tab | KeyCode::Enter => {
                    if !self.time_increments.is_empty() {
                        self.booking_focus = BookingFocus::TimeSlots;
                    }
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.current_view = ViewState::LocationSelection;
                    self.selected_location = None;
                    self.booking_error = None;
                }
                _ => {}
            },
            BookingFocus::TimeSlots => match key_event.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    let len = self.time_increments.len();
                    if len > 0 {
                        let sel = self.timeslots_list_state.selected().unwrap_or(0);
                        let new = if sel == 0 { len - 1 } else { sel - 1 };
                        self.timeslots_list_state.select(Some(new));
                        self.clamp_duration();
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let len = self.time_increments.len();
                    if len > 0 {
                        let sel = self.timeslots_list_state.selected().unwrap_or(0);
                        let new = if sel >= len - 1 { 0 } else { sel + 1 };
                        self.timeslots_list_state.select(Some(new));
                        self.clamp_duration();
                    }
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    if self.selection_duration > 1 {
                        self.selection_duration -= 1;
                    }
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    self.selection_duration += 1;
                    self.clamp_duration();
                }
                KeyCode::Enter => {
                    self.submit_booking();
                }
                KeyCode::Esc | KeyCode::Tab => {
                    self.booking_focus = BookingFocus::Spaces;
                }
                KeyCode::Char('q') => {
                    self.current_view = ViewState::LocationSelection;
                    self.selected_location = None;
                    self.booking_error = None;
                }
                _ => {}
            },
        }
        Ok(())
    }

    fn handle_confirmation_key(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') => {
                self.current_view = ViewState::LocationSelection;
                self.selected_location = None;
            }
            _ => {}
        }
        Ok(())
    }

    /// Recalculate availability for the currently selected space from cached bookings.
    fn recalculate_availability(&mut self) {
        if let Some(selected_idx) = self.spaces_list_state.selected() {
            if let Some(space_id) = self.selected_location_space_ids.get(selected_idx).cloned() {
                let date = self.availability_date.clone();
                if let Some(ref mut skedda) = self.skedda {
                    let bookings = skedda.fetch_bookings(&date).unwrap_or_default();
                    self.available_slots =
                        Skedda::calculate_availability(&space_id, &date, &bookings);
                }
                self.selected_space_id = Some(space_id);
                self.time_increments = skedda::generate_time_increments(&self.available_slots);
                if self.time_increments.is_empty() {
                    self.timeslots_list_state.select(None);
                } else {
                    self.timeslots_list_state.select(Some(0));
                }
                self.clamp_duration();
            }
        }
    }

    /// Ensure selection_duration doesn't cross block boundaries or exceed available slots.
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

    /// Compute the start and end NaiveTime of the current selection.
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

    /// Submit the booking via the API.
    fn submit_booking(&mut self) {
        self.booking_error = None;
        if let Some((start_time, end_time)) = self.selected_time_range() {
            let space_id = self.selected_space_id.clone().unwrap_or_default();
            let date = self.availability_date.clone();
            if let Some(ref mut skedda) = self.skedda {
                match skedda.create_booking(&space_id, &date, &start_time, &end_time, "Booking") {
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

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Renders the user interface.
    pub fn render(&mut self, frame: &mut ratatui::Frame) {
        ui::render(self, frame);
    }
}
