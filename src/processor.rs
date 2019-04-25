use std::collections::VecDeque;
use std::iter::{Iterator, FromIterator};

use super::parser::{bash, Sequence};

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
        self.parsed = bash::parse(self.raw())
    }
}
