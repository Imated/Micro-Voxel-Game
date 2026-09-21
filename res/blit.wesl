struct VertexOutput {
    @builtin(position)
    clip_position: vec4<f32>,
    @location(1)
    uv: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index)in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vec2<f32>(f32((in_vertex_index << 1) & 2), f32(in_vertex_index & 2));
    out.clip_position = vec4<f32>(out.uv * 2 - 1, 0, 1);
    out.uv.y = 1 - out.uv.y;
    return out;
}

@group(0) @binding(0)
var render_texture: texture_2d<f32>;

@group(0) @binding(1)
var render_sampler: sampler;

fn aces(x: vec3<f32>) -> vec3<f32> {
    return clamp((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14),
                 vec3<f32>(0.0), vec3<f32>(1.0));
}

@fragment
fn fs_main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    let hdr = textureSample(render_texture, render_sampler, in.uv).xyz;
    var out = aces(hdr * 1.15);
    let luma = dot(out, vec3<f32>(0.2126, 0.7152, 0.0722));
    let shadow_w = (1.0 - luma) * (1.0 - luma);
    let high_w = luma * luma;
    out += shadow_w * vec3<f32>(-0.012, 0.004, 0.022);
    out *= vec3<f32>(1.0) + high_w * vec3<f32>(0.05, 0.02, -0.04);
    out = mix(vec3<f32>(luma), out, 1.12);
    return vec4<f32>(out, 1.0);
}
