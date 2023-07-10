use {
    crate::{
        layer::{ActiveLayer, Builder, Layer},
        pipeline::Pipeline,
        postproc::PostProcessor,
        render::Render,
    },
    wgpu::{CommandEncoder, TextureView},
};

/// The type that represented a current frame
/// and creates new [layers](crate::ActiveLayer).
pub struct Frame<'d> {
    render: &'d Render,
    frame_view: TextureView,
    encoder: Encoder,
}

impl<'d> Frame<'d> {
    pub(crate) fn new(render: &'d Render, frame_view: TextureView) -> Self {
        Self {
            render,
            frame_view,
            encoder: Encoder::default(),
        }
    }

    /// Starts the [layer](crate::Layer).
    pub fn layer<'l, S, T>(&'l mut self, layer: &'l Layer<S, T>) -> Builder<'d, 'l, S, T> {
        Builder::new(self, layer.pipeline())
    }

    pub(crate) fn start_layer<'l, S, T>(
        &'l mut self,
        pipeline: &'l Pipeline,
        clear_color: Option<[f64; 4]>,
        clear_depth: bool,
    ) -> ActiveLayer<'l, S, T> {
        use wgpu::*;

        // Before start a new layer, finish the previous one if it exists
        self.encoder.finish(self.render);

        let mut pass = self
            .encoder
            .get(self.render)
            .begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: self.render.framebuffer().render_view(),
                    resolve_target: None,
                    ops: Operations {
                        load: clear_color.map_or(LoadOp::Load, |[r, g, b, a]| {
                            LoadOp::Clear(Color { r, g, b, a })
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: self.render.framebuffer().depth_view(),
                    depth_ops: Some(Operations {
                        load: if clear_depth {
                            LoadOp::Clear(1.)
                        } else {
                            LoadOp::Load
                        },
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

        pass.set_pipeline(pipeline.as_ref());

        let screen = self.render.screen();
        let view_size = screen.virtual_size_with_antialiasing().as_vec2();
        pass.set_viewport(0., 0., view_size.x, view_size.y, 0., 1.);

        ActiveLayer::new(pass, screen.virtual_size().into(), pipeline.slots())
    }

    pub(crate) fn commit_in_frame(&mut self) {
        use wgpu::*;

        {
            let mut pass = self
                .encoder
                .get(self.render)
                .begin_render_pass(&RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &self.frame_view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Load,
                            store: false,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

            let post = self.render.post_processor();
            pass.set_pipeline(post.render_pipeline());
            pass.set_bind_group(PostProcessor::DATA_GROUP, post.data_bind_group(), &[]);
            pass.set_bind_group(
                PostProcessor::TEXTURE_GROUP,
                self.render.framebuffer().render_bind_group(),
                &[],
            );

            pass.draw(0..4, 0..1);
        }

        self.encoder.finish(self.render);
    }
}

#[derive(Default)]
struct Encoder(Option<CommandEncoder>);

impl Encoder {
    fn get(&mut self, render: &Render) -> &mut CommandEncoder {
        use wgpu::CommandEncoderDescriptor;

        self.0.get_or_insert_with(|| {
            render
                .state()
                .device()
                .create_command_encoder(&CommandEncoderDescriptor::default())
        })
    }

    fn finish(&mut self, render: &Render) {
        if let Some(encoder) = self.0.take() {
            render.state().queue().submit([encoder.finish()]);
        }
    }
}
