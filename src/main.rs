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
        term.write_bytes("» ".as_bytes());
        term.write_bytes(shell.line().as_bytes());

        'event: loop {
            let event = term.read();
            match event {
                Event::Char(c) => term.write_char(c),
                Event::Return  => term.write_char('\n'),
                _              => ()
            }

            let action = shell.event(event);
            match action {
                Action::Back => term.write(action),
                Action::ClearLine => {
                    term.write(action);
                    continue 'command;
                },
                Action::ClearScreen => {
                    term.write(action);
                    continue 'command;
                },
                Action::Process => break 'event,
                Action::Exit => break 'command,
                _ => (),
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
