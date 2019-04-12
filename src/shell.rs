use std::path::PathBuf;

use nix::sys::stat;

use super::command;
use super::common::{Event, Action};
use super::parser::Parser;

pub struct Shell {
    bin_dirs: Vec<String>,
    parser: Parser,
}

impl Shell {
    pub fn new() -> Shell {
        let mut bin_dirs = Vec::new();
        bin_dirs.push(String::from("/usr/bin/"));
        Shell {
            bin_dirs,
            parser: Parser::new()
        }
    }

    pub fn event(&mut self, event: Event) -> Action {
        match event {
            Event::Char('\n') => Action::Process,
            Event::Char(c) => {
                self.parser.push(c);
                Action::None
            },
        }
    }

    pub fn process(&mut self) -> Action {
        let command = self.parser.command();
        let args = self.parser.args();
        self.parser.clear();

        if let Some(command) = command {
            let _retcode = match command.as_str() {
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
