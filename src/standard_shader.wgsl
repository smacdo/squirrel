// TODO: Consider using structs to represent the packed lighting data, and 
// structs to represent unpacked lights/materials. Refactor the functions to
// take those parameters which should make this all a lot less confusing.
struct PerFrameUniforms {
    view_projection: mat4x4<f32>,
    view_pos: vec4<f32>,
    light_dir: vec4<f32>, // directional light, .w is ambient amount.
    light_color: vec4<f32>, // directional light, .w is specular amount.
    time_elapsed_seconds: f32,
    output_is_srgb: u32, // TODO(scott): Pack bit flags in here.
};

struct PerModelUniforms {
    local_to_world: mat4x4<f32>,
    world_to_local: mat4x4<f32>,
    light_pos: vec4<f32>,   // .w is ambient amount
    light_color: vec4<f32>, // .w is specular amount
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
var tex_sampler: sampler;

@group(2) @binding(2)
var diffuse_texture: texture_2d<f32>;

@group(2) @binding(3)
var specular_texture: texture_2d<f32>;

@group(2) @binding(4)
var emissive_texture: texture_2d<f32>;

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
    let frag_normal = normalize(v_in.normal);

    // Sample the diffuse and specular texture maps. For materials that do not
    // have an associated texture map use a default 1x1 white pixel. Use a 1x1
    // black pixel to default the emissive map.
    let diffuse_tex_color = textureSample(diffuse_texture, tex_sampler, v_in.tex_coords).xyz;
    let specular_tex_color = textureSample(specular_texture, tex_sampler, v_in.tex_coords).xyz;
    let emissive_tex_color = textureSample(emissive_texture, tex_sampler, v_in.tex_coords).xyz;

    // Directional lighting.
    //  Need to invert direction beecause directional light is specified as dir
    //  from light source towards fragment but lighting function expects it to
    //  be fragment to light.
    let dir_light_dir = normalize(-per_frame.light_dir.xyz);
    let dir_light_color = per_frame.light_color.xyz;
    let dir_ambient_contrib = per_frame.light_dir.w;
    let dir_specular_contrib = per_frame.light_color.w;

    var frag_color = vec4<f32>(directional_light(
        v_in.position_ws,        // fragment world space position
        frag_normal,             // fragment normal direction (normalized)
        per_frame.view_pos.xyz,  // camera world space position
        dir_light_dir,           // fragment to directional light direction (normalized)
        dir_light_color,         // color of directional light
        dir_ambient_contrib,     // amount of ambient contribution
        1.0,                     // amount of diffuse contribution
        dir_specular_contrib,    // amount of specular contribution
        diffuse_tex_color * per_submesh.ambient_color,       // material ambient color
        diffuse_tex_color * per_submesh.diffuse_color,       // material diffuse color
        specular_tex_color * per_submesh.specular_color.xyz, // material specular color
        per_submesh.specular_color.w, // material specular shininess
        emissive_tex_color            // material emissive color
    ), 1.0);

    // Point lighting.
    // NOTE: this isn't really point lights as they lack falloff parameters.
    let light_pos = per_model.light_pos.xyz;
    let light_color = per_model.light_color.xyz;
    let light_ambient_contrib = per_model.light_pos.w;
    let light_specular_contrib = per_model.light_color.w;

    frag_color += vec4<f32>(point_light(
        v_in.position_ws,        // fragment world space position
        frag_normal,             // fragment normal direction (normalized)
        per_frame.view_pos.xyz,  // camera world space position
        light_pos,               // point light world space position
        light_color,             // color of point light
        light_ambient_contrib,   // amount of ambient contribution
        1.0,                     // amount of diffuse contribution
        light_specular_contrib,  // amount of specular contribution
        diffuse_tex_color * per_submesh.ambient_color,       // material ambient color
        diffuse_tex_color * per_submesh.diffuse_color,       // material diffuse color
        specular_tex_color * per_submesh.specular_color.xyz, // material specular color
        per_submesh.specular_color.w, // material specular shininess
        emissive_tex_color            // material emissive color
    ), 1.0);

    // Should the color be converted from linear to sRGB in the pixel shader?
    // Otherwise simply return it in lienar space.
    if (per_frame.output_is_srgb == 0) {
        return from_linear_rgb(frag_color);
    } else {
        return frag_color;
    }
}

