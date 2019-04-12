mod common;
mod parser;
mod shell;
mod terminal;

use std::os::unix::io::AsRawFd;

use common::{Event, Action};
use shell::Shell;
use terminal::{Terminal, TerminalState};

fn main() {
    let mut shell = Shell::new();
    let mut term = Terminal::new(std::io::stdin().as_raw_fd(), std::io::stdout().as_raw_fd());
    term.set_state(TerminalState::Custom);

    'command: loop {
        term.write("» ".as_bytes());

        'event: loop {
            let event = term.read();
            let Event::Char(c) = event;
            term.write_char(c);

            match shell.event(event) {
                Action::Process => break 'event,
                Action::Exit    => break 'command,
                _               => (),
            };

        }

        term.set_state(TerminalState::Initial);
        match shell.process() {
            Action::Exit => break,
            _            => (),
        };
        term.set_state(TerminalState::Custom);
    }
}
