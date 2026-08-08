//! Shared visual styling. One place to change the palette instead of
//! hunting through every widget.

use ratatui::style::{Color, Modifier, Style};

pub fn border() -> Style {
    Style::default().fg(Color::DarkGray)
}

pub fn title() -> Style {
    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
}

pub fn placeholder_text() -> Style {
    Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)
}

pub fn log_text() -> Style {
    Style::default().fg(Color::Gray)
}

pub fn command_prompt() -> Style {
    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
}

pub fn status_running() -> Style {
    Style::default().fg(Color::Green)
}
