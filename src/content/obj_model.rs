use std::{path::Path, rc::Rc};

use wgpu::util::DeviceExt;

use crate::{
    content::load_texture_file,
    platform::load_as_string,
    renderer::{self, models, shaders, shading},
};

use super::DefaultTextures;

// TODO: Support loading emissive maps from mtl files.

/// Creates a new `Mesh` from an obj model.
#[tracing::instrument(level = "info")]
pub async fn load_obj_mesh<P>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layouts: &shaders::BindGroupLayouts,
    default_textures: &DefaultTextures,
    obj_file_path: P,
) -> anyhow::Result<renderer::models::Mesh>
where
    P: AsRef<Path> + std::fmt::Debug,
{
    let obj_text = load_as_string(obj_file_path.as_ref()).await?;
    let obj_cursor = std::io::Cursor::new(obj_text); // TODO: move inline?
    let mut obj_buf_reader = std::io::BufReader::new(obj_cursor);

    // Parse the .obj file to get a list of models (actually meshes) and materials
    // definitions.
    let (obj_models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_buf_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |mtl_file_path| async move {
            // TODO: Break this out - can caching be supported?
            // TODO: Convert the unwrap to a returnable error.
            let mtl_text = load_as_string(&mtl_file_path).await.unwrap();
            tobj::load_mtl_buf(&mut std::io::BufReader::new(std::io::Cursor::new(mtl_text)))
        },
    )
    .await?;

    // Create materials for each of the MTL material definitions.
    let obj_materials = obj_materials?;
    let mut materials = Vec::with_capacity(obj_materials.len());

    for obj_mtl in obj_materials.into_iter() {
        materials.push(create_material(device, queue, obj_mtl, default_textures).await?);
    }

    // Creates meshes for each of the obj models.
    create_mesh(
        device,
        layouts,
        &obj_models,
        &materials,
        obj_file_path
            .as_ref()
            .to_str()
            .unwrap_or("invalid utf8 chars in obj file path"),
    )
}

/// Creates a `shading::Material` object from a given obj model's .mtl material.
pub async fn create_material(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mat: tobj::Material,
    default_textures: &DefaultTextures,
) -> anyhow::Result<shading::Material> {
    pub async fn create_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        maybe_file_path: Option<String>,
        default_texture: &Rc<wgpu::Texture>,
    ) -> anyhow::Result<Rc<wgpu::Texture>> {
        match maybe_file_path {
            Some(file_path) => Ok(Rc::new(load_texture_file(device, queue, &file_path).await?)),
            None => Ok(default_texture.clone()),
        }
    }

    Ok(shading::Material {
        ambient_color: mat
            .ambient
            .map(|v| v.into())
            .unwrap_or(shading::DEFAULT_AMBIENT_COLOR),
        diffuse_color: mat
            .diffuse
            .map(|v| v.into())
            .unwrap_or(shading::DEFAULT_DIFFUSE_COLOR),
        diffuse_map: create_texture(
            device,
            queue,
            mat.diffuse_texture,
            &default_textures.diffuse_map,
        )
        .await?,
        specular_color: mat
            .specular
            .map(|v| v.into())
            .unwrap_or(shading::DEFAULT_SPECULAR_COLOR),
        specular_map: create_texture(
            device,
            queue,
            mat.shininess_texture,
            &default_textures.specular_map,
        )
        .await?,
        specular_power: mat.shininess.unwrap_or(shading::DEFAULT_SPECULAR_POWER),
        emissive_map: default_textures.emissive_map.clone(),
    })
}

