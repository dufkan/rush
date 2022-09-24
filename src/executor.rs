use std::collections::HashMap;
use std::path::PathBuf;
use std::process;
use std::ffi::CString;
use std::os::unix::io::RawFd;

use super::builtin;

use nix::sys::stat::Mode;
use nix::fcntl::{self, OFlag};
use nix::unistd::{self, fork, ForkResult};
use nix::sys::wait::{self, WaitStatus};
use nix::sys::signal::{self, Signal, SigHandler};

#[derive(Clone)]
pub enum ExecuteeKind {
    StrongBuiltin(String),
    WeakBuiltin(String),
    Binary(PathBuf),
}

pub enum RedirectKind {
    Dup(RawFd),
    Mov(RawFd),
    Read(String),
    Write(String),
    Append(String),
    RW(String),
}

pub struct Executee {
    kind: ExecuteeKind,
    args: Vec<String>,
    vars: HashMap<String, String>,
    redirect: Vec<(RedirectKind, RawFd)>,
}

impl Executee {
    pub fn new() -> Executee {
        Executee {
            kind: ExecuteeKind::WeakBuiltin(String::from("void")),
            args: Vec::new(),
            vars: HashMap::new(),
            redirect: Vec::new(),
        }
    }

    pub fn set_kind(&mut self, kind: ExecuteeKind) {
        self.kind = kind;
    }

    pub fn arg(&mut self, word: String) {
        self.args.push(word);
    }

    fn redirect(&mut self, src: RedirectKind, dst: RawFd) {
        self.redirect.push((src, dst));
    }

    pub fn fd_duplicate(&mut self, src: RawFd, dst: RawFd) {
        self.redirect(RedirectKind::Dup(src), dst);
    }

    pub fn fd_move(&mut self, src: RawFd, dst: RawFd) {
        self.redirect(RedirectKind::Mov(src), dst);
    }

    pub fn file_write(&mut self, file: String, fd: RawFd) {
        self.redirect(RedirectKind::Write(file), fd);
    }

    pub fn file_read(&mut self, file: String, fd: RawFd) {
        self.redirect(RedirectKind::Read(file), fd);
    }

    pub fn file_append(&mut self, file: String, fd: RawFd) {
        self.redirect(RedirectKind::Append(file), fd);
    }

    pub fn file_rw(&mut self, file: String, fd: RawFd) {
        self.redirect(RedirectKind::RW(file), fd);
    }

    pub fn args(&self) -> &Vec<String> {
        &self.args
    }

    pub fn cargs(&self) -> Vec<CString> {
        self.args.iter()
            .map(String::clone)
            .map(CString::new)
            .map(Result::unwrap)
            .collect()
    }

    pub fn cvars(&self) -> Vec<CString> {
        self.vars.iter()
            .map(|var| CString::new(format!("{}={}", var.0, var.1)).unwrap())
            .collect()
    }
}

