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
                        Rule::redirect => parse_redirect(pair),
                        Rule::word => AtomKind::Word(String::from(span.as_str())),
                        Rule::pipe => AtomKind::Pipe,
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

fn parse_redirect(redirect: Pair<Rule>) -> AtomKind {
    let redirect = redirect.into_inner().next().unwrap();
    match redirect.as_rule() {
        Rule::redirect_move_in => {
            let mut params = redirect.into_inner().rev();
            let src = params.next().unwrap().as_str().parse().unwrap();
            let dst = params.next().as_ref().map(Pair::as_str).unwrap_or("0").parse().unwrap();
            AtomKind::FdMov(src, dst)
        },
        Rule::redirect_move_out => {
            let mut params = redirect.into_inner().rev();
            let src = params.next().unwrap().as_str().parse().unwrap();
            let dst = params.next().as_ref().map(Pair::as_str).unwrap_or("1").parse().unwrap();
            AtomKind::FdMov(src, dst)
        },
        Rule::redirect_duplicate_in => {
            let mut params = redirect.into_inner().rev();
            let src = params.next().unwrap().as_str().parse().unwrap();
            let dst = params.next().as_ref().map(Pair::as_str).unwrap_or("0").parse().unwrap();
            AtomKind::FdDup(src, dst)
        },
        Rule::redirect_duplicate_out => {
            let mut params = redirect.into_inner().rev();
            let src = params.next().unwrap().as_str().parse().unwrap();
            let dst = params.next().as_ref().map(Pair::as_str).unwrap_or("1").parse().unwrap();
            AtomKind::FdDup(src, dst)
        },
        Rule::redirect_write => {
            let mut params = redirect.into_inner().rev();
            let file = String::from(params.next().unwrap().as_str());
            let fd = params.next().as_ref().map(Pair::as_str).unwrap_or("1").parse().unwrap();
            AtomKind::FileWrite(file, fd)
        },
        Rule::redirect_read => {
            let mut params = redirect.into_inner().rev();
            let file = String::from(params.next().unwrap().as_str());
            let fd = params.next().as_ref().map(Pair::as_str).unwrap_or("0").parse().unwrap();
            AtomKind::FileRead(file, fd)
        },
        Rule::redirect_append => {
            let mut params = redirect.into_inner().rev();
            let file = String::from(params.next().unwrap().as_str());
            let fd = params.next().as_ref().map(Pair::as_str).unwrap_or("1").parse().unwrap();
            AtomKind::FileAppend(file, fd)
        },
        Rule::redirect_rw => {
            let mut params = redirect.into_inner().rev();
            let file = String::from(params.next().unwrap().as_str());
            let fd = params.next().as_ref().map(Pair::as_str).unwrap_or("0").parse().unwrap();
            AtomKind::FileRW(file, fd)
        },
        Rule::redirect_std_write => {
            let file = String::from(redirect.into_inner().next().unwrap().as_str());
            AtomKind::StdWrite(file)
        },
        Rule::redirect_std_append => {
            let file = String::from(redirect.into_inner().next().unwrap().as_str());
            AtomKind::StdAppend(file)
        },
        _ => unreachable!()
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