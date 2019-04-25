mod config;
mod builtin;
mod executor;
mod parser;
mod processor;
mod shell;

use std::path::PathBuf;
use std::io::Write;
use std::process::exit;
use std::sync::atomic::{self, AtomicU32};

use nix::sys::signal::{self, Signal, SigHandler};
use directories::ProjectDirs;

use shell::{Action, Shell};
use termion::input::TermRead;
use termion::raw::IntoRawMode;

static mut TERM_SIZE: AtomicU32 = AtomicU32::new(0);
static mut SHELL: Option<Shell> = None;

extern fn handle_sigwinch(_: nix::libc::c_int) {
    let new_size = termion::terminal_size().unwrap();
    let new_size = ((new_size.0 as u32) << 16) | (new_size.1 as u32);
    unsafe { TERM_SIZE.store(new_size, atomic::Ordering::Relaxed); }
}

/// Prints terminal prompt
/// 
/// Nothing more.
fn print_prompt(prompt: &str) {
    print!("\r{}", prompt);
}

/// Prints current terminal input
/// 
/// Moves cursor back by `prev_position`, prints `line`, positions cursor back
/// by `position` characters.
fn print_line(line: &str, position: usize, prev_position: usize) -> usize {
    for _ in 0..prev_position {
        print!("\x08");
    }

    print!("{}{} \x08", termion::clear::AfterCursor, line);

    for _ in 0..position {
        print!("\x08");
    }

    line.len() - position
}


fn interactive() {
    unsafe { signal::signal(Signal::SIGINT, SigHandler::SigIgn) }.unwrap();
    unsafe { signal::signal(Signal::SIGWINCH, SigHandler::Handler(handle_sigwinch)) }.unwrap();
    handle_sigwinch(0);

    let shell = unsafe { SHELL.as_mut().unwrap() };

    'command: loop {
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout().into_raw_mode().unwrap_or_else(|_| {
            eprintln!("rush: Could not enter raw mode.");
            exit(1);
        });

        print_prompt(&shell.prompt());
        let mut prev_position = print_line(&shell.line(), shell.position(), 0);
        stdout.flush().unwrap();

        'event: for event in stdin.events() {
            let event = event.unwrap();

            if let Some(action) = shell.event(&event) {
                match action {
                    Action::Exit => {
                        break 'command
                        print!("\r\n");
                    },
                    Action::Process => break 'event,
                    Action::ClearScreen => {
                        print!("{}{}", termion::clear::All, termion::cursor::Goto(1,1));
                        continue 'command;
                    },
                }
            }

            prev_position = print_line(&shell.line(), shell.position(), prev_position);
            stdout.flush().unwrap();
        }

        print_line(&shell.line(), 0, prev_position);
        print!("\r\n");
        stdout.flush().unwrap();

        std::mem::drop(stdout);

        shell.process();
    }
}

fn execute(command: &str) {
    let shell = unsafe { SHELL.as_mut().unwrap() };
    shell.set_line(command);
    exit(shell.process() as i32);
}

fn main() {
    use clap::{App, Arg, crate_authors, crate_description, crate_version};

    let matches = App::new("rush")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("command")
            .short("c")
            .long("command")
            .value_name("COMMAND")
            .help("Executes COMMAND")
            .takes_value(true))
        .arg(Arg::with_name("config")
            .long("config")
            .value_name("CONFIG")
            .help("Use CONFIG file"))
        .get_matches();

    let config = matches.value_of("config").map(PathBuf::from).or({
        if let Some(path) = ProjectDirs::from("", "", "rush") {
            Some(path.config_dir().join("config.toml"))
        } else {
            None
        }
    });

    unsafe { SHELL = Some(Shell::new(config.as_ref().map(PathBuf::as_path))) };

    if let Some(command) = matches.value_of("command") {
        execute(command);
    } else {
        interactive();
    }
}