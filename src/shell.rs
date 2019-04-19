use std::path::PathBuf;
use std::collections::HashMap;

use nix::sys::stat;
use termion::event::{Event, Key};

use super::command;
use super::processor::{Processor, Sequence, SequenceKind, Command, CommandKind, Atom, AtomKind};

pub enum Action {
    Process,
    Exit,
    ClearScreen,
}

pub struct Shell {
    bin_dirs: Vec<String>,
    history: Vec<String>,
    history_idx: usize,
    processor: Processor,
    prompt: String,
    vars: HashMap<String, String>,
}

impl Shell {
    pub fn new() -> Shell {
        let prompt = String::from("» ");
        let vars: HashMap<_, _> = std::env::vars().collect();

        let mut bin_dirs = Vec::new();
        bin_dirs.push(String::from("/bin"));
        bin_dirs.push(String::from("/usr/bin"));
        if let Some(path) = vars.get("PATH") {
            bin_dirs.extend(path.split(":").map(String::from).collect::<Vec<_>>());
        }

        Shell {
            bin_dirs,
            history: Vec::new(),
            history_idx: 1,
            processor: Processor::new(),
            prompt,
            vars
        }
    }

    pub fn event(&mut self, event: &Event) -> Option<Action> {
        match event {
            Event::Key(key) => match key {
                Key::Char('\n') => Some(Action::Process),
                Key::Char(c) => {
                    self.processor.push(*c);
                    None
                },
                Key::Ctrl('c') => {
                    self.processor.clear();
                    None
                },
                Key::Ctrl('d') => {
                    if self.processor.is_empty() {
                        Some(Action::Exit)
                    } else {
                        None
                    }
                },
                Key::Backspace => {
                    self.processor.pop_prev();
                    None
                },
                Key::Delete => {
                    self.processor.pop_next();
                    None
                },
                Key::Ctrl('l') => Some(Action::ClearScreen),
                Key::Up => {
                    if self.history_idx > 0 && self.history.len() > 0 {
                        self.history_idx -= 1;
                        self.processor.set(self.history[self.history_idx].clone());
                    }
                    None
                },
                Key::Down => {
                    if self.history_idx + 1 < self.history.len() {
                        self.history_idx += 1;
                        self.processor.set(self.history[self.history_idx].clone());
                    } else {
                        self.history_idx = self.history.len();
                        self.processor.clear()
                    }
                    None
                },
                Key::Left => {
                    self.processor.left();
                    None
                },
                Key::Right => {
                    self.processor.right();
                    None
                }
                _ => None,
            },
            _ => None
        }
    }

    pub fn process(&mut self) -> usize {
        let sequence = self.processor.get().get();
        self.history.push(self.processor.raw());
        self.history_idx = self.history.len();
        self.processor.clear();

        let mut retcode = 0;
        for command in sequence {
            retcode = match command.1.kind() {
                CommandKind::Execute => self.process_execute(command.1.atoms()),
                CommandKind::Assign => self.process_assign(command.1.atoms()),
            }
        }
        retcode
    }

    fn process_execute(&mut self, atoms: Vec<Atom>) -> usize {
        let args: Vec<_> = atoms.iter()
            .map(|atom| {
                if let AtomKind::Word(word) = atom.kind() {
                    Some(word)
                } else {
                    None
                }
            })
            .filter(Option::is_some)
            .map(Option::unwrap)
            .collect();
        if args.is_empty() {
            return 0;
        }

        match args.first().unwrap().as_str() {
            "cd" => command::cd(&args) as usize,
            "self" => command::state(&args, &self.vars, &self.bin_dirs) as usize,
            command => {
                if let Some(bin) = self.find_bin(command) {
                    command::run(&bin, &args, &self.vars) as usize
                } else {
                    eprintln!("rush: Unknown command.");
                    1
                }
            }
        }
    }

    fn process_assign(&mut self, atoms: Vec<Atom>) -> usize {
        let args: Vec<_> = atoms.iter()
            .map(|atom| {
                if let AtomKind::Word(word) = atom.kind() {
                    Some(word)
                } else {
                    None
                }
            })
            .filter(Option::is_some)
            .map(Option::unwrap)
            .collect();
        let mut args = args.into_iter();

        self.vars.insert(args.next().unwrap(), args.next().unwrap());
        0
    }

    pub fn prompt(&self) -> String {
        self.prompt.clone()
    }

    pub fn line(&self) -> String {
        self.processor.raw()
    }

    pub fn position(&self) -> usize {
        self.processor.position()
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
