pub struct Parser {
    raw: String,
    args: Vec<String>,
}

impl Parser {
    pub fn new() -> Parser {
        Parser { raw: String::new(), args: Vec::new() }
    }

    pub fn clear(&mut self) {
        self.raw.clear();
        self.parse();
    }

    pub fn push(&mut self, c: char) {
        self.raw.push(c);
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
        let mut args = Vec::new();
        for item in &self.args {
            args.push(item.clone());
        }
        args
    }

    fn parse(&mut self) {
        self.args = self.raw.split_whitespace().map(String::from).collect();
    }
}