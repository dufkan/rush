pub mod bash;

#[derive(Clone, Debug)]
pub enum AtomKind {
    Word(String),
    Pipe,
}

#[derive(Clone, Debug)]
pub struct Atom {
    kind: AtomKind,
    start: usize,
    end: usize,
}

impl Atom {
    pub fn new(kind: AtomKind, start: usize, end: usize) -> Atom {
        Atom { kind, start, end }
    }

    pub fn kind(&self) -> AtomKind {
        self.kind.clone()
    }
}

#[derive(Clone, Debug)]
pub enum CommandKind {
    Execute,
    Assign,
}

#[derive(Clone, Debug)]
pub struct Command {
    kind: CommandKind,
    atoms: Vec<Atom>,
}

impl Command {
    pub fn new(kind: CommandKind, atoms: Vec<Atom>) -> Command {
        Command { kind, atoms }    
    }

    pub fn kind(&self) -> CommandKind {
        self.kind.clone()
    }

    pub fn atoms(&self) -> Vec<Atom> {
        self.atoms.clone()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SequenceKind {
    Seq,
    And,
    Or
}

#[derive(Clone, Debug)]
pub struct Sequence {
    commands: Vec<(SequenceKind, Command)>,
}

impl Sequence {
    pub fn new() -> Sequence {
        Sequence { commands: Vec::new() }
    }

    pub fn add(&mut self, seq: SequenceKind, command: Command) {
        self.commands.push((seq, command))
    }

    pub fn get(self) -> Vec<(SequenceKind, Command)> {
        self.commands
    }
}