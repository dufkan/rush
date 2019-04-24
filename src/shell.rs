use std::path::PathBuf;
use std::collections::HashMap;

use directories::ProjectDirs;
use nix::sys::stat;
use termion::event::{Event, Key};

use super::config::Config;
use super::executor::{self, Executee, ExecuteeKind};
use super::processor::{Processor, SequenceKind, CommandKind, Atom, AtomKind};

pub enum Action {
    Process,
    Exit,
    ClearScreen,
}

pub struct Shell {
    config: Config,
    bin_dirs: Vec<String>,
    history: Vec<String>,
    history_idx: usize,
    processor: Processor,
    prompt: String,
    vars: HashMap<String, String>,
}

impl Shell {
    pub fn new() -> Shell {
        let config = if let Some(project_dirs) = ProjectDirs::from("", "", "rush") {
            Config::load(&project_dirs.config_dir().join("config.toml"))
        } else {
            Config::default()  
        };

        let vars = if config.respect_vars {
            std::env::vars().collect()
        } else {
            HashMap::new()
        };

        let mut bin_dirs = config.bin_dirs.clone();

        if config.respect_path {
            if let Some(path) = vars.get("PATH") {
                bin_dirs.extend(path.split(":").map(String::from).collect::<Vec<_>>());
            }
        }

        let prompt = config.prompt.clone();

        Shell {
            config,
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

    pub fn vars(&self) -> &HashMap<String, String> {
        &self.vars
    }

    pub fn bin_dirs(&self) -> &Vec<String> {
        &self.bin_dirs
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn process(&mut self) -> usize {
        let sequence = self.processor.get().get();
        self.history.push(self.processor.raw());
        self.history_idx = self.history.len();
        self.processor.clear();

        let mut retcode = 0;
        for command in sequence {
            match command.0 {
                SequenceKind::And if retcode != 0 => continue,
                SequenceKind::Or if retcode == 0 => continue,
                _ => (),
            }
            retcode = match command.1.kind() {
                CommandKind::Execute => self.process_execute(command.1.atoms()),
                CommandKind::Assign => self.process_assign(command.1.atoms()),
            }
        }
        retcode
    }

    fn process_execute(&mut self, atoms: Vec<Atom>) -> usize {
        let mut execs = Vec::new();
        let mut exec = Vec::new();
        for atom in atoms {
            match atom.kind() {
                AtomKind::Word(word) => exec.push(word),
                AtomKind::Pipe => {
                    execs.push(exec);
                    exec = Vec::new();
                },
            }
        };
        if !exec.is_empty() {
            execs.push(exec);
        }


        let mut execs: Vec<_> = execs.iter()
            .map(|words| {
                let kind = match words[0].as_str() {
                    "cd" => ExecuteeKind::StrongBuiltin(String::from("cd")),
                    "state" | "self" => ExecuteeKind::WeakBuiltin(String::from("state")),
                    other => {
                        if let Some(path) = self.find_bin(other) {
                            ExecuteeKind::Binary(path)
                        } else {
                            ExecuteeKind::WeakBuiltin(String::from("fail"))
                        }
                    }
                };
                Executee::new(kind, &words, self.vars())
            })
            .collect();

        if execs.len() == 1 {
            executor::execute_single(&execs[0]) as usize
        } else {
            executor::execute_group(&mut execs[..]) as usize
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
