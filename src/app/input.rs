use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{App, BookingFocus, LoginField, LoginMode, ViewState, LOCATIONS, MAIN_MENU_ITEMS};
use crate::credentials;
use crate::event::{AppEvent, Event};
use crate::skedda::Skedda;

impl App<'_> {
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

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
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

    fn handle_paste(&mut self, text: String) -> color_eyre::Result<()> {
        let trimmed = text.trim().to_string();
        if self.current_view == ViewState::Login {
            match self.login_field_focus {
                LoginField::Username => self.username_input.push_str(&trimmed),
                LoginField::Password => self.password_input.push_str(&trimmed),
                LoginField::Cookie => self.cookie_input.push_str(&trimmed),
            }
        } else if self.current_view == ViewState::BookingForm
            && self.booking_focus == BookingFocus::TitleInput
        {
            self.booking_title.push_str(&trimmed);
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

    fn switch_login_mode(&mut self) {
        self.login_mode = match self.login_mode {
            LoginMode::EmailPassword => LoginMode::SessionCookie,
            LoginMode::SessionCookie => LoginMode::EmailPassword,
        };
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
            ViewState::MainMenu => self.handle_menu_key(key_event),
            ViewState::LocationSelection => self.handle_location_key(key_event),
            ViewState::BookingManager => self.handle_manage_booking_key(key_event),
            ViewState::BookingForm => self.handle_booking_key(key_event),
            ViewState::Confirmation => self.handle_confirmation_key(key_event),
            ViewState::Login => Ok(()), // handled in handle_login_key
        }
    }

    fn handle_menu_key(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.events.send(AppEvent::Quit);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let len = self.main_menu_items.len();
                let selected = self.main_menu_list_state.selected().unwrap_or(0);
                let new = if selected == 0 {
                    len.saturating_sub(1)
                } else {
                    selected.saturating_sub(1)
                };
                self.main_menu_list_state.select(Some(new));
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let len = self.main_menu_items.len();
                let selected = self.main_menu_list_state.selected().unwrap_or(0);
                let new = if selected >= len.saturating_sub(1) {
                    0
                } else {
                    selected.saturating_add(1)
                };
                self.main_menu_list_state.select(Some(new));
            }
            KeyCode::Enter => {
                if let Some(selected) = self.main_menu_list_state.selected() {
                    match MAIN_MENU_ITEMS[selected] {
                        "Create a booking" => {
                            self.current_view = ViewState::LocationSelection;
                        },
                        "Manage bookings" => {
                            if let Some(ref mut skedda) = self.skedda {
                                skedda.fetch_space_ids();
                                self.venue_space_ids = skedda.venue_space_ids.clone();
                                self.user_bookings = skedda.get_user_bookings();
                                self.user_booking_list_state.select(
                                    if self.user_bookings.is_empty() { None } else { Some(0) }
                                );
                                self.current_view = ViewState::BookingManager;
                            }
                        },
                        _ => todo!()
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_manage_booking_key(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        if self.confirm_delete {
            match key_event.code {
                KeyCode::Enter | KeyCode::Char('y') => {
                    if let Some(selected) = self.user_booking_list_state.selected() {
                        if let Some(booking) = self.user_bookings.get(selected) {
                            let booking_id = booking
                                .get("id")
                                .and_then(|v| {
                                    v.as_str()
                                        .map(String::from)
                                        .or_else(|| v.as_i64().map(|n| n.to_string()))
                                });

                            if let Some(id) = booking_id {
                                if let Some(ref mut skedda) = self.skedda {
                                    match skedda.delete_booking(&id) {
                                        Ok(()) => {
                                            self.user_bookings = skedda.get_user_bookings();
                                            if self.user_bookings.is_empty() {
                                                self.user_booking_list_state.select(None);
                                            } else {
                                                let new_sel =
                                                    selected.min(self.user_bookings.len() - 1);
                                                self.user_booking_list_state
                                                    .select(Some(new_sel));
                                            }
                                            self.booking_error = None;
                                        }
                                        Err(e) => {
                                            self.booking_error = Some(e.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                    self.confirm_delete = false;
                }
                KeyCode::Esc | KeyCode::Char('n') => {
                    self.confirm_delete = false;
                }
                _ => {}
            }
            return Ok(());
        }

        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.confirm_delete = false;
                self.current_view = ViewState::MainMenu;
                self.booking_error = None;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let len = self.user_bookings.len();
                if len > 0 {
                    let selected = self.user_booking_list_state.selected().unwrap_or(0);
                    let new = if selected == 0 { len - 1 } else { selected - 1 };
                    self.user_booking_list_state.select(Some(new));
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let len = self.user_bookings.len();
                if len > 0 {
                    let selected = self.user_booking_list_state.selected().unwrap_or(0);
                    let new = if selected >= len - 1 { 0 } else { selected + 1 };
                    self.user_booking_list_state.select(Some(new));
                }
            }
            KeyCode::Enter => {
                if self.user_booking_list_state.selected().is_some() {
                    self.confirm_delete = true;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_location_key(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.current_view = ViewState::MainMenu;
                self.selected_location = None;
                self.booking_error = None;
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
                        self.availability_date = String::new();
                        self.booking_title = String::new();
                        self.initialize_week_dates();

                        if let Some(ref mut skedda) = self.skedda {
                            skedda.fetch_space_ids();
                            self.venue_space_ids = skedda.venue_space_ids.clone();
                            self.selected_location_space_ids =
                                skedda.fetch_location_space_ids(location_name);
                        } else {
                            self.venue_space_ids.clear();
                            self.selected_location_space_ids.clear();
                            self.available_slots = Vec::new();
                            self.selected_space_id = None;
                            self.week_dates.clear();
                            self.week_availability.clear();
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
                    if !self.week_dates.is_empty() {
                        self.booking_focus = BookingFocus::DateSelection;
                    }
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.current_view = ViewState::LocationSelection;
                    self.selected_location = None;
                    self.booking_error = None;
                }
                _ => {}
            },
            BookingFocus::DateSelection => match key_event.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    let len = self.week_dates.len();
                    if len > 0 {
                        let sel = self.date_list_state.selected().unwrap_or(0);
                        let new = if sel == 0 { len - 1 } else { sel - 1 };
                        self.date_list_state.select(Some(new));
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let len = self.week_dates.len();
                    if len > 0 {
                        let sel = self.date_list_state.selected().unwrap_or(0);
                        let new = if sel >= len - 1 { 0 } else { sel + 1 };
                        self.date_list_state.select(Some(new));
                    }
                }
                KeyCode::Enter => {
                    self.select_date();
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
                    self.booking_title = String::new();
                    self.booking_focus = BookingFocus::TitleInput;
                }
                KeyCode::Esc | KeyCode::Tab => {
                    self.booking_focus = BookingFocus::DateSelection;
                }
                KeyCode::Char('q') => {
                    self.current_view = ViewState::LocationSelection;
                    self.selected_location = None;
                    self.booking_error = None;
                }
                _ => {}
            },
            BookingFocus::TitleInput => match key_event.code {
                KeyCode::Char(c) => {
                    self.booking_title.push(c);
                }
                KeyCode::Backspace => {
                    self.booking_title.pop();
                }
                KeyCode::Enter => {
                    self.submit_booking();
                }
                KeyCode::Esc => {
                    self.booking_focus = BookingFocus::TimeSlots;
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
}
