struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) map: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    switch in_vertex_index {
        case 0u {
            out.clip_position = vec4(1., -1., 0., 1.);
            out.map = vec2(1., 1.);
        }
        case 1u {
            out.clip_position = vec4(-1., -1., 0., 1.);
            out.map = vec2(0., 1.);
        }
        case 2u {
            out.clip_position = vec4(1., 1., 0., 1.);
            out.map = vec2(1., 0.);
        }
        case 3u {
            out.clip_position = vec4(-1., 1., 0., 1.);
            out.map = vec2(0., 0.);
        }
        default {}
    }
    return out;
}

struct Data {
    size: vec2<f32>,
    _pad: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> data: Data;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // This fixes the annoying effect of a sprite stretching
    // caused by f32 rounding up or down depending on its sign.
    var map = in.map;
    if map.x < 0.5 {
        map.x -= 0.5 / data.size.x;
    }

    let eps = 0.12;
    let ex = 1. / data.size.x;
    let ey = 1. / data.size.y;
    let l = textureSample(t_diffuse, s_diffuse, vec2(map.x - ex, map.y)).rgb;
    let r = textureSample(t_diffuse, s_diffuse, vec2(map.x + ex, map.y)).rgb;
    let u = textureSample(t_diffuse, s_diffuse, vec2(map.x, map.y - ey)).rgb;
    let d = textureSample(t_diffuse, s_diffuse, vec2(map.x, map.y + ey)).rgb;
    var q = textureSample(t_diffuse, s_diffuse, map);

    let horf = abs(u - d) <= vec3(eps);
    let verf = abs(l - r) <= vec3(eps);
    let dl = max(abs(l - d), abs(l - u));
    let dr = max(abs(r - d), abs(r - u));

    if !horf.r && !verf.r {
        if dl.r > dr.r {
            q.r = mix(q.r, dl.r, 0.5);
        } else {
            q.r = mix(q.r, dr.r, 0.5);
        }
    }

    if !horf.g && !verf.g {
        if dl.g > dr.g {
            q.g = mix(q.g, dl.g, 0.5);
        } else {
            q.g = mix(q.g, dr.g, 0.5);
        }
    }

    if !horf.b && !verf.b {
        if dl.b > dr.b {
            q.b = mix(q.b, dl.b, 0.5);
        } else {
            q.b = mix(q.b, dr.b, 0.5);
        }
    }

    return q;
}
