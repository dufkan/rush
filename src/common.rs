pub enum Event {
    Left,
    Right,
    Up,
    Down,
    Backspace,
    Return,
    Tab,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    Delete,
    Null,
    F(u8),
    Char(char),
    Ctrl(char),
    Alt(char),
    None,
}

pub enum Action {
    Process,
    Exit,
    None,
}
