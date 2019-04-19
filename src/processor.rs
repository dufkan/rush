use std::collections::VecDeque;
use std::iter::{Iterator, FromIterator};

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "bash.pest"]
struct BashParser;

#[derive(Clone, Debug)]
pub enum AtomKind {
    Word(String),
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

#[derive(Clone, Debug)]
pub enum SequenceKind {
    Seq,
}

#[derive(Clone, Debug)]
pub struct Sequence {
    commands: Vec<(SequenceKind, Command)>,
}

impl Sequence {
    pub fn new() -> Sequence {
        Sequence { commands: Vec::new() }
    }

    pub fn seq(&mut self, command: Command) {
        self.commands.push((SequenceKind::Seq, command))
    }

    pub fn get(self) -> Vec<(SequenceKind, Command)> {
        self.commands
    }
}

pub struct Processor {
    raw: (VecDeque<char>, VecDeque<char>),
    parsed: Sequence,
}

impl Processor {
    pub fn new() -> Processor {
        Processor { 
            raw: (VecDeque::new(), VecDeque::new()),
            parsed: Sequence::new()
        }
    }

    pub fn clear(&mut self) {
        self.raw.0.clear();
        self.raw.1.clear();
        self.parse();
    }

    pub fn push(&mut self, c: char) {
        self.raw.0.push_back(c);
        self.parse();
    }

    pub fn get(&self) -> Sequence {
        self.parsed.clone()
    }

    pub fn is_empty(&self) -> bool {
        self.raw.0.is_empty() && self.raw.1.is_empty()
    }

    pub fn pop_prev(&mut self) -> Option<char> {
        self.raw.0.pop_back()
    }

    pub fn pop_next(&mut self) -> Option<char> {
        self.raw.1.pop_front()
    }

    pub fn raw(&self) -> String {
        String::from_iter(self.raw.0.iter().chain(self.raw.1.iter()))
    }

    pub fn set(&mut self, raw: String) {
        self.raw.0 = VecDeque::from_iter(raw.chars());
        self.raw.1.clear();
        self.parse();
    }

    pub fn left(&mut self) -> bool {
        if let Some(c) = self.raw.0.pop_back() {
            self.raw.1.push_front(c);
            true
        } else {
            false
        }
    }

    pub fn right(&mut self) -> bool {
        if let Some(c) = self.raw.1.pop_front() {
            self.raw.0.push_back(c);
            true
        } else {
            false
        }
    }

    pub fn position(&self) -> usize {
        self.raw.1.len()
    }

    pub fn parse(&mut self) {
        self.parsed = self.parse_bash()
    }

    fn parse_bash(&mut self) -> Sequence {
        let input = self.raw();
        let mut sequence = Sequence::new();
        let parsed = BashParser::parse(Rule::line, &input);
        if parsed.is_err() {
            return sequence;
        }

        let mut line = parsed.unwrap().next().unwrap().into_inner();
        let command = line.next().unwrap().into_inner().next().unwrap();
        match command.as_rule() {
            Rule::execute => {
                let atoms: Vec<_> = command.into_inner()
                    .map(|pair| pair.as_span())
                    .map(|span| {
                        Atom::new(
                            AtomKind::Word(String::from(span.as_str())),
                            span.start(),
                            span.end()
                        )
                    })
                    .collect();
                sequence.seq(Command::new(CommandKind::Execute, atoms));
            },
            Rule::assign => {
                let atoms: Vec<_> = command.into_inner()
                    .map(|pair| pair.as_span())
                    .map(|span| {
                        Atom::new(
                            AtomKind::Word(String::from(span.as_str())),
                            span.start(),
                            span.end()
                        )
                    })
                    .collect();
                sequence.seq(Command::new(CommandKind::Assign, atoms));
            },
            _ => unreachable!(),
        };
        sequence
    }
}
