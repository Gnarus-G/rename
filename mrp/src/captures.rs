#[derive(Debug, PartialEq)]
struct Capture<'source, 'input> {
    name: &'source str,
    value: &'input str,
}

#[derive(Debug, PartialEq)]
pub struct Captures<'source, 'input> {
    inner: Vec<Capture<'source, 'input>>,
}

impl<'source, 'input> Captures<'source, 'input> {
    pub fn new() -> Self {
        Self { inner: vec![] }
    }
    pub fn put(&mut self, name: &'source str, value: &'input str) {
        self.inner.push(Capture { name, value });
    }
    pub fn get(&self, name: &str) -> Option<&str> {
        self.inner.iter().find(|c| c.name == name).map(|c| c.value)
    }
}
