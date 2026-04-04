struct TextVertexInput {
    @location(0) clip_position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct TextVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(input: TextVertexInput) -> TextVertexOutput {
    var out: TextVertexOutput;
    out.clip_position = vec4<f32>(input.clip_position, 0.0, 1.0);
    out.tex_coords = input.tex_coords;
    out.color = input.color;
    return out;
}

@group(0) @binding(0)
var t_glyph: texture_2d<f32>;
@group(0) @binding(1)
var s_glyph: sampler;

@fragment
fn fs_main(in: TextVertexOutput) -> @location(0) vec4<f32> {
    let alpha = textureSample(t_glyph, s_glyph, in.tex_coords).r;
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
