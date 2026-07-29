import lygia::color::tonemap::aces::tonemapACES4;

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

@fragment
fn fs_main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    let hdr = textureSample(render_texture, render_sampler, in.uv);
    return vec4<f32>(hdr.xyz, 1.0);
}
