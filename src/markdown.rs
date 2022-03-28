pub struct Document {
    lines: Vec<String>,
}

impl Document {
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }

    pub fn append(&mut self, mut other: Self) {
        self.lines.append(&mut other.lines);
    }

    pub fn h1(&mut self, s: &str) {
        self.lines.push(format!("# {}", s));
    }

    pub fn h2(&mut self, s: &str) {
        self.lines.push(format!("## {}", s));
    }

    pub fn h3(&mut self, s: &str) {
        self.lines.push(format!("### {}", s));
    }

    pub fn h4(&mut self, s: &str) {
        self.lines.push(format!("#### {}", s));
    }

    pub fn h5(&mut self, s: &str) {
        self.lines.push(format!("##### {}", s));
    }

    pub fn p(&mut self, s: &str) {
        self.lines.push(String::from(s));
    }
}
