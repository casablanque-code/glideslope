mod aircraft;
mod app;
mod core;
mod crew;
mod parser;
mod sim;
mod ui;

use app::app::App;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, Stdout};

fn main() -> io::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(io::stderr) // stdout is the terminal UI; logs go to stderr
        .init();

    let mut terminal = setup_terminal()?;

    // If App::run panics, the terminal would otherwise be left in raw /
    // alternate-screen mode and the user's shell would look broken until
    // they blindly typed `reset`. Restore it first, then resume the panic
    // so the real error still surfaces.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal();
        default_hook(panic_info);
    }));

    let mut app = App::new();
    let result = app.run(&mut terminal);

    restore_terminal()?;
    result
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
