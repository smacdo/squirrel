struct PerFrameUniforms {
    view_projection: mat4x4<f32>,
    time_elapsed_seconds: f32,
};

struct PerModelUniforms {
    local_to_world: mat4x4<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
}

struct VertexOutput {
    /// Vertex output in "clip space" which can be visualized as:
    ///  (.u must be set to 1.0).
    ///
    ///  <----------X---------->
    /// ^ 
    /// |          +1
    /// |           
    /// Y    -1     .     +1
    /// | 
    /// |          -1
    /// v
    ///
    /// See: https://webgpufundamentals.org/webgpu/lessons/webgpu-fundamentals.html
    @builtin(position) position_cs: vec4<f32>,
    /// RGB color of the vertex.
    @location(0) color: vec3<f32>,
    /// UV texture coordinates of the vertex.
    @location(1) tex_coords: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> per_frame: PerFrameUniforms;

@group(1) @binding(0)
var<uniform> per_model: PerModelUniforms;

@group(2) @binding(0)
var diffuse_texture: texture_2d<f32>;
@group(2) @binding(1)
var diffuse_sampler: sampler;

@vertex
fn vs_main(mesh: VertexInput) -> VertexOutput {
    var v: VertexOutput;

    v.color = mesh.color;
    v.tex_coords = mesh.tex_coords;
    v.position_cs = per_frame.view_projection
        * per_model.local_to_world
        * vec4<f32>(mesh.position, 1.0);

    return v;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(diffuse_texture, diffuse_sampler, in.tex_coords);

    // TODO(scott): Re-enable vertex coloring.
    //let vert_color = vec4<f32>(in.color, 1.0);
    //let frag_color = tex_color * vert_color;

    return tex_color;
}