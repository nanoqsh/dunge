struct Camera {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) map: vec2<f32>,
}

struct InstanceInput {
    @location(2) row0: vec4<f32>,
    @location(3) row1: vec4<f32>,
    @location(4) row2: vec4<f32>,
    @location(5) row3: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) map: vec2<f32>,
    @location(1) world: vec3<f32>,
}

@vertex
fn vs_main(vert: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model = mat4x4<f32>(
        instance.row0,
        instance.row1,
        instance.row2,
        instance.row3,
    );

    var out: VertexOutput;
    let world = model * vec4<f32>(vert.pos, 1.);
    out.pos = camera.view_proj * world;
    out.map = vert.map;
    out.world = world.xyz;
    return out;
}

@group(1) @binding(0)
var tdiff: texture_2d<f32>;
@group(1) @binding(1)
var sdiff: sampler;

struct Source {
    pos: vec3<f32>,
    rad: f32,
    col: vec3<f32>,
    flags: u32,
}

@group(2) @binding(0)
var<uniform> sources: array<Source, 64>;
@group(2) @binding(1)
var<uniform> n_sources: u32;

@group(3) @binding(0)
var<uniform> ambient: vec3<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let out = textureSample(tdiff, sdiff, in.map);
    if out.w < 0.9 {
        discard;
    }

    var diffuse = vec3(0., 0., 0.);
    for (var i: u32 = 0u; i < n_sources; i++) {
        let source = sources[i];

        if in.world.x > source.pos.x - source.rad && in.world.x < source.pos.x + source.rad
        && in.world.y > source.pos.y - source.rad && in.world.y < source.pos.y + source.rad
        && in.world.z > source.pos.z - source.rad && in.world.z < source.pos.z + source.rad {
            let len = length(in.world - source.pos);
            if len < source.rad {
                var sharp = 1.;
                if (source.flags & 1u) == 0u {
                    sharp -= (len / source.rad);
                }

                var gloom: vec3<f32>;
                if (source.flags & 2u) == 0u {
                    gloom = vec3(1.);
                } else {
                    gloom = -ambient;
                }

                diffuse += gloom * sharp * source.col;
            }
        }
    }

    let result = (ambient + diffuse) * out.rgb;
    return vec4(result, out.a);
}
