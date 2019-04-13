use std::path::PathBuf;

use nix::sys::stat;
use termion::event::{Event, Key};

use super::command;
use super::parser::Parser;

pub enum Action {
    Process,
    Exit,
    ClearScreen,
}

pub struct Shell {
    bin_dirs: Vec<String>,
    history: Vec<String>,
    history_idx: usize,
    parser: Parser,
}

impl Shell {
    pub fn new() -> Shell {
        let mut bin_dirs = Vec::new();
        bin_dirs.push(String::from("/bin/"));
        bin_dirs.push(String::from("/usr/bin/"));
        Shell {
            bin_dirs,
            history: Vec::new(),
            history_idx: 1,
            parser: Parser::new()
        }
    }

    pub fn event(&mut self, event: &Event) -> Option<Action> {
        match event {
            Event::Key(key) => match key {
                Key::Char('\n') => Some(Action::Process),
                Key::Char(c) => {
                    self.parser.push(*c);
                    None
                },
                Key::Ctrl('c') => {
                    self.parser.clear();
                    None
                },
                Key::Ctrl('d') => {
                    if self.parser.is_empty() {
                        Some(Action::Exit)
                    } else {
                        None
                    }
                },
                Key::Backspace => {
                    self.parser.pop_prev();
                    None
                },
                Key::Delete => {
                    self.parser.pop_next();
                    None
                },
                Key::Ctrl('l') => Some(Action::ClearScreen),
                Key::Up => {
                    if self.history_idx > 0 && self.history.len() > 0 {
                        self.history_idx -= 1;
                        self.parser.set(self.history[self.history_idx].clone());
                    }
                    None
                },
                Key::Down => {
                    if self.history_idx < self.history.len() - 1 {
                        self.history_idx += 1;
                        self.parser.set(self.history[self.history_idx].clone());
                    } else {
                        self.history_idx = self.history.len();
                        self.parser.clear()
                    }
                    None
                },
                _ => None,
            },
            _ => None
        }
    }

    pub fn process(&mut self) -> Option<Action> {
        let command = self.parser.command();
        let args = self.parser.args();
        self.history.push(self.parser.raw());
        self.history_idx = self.history.len();
        self.parser.clear();

        if let Some(command) = command {
            let _retcode = match command.as_str() {
                "cd" => command::cd(&args),
                "exit" => return Some(Action::Exit),
                _      => {
                    if let Some(bin) = self.find_bin(&command) {
                        command::run(&bin, &args)
                    } else {
                        eprintln!("rush: Unknown command.");
                        1
                    }
                }
            };
        };

        None
    }

    pub fn line(&self) -> String {
        self.parser.raw().clone()
    }

    fn find_bin(&self, command: &str) -> Option<PathBuf> {
        for path in &self.bin_dirs {
            let path: PathBuf = [path, command].iter().collect();
            if let Ok(_) = stat::stat(&path) {
                return Some(path);
            }
        }
        None
    }
}
