use std::collections::HashMap;

use chrono::Local;
use crate::credentials;
use crate::event::{AppEvent, Event, EventHandler};
use crate::skedda::{AvailableSlot, Skedda};
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
pub enum LoginField {
    Username,
    Password,
}

/// Application.
pub struct App<'a> {
    pub running: bool,
    pub locations: Vec<ListItem<'a>>,
    pub events: EventHandler,
    pub list_state: ListState,
    pub current_view: ViewState,
    pub selected_location: Option<String>,
    pub test_http: bool,
    pub venue_space_ids: HashMap<String, String>,
    pub selected_location_space_ids: Vec<String>,
    pub available_slots: Vec<AvailableSlot>,
    pub selected_space_id: Option<String>,
    pub availability_date: String,
    // Login fields
    pub username_input: String,
    pub password_input: String,
    pub login_field_focus: LoginField,
    pub auth_error: Option<String>,
    // Persistent authenticated client
    pub skedda: Option<Skedda>,
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
            list_state: ListState::default().with_selected(Some(0)),
            current_view: ViewState::Login,
            selected_location: None,
            test_http: false,
            selected_location_space_ids: Vec::new(),
            venue_space_ids: HashMap::new(),
            available_slots: Vec::new(),
            selected_space_id: None,
            availability_date: String::new(),
            username_input: String::new(),
            password_input: String::new(),
            login_field_focus: LoginField::Username,
            auth_error: None,
            skedda: None,
        }
    }
}

impl App<'_> {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        let mut app = Self::default();

        // Try loading saved credentials
        if let Ok(Some(creds)) = credentials::load_credentials() {
            if let Ok(mut skedda) = Skedda::new() {
                if skedda.authenticate(&creds.username, &creds.password).is_ok() {
                    app.skedda = Some(skedda);
                    app.current_view = ViewState::LocationSelection;
                } else {
                    let _ = credentials::clear_credentials();
                    app.current_view = ViewState::Login;
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

    fn handle_login_key(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Esc => {
                self.events.send(AppEvent::Quit);
            }
            KeyCode::Tab | KeyCode::BackTab => {
                self.login_field_focus = match self.login_field_focus {
                    LoginField::Username => LoginField::Password,
                    LoginField::Password => LoginField::Username,
                };
            }
            KeyCode::Char(c) => {
                match self.login_field_focus {
                    LoginField::Username => self.username_input.push(c),
                    LoginField::Password => self.password_input.push(c),
                }
            }
            KeyCode::Backspace => {
                match self.login_field_focus {
                    LoginField::Username => { self.username_input.pop(); }
                    LoginField::Password => { self.password_input.pop(); }
                }
            }
            KeyCode::Enter => {
                self.auth_error = None;
                match Skedda::new() {
                    Ok(mut skedda) => {
                        match skedda.authenticate(&self.username_input, &self.password_input) {
                            Ok(()) => {
                                let _ = credentials::save_credentials(
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
                        self.auth_error = Some(format!("Client error: {}", e));
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_app_key(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => match self.current_view {
                ViewState::LocationSelection => self.events.send(AppEvent::Quit),
                ViewState::BookingForm | ViewState::Confirmation => {
                    self.current_view = ViewState::LocationSelection;
                    self.selected_location = None;
                }
                ViewState::Login => {} // handled in handle_login_key
            },
            KeyCode::Char('t') => {
                self.test_http = true;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let list_len = match self.current_view {
                    ViewState::LocationSelection => self.locations.len(),
                    ViewState::BookingForm => self.selected_location_space_ids.len(),
                    ViewState::Confirmation | ViewState::Login => 0,
                };
                let selected = self.list_state.selected().unwrap_or(0);
                let new_selected = if selected == 0 {
                    list_len.saturating_sub(1)
                } else {
                    selected.saturating_sub(1)
                };
                self.list_state.select(Some(new_selected));
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let list_len = match self.current_view {
                    ViewState::LocationSelection => self.locations.len(),
                    ViewState::BookingForm => self.selected_location_space_ids.len(),
                    ViewState::Confirmation | ViewState::Login => 0,
                };
                let selected = self.list_state.selected().unwrap_or(0);
                let new_selected = if selected >= list_len.saturating_sub(1) {
                    0
                } else {
                    selected.saturating_add(1)
                };
                self.list_state.select(Some(new_selected));
            }
            KeyCode::Enter => {
                match self.current_view {
                    ViewState::LocationSelection => {
                        if let Some(selected) = self.list_state.selected() {
                            if selected < self.locations.len() {
                                let location_name = LOCATIONS[selected];
                                self.selected_location = Some(location_name.to_string());

                                if let Some(ref mut skedda) = self.skedda {
                                    skedda.fetch_space_ids();
                                    self.venue_space_ids = skedda.venue_space_ids.clone();
                                    self.selected_location_space_ids =
                                        skedda.fetch_location_space_ids(location_name);
                                    let today = Local::now().format("%Y-%m-%d").to_string();
                                    if let Some(first_id) =
                                        self.selected_location_space_ids.first().cloned()
                                    {
                                        let bookings = skedda
                                            .fetch_bookings(&today)
                                            .unwrap_or_default();
                                        self.available_slots = Skedda::calculate_availability(
                                            &first_id,
                                            &today,
                                            &bookings,
                                        );
                                        self.selected_space_id = Some(first_id);
                                        self.availability_date = today;
                                    } else {
                                        self.available_slots = Vec::new();
                                        self.selected_space_id = None;
                                        self.availability_date = today;
                                    }
                                } else {
                                    self.venue_space_ids.clear();
                                    self.selected_location_space_ids.clear();
                                    self.available_slots = Vec::new();
                                    self.selected_space_id = None;
                                    self.availability_date.clear();
                                }
                                self.list_state.select(
                                    if self.selected_location_space_ids.is_empty() {
                                        None
                                    } else {
                                        Some(0)
                                    },
                                );
                                self.current_view = ViewState::BookingForm;
                            }
                        }
                    }
                    ViewState::BookingForm => {
                        self.current_view = ViewState::Confirmation;
                    }
                    ViewState::Confirmation => {
                        self.current_view = ViewState::LocationSelection;
                        self.selected_location = None;
                    }
                    ViewState::Login => {} // handled in handle_login_key
                }
            }
            _ => {}
        }
        Ok(())
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
