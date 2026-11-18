use std::io;

use crossterm::{
    event::DisableBracketedPaste,
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};

pub fn install_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), DisableBracketedPaste, LeaveAlternateScreen, crossterm::cursor::Show);
        default_hook(info);
    }));
}