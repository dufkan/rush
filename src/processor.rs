use std::collections::VecDeque;
use std::iter::{Iterator, FromIterator};

use pest::Parser;
use pest::iterators::Pair;
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
        let popped = self.raw.0.pop_back();
        self.parse();
        popped
    }

    pub fn pop_next(&mut self) -> Option<char> {
        let popped = self.raw.1.pop_front();
        self.parse();
        popped
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
        self.parsed = Self::parse_bash(self.raw())
    }

    fn parse_bash(input: String) -> Sequence {
        let mut sequence = Sequence::new();
        let parsed = BashParser::parse(Rule::line, &input);
        if parsed.is_err() {
            return sequence;
        }

        let mut seq = SequenceKind::Seq;

        for pair in parsed.unwrap() {
            match pair.as_rule() {
                Rule::command => sequence.add(seq, Self::parse_bash_command(pair)),
                Rule::separator => {
                    seq = Self::parse_bash_separator(pair);
                },
                Rule::EOI => (),
                _ => unreachable!(),
            }
        }

        sequence
    }

    fn parse_bash_command(command: Pair<Rule>) -> Command {
        assert!(command.as_rule() == Rule::command);
        let command = command.into_inner().next().unwrap();
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
                    Command::new(CommandKind::Execute, atoms)
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
                    Command::new(CommandKind::Assign, atoms)
            },
            _ => unreachable!(),
        }
    }

    fn parse_bash_separator(separator: Pair<Rule>) -> SequenceKind {
        assert!(separator.as_rule() == Rule::separator);
        match separator.as_str() {
            ";" => SequenceKind::Seq,
            "||" => SequenceKind::Or,
            "&&" => SequenceKind::And,
            _ => unreachable!(),
        }
    }
}
