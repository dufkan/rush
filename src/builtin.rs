use std::collections::HashMap;

use nix::errno::{self, Errno};
use nix::unistd;

use super::config::Config;
use super::SHELL;

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

pub fn state(args: &[String]) -> u8 {
    let shell = unsafe { SHELL.as_ref().unwrap() };
    if args.len() == 2 {
        match args.get(1).unwrap().as_str() {
            "vars" => state_vars(shell.vars()),
            "bin" | "bin_dirs" => state_bin_dirs(shell.bin_dirs()),
            "config" => state_config(shell.config()),
            _ => eprintln!("{}: Unknown option. ", args.get(0).unwrap()),
        };
    } else {
        println!("VARS");
        state_vars(shell.vars());
        println!("\nBIN_DIRS");
        state_bin_dirs(shell.bin_dirs());
        println!("\nCONFIG");
        state_config(shell.config());
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

fn state_config(config: &Config) {
    print!("{}", config.to_string());
}

pub fn fail(args: &[String]) -> u8 {
    eprintln!("rush: Unknown command {}.", args[0]);
    1
}

pub fn void() -> u8 {
    0
}