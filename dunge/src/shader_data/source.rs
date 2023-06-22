use {crate::layout::Plain, std::iter};

/// Light source parameters.
#[derive(Clone, Copy)]
pub struct Source {
    pub col: [f32; 3],
    pub pos: [f32; 3],
    pub rad: f32,
}

pub(crate) struct SourceArray {
    buf: Box<[SourceUniform]>,
    len: u32,
}

impl SourceArray {
    pub fn new(sources: &[Source], max_size: usize) -> Self {
        assert!(sources.len() <= max_size);
        let mut buf = vec![SourceUniform::default(); max_size];
        for (uniform, &source) in iter::zip(&mut buf, sources) {
            *uniform = SourceUniform::new(source);
        }

        Self {
            buf: buf.into_boxed_slice(),
            len: sources.len() as u32,
        }
    }

    pub fn update(&mut self, offset: usize, sources: &[Source]) -> Result<(), UpdateError> {
        let buf = self.buf.get_mut(offset..).ok_or(UpdateError::Offset)?;
        if sources.len() > buf.len() {
            return Err(UpdateError::Len);
        }

        for (uniform, &source) in iter::zip(buf, sources) {
            *uniform = SourceUniform::new(source);
        }

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

    pub fn len(&self) -> u32 {
        self.len
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
    fn new(Source { col, pos, rad }: Source) -> Self {
        Self {
            col,
            pos,
            rad,
            pad: 0,
        }
    }
}

unsafe impl Plain for SourceUniform {}
