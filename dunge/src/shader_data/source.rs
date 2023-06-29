use crate::{
    color::{Color, Rgb},
    layout::Plain,
    shader_data::len::LenUniform,
};

/// Light source parameters.
#[derive(Clone, Copy)]
pub struct Source {
    pub col: Rgb,
    pub pos: [f32; 3],
    pub rad: f32,
}

impl Source {
    pub(crate) fn into_uniform(self) -> SourceUniform {
        let Color(col) = self.col;
        SourceUniform::new(col, self.rad, self.pos)
    }
}

pub(crate) struct SourceArray {
    len: u32,
    buf: Box<[SourceUniform]>,
}

impl SourceArray {
    pub fn new(mut sources: Vec<SourceUniform>, max_size: usize) -> Self {
        assert!(sources.len() <= max_size, "too many light sources");
        sources.resize(max_size, SourceUniform::default());
        Self {
            len: sources.len() as u32,
            buf: sources.into_boxed_slice(),
        }
    }

    pub fn update(&mut self, offset: usize, sources: &[SourceUniform]) -> Result<(), UpdateError> {
        let buf = self.buf.get_mut(offset..).ok_or(UpdateError::Offset)?;
        if sources.len() > buf.len() {
            return Err(UpdateError::Len);
        }

        buf[..sources.len()].copy_from_slice(sources);
        Ok(())
    }

    pub fn set_len(&mut self, len: u32) -> Result<(), SetLenError> {
        if len as usize > self.buf.len() {
            return Err(SetLenError);
        }

        self.len = len;
        Ok(())
    }

    pub fn buf(&self) -> &[SourceUniform] {
        &self.buf
    }

    pub fn len(&self) -> LenUniform {
        LenUniform::new(self.len)
    }
}

#[derive(Debug)]
pub enum UpdateError {
    Offset,
    Len,
}

#[derive(Debug)]
pub struct SetLenError;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub(crate) struct SourceUniform {
    col: [f32; 3],
    rad: f32,
    pos: [f32; 3],
    pad: u32,
}

impl SourceUniform {
    fn new(col: [f32; 3], rad: f32, pos: [f32; 3]) -> Self {
        Self {
            col,
            rad,
            pos,
            pad: 0,
        }
    }
}

unsafe impl Plain for SourceUniform {}
