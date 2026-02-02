use crate::app::App;
use crossterm::event::{DisableBracketedPaste, EnableBracketedPaste};
use crossterm::execute;

pub mod app;
pub mod backdrop;
pub mod credentials;
pub mod event;
pub mod map_ui;
pub mod maps;
pub mod skedda;
pub mod theme;
pub mod ui;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    execute!(std::io::stdout(), EnableBracketedPaste)?;
    let result = App::new().run(terminal);
    execute!(std::io::stdout(), DisableBracketedPaste)?;
    ratatui::restore();
    result
}
