//! Workload types.

use {
    crate::{shader::ShaderData, state::State},
    std::marker::PhantomData,
};

pub struct Workload<I> {
    compute: wgpu::ComputePipeline,
    inp: PhantomData<I>,
}

impl<I> Workload<I> {
    pub(crate) fn new(state: &State, shader: &ShaderData) -> Self {
        let desc = wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(shader.layout()),
            module: shader.module(),
            entry_point: Some("cs"),
            compilation_options: wgpu::PipelineCompilationOptions {
                // dunge guarantees that the memory of all
                // buffers passed to shaders is initialized,
                // so there is no need to zero initialize memory
                zero_initialize_workgroup_memory: false,
                ..wgpu::PipelineCompilationOptions::default()
            },
            cache: None,
        };

        let compute = state.device().create_compute_pipeline(&desc);

        Self {
            compute,
            inp: PhantomData,
        }
    }

    pub(crate) fn compute(&self) -> &wgpu::ComputePipeline {
        &self.compute
    }
}
