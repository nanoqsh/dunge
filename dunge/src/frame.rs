use {
    crate::{
        _vertex::Vertex,
        color::Linear,
        error::ResourceNotFound,
        handles::LayerHandle,
        layer::{Builder, Layer},
        pipeline::Pipeline,
        render::Render,
        resources::Resources,
        shader,
    },
    wgpu::{CommandEncoder, TextureView},
};

/// The type that represented a current frame
/// and creates new [layers](crate::Layer).
pub struct Frame<'d> {
    render: &'d Render,
    resources: &'d Resources,
    encoder: Encoder,
    frame_view: TextureView,
    drawn_in_frame: bool,
}

impl<'d> Frame<'d> {
    pub(crate) fn new(
        render: &'d Render,
        resources: &'d Resources,
        frame_view: TextureView,
    ) -> Self {
        Self {
            render,
            resources,
            encoder: Encoder::default(),
            frame_view,
            drawn_in_frame: false,
        }
    }

    /// Draws the frame in the screen buffer.
    ///
    /// You usually don't need to call this method manually.
    /// It is called automatically at the end of the [`Frame`] lifetime.
    /// It can be useful if you want to partially render a frame in multiple layers.
    ///
    /// # Example
    /// ```
    /// # #[derive(Clone, Copy)]
    /// # struct Frame;
    /// # impl Frame {
    /// #     fn texture_layer(self) -> Self { self }
    /// #     fn start(self) -> Self { self }
    /// #     fn commit_in_frame(self) {}
    /// # }
    /// # let frame = Frame;
    /// // Create a new layer
    /// let mut layer = frame
    ///     .texture_layer()
    ///     .start();
    ///
    /// // Draw something in the layer
    ///
    /// // Drop the layer to release a frame
    /// drop(layer);
    ///
    /// // Commit the layer in frame
    /// frame.commit_in_frame();
    /// ```
    pub fn commit_in_frame(&mut self) {
        use wgpu::*;

        if !self.drawn_in_frame {
            return;
        }

        self.drawn_in_frame = false;

        {
            let mut pass = self
                .encoder
                .get(self.render)
                .begin_render_pass(&RenderPassDescriptor {
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
                shader::POST_TEXTURE_GROUP,
                self.render.framebuffer().render_bind_group(),
                &[],
            );
            pass.set_bind_group(
                shader::POST_DATA_GROUP,
                self.render.post_shader_data().bind_group(),
                &[],
            );

            pass.draw(0..4, 0..1);
        }

        self.encoder.finish(self.render);
    }

    /// Starts a [layer](crate::handles::LayerHandle).
    ///
    /// # Errors
    /// Returns [`ResourceNotFound`] if given instance handler was deleted.
    pub fn layer<V, T>(
        &mut self,
        handle: LayerHandle<V, T>,
    ) -> Result<Builder<'_, 'd, V, T>, ResourceNotFound> {
        Ok(Builder::new(self, self.resources.layers.get(handle.id())?))
    }

    /// Creates a new [layer](crate::Layer).
    pub(crate) fn start_layer<'l, V, T>(
        &'l mut self,
        pipeline: &'l Pipeline,
        clear_color: Option<Linear<f32>>,
        clear_depth: bool,
    ) -> Layer<V, T>
    where
        V: Vertex,
    {
        use wgpu::*;

        // Before start a new layer, finish the previous one if it exists
        self.encoder.finish(self.render);

        let mut pass = self
            .encoder
            .get(self.render)
            .begin_render_pass(&RenderPassDescriptor {
                label: Some("layer render pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: self.render.framebuffer().render_view(),
                    resolve_target: None,
                    ops: Operations {
                        load: clear_color.map_or(LoadOp::Load, |Linear([r, g, b, a])| {
                            LoadOp::Clear(Color {
                                r: r as f64,
                                g: g as f64,
                                b: b as f64,
                                a: a as f64,
                            })
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

        let (vw, vh) = self.render.screen().virtual_size();
        pass.set_viewport(0., 0., vw as f32, vh as f32, 0., 1.);

        Layer::new(
            pass,
            (vw, vh),
            self.render.context().queue(),
            self.resources,
            &mut self.drawn_in_frame,
        )
    }
}

#[derive(Default)]
struct Encoder(Option<CommandEncoder>);

impl Encoder {
    fn get(&mut self, render: &Render) -> &mut CommandEncoder {
        use wgpu::CommandEncoderDescriptor;

        self.0.get_or_insert_with(|| {
            render
                .context()
                .device()
                .create_command_encoder(&CommandEncoderDescriptor::default())
        })
    }

    fn finish(&mut self, render: &Render) {
        if let Some(encoder) = self.0.take() {
            render.context().queue().submit([encoder.finish()]);
        }
    }
}
