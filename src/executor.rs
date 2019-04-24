use std::collections::HashMap;
use std::path::PathBuf;
use std::process;
use std::ffi::CString;
use std::os::unix::io::RawFd;

use super::builtin;

use nix::unistd::{self, fork, ForkResult};
use nix::sys::wait::{self, WaitStatus};
use nix::sys::signal::{self, Signal, SigHandler};

#[derive(Clone)]
pub enum ExecuteeKind {
    StrongBuiltin(String),
    WeakBuiltin(String),
    Binary(PathBuf),
}

pub struct Executee {
    kind: ExecuteeKind,
    args: Vec<String>,
    vars: HashMap<String, String>,
    redirects: Vec<(RawFd, RawFd)>,
}

impl Executee {
    pub fn new(kind: ExecuteeKind, args: &Vec<String>, vars: &HashMap<String, String>) -> Executee {
        Executee {
            kind,
            args: args.clone(),
            vars: vars.clone(),
            redirects: Vec::new(),
        }
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
    for redirect in &executee.redirects {
        unistd::dup2(redirect.0, redirect.1).unwrap();
    }
    match executee.kind.clone() {
        ExecuteeKind::WeakBuiltin(name) => {
            let retcode = match name.as_str() {
                "state" => builtin::state(&executee.args),
                "fail" => builtin::fail(&executee.args),
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
        match unistd::fork() {
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
    match unistd::fork() {
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
                executees[i].redirects.push((pipe.1, 1));
                if let Ok(ForkResult::Child) = fork() {
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