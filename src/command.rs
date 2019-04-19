use std::collections::HashMap;
use std::ffi::CString;
use std::path::Path;

use nix::errno::{self, Errno};
use nix::sys::signal::{self, Signal, SigHandler};
use nix::sys::wait::{self, WaitStatus};
use nix::unistd::{self, ForkResult};

pub fn run(bin: &Path, args: &[String], vars: &HashMap<String, String>) -> u8 {
    let bin = CString::new(bin.to_str().unwrap()).unwrap();
    let cargs: Vec<_> = args.iter()
        .map(String::clone)
        .map(CString::new)
        .map(Result::unwrap)
        .collect();
    let cvars: Vec<_> = vars.iter()
        .map(|var| CString::new(format!("{}={}", var.0, var.1)).unwrap())
        .collect();

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
            nix::unistd::execve(&bin, &cargs, &cvars).unwrap();
            panic!("Child did not exec!");
        },
        Err(_) => {
            panic!("Fork failed!");
        },
    }
}

pub fn cd(args: &[String]) -> u8 {
    let path = match args.len() {
        1 => {
            eprintln!("cd: Not enough arguments.");
            return 1;
        },
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

pub fn state(args: &[String], vars: &HashMap<String, String>, bin_dirs: &Vec<String>) -> u8 {
    if args.len() == 2 {
        match args.get(1).unwrap().as_str() {
            "vars" => state_vars(vars),
            "bin" | "bin_dirs" => state_bin_dirs(bin_dirs),
            _ => eprintln!("{}: Unknown option. ", args.get(0).unwrap()),
        };
    } else {
        println!("VARS");
        state_vars(vars);
        println!("\nBIN_DIRS");
        state_bin_dirs(bin_dirs);
    }
    0

}

fn state_vars(vars: &HashMap<String, String>) {
    for var in vars {
        println!("{}={}", var.0, var.1);
    }
}

fn state_bin_dirs(bin_dirs: &Vec<String>) {
    for dir in bin_dirs {
        println!("{}", dir);
    }
}