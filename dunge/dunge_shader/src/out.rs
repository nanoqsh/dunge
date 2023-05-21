use std::fmt;

pub(crate) struct Out {
    buf: String,
}

impl Out {
    pub fn new() -> Self {
        Self {
            buf: String::with_capacity(128),
        }
    }

    pub fn write<D>(&mut self, d: D)
    where
        D: fmt::Display,
    {
        use fmt::Write;

        _ = write!(self.buf, "{d}");
    }

    pub fn write_str(&mut self, s: &str) {
        self.buf.push_str(s);
    }

    pub fn buf(&self) -> &str {
        &self.buf
    }
}
