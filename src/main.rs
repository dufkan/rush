mod command;
mod common;
mod parser;
mod shell;
mod terminal;

use std::os::unix::io::AsRawFd;

use nix::sys::signal::{self, Signal, SigHandler};

use common::{Event, Action};
use shell::Shell;
use terminal::{Terminal, TerminalState};

fn main() {
    let mut shell = Shell::new();
    let mut term = Terminal::new(std::io::stdin().as_raw_fd(), std::io::stdout().as_raw_fd());
    term.set_state(TerminalState::Custom);

    unsafe { signal::signal(Signal::SIGINT, SigHandler::SigIgn) }.unwrap();

    'command: loop {
        term.write("» ".as_bytes());

        'event: loop {
            let event = term.read();
            match event {
                Event::Char(c) => term.write_char(c),
                Event::Return  => term.write_char('\n'),
                _              => ()
            }

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
