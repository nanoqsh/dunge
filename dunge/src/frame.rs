use {
    crate::{
        color::{IntoLinear, Linear},
        layer::Layer,
        render::{GetPipeline, Render},
        shader_consts,
        vertex::{ColorVertex, TextureVertex},
    },
    wgpu::{CommandEncoder, TextureView},
};

/// A struct represented a current frame
/// and exists during a frame render.
pub struct Frame<'d> {
    render: &'d Render,
    encoder: CommandEncoder,
    frame_view: TextureView,
}

impl<'d> Frame<'d> {
    pub(crate) fn new(render: &'d Render, frame_view: TextureView) -> Self {
        use wgpu::*;

        let encoder = render
            .device()
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        Self {
            render,
            encoder,
            frame_view,
        }
    }

    /// Draws the frame in the screen buffer.
    pub(crate) fn draw_frame(&mut self) {
        use wgpu::*;

        let mut pass = self.encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("post render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &self.frame_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        pass.set_pipeline(self.render.post_pipeline().as_ref());
        pass.set_bind_group(
            shader_consts::post::T_DIFFUSE.group,
            self.render.render_frame().bind_group(),
            &[],
        );
        pass.set_bind_group(
            shader_consts::post::SCREEN.group,
            self.render.screen().bind_group(),
            &[],
        );

        pass.draw(0..4, 0..1);
    }

    pub(crate) fn submit(self) {
        self.render.queue().submit([self.encoder.finish()]);
    }

    pub fn start_texture_layer<C>(&mut self, col: C) -> Layer<TextureVertex>
    where
        C: IntoLayerColor,
    {
        Self::start_layer(self, col.into_layer_color())
    }

    pub fn start_color_layer<C>(&mut self, col: C) -> Layer<ColorVertex>
    where
        C: IntoLayerColor,
    {
        Self::start_layer(self, col.into_layer_color())
    }

    fn start_layer<V>(&mut self, col: Option<Linear<f64>>) -> Layer<V>
    where
        Render: GetPipeline<V>,
    {
        use wgpu::*;

        let load = col.map_or(LoadOp::Load, |Linear([r, g, b, a])| {
            LoadOp::Clear(Color { r, g, b, a })
        });

        let mut pass = self.encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("main render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: self.render.render_frame().view(),
                resolve_target: None,
                ops: Operations { load, store: true },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: self.render.depth_frame().view(),
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        pass.set_pipeline(self.render.get_pipeline().as_ref());

        Layer::new(
            pass,
            self.render.size().as_virtual(),
            self.render.queue(),
            self.render.resources(),
        )
    }
}

pub trait IntoLayerColor {
    fn into_layer_color(self) -> Option<Linear<f64>>;
}

impl IntoLayerColor for () {
    fn into_layer_color(self) -> Option<Linear<f64>> {
        None
    }
}

impl<L> IntoLayerColor for L
where
    L: IntoLinear,
{
    fn into_layer_color(self) -> Option<Linear<f64>> {
        Some(self.into_linear())
    }
}