/// Create a mesh out of the models in an obj model file.
///
/// `obj_meshes`: A list of all the obj models defined by the .obj file.
/// `materials`: A list of all the materials referenced in the .obj file.
/// `name`: Caller provided name for the mesh.
fn create_mesh(
    device: &wgpu::Device,
    layouts: &shaders::BindGroupLayouts,
    obj_meshes: &[tobj::Model],
    materials: &[shading::Material],
    name: &str,
) -> anyhow::Result<models::Mesh> {
    // Allocate a single vertex and index buffer for the entire obj mesh.
    let vertex_count: usize = obj_meshes.iter().map(|m| m.mesh.positions.len()).sum();
    let index_count: usize = obj_meshes.iter().map(|m| m.mesh.indices.len()).sum();

    let mut vertices: Vec<models::Vertex> = Vec::with_capacity(vertex_count);
    let mut indices: Vec<u32> = Vec::with_capacity(index_count);

    // Concatenate vertex and index buffer of each obj mesh into a single mesh
    // with a single vertex and index buffer. Each obj "mesh" should be converted
    // into a matching submesh.
    let mut submeshes: Vec<models::Submesh> = Vec::with_capacity(obj_meshes.len());

    for obj_mesh in obj_meshes {
        submeshes.push(process_obj_mesh(
            device,
            layouts,
            obj_mesh,
            &mut vertices,
            &mut indices,
            materials,
        )?);
    }

    // Copy the newly assembled vertex buffer into a hardware GPU vertex buffer.
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(&format!("{name} vertex buffer")),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    // Create a hardware GPU index buffer using the tobj mesh's indices. No need
    // to assemble an index buffer!
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(&format!("{name} index buffer")),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    Ok(models::Mesh::new(
        vertex_buffer,
        index_buffer,
        indices.len() as u32,
        wgpu::IndexFormat::Uint32,
        submeshes,
    ))
}

/// Append the vertices and indices of a obj model into a shared vertex and
/// index buffer for the entire mesh. This function will return a `Submesh` that
/// references the appended data.
///
/// `model`: The `tobj::Model` to copy from.
/// `vertices`: Vertices from the model are appended to this vertex buffer vec.
/// `indices`: Indices from the model are appended to this index buffer vec.
/// `materials`: A list of all the materials loaded by the obj model file.
fn process_obj_mesh(
    device: &wgpu::Device,
    layouts: &shaders::BindGroupLayouts,
    model: &tobj::Model,
    vertices: &mut Vec<models::Vertex>,
    indices: &mut Vec<u32>,
    materials: &[shading::Material],
) -> anyhow::Result<models::Submesh> {
    // This method assumes that `obj_model` was loaded with `triangulate = True`,
    // and `single_index = True`.
    assert!(
        model.mesh.face_arities.is_empty(),
        "expected triangulate = true"
    );
    assert!(
        model.mesh.normal_indices.is_empty(),
        "expected single_index = true"
    );
    assert!(
        model.mesh.texcoord_indices.is_empty(),
        "expected single_index = true"
    );

    // Assemble a vertex buffer from tobj's mesh data. By forcing `single_index`
    // the mesh's position, texture and normal buffers will have each face stored
    // sequentially. (eg position[0] = texture[0] = normal[0]).
    //
    // It's also possible for the obj file to have an empty normal buffer which
    // means the obj data didn't specify any normals.
    assert!(
        model.mesh.positions.len() % 3 == 0,
        "expected triangulate = true"
    );

    let has_normals = !model.mesh.normals.is_empty();

    // The obj mesh's index buffer do not account for vertex buffer sharing.
    // Record the size of the shared buffer prior to copying and use this as the
    // submesh's vertex offset.
    let base_vertex = vertices.len() as i32;
    let base_index = indices.len() as u32;

    // Append this model's vertices and indices to the merged vertex and index
    // buffers.
    (0..model.mesh.positions.len() / 3)
        .map(|vp_i| models::Vertex {
            position: [
                model.mesh.positions[vp_i * 3],
                model.mesh.positions[vp_i * 3 + 1],
                model.mesh.positions[vp_i * 3 + 2],
            ],
            tex_coords: [
                model.mesh.texcoords[vp_i * 2],
                model.mesh.texcoords[vp_i * 2 + 1],
            ],
            normal: if has_normals {
                [
                    model.mesh.normals[vp_i * 3],
                    model.mesh.normals[vp_i * 3 + 1],
                    model.mesh.normals[vp_i * 3 + 2],
                ]
            } else {
                [0.0, 0.0, 0.0]
            },
        })
        .for_each(|v| vertices.push(v));

    model.mesh.indices.iter().for_each(|i| indices.push(*i));

    Ok(models::Submesh::new(
        device,
        layouts,
        base_index..(base_index + model.mesh.indices.len() as u32),
        base_vertex,
        &materials[model
            .mesh
            .material_id
            .expect("TODO: Make material optional, let renderer handle empty material")],
    ))
}