fn execute(executee: &Executee) -> ! {
    unsafe { signal::signal(Signal::SIGINT, SigHandler::SigDfl) }.unwrap();
    for redirect in &executee.redirect {
        let mut close_src = false;
        let src = match &redirect.0 {
            RedirectKind::Dup(fd) => *fd,
            RedirectKind::Mov(fd) => {
                close_src = true;
                *fd
            },
            RedirectKind::Write(file) => {
                let mut oflag = OFlag::empty();
                oflag.insert(OFlag::O_WRONLY);
                oflag.insert(OFlag::O_CREAT);
                oflag.insert(OFlag::O_TRUNC);
                let mut mode = Mode::empty();
                mode.insert(Mode::S_IWUSR);
                mode.insert(Mode::S_IRUSR);
                mode.insert(Mode::S_IRGRP);
                mode.insert(Mode::S_IROTH);
                fcntl::open(file.as_str(), oflag, mode).unwrap_or_else(|_| {
                    eprintln!("rush: Could not open file {}.", file);
                    process::exit(1 as i32);
                })
            },
            RedirectKind::Read(file) => {
                let mut oflag = OFlag::empty();
                oflag.insert(OFlag::O_RDONLY);
                let mode = Mode::empty();
                fcntl::open(file.as_str(), oflag, mode).unwrap_or_else(|_| {
                    eprintln!("rush: Could not open file {}.", file);
                    process::exit(1 as i32);
                })
            },
            RedirectKind::Append(file) => {
                let mut oflag = OFlag::empty();
                oflag.insert(OFlag::O_WRONLY);
                oflag.insert(OFlag::O_CREAT);
                oflag.insert(OFlag::O_APPEND);
                let mut mode = Mode::empty();
                mode.insert(Mode::S_IWUSR);
                mode.insert(Mode::S_IRUSR);
                mode.insert(Mode::S_IRGRP);
                mode.insert(Mode::S_IROTH);
                fcntl::open(file.as_str(), oflag, mode).unwrap_or_else(|_| {
                    eprintln!("rush: Could not open file {}.", file);
                    process::exit(1 as i32);
                })
            },
            RedirectKind::RW(file) => {
                let mut oflag = OFlag::empty();
                oflag.insert(OFlag::O_RDWR);
                oflag.insert(OFlag::O_CREAT);
                oflag.insert(OFlag::O_TRUNC);
                let mut mode = Mode::empty();
                mode.insert(Mode::S_IWUSR);
                mode.insert(Mode::S_IRUSR);
                mode.insert(Mode::S_IRGRP);
                mode.insert(Mode::S_IROTH);
                fcntl::open(file.as_str(), oflag, mode).unwrap_or_else(|_| {
                    eprintln!("rush: Could not open file {}.", file);
                    process::exit(1 as i32);
                })
            },
        };
        let dst = redirect.1;
        unistd::dup2(src, dst).unwrap();
        if close_src {
            unistd::close(src).unwrap();
        }
    }
    match executee.kind.clone() {
        ExecuteeKind::WeakBuiltin(name) => {
            let retcode = match name.as_str() {
                "state" => builtin::state(&executee.args),
                "fail" => builtin::fail(&executee.args),
                "void" => builtin::void(),
                _ => 1,
            };
            process::exit(retcode as i32);
        }
        ExecuteeKind::Binary(bin) => {
            unistd::execve(&CString::new(bin.to_str().unwrap()).unwrap(), &executee.cargs(), &executee.cvars()).unwrap();
        },
        _ => process::exit(1)
    };
    panic!("Child did not exec!");
}

pub fn execute_single(executee: &Executee) -> u8 {
    if let ExecuteeKind::StrongBuiltin(name) = executee.kind.clone() {
        match name.as_str() {
            "cd" => builtin::cd(&executee.args),
            _ => 1
        }
    } else {
        match unsafe { unistd::fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                let status = wait::waitpid(child, None).unwrap();
                match status {
                    WaitStatus::Exited(_, retcode) => {
                        retcode as u8
                    },
                    _ => 0,
                }
            },
            Ok(ForkResult::Child) => {
                execute(executee);
            },
            Err(_) => {
                panic!("Fork failed!");
            },
        }
    }
}

pub fn execute_group(executees: &mut [Executee]) -> u8 {
    match unsafe { unistd::fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            let status = wait::waitpid(child, None).unwrap();
            match status {
                WaitStatus::Exited(_, retcode) => {
                    retcode as u8
                },
                _ => 0,
            }
        },
        Ok(ForkResult::Child) => {
            unsafe { signal::signal(Signal::SIGINT, SigHandler::SigDfl) }.unwrap();
            
            for i in 0..(executees.len() - 1) {
                let pipe = unistd::pipe().unwrap();
                executees[i].fd_duplicate(pipe.1, 1);
                if let Ok(ForkResult::Child) = unsafe { fork() } {
                    execute(&executees[i]);
                }
                unistd::close(pipe.1).unwrap();
                unistd::dup2(pipe.0, 0).unwrap();
            }

            let last = executees.last().unwrap();
            execute(last);
        },
        Err(_) => {
            panic!("Fork failed!");
        },
    }
}
