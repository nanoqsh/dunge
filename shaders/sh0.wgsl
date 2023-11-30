struct InstanceInput {
    @location(0) r0: vec4<f32>,
    @location(1) r1: vec4<f32>,
    @location(2) r2: vec4<f32>,
    @location(3) r3: vec4<f32>,
}

struct VertexInput {
    @location(4) pos: vec3<f32>,
    @location(5) map: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) map: vec2<f32>,
    @location(1) world: vec3<f32>,
    @location(2) space_0: vec3<f32>,
}

struct Camera {
    view: mat4x4<f32>,
}

struct Source {
    col: vec3<f32>,
    rad: f32,
    pos: vec3<f32>,
}

struct Len {
    n: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
}

struct Space {
    model: mat4x4<f32>,
    col: vec3<f32>,
}


@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<uniform> ambient: vec4<f32>;
@group(1) @binding(0) var tmap_0: texture_2d<f32>;
@group(1) @binding(1) var smap: sampler;
@group(2) @binding(0) var<uniform> sources_array_0: array<Source, 3>;
@group(2) @binding(1) var<uniform> sources_len_0: Len;
@group(3) @binding(0) var<uniform> spaces: array<Space, 1>;
@group(3) @binding(1) var tspace_0: texture_3d<f32>;
@group(3) @binding(2) var sspace: sampler;


@vertex
fn vsmain(inst: InstanceInput, input: VertexInput) -> VertexOutput {
    let model = mat4x4<f32>(
        inst.r0,
        inst.r1,
        inst.r2,
        inst.r3,
    );

    var out: VertexOutput;
    let world = model * vec4(input.pos, 1.);
    out.pos = camera.view * world;
    out.map = input.map;
    out.world = world.xyz;
    out.space_0 = (spaces[0].model * world).xzy;

    return out;
}

@fragment
fn fsmain(out: VertexOutput) -> @location(0) vec4<f32> {
    var col: vec3<f32>;
    let tex = textureSample(tmap_0, smap, out.map);
    if tex.a < 0.9 { discard; }
    
    var sources = vec3(0.); 
    for (var i = 0u; i < sources_len_0.n; i++) {
        let src = sources_array_0[i];
        if out.world.x > src.pos.x - src.rad && out.world.x < src.pos.x + src.rad && out.world.y > src.pos.y - src.rad && out.world.y < src.pos.y + src.rad && out.world.z > src.pos.z - src.rad && out.world.z < src.pos.z + src.rad {
            let len = length(out.world - src.pos);
            if len < src.rad {
                let e = 1. - (len / src.rad);
                sources += e * e * src.col;
            }
        }
    }
    
    var space = vec3(0.);
    var space_a = textureSampleLevel(tspace_0, sspace, out.space_0, 0.);
    space = space_a.rgb * spaces[0].col;
    
    let light = ambient.rgb + sources + space;
    col = light * tex.rgb;

    return vec4(col, 1.);
}
