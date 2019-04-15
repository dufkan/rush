mod command;
mod parser;
mod shell;

use std::io::Write;
use std::sync::atomic::{self, AtomicU32};

use nix::sys::signal::{self, Signal, SigHandler};

use shell::{Action, Shell};
use termion::input::TermRead;
use termion::raw::IntoRawMode;

static mut TERM_SIZE: AtomicU32 = AtomicU32::new(0);

extern fn handle_sigwinch(_: nix::libc::c_int) {
    let new_size = termion::terminal_size().unwrap();
    let new_size = ((new_size.0 as u32) << 16) | (new_size.1 as u32);
    unsafe { TERM_SIZE.store(new_size, atomic::Ordering::Relaxed); }
}

/// Prints terminal line with current input
/// 
/// Moves cursor up by `prev_cursor_line` lines, prints `line`, positions cursor
/// back by `position` characters.
fn print_line(line: &str, position: usize, prev_cursor_line: usize) -> usize {
    if prev_cursor_line > 0 {
        print!("{}", termion::cursor::Up(prev_cursor_line as u16));
    }

    print!("\r{} {}\x08", line, termion::clear::AfterCursor);

    for _ in 0..position {
        print!("\x08");
    }

    let width = unsafe { (TERM_SIZE.load(atomic::Ordering::Relaxed) >> 16) as usize };
    let len = line.len() - 1;
    (len - position) / width
}

fn main() {
    let mut shell = Shell::new();

    unsafe { signal::signal(Signal::SIGINT, SigHandler::SigIgn) }.unwrap();
    unsafe { signal::signal(Signal::SIGWINCH, SigHandler::Handler(handle_sigwinch)) }.unwrap();
    handle_sigwinch(0);

    let mut cursor_line = 0;

    'command: loop {
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout().into_raw_mode().unwrap();

        let (line, position) = shell.line();
        cursor_line = print_line(&line, position, cursor_line);
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

            let (line, position) = shell.line();
            cursor_line = print_line(&line, position, cursor_line);
            stdout.flush().unwrap();
        }

        cursor_line = 0;
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