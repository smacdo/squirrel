//============================================================================//
// Uniform Buffers                                                            //
//============================================================================//
// TODO: Consider using structs to represent the packed lighting data, and 
// structs to represent unpacked lights/materials. Refactor the functions to
// take those parameters which should make this all a lot less confusing.
struct PerFrameUniforms {
    /// Camera view projection.
    view_projection: mat4x4<f32>,
    /// Camera world space position.
    view_pos: vec4<f32>,
    directional_light: PackedDirectionalLight,
    time_elapsed_seconds: f32,
    output_is_srgb: u32, // TODO(scott): Pack bit flags in here.
};

struct PerModelUniforms {
    /// Model -> world transform.
    local_to_world: mat4x4<f32>,
    /// World -> model transform.
    world_to_local: mat4x4<f32>,
    /// Point light.
    point_light: PackedPointLight,
}

struct PerSubmeshUniforms {
    material: PerSubmeshMaterial
}

//============================================================================//
// Shader inputs                                                              //
//============================================================================//
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

//============================================================================//
// Vertex shader                                                              //
//============================================================================//
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

//============================================================================//
// Pixel shader                                                               //
//============================================================================//
@fragment
fn fs_main(v_in: VertexOutput) -> @location(0) vec4<f32> {
    let frag_normal = normalize(v_in.normal);
    let material = unpack_per_submesh_material(
            per_submesh.material,
            v_in.tex_coords,
            tex_sampler,
            diffuse_texture,
            specular_texture,
            emissive_texture);

    // Directional lighting.
    var frag_color = vec4<f32>(directional_light(
        v_in.position_ws,        // fragment world space position
        frag_normal,             // fragment normal direction (normalized)
        per_frame.view_pos.xyz,  // camera world space position
        unpack_directional_light(per_frame.directional_light),
        material,
    ), 1.0);

    // Point lighting.
    frag_color += vec4<f32>(point_light(
        v_in.position_ws,        // fragment world space position
        frag_normal,             // fragment normal direction (normalized)
        per_frame.view_pos.xyz,  // camera world space position
        unpack_point_light(per_model.point_light),
        material,
    ), 1.0);

    // Should the color be converted from linear to sRGB in the pixel shader?
    // Otherwise simply return it in lienar space.
    if (per_frame.output_is_srgb == 0) {
        return from_linear_rgb(frag_color);
    } else {
        return frag_color;
    }
}

//============================================================================//
// Shared types and functions                                                 //
//============================================================================//
struct PerSubmeshMaterial {
    ambient_color: vec3<f32>,  // .w is unused.
    diffuse_color: vec3<f32>,  // .w is unused.
    specular_color: vec4<f32>, // .w is power.
};

struct Material {
    ambient_color: vec3<f32>,
    diffuse_color: vec3<f32>,
    specular_color: vec3<f32>,
    specular_shininess: f32,
    emissive_color: vec3<f32>,
};

fn unpack_per_submesh_material(
        material_constants: PerSubmeshMaterial,
        tex_uv: vec2<f32>,
        tex_sampler: sampler,
        diffuse_map: texture_2d<f32>,
        specular_map: texture_2d<f32>,
        emissive_map: texture_2d<f32>,
) -> Material {
    // Sample the material's texture maps. If a texture map is not specified
    // then either use a 1x1 white pixel to let the constant color through or
    // use a 1x1 black pixel to disable that contribution.
    //
    // A sane default is probably white = 1 for the diffuse texture map, and a
    // black = 0 for the specular and emissive texture map.
    let diffuse_tex_color = textureSample(diffuse_map, tex_sampler, tex_uv).xyz;
    let specular_tex_color = textureSample(specular_map, tex_sampler, tex_uv).xyz;
    let emissive_tex_color = textureSample(emissive_map, tex_sampler, tex_uv).xyz;

    // Combine the texture maps with the material's constant color values before
    // returning the material.
    var m: Material;

    m.ambient_color = material_constants.ambient_color.xyz * diffuse_tex_color;
    m.diffuse_color = material_constants.diffuse_color.xyz * diffuse_tex_color;
    m.specular_color = material_constants.specular_color.xyz * specular_tex_color;
    m.emissive_color = emissive_tex_color.xyz;

    m.specular_shininess = material_constants.specular_color.w;

    return m;
}

struct PackedDirectionalLight {
    /// Direction from light to source.
    ///   .xyz is normalized
    ///   .w is ambient contribution.
    direction: vec4<f32>,
    /// Color
    ///   .w is specular contribution.
    color: vec4<f32>,
}

