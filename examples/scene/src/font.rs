use {
    crate::{FontShader, FontVert},
    dunge::{
        Context, Format, Instance, Layer, Mesh, MeshData, ModelTransform, TextureData, Textures,
    },
    serde::Deserialize,
    std::collections::HashMap,
};

pub(crate) struct Font {
    pub map: Textures<FontShader>,
    size: (u32, u32),
    atlas: Atlas,
    pub instance: Instance,
    pub mesh: Mesh<FontVert>,
    pub n: u32,
}

impl Font {
    const MAX_SYMBOLS: usize = 32;

    pub fn new(context: &mut Context, layer: &Layer<FontShader>) -> Self {
        let image = utils::decode_gray_png(include_bytes!("atlas.png"));
        let size = image.dimensions();
        let data = TextureData::new(&image, size, Format::Gray).expect("create atlas texture");

        let map = context.textures_builder().with_map(data).build(layer);
        let atlas = serde_json::from_str(include_str!("atlas.json")).expect("read atlas map");
        let instance = context.create_instances(&[ModelTransform::default()]);

        let quads = vec![[FontVert::default(); 4]; Self::MAX_SYMBOLS];
        let data = MeshData::from_quads(&quads).expect("create atlas mesh");
        let mesh = context.create_mesh(&data);

        Self {
            map,
            size,
            atlas,
            instance,
            mesh,
            n: 0,
        }
    }

    pub fn write(&mut self, s: &str, (sw, sh): (u32, u32)) {
        const PADDING: i32 = 16;
        const FONT_SIZE: i32 = 2;
        const SPACE_WIDTH: i32 = 4;

        let mut px = -(sw as i32 - PADDING);
        let py = sh as i32 - PADDING;
        let (mw, mh) = {
            let (mw, mh) = self.size;
            (mw as f32, mh as f32)
        };

        let (sw, sh) = (sw as f32, sh as f32);
        let vert = |x, y, u, v| FontVert {
            pos: [x, y],
            map: [u, v],
        };

        self.n = 0;
        let mut quads = Vec::with_capacity(Self::MAX_SYMBOLS * 4);
        for c in s.chars().take(Self::MAX_SYMBOLS) {
            self.n += 2;
            if c == ' ' {
                px += SPACE_WIDTH * FONT_SIZE + FONT_SIZE;
                continue;
            }

            let Rect { u, v, w, h } = self.atlas.get(c);
            let (x, y) = (px as f32 / sw, py as f32 / sh);
            let (dx, dy) = (
                w as f32 / sw * FONT_SIZE as f32,
                h as f32 / sh * FONT_SIZE as f32,
            );

            let (u, v) = (u as f32 / mw, v as f32 / mh);
            let (du, dv) = (w as f32 / mw, h as f32 / mh);
            quads.extend([
                vert(x, y, u, v),
                vert(x + dx, y, u + du, v),
                vert(x + dx, y - dy, u + du, v + dv),
                vert(x, y - dy, u, v + dv),
            ]);

            px += w as i32 * FONT_SIZE + FONT_SIZE;
        }

        self.mesh.update_verts(&quads).expect("update font mesh");
    }
}

#[derive(Deserialize)]
struct Atlas(HashMap<char, Rect>);

impl Atlas {
    fn get(&self, c: char) -> Rect {
        match self.0.get(&c) {
            Some(&rect) => rect,
            None => panic!("unknown character {c:?}"),
        }
    }
}

#[derive(Clone, Copy, Deserialize)]
#[serde(from = "[u32; 4]")]
struct Rect {
    u: u32,
    v: u32,
    w: u32,
    h: u32,
}

impl From<[u32; 4]> for Rect {
    fn from([u, v, w, h]: [u32; 4]) -> Self {
        Self { u, v, w, h }
    }
}
