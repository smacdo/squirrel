struct PerFrameUniforms {
    view_projection: mat4x4<f32>,
    time_elapsed_seconds: f32,
    output_is_srgb: u32, // TODO(scott): Pack bit flags in here.
};

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct InstanceInput {
    @location(2) local_to_world_0: vec4<f32>,
    @location(3) local_to_world_1: vec4<f32>,
    @location(4) local_to_world_2: vec4<f32>,
    @location(5) local_to_world_3: vec4<f32>,
    @location(6) tint_color: vec4<f32>, // .w is unused
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

@vertex
fn vs_main(mesh: VertexInput, instance: InstanceInput) -> VertexOutput {
    var v: VertexOutput;

    let local_to_world = mat4x4<f32>(
        instance.local_to_world_0,
        instance.local_to_world_1,
        instance.local_to_world_2,
        instance.local_to_world_3,
    );

    v.color = instance.tint_color.xyz;
    v.tex_coords = mesh.tex_coords;
    v.position_cs = per_frame.view_projection
        * local_to_world
        * vec4<f32>(mesh.position, 1.0);

    return v;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let frag_color = vec4<f32>(in.color, 1.0);

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