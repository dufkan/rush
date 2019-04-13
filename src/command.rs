use std::ffi::CString;
use std::path::Path;

use nix::errno::{self, Errno};
use nix::sys::signal::{self, Signal, SigHandler};
use nix::sys::wait::{self, WaitStatus};
use nix::unistd::{self, ForkResult};

pub fn run(bin: &Path, args: &[String]) -> u8 {
    let cargs: Vec<_> = args.iter()
        .map(String::clone)
        .map(CString::new)
        .map(Result::unwrap)
        .collect();
    let bin = CString::new(bin.to_str().unwrap()).unwrap();

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
            unsafe { signal::signal(Signal::SIGINT, SigHandler::SigIgn) }.unwrap();
            nix::unistd::execv(&bin, &cargs).unwrap();
            panic!("Child did not exec!");
        },
        Err(_) => {
            panic!("Fork failed!");
        },
    }
}

pub fn cd(args: &[String]) -> u8 {
    let path = match args.len() {
        1 => "~",
        2 => args[1].as_str(),
        _ => {
            eprintln!("cd: Too many arguments.");
            return 1;
        }
    };

    if unistd::chdir(path).is_err() {
        match errno::Errno::last() {
            Errno::EACCES => eprintln!("cd: Search permission denied."),
            Errno::EFAULT => eprintln!("cd: Path \"{}\" points outside accessible address space.", &args[1]),
            Errno::EIO => eprintln!("cd: And I/O error occurred."),
            Errno::ELOOP => eprintln!("cd: Too many symbolic links encountered."),
            Errno::ENAMETOOLONG => eprintln!("cd: Path is too long."),
            Errno::ENOENT => eprintln!("cd: The directory \"{}\" does not exist.", &args[1]),
            Errno::ENOMEM => eprintln!("cd: Insufficient kernel memory."),
            Errno::ENOTDIR => eprintln!("cd: \"{}\" is not a directory.", &args[1]),
            _ => eprintln!("cd: Unknown error."),
        };
        1
    } else {
        0
    }
}