use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct Templater<'a> {
    subs: HashMap<&'a str, &'a str>,
}

impl<'a> Templater<'a> {
    pub fn insert(&mut self, key: &'a str, value: &'a str) -> &mut Self {
        self.subs.insert(key, value);
        self
    }

    pub fn format(&self, template: &'a str) -> Result<String, Error<'a>> {
        let mut out = String::with_capacity(template.len());
        let mut state = State::Tail(template);
        loop {
            match state.next() {
                Entry::Str(s) => out.push_str(s),
                Entry::Key(key) => {
                    let value = self.subs.get(key).ok_or(Error::KeyNotFound(key))?;
                    out.push_str(value);
                }
                Entry::Fail => return Err(Error::Parse),
                Entry::End => return Ok(out),
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Error<'a> {
    Parse,
    KeyNotFound(&'a str),
}

enum State<'a> {
    Tail(&'a str),
    Entry { key: &'a str, tail: &'a str },
    End,
}

impl<'a> State<'a> {
    fn next(&mut self) -> Entry<'a> {
        match *self {
            Self::Tail(tail) => {
                let Some((head, tail)) = tail.split_once("[[") else {
                    *self = Self::End;
                    return Entry::Str(tail);
                };

                let Some((key, tail)) = tail.split_once("]]") else {
                    *self = Self::End;
                    return Entry::Fail;
                };

                *self = Self::Entry { key, tail };
                Entry::Str(head)
            }
            Self::Entry { key, tail } => {
                *self = Self::Tail(tail);
                Entry::Key(key)
            }
            Self::End => Entry::End,
        }
    }
}

enum Entry<'a> {
    Str(&'a str),
    Key(&'a str),
    Fail,
    End,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn templater() {
        let template = "[[a]] + [[b]] = [[c]]";
        let mut t = Templater::default();
        t.insert("a", "1");
        t.insert("b", "2");
        t.insert("c", "3");
        assert_eq!(t.format(template), Ok("1 + 2 = 3".to_owned()));
    }

    #[test]
    fn templater_parse_error() {
        let template = "[[a";
        let t = Templater::default();
        assert_eq!(t.format(template), Err(Error::Parse));
    }

    #[test]
    fn templater_key_error() {
        let template = "[[a]]";
        let t = Templater::default();
        assert_eq!(t.format(template), Err(Error::KeyNotFound("a")));
    }
}
