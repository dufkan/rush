use std::os::unix::io::RawFd;
use std::slice;

use nix::sys::termios::{self, LocalFlags, InputFlags, SetArg};
use nix::unistd;

pub enum TerminalState {
    Initial,
    Custom,
}

pub struct Terminal {
    fd_in: RawFd,
    fd_out: RawFd,
    termios_initial: termios::Termios,
    termios_custom: termios::Termios,
}

impl Terminal {
    pub fn new(fd_in: RawFd, fd_out: RawFd) -> Terminal {
        let termios_initial = termios::tcgetattr(fd_in).unwrap();
        let mut termios_custom = termios_initial.clone();
        termios_custom.local_flags.remove(LocalFlags::ICANON);
        termios_custom.local_flags.remove(LocalFlags::ECHO);
        termios_custom.local_flags.remove(LocalFlags::ISIG);
        termios_custom.input_flags.remove(InputFlags::IXON);

        Terminal {
            fd_in,
            fd_out,
            termios_initial,
            termios_custom,
        }
    }

    pub fn set_state(&mut self, state: TerminalState) {
        let termios_next = match state {
            TerminalState::Custom => &self.termios_custom,
            TerminalState::Initial => &self.termios_initial,
        };
        termios::tcsetattr(self.fd_in, SetArg::TCSANOW, termios_next).unwrap();
    }

    pub fn write(&mut self, bytes: &[u8]) {
        unistd::write(self.fd_out, bytes).unwrap();
    }

    pub fn read(&mut self) -> u8 {
        let mut byte = 0;
        unistd::read(self.fd_in, slice::from_mut(&mut byte)).unwrap();
        byte
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.set_state(TerminalState::Initial);
    }
}