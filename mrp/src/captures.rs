#[derive(Debug, PartialEq)]
struct Capture<'i> {
    name: &'i str,
    value: &'i str,
}

#[derive(Debug, PartialEq)]
pub struct Captures<'i> {
    inner: Vec<Capture<'i>>,
}

impl<'i> Captures<'i> {
    pub fn new() -> Self {
        Self { inner: vec![] }
    }
    pub fn put(&mut self, name: &'i str, value: &'i str) {
        self.inner.push(Capture { name, value });
    }
    pub fn get(&self, name: &str) -> Option<&str> {
        self.inner.iter().find(|c| c.name == name).map(|c| c.value)
    }
}
