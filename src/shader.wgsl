
@group(0) @binding(0)
var diffuse_texture: texture_2d<f32>;
@group(0) @binding(1)
var diffuse_sampler: sampler;


struct CameraUniform {
    view_projection: mat4x4<f32>,
};

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(mesh: VertexInput) -> VertexOutput {
    var v: VertexOutput;

    v.color = mesh.color;
    v.tex_coords = mesh.tex_coords;
    v.clip_position = camera.view_projection * vec4<f32>(mesh.position, 1.0);
    v.clip_position =  vec4<f32>(mesh.position, 1.0);

    return v;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(diffuse_texture, diffuse_sampler, in.tex_coords);
    let vert_color = vec4<f32>(in.color, 1.0);
    return tex_color * vert_color;
}