use std::{
    fmt::{Display, Write},
    ops,
};

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
        D: Display,
    {
        _ = write!(self.buf, "{d}");
        self
    }

    pub fn write_f32(&mut self, f: f32) -> &mut Self {
        if f.is_finite() {
            _ = write!(self.buf, "{f:?}");
        } else {
            self.buf.push_str("0.0");
        }

        self
    }

    pub fn write_str(&mut self, s: &str) -> &mut Self {
        self.buf.push_str(s);
        self
    }

    pub fn separated<'a>(&'a mut self, sep: &'a str) -> Separated<'a> {
        Separated {
            out: self,
            sep,
            add: false,
        }
    }
}

impl ops::Deref for Out {
    type Target = str;

    fn deref(&self) -> &Self::Target {
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
