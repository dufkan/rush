use super::common::{Event, Action};
use super::parser::Parser;

pub struct Shell {
    parser: Parser,
}

impl Shell {
    pub fn new() -> Shell {
        Shell { parser: Parser::new() }
    }

    pub fn event(&mut self, event: Event) -> Action {
        match event {
            Event::Char('\n') => Action::Process,
            Event::Char(c) => {
                self.parser.push(c);
                Action::None
            },
        }
    }

    pub fn process(&mut self) -> Action {
        if let Some(command) = self.parser.command() {
            println!("{}", command);
            let args = self.parser.args();
            for i in 0..args.len() {
                println!("  {}: {}", i, args[i]);
            }

            if command == "exit" {
                return Action::Exit;
            }
        } else {
            println!("Nothing to process.");
        }
        self.parser.clear();
        Action::None
    }
}
