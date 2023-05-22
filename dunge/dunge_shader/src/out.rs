use std::fmt::{self, Write};

pub(crate) struct Out {
    buf: String,
}

impl Out {
    pub fn new() -> Self {
        Self {
            buf: String::with_capacity(128),
        }
    }

    pub fn write<D>(&mut self, d: D) -> &mut Self
    where
        D: fmt::Display,
    {
        _ = write!(self.buf, "{d}");
        self
    }

    pub fn write_str(&mut self, s: &str) -> &mut Self {
        self.buf.push_str(s);
        self
    }

    pub fn separated<'a>(&'a mut self, sep: &'a str) -> Separated {
        Separated {
            out: self,
            sep,
            add: false,
        }
    }

    pub fn buf(&self) -> &str {
        &self.buf
    }
}

pub(crate) struct Separated<'a> {
    out: &'a mut Out,
    sep: &'a str,
    add: bool,
}

impl Separated<'_> {
    pub fn out(&mut self) -> &mut Out {
        if self.add {
            self.out.write_str(self.sep);
        } else {
            self.add = true;
        }

        self.out
    }

    pub fn write_default(&mut self, s: &str) {
        if !self.add {
            self.out.write_str(s);
        }
    }
}
