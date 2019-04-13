mod command;
mod parser;
mod shell;

use std::io::{Write};

use nix::sys::signal::{self, Signal, SigHandler};

use shell::{Action, Shell};
use termion::input::TermRead;
use termion::raw::IntoRawMode;

fn main() {
    let mut shell = Shell::new();
    unsafe { signal::signal(Signal::SIGINT, SigHandler::SigIgn) }.unwrap();

    'command: loop {
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout().into_raw_mode().unwrap();

        print!("{}\r» {}", termion::clear::CurrentLine, shell.line());
        stdout.flush().unwrap();

        'event: for event in stdin.events() {

            let event = event.unwrap();

            if let Some(action) = shell.event(&event) {
                match action {
                    Action::Exit => break 'command,
                    Action::Process => break 'event,
                    Action::ClearScreen => print!("{}{}", termion::clear::All, termion::cursor::Goto(1,1)),
                    _ => (),
                }
            }

            print!("{}\r» {}", termion::clear::CurrentLine, shell.line());
            stdout.flush().unwrap();
        }

        print!("\r\n");
        stdout.flush().unwrap();

        std::mem::drop(stdout);
        if let Some(action) = shell.process() {
            match action {
                Action::Exit => break 'command,
                Action::ClearScreen => print!("{}{}", termion::clear::All, termion::cursor::Goto(1,1)),
                _ => (),
            }
        }
    }
}