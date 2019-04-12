use std::ffi::CString;
use std::path::Path;

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
            nix::unistd::execv(&bin, &cargs).unwrap();
            panic!("Child did not exec!");
        },
        Err(_) => {
            panic!("Fork failed!");
        },
    }
}
