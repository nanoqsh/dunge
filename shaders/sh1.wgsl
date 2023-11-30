struct InstanceInput {
    @location(0) r0: vec4<f32>,
    @location(1) r1: vec4<f32>,
    @location(2) r2: vec4<f32>,
    @location(3) r3: vec4<f32>,
}

struct InstanceColorInput {
    @location(4) col: vec3<f32>,
}

struct VertexInput {
    @location(5) pos: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) instcol: vec3<f32>,
}

struct Camera {
    view: mat4x4<f32>,
}


@group(0) @binding(0) var<uniform> camera: Camera;


@vertex
fn vsmain(inst: InstanceInput, instcol: InstanceColorInput, input: VertexInput) -> VertexOutput {
    let model = mat4x4<f32>(
        inst.r0,
        inst.r1,
        inst.r2,
        inst.r3,
    );

    var out: VertexOutput;
    let world = model * vec4(input.pos, 1.);
    out.pos = camera.view * world;
    out.instcol = instcol.col;

    return out;
}

@fragment
fn fsmain(out: VertexOutput) -> @location(0) vec4<f32> {
    var col: vec3<f32>;
    col = out.instcol;

    return vec4(col, 1.);
}
