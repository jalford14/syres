use crate::event::{AppEvent, Event, EventHandler};
use crate::ui;

use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    widgets::{ListState, ListItem},
};

const LOCATIONS: [&str; 12] = [
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
];

#[derive(Debug, Clone, PartialEq)]
pub enum ViewState {
    LocationSelection,
    BookingForm,
    Confirmation,
}

/// Application.
#[derive(Debug)]
pub struct App<'a> {
    /// Is the application running?
    pub running: bool,
    pub counter: u64,
    pub locations: Vec<ListItem<'a>>,
    pub events: EventHandler,
    pub list_state: ListState,
    pub current_view: ViewState,
    pub selected_location: Option<String>,
    pub test_http: bool,
    pub selected_location_space_ids: Vec<String>,
}

impl Default for App<'_> {
    fn default() -> Self {
        Self {
            running: true,
            counter: 0,
            locations: LOCATIONS.iter().map(|&s| ListItem::new(s.to_string())).collect(),
            events: EventHandler::new(),
            list_state: ListState::default().with_selected(Some(0)),
            current_view: ViewState::LocationSelection,
            selected_location: None,
            test_http: false,
            selected_location_space_ids: Vec::new(),
        }
    }
}

impl App<'_> {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            // Check if we need to test HTTP client
            if self.test_http {
                self.test_http = false;
                if let Err(e) = self.test_http_client() {
                    eprintln!("HTTP test failed: {:?}", e);
                }
            }
            
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
                AppEvent::Up => self.increment_counter(),
                AppEvent::Down => self.decrement_counter(),
                AppEvent::Quit => self.quit(),
            },
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                match self.current_view {
                    ViewState::LocationSelection => self.events.send(AppEvent::Quit),
                    ViewState::BookingForm | ViewState::Confirmation => {
                        self.current_view = ViewState::LocationSelection;
                        self.selected_location = None;
                        self.selected_location_space_ids.clear();
                    }
                }
            }
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit)
            }
            KeyCode::Char('t') => {
                // Set flag to test HTTP client
                self.test_http = true;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let selected = self.list_state.selected().unwrap_or(0);
                let new_selected = if selected == 0 {
                    self.locations.len().saturating_sub(1)
                } else {
                    selected.saturating_sub(1)
                };
                self.list_state.select(Some(new_selected));
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let selected = self.list_state.selected().unwrap_or(0);
                let new_selected = if selected >= self.locations.len().saturating_sub(1) {
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
                                // Get the location name from the ListItem
                                let location_name = LOCATIONS[selected];
                                self.selected_location = Some(location_name.to_string());
                                let _ = self.test_http_client();
                                self.current_view = ViewState::BookingForm;
                            }
                        }
                    }
                    ViewState::BookingForm => {
                        self.current_view = ViewState::Confirmation;
                    }
                    ViewState::Confirmation => {
                        // Could reset to location selection or quit
                        self.current_view = ViewState::LocationSelection;
                        self.selected_location = None;
                    }
                }
            }
            // Other handlers you could add here.
            _ => {}
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn increment_counter(&mut self) {
        self.counter = self.counter.saturating_add(1);
    }

    pub fn decrement_counter(&mut self) {
        self.counter = self.counter.saturating_sub(1);
    }

    /// Renders the user interface.
    pub fn render(&mut self, frame: &mut ratatui::Frame) {
        ui::render(self, frame);
    }

    /// Test the HTTP client functionality
    pub fn test_http_client(&mut self) -> anyhow::Result<()> {
        use crate::skedda_client::SkeddaClient;
        
        let rt = tokio::runtime::Runtime::new()?;
        
        rt.block_on(async {
            // Create client
            let client = SkeddaClient::new()?;
            
            //venue
            //mapsStructure
            //maps
            //id, name
            
            //spaces[]
            //id
            //so you can "zip" the spaceIds from venue with ids you get from spaces
            // doing that will give you the name and info

            //venue
            //spaceTags
            let webs_data = client.get_booking_data().await?;
            // s.Book(domain, venue.ID, spaceIDs, title, from, till)
            // domain: "switchyards.skedda.com"
            // venue.ID: webs_data["venue"][0]["id"]
            // venue.ID: webs_data["venue"][0]["id"]
            let webs_response = &webs_data["venue"][0]["spacePresentation"]["spaceTags"];
            if let serde_json::Value::Array(items) = webs_response {
                for item in items {
                    if let serde_json::Value::Object(obj) = item {
                        if self.selected_location == obj.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()) {
                            if let Some(serde_json::Value::Array(space_ids)) = obj.get("spaceIds") {
                                self.selected_location_space_ids = space_ids
                                    .iter()
                                    .filter_map(|v| v.as_i64())
                                    .map(|n| n.to_string())    
                                    .collect();
                            }
                        }
                    }
                } 
            } else {
                    println!("Unexpected response format: {:?}", webs_response);
            }
            Ok::<(), anyhow::Error>(())
        })
    }
}
