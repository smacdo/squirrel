struct PerFrameUniforms {
    view_projection: mat4x4<f32>,
    view_pos: vec4<f32>,
    time_elapsed_seconds: f32,
    output_is_srgb: u32, // TODO(scott): Pack bit flags in here.
};

struct PerModelUniforms {
    local_to_world: mat4x4<f32>,
    world_to_local: mat4x4<f32>,
    light_pos: vec4<f32>,   // .w is specular whiteness amount
    light_color: vec4<f32>, // .w is ambient amount
}

struct PerSubmeshUniforms {
    ambient_color: vec3<f32>,
    diffuse_color: vec3<f32>,
    specular_color: vec4<f32>, // .w is power.
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
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
    /// Vertex position in world space (rather than clip space) to allow world
    /// space lighting calculations in the fragment shader.
    @location(0) position_ws: vec3<f32>,
    /// Normal vector from the vertex.
    @location(1) normal: vec3<f32>,
    /// UV texture coordinates of the vertex.
    @location(2) tex_coords: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> per_frame: PerFrameUniforms;

@group(1) @binding(0)
var<uniform> per_model: PerModelUniforms;

@group(2) @binding(0)
var<uniform> per_submesh: PerSubmeshUniforms;

@group(2) @binding(1)
var diffuse_texture: texture_2d<f32>;
@group(2) @binding(2)
var diffuse_sampler: sampler;

@vertex
fn vs_main(v_in: VertexInput) -> VertexOutput {
    var v_out: VertexOutput;

    v_out.position_cs = per_frame.view_projection
        * per_model.local_to_world
        * vec4<f32>(v_in.position, 1.0);
    v_out.position_ws = (per_model.local_to_world * vec4<f32>(v_in.position, 1.0)).xyz;
    v_out.normal = (transpose(per_model.world_to_local) * vec4<f32>(v_in.normal, 1.0)).xyz;
    v_out.tex_coords = v_in.tex_coords;


    return v_out;
}

@fragment
fn fs_main(v_in: VertexOutput) -> @location(0) vec4<f32> {
    // TODO: Restore the old shader code once lighting implementation completed.
    //let tex_color = textureSample(diffuse_texture, diffuse_sampler, v_in.tex_coords);
    //let vert_color = vec4<f32>(v_in.color, 1.0);
    //let frag_color = tex_color * vert_color;

    // Unpack lighting into separate variables.
    let light_pos = per_model.light_pos.xyz;
    let light_color = per_model.light_color.xyz;
    let light_ambient = per_model.light_pos.w;
    let light_specular = per_model.light_color.w;

    // Ambient lighting.
    let ambient_color = light_color * light_ambient * per_submesh.ambient_color;

    // Diffuse lighting.
    // The light direction is a vector pointing from this fragment to the light.
    let normal = normalize(v_in.normal);
    let light_dir = normalize(light_pos - v_in.position_ws);
    let diffuse_amount = max(dot(normal, light_dir), 0.0);
    let diffuse_color = light_color * diffuse_amount * per_submesh.diffuse_color;

    // Specular lighting.
    let view_dir = normalize(per_frame.view_pos.xyz - v_in.position_ws);
    let reflect_dir = reflect(-light_dir, normal);
    let specular_amount = pow(max(dot(view_dir, reflect_dir), 0.0), per_submesh.specular_color.w);
    let specular_color = vec3<f32>(1.0) * light_specular * specular_amount * per_submesh.specular_color.xyz;

    // Final color is an additive combination of ambient, diffuse and specular.
    let frag_color = vec4<f32>(ambient_color + diffuse_color + specular_color, 1.0);

    // Should the color be converted from linear to sRGB in the pixel shader?
    // Otherwise simply return it in lienar space.
    if (per_frame.output_is_srgb == 0) {
        return from_linear_rgb(frag_color);
    } else {
        return frag_color;
    }
}

//============================================================================//
// Shared utility functions.
// TODO(scott): Move these to a utility functions library.
//============================================================================//

// linear -> srgb
// https://en.wikipedia.org/wiki/SRGB
fn from_linear_color(x: f32) -> f32 {
    var y = 12.92 * x;

    if (x > 0.0031308) {
        let a = 0.055;
        y = (1.0 + a) * pow(x, 1.0/2.4) - a;
    }

    return y;
}

fn from_linear_rgb(c: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(
        from_linear_color(c.r),
        from_linear_color(c.g),
        from_linear_color(c.b),
        c.a
    );
}

/*
// TODO(scott): Get this optimized solution to work from GLSL
// https://gamedev.stackexchange.com/questions/92015/optimized-linear-to-srgb-glsl
fn from_linear_rgb(linear_rgb: vec4<f32>) -> vec4<f32> {
    let cutoff: vec4<bool> = lessThan(linear_rgb.rgb, vec3<f32>(0.0031308));
    let higher = vec3<f32>(1.055) * pow(linear_rgb.rgb, vec3<f32>(1.0 / 2.4)) - vec3<f32>(0.055);
    let lower = linear_rgb.rgb * vec3<f32>(12.2);

    return vec4<f32>(mix(higher, lower, cutoff), linear_rgb.a);
}
*/