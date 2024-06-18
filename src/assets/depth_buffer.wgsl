
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position_cs: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@group(0) @binding(0)
var depth_texture: texture_2d<f32>;
@group(0) @binding(1)
var depth_sampler: sampler;

@vertex
fn vs_main(model: VertexInput,) -> VertexOutput {
    var out: VertexOutput;

    out.tex_coords = model.tex_coords;
    out.position_cs = vec4<f32>(model.position, 1.0);
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let near = 0.1; // TODO: Replace with zNear of perspective projection.
    let far = 100.0; // TODO: Replace with zFar of perspective projection.
    let depth = textureSample(depth_texture, depth_sampler, in.tex_coords).x;
    let r = (2.0 * near) / (far + near - depth * (far - near));
    return vec4<f32>(r, r, r, 1.0);
}