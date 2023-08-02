use {serde::Deserialize, std::collections::HashMap};

#[derive(Deserialize)]
pub struct Atlas(HashMap<char, Rect>);

impl Atlas {
    pub fn get(&self, c: char) -> Rect {
        match self.0.get(&c) {
            Some(&rect) => rect,
            None => panic!("unknown character {c:?}"),
        }
    }
}

#[derive(Clone, Copy, Deserialize)]
#[serde(from = "[u32; 4]")]
pub struct Rect {
    pub u: u32,
    pub v: u32,
    pub w: u32,
    pub h: u32,
}

impl From<[u32; 4]> for Rect {
    fn from([u, v, w, h]: [u32; 4]) -> Self {
        Self { u, v, w, h }
    }
}
