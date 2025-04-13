//! Workload types.

use {
    crate::{shader::ShaderData, state::State},
    wgpu::ComputePipeline,
};

pub struct Workload {
    compute: ComputePipeline,
}

impl Workload {
    pub(crate) fn new(state: &State, shader: &ShaderData) -> Self {
        use wgpu::*;

        let desc = ComputePipelineDescriptor {
            label: None,
            layout: Some(shader.layout()),
            module: shader.module(),
            entry_point: Some("cs"),
            compilation_options: PipelineCompilationOptions {
                // dunge guarantees that the memory of all
                // buffers passed to shaders is initialized,
                // so there is no need to zero initialize memory
                zero_initialize_workgroup_memory: false,
                ..PipelineCompilationOptions::default()
            },
            cache: None,
        };

        let compute = state.device().create_compute_pipeline(&desc);
        Self { compute }
    }

    #[expect(dead_code)]
    pub(crate) fn compute(&self) -> &wgpu::ComputePipeline {
        &self.compute
    }
}
