use std::os::unix::io::RawFd;
use std::slice;
use std::str;

use nix::sys::termios::{self, LocalFlags, InputFlags, SetArg};
use nix::unistd;

use super::common::Event;

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

    pub fn write_char(&mut self, c: char) {
        let mut buff = [0; 4];
        c.encode_utf8(&mut buff);
        self.write(&buff);
    }

    pub fn read(&mut self) -> Event {
        match self.read_byte() {
            b'\n' | b'\r'         => Event::Return,
            b'\t'                 => Event::Tab,
            b'\x7f'               => Event::Backspace,
            b'\x1b'               => self.read_escape(),
            c @ b'\x01'...b'\x1f' => Event::Ctrl((c - 0x1 + b'A') as char),
            c @ b'\x20'...b'\x7e' => self.read_utf8(c),
            b'\0'                 => Event::Null,
            c                     => self.read_utf8(c),
        }
    }

    fn read_byte(&mut self) -> u8 {
        let mut byte = 0u8;
        unistd::read(self.fd_in, slice::from_mut(&mut byte)).unwrap();
        byte
    }

    fn read_escape(&mut self) -> Event {
        match self.read_byte() {
            b'O' => match self.read_byte() {
                b'P' => Event::F(1),
                b'Q' => Event::F(2),
                b'R' => Event::F(3),
                b'S' => Event::F(4),
                _    => Event::None,
            }
            b'[' => self.read_csi(),
            c    => {
                if let Event::Char(c) = self.read_utf8(c) {
                    Event::Alt(c)
                } else {
                    Event::None
                }
            },
        }
    }

    fn read_csi(&mut self) -> Event {
        match self.read_byte() {
            b'[' => {
                match self.read_byte() {
                    c @ b'A'...b'E' => Event::F(1 + c - b'A'),
                    _ => Event::None,
                }
            },
            b'A' => Event::Up,
            b'B' => Event::Down,
            b'C' => Event::Right,
            b'D' => Event::Left,
            b'F' => Event::End,
            b'H' => Event::Home,
            c @ b'0'...b'9' => {
                let mut buff = [0; 2];
                buff[0] = c;
                if buff[0] < b'1' || buff[0] > b'8' {
                    return Event::None;
                }

                buff[1] = self.read_byte();
                if buff[1] != b'~' && self.read_byte() != b'~' {
                    return Event::None; 
                }

                match str::from_utf8(&buff).unwrap() {
                    "1~"  |
                    "7~"  => Event::Home,
                    "2~"  => Event::Insert,
                    "3~"  => Event::Delete,
                    "4~"  |
                    "8~"  => Event::End,
                    "5~"  => Event::PageUp,
                    "6~"  => Event::PageDown,
                    "11" => Event::F(1),
                    "12" => Event::F(2),
                    "13" => Event::F(3),
                    "14" => Event::F(4),
                    "15" => Event::F(5),
                    "17" => Event::F(6),
                    "18" => Event::F(7),
                    "19" => Event::F(8),
                    "20" => Event::F(9),
                    "21" => Event::F(10),
                    "23" => Event::F(11),
                    "24" => Event::F(12),
                    _    => Event::None,
                }
            },
            _    => Event::None,
        }
    }

    fn read_utf8(&mut self, first: u8) -> Event {
        if first.is_ascii() {
            return Event::Char(first as char);
        }

        let mut buff = [0; 4];
        buff[0] = first;
        for i in 1..4 {
            buff[i] = self.read_byte();
            if let Ok(s) = str::from_utf8(&buff[..=i]) {
                return Event::Char(s.chars().next().unwrap());
            }
        }

        Event::None
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.set_state(TerminalState::Initial);
    }
}
