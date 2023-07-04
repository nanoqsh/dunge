use {
    crate::{
        color::{Color, Rgb},
        shader_data::len::LenUniform,
    },
    bytemuck::{Pod, Zeroable},
};

/// Light source parameters.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Source {
    col: [f32; 3],
    rad: f32,
    pos: [f32; 3],
    pad: u32,
}

impl Source {
    pub fn new<P>(Color(col): Rgb, pos: P, rad: f32) -> Self
    where
        P: Into<[f32; 3]>,
    {
        Self {
            col,
            rad,
            pos: pos.into(),
            pad: 0,
        }
    }
}

pub(crate) struct SourceArray {
    len: u32,
    buf: Box<[Source]>,
}

impl SourceArray {
    pub fn new(mut sources: Vec<Source>, max_size: usize) -> Self {
        assert!(sources.len() <= max_size, "too many light sources");
        sources.resize(max_size, Source::zeroed());
        Self {
            len: sources.len() as u32,
            buf: sources.into_boxed_slice(),
        }
    }

    pub fn update(&mut self, offset: usize, sources: &[Source]) -> Result<(), UpdateError> {
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

    pub fn buf(&self) -> &[Source] {
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
