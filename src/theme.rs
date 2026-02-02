use ratatui::style::{Color, Modifier, Style};

// ── Lamp-Lit Lounge palette ──────────────────────────────────────────

/// Warm amber — focused borders, key hints, primary accent
pub const LAMP: Color = Color::Rgb(204, 153, 0);

/// Burnt orange — selected item backgrounds
pub const EMBER: Color = Color::Rgb(176, 82, 40);

/// Warm red — errors
pub const RUST: Color = Color::Rgb(160, 60, 40);

/// Sage green — confirmation, map spaces, tertiary
pub const OLIVE: Color = Color::Rgb(107, 124, 94);

/// Warm off-white — body text
pub const CREAM: Color = Color::Rgb(216, 200, 170);

/// Brightest text — focused labels, tab highlights
pub const WARM_WHITE: Color = Color::Rgb(235, 225, 210);

/// Warm gray — help descriptions, inactive tabs
pub const DIM: Color = Color::Rgb(90, 82, 72);

/// Deep warm gray — unfocused borders
pub const SHADOW: Color = Color::Rgb(58, 53, 48);

/// Near-black — backgrounds, fg on colored selections
pub const WARM_BLACK: Color = Color::Rgb(24, 22, 20);

// ── Style helpers ────────────────────────────────────────────────────

/// Title/primary border (LAMP amber).
pub fn title_border() -> Style {
    Style::default().fg(LAMP)
}

/// Focused panel border (bright LAMP).
pub fn focused_border() -> Style {
    Style::default().fg(LAMP)
}

/// Unfocused panel border (dim SHADOW).
pub fn unfocused_border() -> Style {
    Style::default().fg(SHADOW)
}

/// Standard body text (CREAM).
pub fn body_text() -> Style {
    Style::default().fg(CREAM)
}

/// Dim descriptive text (DIM warm gray).
pub fn dim_text() -> Style {
    Style::default().fg(DIM)
}

/// Focused label (WARM_WHITE + bold).
pub fn focused_label() -> Style {
    Style::default().fg(WARM_WHITE).add_modifier(Modifier::BOLD)
}

/// Unfocused label (CREAM).
pub fn unfocused_label() -> Style {
    Style::default().fg(CREAM)
}

/// Key-hint spans in help text (LAMP + bold).
pub fn key_hint() -> Style {
    Style::default().fg(LAMP).add_modifier(Modifier::BOLD)
}

/// Error messages (warm RUST).
pub fn error_text() -> Style {
    Style::default().fg(RUST)
}

/// Selected list item (WARM_BLACK on EMBER + bold).
pub fn selected_item() -> Style {
    Style::default()
        .fg(WARM_BLACK)
        .bg(EMBER)
        .add_modifier(Modifier::BOLD)
}

/// Time-slot selection highlight (WARM_BLACK on LAMP).
pub fn time_selection() -> Style {
    Style::default().fg(WARM_BLACK).bg(LAMP)
}

/// Time-slot selection cursor (WARM_BLACK on LAMP + bold).
pub fn time_selection_cursor() -> Style {
    Style::default()
        .fg(WARM_BLACK)
        .bg(LAMP)
        .add_modifier(Modifier::BOLD)
}

/// Cursor line when not part of a selection (LAMP + bold).
pub fn cursor_line() -> Style {
    Style::default().fg(LAMP).add_modifier(Modifier::BOLD)
}

/// Confirmation view border (OLIVE).
pub fn confirmation_border() -> Style {
    Style::default().fg(OLIVE)
}

/// Map — selected space label (LAMP + bold).
pub fn map_selected() -> Style {
    Style::default().fg(LAMP).add_modifier(Modifier::BOLD)
}

/// Map — unselected space label (CREAM).
pub fn map_unselected() -> Style {
    Style::default().fg(CREAM)
}

/// Tab highlight (WARM_WHITE + bold).
pub fn tab_highlight() -> Style {
    Style::default()
        .fg(WARM_WHITE)
        .add_modifier(Modifier::BOLD)
}

/// Inactive tab style (DIM).
pub fn tab_inactive() -> Style {
    Style::default().fg(DIM)
}
