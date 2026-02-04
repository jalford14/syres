use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Masked, Span, Text},
    widgets::{Block, BorderType, Clear, Paragraph, Tabs},
};

use crate::app::{App, LoginField, LoginMode};
use crate::theme;

pub(super) fn render_login(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    let popup_area = super::centered_rect(60, 80, area);

    frame.render_widget(Clear, popup_area);

    let outer_block = Block::bordered()
        .title(" syres ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(theme::title_border());

    let inner = outer_block.inner(popup_area);
    frame.render_widget(outer_block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // spacer
            Constraint::Length(3), // tabs (with border)
            Constraint::Length(1), // spacer
            Constraint::Length(9), // form fields
            Constraint::Length(1), // spacer
            Constraint::Length(2), // error
            Constraint::Min(1),   // help
        ])
        .split(inner);

    // -- Mode tabs --
    let tab_titles = vec![" Email/Password ", " Session Cookie "];
    let mode_index = match app.login_mode {
        LoginMode::EmailPassword => 0,
        LoginMode::SessionCookie => 1,
    };
    let tab_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(theme::unfocused_border());
    let tabs = Tabs::new(tab_titles)
        .block(tab_block)
        .select(mode_index)
        .style(theme::tab_inactive())
        .highlight_style(theme::tab_highlight())
        .divider("\u{2502}");
    frame.render_widget(tabs, chunks[1]);

    // -- Form fields --
    match app.login_mode {
        LoginMode::EmailPassword => render_email_password_form(app, frame, chunks[3]),
        LoginMode::SessionCookie => render_cookie_form(app, frame, chunks[3]),
    }

    // -- Error message --
    if let Some(ref err) = app.auth_error {
        let error_msg = Paragraph::new(err.as_str())
            .style(theme::error_text())
            .alignment(Alignment::Center);
        frame.render_widget(error_msg, chunks[5]);
    }

    // -- Help text --
    let key_style = theme::key_hint();
    let desc_style = theme::dim_text();
    let help_lines = vec![
        Line::from(vec![
            Span::styled("\u{2190}/\u{2192}", key_style),
            Span::styled(" switch mode  ", desc_style),
            Span::styled("Tab", key_style),
            Span::styled(" next field  ", desc_style),
            Span::styled("Enter", key_style),
            Span::styled(" login", desc_style),
        ]),
        Line::from(vec![Span::styled("Esc to quit", desc_style)]),
    ];
    let help = Paragraph::new(help_lines).alignment(Alignment::Center);
    frame.render_widget(help, chunks[6]);
}

fn render_email_password_form(app: &App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // email label
            Constraint::Length(3), // email input
            Constraint::Length(1), // spacer
            Constraint::Length(1), // password label
            Constraint::Length(3), // password input
        ])
        .split(area);

    // Email label
    let email_focused = app.login_field_focus == LoginField::Username;
    let email_label_style = if email_focused {
        theme::focused_label()
    } else {
        theme::unfocused_label()
    };
    frame.render_widget(
        Paragraph::new("  Email").style(email_label_style),
        chunks[0],
    );

    // Email input
    let email_border_style = if email_focused {
        theme::focused_border()
    } else {
        theme::unfocused_border()
    };
    let email_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(email_border_style);
    let email_display = if email_focused {
        format!("{}\u{2588}", app.username_input)
    } else {
        app.username_input.clone()
    };
    let email_input = Paragraph::new(email_display)
        .block(email_block)
        .style(theme::body_text());
    frame.render_widget(email_input, chunks[1]);

    // Password label
    let password_focused = app.login_field_focus == LoginField::Password;
    let password_label_style = if password_focused {
        theme::focused_label()
    } else {
        theme::unfocused_label()
    };
    frame.render_widget(
        Paragraph::new("  Password").style(password_label_style),
        chunks[3],
    );

    let password_border_style = if password_focused {
        theme::focused_border()
    } else {
        theme::unfocused_border()
    };
    let password_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(password_border_style);
    let password_input = if password_focused {
        let masked_val = Masked::new(app.password_input.as_str(), '\u{2022}')
            .value()
            .to_string();
        Paragraph::new(format!("{masked_val}\u{2588}"))
    } else {
        Paragraph::new(Text::from(Masked::new(
            app.password_input.as_str(),
            '\u{2022}',
        )))
    };
    let password_input = password_input
        .block(password_block)
        .style(theme::body_text());
    frame.render_widget(password_input, chunks[4]);
}

fn render_cookie_form(app: &App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // cookie label
            Constraint::Length(3), // cookie input
            Constraint::Length(1), // spacer
            Constraint::Min(1),   // instructions
        ])
        .split(area);

    // Cookie label
    let cookie_focused = app.login_field_focus == LoginField::Cookie;
    let cookie_label_style = if cookie_focused {
        theme::focused_label()
    } else {
        theme::unfocused_label()
    };
    frame.render_widget(
        Paragraph::new("  X-Skedda-ApplicationCookie").style(cookie_label_style),
        chunks[0],
    );

    // Cookie input
    let cookie_border_style = if cookie_focused {
        theme::focused_border()
    } else {
        theme::unfocused_border()
    };
    let cookie_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(cookie_border_style);

    let cookie_input = if cookie_focused {
        Paragraph::new(format!("{}\u{2588}", app.cookie_input))
    } else if app.cookie_input.is_empty() {
        Paragraph::new("")
    } else {
        let masked_val: String = Masked::new(app.cookie_input.as_str(), '\u{2022}')
            .value()
            .chars()
            .take(8)
            .collect();
        Paragraph::new(format!("{masked_val}  (cookie set)"))
    };
    let cookie_input = cookie_input
        .block(cookie_block)
        .style(theme::body_text());
    frame.render_widget(cookie_input, chunks[1]);

    let instructions = vec![
        Line::from(Span::styled(
            "  To get your session cookie:",
            theme::dim_text(),
        )),
        Line::from(Span::styled(
            "  1. Log in at switchyards.skedda.com with Oauth",
            theme::dim_text(),
        )),
        Line::from(Span::styled(
            "  2. Open Dev Tools \u{2192} Application \u{2192} Cookies",
            theme::dim_text(),
        )),
        Line::from(Span::styled(
            "  3. Copy the X-Skedda-ApplicationCookie value",
            theme::dim_text(),
        )),
    ];
    frame.render_widget(Paragraph::new(instructions), chunks[3]);
}
