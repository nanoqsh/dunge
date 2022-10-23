use {
    crate::{
        color::Linear,
        layer::{Layer, LayerBuilder},
        render::{GetPipeline, Render},
        shader_consts,
        vertex::{ColorVertex, FlatVertex, TextureVertex},
    },
    wgpu::{CommandEncoder, TextureView},
};

/// The type that represented a current frame
/// and creates new [layers](crate::Layer).
pub struct Frame<'d> {
    render: &'d Render,
    encoder: CommandEncoder,
    frame_view: TextureView,
    antialiasing: bool,
    drawn_in_frame: bool,
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
            antialiasing: false,
            drawn_in_frame: false,
        }
    }

    /// Sets the value of whether to apply antialiasing to the frame or not.
    pub fn set_antialiasing(&mut self, enable: bool) {
        self.antialiasing = enable;
    }

    /// Draws the frame in the screen buffer.
    ///
    /// You usually don't need to call this method manually.
    /// It is called automatically at the end of the [`Frame`] lifetime.
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

        pass.set_pipeline(self.render.post_pipeline(self.antialiasing).as_ref());
        pass.set_bind_group(
            shader_consts::post::T_DIFFUSE.group,
            self.render.render_frame().bind_group(),
            &[],
        );
        pass.set_bind_group(
            shader_consts::post::DATA.group,
            self.render.post_shader_data().bind_group(),
            &[],
        );

        pass.draw(0..4, 0..1);
    }

    pub(crate) fn submit(self) {
        self.render.queue().submit([self.encoder.finish()]);
    }

    pub fn texture_layer<'l>(&'l mut self) -> LayerBuilder<'l, 'd, TextureVertex> {
        LayerBuilder::new(self)
    }

    pub fn color_layer<'l>(&'l mut self) -> LayerBuilder<'l, 'd, ColorVertex> {
        LayerBuilder::new(self)
    }

    pub fn flat_layer<'l>(&'l mut self) -> LayerBuilder<'l, 'd, FlatVertex> {
        LayerBuilder::new(self)
    }

    /// Creates a new [layer](crate::Layer).
    pub(crate) fn start_layer<V>(
        &mut self,
        clear_color: Option<Linear<f64>>,
        clear_depth: bool,
    ) -> Layer<V>
    where
        Render: GetPipeline<V>,
    {
        use wgpu::*;

        let mut pass = self.encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("main render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: self.render.render_frame().view(),
                resolve_target: None,
                ops: Operations {
                    load: clear_color.map_or(LoadOp::Load, |Linear([r, g, b, a])| {
                        LoadOp::Clear(Color { r, g, b, a })
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: self.render.depth_frame().view(),
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

        pass.set_pipeline(self.render.get_pipeline().as_ref());

        Layer::new(
            pass,
            self.render.size().as_virtual(),
            self.render.queue(),
            self.render.resources(),
            &mut self.drawn_in_frame,
        )
    }
}
