use std::path::{Path, PathBuf};
use std::collections::HashMap;

use nix::sys::stat;
use termion::event::{Event, Key};

use super::config::Config;
use super::executor::{self, Executee, ExecuteeKind};
use super::input::Input;
use super::parser::{SequenceKind, CommandKind, Atom, AtomKind};

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
    input: Input,
    prompt: String,
    vars: HashMap<String, String>,
}

impl Shell {
    pub fn new(config: Option<&Path>) -> Shell {
        let config = config
            .map(|path| {
                if let Some(config) = Config::load(path) {
                    config
                } else {
                    eprintln!("rush: No config found at {}.", path.to_string_lossy());
                    Config::default()
                }
            })
            .unwrap_or(Config::default());

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
            input: Input::new(),
            prompt,
            vars
        }
    }

    pub fn event(&mut self, event: &Event) -> Option<Action> {
        match event {
            Event::Key(key) => match key {
                Key::Char('\n') => Some(Action::Process),
                Key::Char(c) => {
                    self.input.push(*c);
                    None
                },
                Key::Ctrl('c') => {
                    self.input.clear();
                    None
                },
                Key::Ctrl('d') => {
                    if self.input.is_empty() {
                        Some(Action::Exit)
                    } else {
                        None
                    }
                },
                Key::Backspace => {
                    self.input.pop_prev();
                    None
                },
                Key::Delete => {
                    self.input.pop_next();
                    None
                },
                Key::Ctrl('l') => Some(Action::ClearScreen),
                Key::Up => {
                    if self.history_idx > 0 && self.history.len() > 0 {
                        self.history_idx -= 1;
                        self.input.set(&self.history[self.history_idx]);
                    }
                    None
                },
                Key::Down => {
                    if self.history_idx + 1 < self.history.len() {
                        self.history_idx += 1;
                        self.input.set(&self.history[self.history_idx]);
                    } else {
                        self.history_idx = self.history.len();
                        self.input.clear()
                    }
                    None
                },
                Key::Left => {
                    self.input.left();
                    None
                },
                Key::Right => {
                    self.input.right();
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
        let sequence = self.input.get().get();
        self.history.push(self.input.raw());
        self.history_idx = self.history.len();
        self.input.clear();

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
        let mut exec = Executee::new();
        for atom in atoms {
            match atom.kind() {
                AtomKind::Word(word) => exec.arg(word),
                AtomKind::Pipe => {
                    execs.push(exec);
                    exec = Executee::new();
                },
                AtomKind::OutFd(src, dst) => exec.redirect_fd(src, dst),
                AtomKind::OutFile(file, fd) => exec.out_file(file, fd),
                AtomKind::InFile(file, fd) => exec.in_file(file, fd)
            }
        };
        execs.push(exec);

        for exec in &mut execs {
            let command = exec.args().first();
            if command.is_some() {
                let command = command.unwrap();
                exec.set_kind(match command.as_str() {
                    "cd" => ExecuteeKind::StrongBuiltin(String::from("cd")),
                    "state" | "self" => ExecuteeKind::WeakBuiltin(String::from("state")),
                    other => {
                        if let Some(path) = self.find_bin(other) {
                            ExecuteeKind::Binary(path)
                        } else {
                            ExecuteeKind::WeakBuiltin(String::from("fail"))
                        }
                    }
                });
            }
        }

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
        self.input.raw()
    }

    pub fn position(&self) -> usize {
        self.input.position()
    }

    pub fn set_line(&mut self, line: &str) {
        self.input.set(line);
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
