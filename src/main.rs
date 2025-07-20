use crate::app::App;

pub mod app;
pub mod event;
pub mod ui;
pub mod http_client;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}
