use std::path::PathBuf;

use nix::sys::stat;

use super::command;
use super::common::{Event, Action};
use super::parser::Parser;

pub struct Shell {
    bin_dirs: Vec<String>,
    history: Vec<String>,
    history_idx: usize,
    parser: Parser,
}

impl Shell {
    pub fn new() -> Shell {
        let mut bin_dirs = Vec::new();
        bin_dirs.push(String::from("/usr/bin/"));
        Shell {
            bin_dirs,
            history: Vec::new(),
            history_idx: 1,
            parser: Parser::new()
        }
    }

    pub fn event(&mut self, event: Event) -> Action {
        match event {
            Event::Char(c) => {
                self.parser.push(c);
                Action::None
            },
            Event::Return => Action::Process,
            Event::Backspace => {
                if !self.parser.is_empty() {
                    self.parser.pop();
                    Action::Back
                } else {
                    Action::None
                }
            },
            Event::Ctrl('D') => {
                if self.parser.is_empty() {
                    Action::Exit
                } else {
                    Action::None
                }
            },
            Event::Ctrl('C') => {
                self.parser.clear();
                self.history_idx = self.history.len();
                Action::ClearLine
            },
            Event::Up => {
                if self.history_idx > 0 && self.history.len() > 0 {
                    self.history_idx -= 1;
                    self.parser.set(self.history[self.history_idx].clone());
                } else {
                    self.parser.clear();
                }
                Action::ClearLine
            },
            Event::Down => {
                if self.history_idx < self.history.len() - 1 {
                    self.history_idx += 1;
                    self.parser.set(self.history[self.history_idx].clone());
                } else {
                    self.parser.clear()
                }
                Action::ClearLine
            },
            Event::Ctrl('L') => Action::ClearScreen,
            _ => Action::None

        }
    }

    pub fn process(&mut self) -> Action {
        let command = self.parser.command();
        let args = self.parser.args();
        self.history.push(self.parser.raw());
        self.history_idx = self.history.len();
        self.parser.clear();

        if let Some(command) = command {
            let _retcode = match command.as_str() {
                "cd" => command::cd(&args),
                "exit" => return Action::Exit,
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

        Action::None
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
