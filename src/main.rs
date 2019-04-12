mod terminal;

use std::slice;
use std::os::unix::io::AsRawFd;

use terminal::{Terminal, TerminalState};

fn main() {
    let mut term = Terminal::new(std::io::stdin().as_raw_fd(), std::io::stdout().as_raw_fd());
    term.set_state(TerminalState::Custom);

    'command: loop {
        term.write("» ".as_bytes());

        'event: loop {
            let event = term.read();
            term.write(slice::from_ref(&event));
            match event {
                b'\n'   => break 'event,
                b'\x1b' => break 'command,
                _       => (),
            }
        }
    }
}