struct DirectionalLight {
    reverse_direction_n: vec3<f32>,
    color: vec3<f32>,
    ambient_contrib: f32,
    diffuse_contrib: f32,
    specular_contrib: f32,
}

fn unpack_directional_light(directional_light: PackedDirectionalLight) -> DirectionalLight {    
    //  Need to invert direction beecause directional light is specified as dir
    //  from light source towards fragment but lighting function expects it to
    //  be fragment to light.
    var d: DirectionalLight;
    
    d.reverse_direction_n = normalize(-directional_light.direction.xyz);
    d.color = directional_light.color.xyz;
    d.ambient_contrib = directional_light.direction.w;
    d.diffuse_contrib = 1.0;
    d.specular_contrib = directional_light.color.w;

    return d;
}

/// Calculate the color contribution from a directional light for a given 
/// material.
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
        light: DirectionalLight,
        material: Material,
) -> vec3<f32> {
    // Ambient.
    let ambient_color = light.color 
        * light.ambient_contrib
        * material.ambient_color;

    // Diffuse.
    let diffuse_color = light_diffuse(
        frag_normal,
        light.reverse_direction_n,
        light.color,
        light.diffuse_contrib,
        material.diffuse_color
    );

    // Specular lighting.
    let view_dir = normalize(view_pos - frag_pos);
    let specular_color = light_specular(
        frag_normal,
        view_dir,
        light.reverse_direction_n,
        vec3<f32>(1.0),
        light.specular_contrib,
        material.specular_color,
        material.specular_shininess
    );

    // Final color is an additive combination of ambient, diffuse and specular.
    // TODO: Bug! Emissive color should only be added once after summing all lights!
    return ambient_color
        + diffuse_color
        + specular_color
        + material.emissive_color;
}

struct PackedPointLight {
    /// Point light world space position. (`w` is the ambient term).
    pos: vec4<f32>, 
    /// Point light color. (`w` is the specular term).
    color: vec4<f32>,
    /// Point light attenuation.
    ///  `x`: constant term.
    ///  `y`: linear term.
    ///  `z`: quadratic term.
    ///  `w`: unused.
    attenuation: vec4<f32>,
}

struct PointLight {
    pos: vec3<f32>,
    color: vec3<f32>,
    ambient_contrib: f32,
    diffuse_contrib: f32,
    specular_contrib: f32,
    attenuation: vec3<f32>,
}

fn unpack_point_light(packed_light: PackedPointLight) -> PointLight {
    var p: PointLight;

    p.pos = packed_light.pos.xyz;
    p.color = packed_light.color.xyz;
    p.ambient_contrib = packed_light.pos.w;
    p.diffuse_contrib = 1.0;
    p.specular_contrib = packed_light.color.w;
    p.attenuation = packed_light.attenuation.xyz;

    return p;
}

/// Calculate the color contribution from a point light for a given material.
///
///  `frag_pos`:  Fragment world space position.
///  `frag_normal`: Fragment normal vector direction (normalized).
///  `view_pos`: Camera world space position.
///  `light_pos`: World space position of the light.
///  `light_color`: Color of the light.
///  `light_attenuation`: Point light attenuation terms (constant, linear, quadratic).
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
        light: PointLight,
        material: Material,
) -> vec3<f32> {
    // Ambient.
    let ambient_color = light.color * light.ambient_contrib * material.ambient_color;

    // Diffuse.
    let light_dir = normalize(light.pos - frag_pos);
    let diffuse_color = light_diffuse(
        frag_normal,
        light_dir,
        light.color,
        light.diffuse_contrib,
        material.diffuse_color
    );

    // Specular lighting.
    let view_dir = normalize(view_pos - frag_pos);
    let specular_color = light_specular(
        frag_normal,
        view_dir,
        light_dir,
        vec3<f32>(1.0),
        light.specular_contrib,
        material.specular_color,
        material.specular_shininess
    );

    // Attenuation.
    // TODO: Insert check for when attenuation tries to divide by zero.
    let distance = length(light.pos - frag_pos);
    let attenuation = 1.0 / (
        light.attenuation.x +
        light.attenuation.y * distance +
        light.attenuation.z * distance * distance
    );

    // Final color is an additive combination of ambient, diffuse and specular.
    // TODO: Bug! Emissive color should only be added once after summing all lights!
    return ambient_color * attenuation
        + diffuse_color * attenuation
        + specular_color * attenuation
        + material.emissive_color;
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