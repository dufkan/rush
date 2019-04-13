use std::collections::VecDeque;
use std::iter::FromIterator;

pub struct Parser {
    raw: (VecDeque<char>, VecDeque<char>),
    args: Vec<String>,
}

impl Parser {
    pub fn new() -> Parser {
        Parser { 
            raw: (VecDeque::new(), VecDeque::new()),
            args: Vec::new()
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

    pub fn command(&self) -> Option<String> {
        if !self.args.is_empty() {
            Some(self.args[0].clone())
        } else {
            None
        }
    }

    pub fn args(&self) -> Vec<String> {
        self.args.iter().map(String::clone).collect()
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

    fn parse(&mut self) {
        self.args = self.raw().split_whitespace().map(String::from).collect();
    }
}
