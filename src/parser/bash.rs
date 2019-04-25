use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;

use super::{Sequence, SequenceKind, Command, CommandKind, Atom, AtomKind};

#[derive(Parser)]
#[grammar = "parser/bash.pest"]
struct BashParser;

pub fn parse(input: String) -> Sequence {
    let mut sequence = Sequence::new();
    let parsed = BashParser::parse(Rule::line, &input);
    if parsed.is_err() {
        return sequence;
    }

    let mut seq = SequenceKind::Seq;

    for pair in parsed.unwrap() {
        match pair.as_rule() {
            Rule::command => sequence.add(seq, parse_command(pair)),
            Rule::separator => {
                seq = parse_separator(pair);
            },
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }

    sequence
}

fn parse_command(command: Pair<Rule>) -> Command {
    assert!(command.as_rule() == Rule::command);
    let command = command.into_inner().next().unwrap();
    match command.as_rule() {
        Rule::execute => {
            let atoms: Vec<_> = command.into_inner()
                .map(|pair| {
                    let span = pair.as_span();
                    let kind = match pair.as_rule() {
                        Rule::redirect => match span.as_str() {
                            "|" => AtomKind::Pipe,
                            _ => unreachable!(),
                        },
                        Rule::word => AtomKind::Word(String::from(span.as_str())),
                        _ => unreachable!(),
                    };
                    Atom::new(kind, span.start(), span.end())
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

fn parse_separator(separator: Pair<Rule>) -> SequenceKind {
    assert!(separator.as_rule() == Rule::separator);
    match separator.as_str() {
        ";" => SequenceKind::Seq,
        "||" => SequenceKind::Or,
        "&&" => SequenceKind::And,
        _ => unreachable!(),
    }
}