/// Calculate the color contribution from a directional light for a given 
//// material.
///
///  `frag_pos`:  Fragment world space position.
///  `frag_normal`: Fragment normal vector direction (normalized).
///  `view_pos`: Camera world space position.
///  `light_dir`: Normalized direction from fragment towards the light source.
///  `light_color`: Color of the light.
///  `light_ambient_contrib`: Ambient lighting modifier [0 = none, 1 = full].
///  `light_diffuse_contrib`: Diffuse lighting modifier [0 = none, 1 = full].
///  `light_specular_contrib`: Specular lighting modifier [0 = none, 1 = full].
///  `mat_ambient_color`: Material ambient color.
///  `mat_diffuse_color`: Material diffuse color.
///  `mat_specular_color`: Material specular color.
///  `mat_shininess`: Material shininess amount.
///  `mat_emissive`: Material emissive color.
fn directional_light(
        frag_pos: vec3<f32>,
        frag_normal: vec3<f32>,
        view_pos: vec3<f32>,
        light_dir: vec3<f32>,
        light_color: vec3<f32>,
        light_ambient_contrib: f32,
        light_diffuse_contrib: f32,
        light_specular_contrib: f32,
        mat_ambient_color: vec3<f32>,
        mat_diffuse_color: vec3<f32>,
        mat_specular_color: vec3<f32>,
        mat_shininess: f32,
        mat_emissive: vec3<f32>,
) -> vec3<f32> {
    // Ambient.
    let ambient_color = light_color 
        * light_ambient_contrib
        * mat_ambient_color;

    // Diffuse.
    let diffuse_color = light_diffuse(
        frag_normal,
        light_dir,
        light_color,
        light_diffuse_contrib,
        mat_diffuse_color
    );

    // Specular lighting.
    let view_dir = normalize(view_pos - frag_pos);
    let specular_color = light_specular(
        frag_normal,
        view_dir,
        light_dir,
        vec3<f32>(1.0),
        light_specular_contrib,
        mat_specular_color,
        mat_shininess
    );

    // Final color is an additive combination of ambient, diffuse and specular.
    return ambient_color
        + diffuse_color
        + specular_color
        + mat_emissive;
}

/// Calculate the color contribution from a point light for a given material.
///
///  `frag_pos`:  Fragment world space position.
///  `frag_normal`: Fragment normal vector direction (normalized).
///  `view_pos`: Camera world space position.
///  `light_pos`: World space position of the light.
///  `light_color`: Color of the light.
///  `light_ambient_contrib`: Ambient lighting modifier [0 = none, 1 = full].
///  `light_diffuse_contrib`: Diffuse lighting modifier [0 = none, 1 = full].
///  `light_specular_contrib`: Specular lighting modifier [0 = none, 1 = full].
///  `mat_ambient_color`: Material ambient color.
///  `mat_diffuse_color`: Material diffuse color.
///  `mat_specular_color`: Material specular color.
///  `mat_shininess`: Material shininess amount.
///  `mat_emissive`: Material emissive color.
fn point_light(
        frag_pos: vec3<f32>,
        frag_normal: vec3<f32>,
        view_pos: vec3<f32>,
        light_pos: vec3<f32>,
        light_color: vec3<f32>,
        light_ambient_contrib: f32,
        light_diffuse_contrib: f32,
        light_specular_contrib: f32,
        mat_ambient_color: vec3<f32>,
        mat_diffuse_color: vec3<f32>,
        mat_specular_color: vec3<f32>,
        mat_shininess: f32,
        mat_emissive: vec3<f32>,
) -> vec3<f32> {
    // Ambient.
    let ambient_color = light_color 
        * light_ambient_contrib
        * mat_ambient_color;

    // Diffuse.
    let light_dir = normalize(light_pos - frag_pos);
    let diffuse_color = light_diffuse(
        frag_normal,
        light_dir,
        light_color,
        light_diffuse_contrib,
        mat_diffuse_color
    );

    // Specular lighting.
    let view_dir = normalize(view_pos - frag_pos);
    let specular_color = light_specular(
        frag_normal,
        view_dir,
        light_dir,
        vec3<f32>(1.0),
        light_specular_contrib,
        mat_specular_color,
        mat_shininess
    );

    // Final color is an additive combination of ambient, diffuse and specular.
    return ambient_color
        + diffuse_color
        + specular_color
        + mat_emissive;
}

/// Calculate the diffuse color contribution from a light for a given material.
///
/// `normal`: Normalized perpendicular vector from surface of fragment.
/// `light_dir`: Normalized vector pointing from fragment to the light.
/// `light_color`: Color of the light.
/// `light_contrib`: Light contribution modifier (0 for none, 1 for full).
/// `mat_color`: Material diffuse color.
fn light_diffuse(
        normal: vec3<f32>,
        light_dir: vec3<f32>,
        light_color: vec3<f32>,
        light_contrib: f32,
        mat_color: vec3<f32>) -> vec3<f32> {
    let diffuse_amount = max(dot(normal, light_dir), 0.0);
    return light_color
        * light_contrib
        * diffuse_amount
        * mat_color;
}

/// Calculate the specular color contribution from a light for a given material.
///
/// `normal`: Normalized perpendicular vector from surface of fragment.
/// `view_dir`:  Normalized vector pointing from fragment to the camera.
/// `light_dir`: Normalized vector pointing from fragment to the light.
/// `light_color`: Color of the light.
/// `light_contrib`: Light contribution modifier (0 for none, 1 for full).
/// `mat_color`: Material color.
/// `mat_shininess`: Material specular shininess component.
fn light_specular(
        normal: vec3<f32>,
        view_dir: vec3<f32>,
        light_dir: vec3<f32>,
        light_color: vec3<f32>,
        light_contrib: f32,
        mat_color: vec3<f32>,
        mat_shininess: f32) -> vec3<f32> {
    let reflect_dir = reflect(-light_dir, normal);
    let specular_amount = pow(max(dot(view_dir, reflect_dir), 0.0), mat_shininess);
    return light_color
        * light_contrib
        * specular_amount
        * mat_color;
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