use {
    crate::{
        framebuffer::Framebuffer,
        layer::{ActiveLayer, Builder, Layer},
        pipeline::Pipeline,
        posteffect::PostEffect,
        postproc::PostProcessor,
        render::State,
        screen::RenderScreen,
        shader_data::Instance,
    },
    wgpu::{CommandEncoder, TextureView},
};

/// The type that represented a current frame
/// and creates new [layers](crate::ActiveLayer).
pub struct Frame<'d> {
    shot: Snapshot<'d>,
    view: TextureView,
    encoder: Encoder,
    drawn: bool,
}

impl<'d> Frame<'d> {
    pub(crate) fn new(shot: Snapshot<'d>, view: TextureView) -> Self {
        Self {
            shot,
            view,
            encoder: Encoder::default(),
            drawn: false,
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

        let shot = &self.shot;

        // Before start a new layer, finish the previous one if it exists
        self.encoder.finish(shot.state);
        self.drawn = false;

        let mut pass = self
            .encoder
            .get(shot.state)
            .begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: shot.framebuffer.render_view(),
                    resolve_target: None,
                    ops: Operations {
                        load: clear_color.map_or(LoadOp::Load, |[r, g, b, a]| {
                            LoadOp::Clear(Color { r, g, b, a })
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: shot.framebuffer.depth_view(),
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

        let view_size = shot.screen.virtual_size_with_antialiasing().as_vec2();
        pass.set_viewport(0., 0., view_size.x, view_size.y, 0., 1.);
        pass.set_pipeline(pipeline.as_ref());
        ActiveLayer::new(
            pass,
            shot.screen.virtual_size().into(),
            pipeline.slots(),
            shot.instance,
        )
    }

    /// Writes a final frame on the screen.
    ///
    /// When you call this, you may experience problems with borrowing frame references.
    /// This is intentional. You should drop the layer object before calling this method.
    ///
    /// To apply a [post-effect](PostEffect) to the frame, call the
    /// [`draw_on_screen_with`](Frame::draw_on_screen_with) method.
    pub fn draw_on_screen(&mut self) {
        self.draw(None);
    }

    /// Writes a final frame on the screen with the [post-effect](PostEffect).
    pub fn draw_on_screen_with(&mut self, ef: &PostEffect) {
        self.draw(Some(ef));
    }

    fn draw(&mut self, ef: Option<&PostEffect>) {
        use wgpu::*;

        // Skip render if not needed
        if self.drawn {
            log::info!("draw frame (skipped)");
            return;
        }

        log::info!("draw frame");
        let shot = &mut self.shot;
        let locked_postproc;
        {
            let mut pass = self
                .encoder
                .get(shot.state)
                .begin_render_pass(&RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &self.view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Load,
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

            let params = shot.screen.frame_parameters();
            let postproc: &PostProcessor = if let Some(ef) = ef {
                locked_postproc = ef.with_parameters(shot.state, params);
                &locked_postproc
            } else {
                shot.postproc.set_parameters(shot.state, params);
                shot.postproc
            };

            pass.set_pipeline(postproc.render_pipeline());
            pass.set_bind_group(PostProcessor::DATA_GROUP, postproc.data_bind_group(), &[]);
            pass.set_bind_group(
                PostProcessor::TEXTURE_GROUP,
                postproc.render_bind_group(shot.state, shot.framebuffer.render_view()),
                &[],
            );

            pass.draw(0..4, 0..1);
        }

        self.encoder.finish(shot.state);
        self.drawn = true;
    }
}

pub(crate) struct Snapshot<'d> {
    pub state: &'d State,
    pub framebuffer: &'d Framebuffer,
    pub postproc: &'d mut PostProcessor,
    pub screen: RenderScreen,
    pub instance: &'d Instance,
}

#[derive(Default)]
struct Encoder(Option<CommandEncoder>);

impl Encoder {
    fn get(&mut self, state: &State) -> &mut CommandEncoder {
        use wgpu::CommandEncoderDescriptor;

        self.0.get_or_insert_with(|| {
            state
                .device()
                .create_command_encoder(&CommandEncoderDescriptor::default())
        })
    }

    fn finish(&mut self, state: &State) {
        if let Some(encoder) = self.0.take() {
            state.queue().submit([encoder.finish()]);
        }
    }
}
