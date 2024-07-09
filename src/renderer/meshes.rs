//! NOTES:
//! Meshes vertex winding order is CCW.
//! Builtin meshes are ordered bottom left to bottom right.
use super::models::Vertex;

/// A list of meshes that can be constructed by the engine without needing to
/// load a model externally.
#[allow(dead_code)]
pub enum BuiltinMesh {
    Triangle,
    Rect,
    Pentagon,
    Cube,
}

/// Gets a builtin mesh for use in rendering. All builtin meshes are unit sized,
/// meaning the vertices in the mesh range from [-1, 1] on the XYZ axis.
#[allow(dead_code)]
pub fn builtin_mesh(mesh_type: BuiltinMesh) -> (&'static [Vertex], &'static [u16]) {
    match mesh_type {
        BuiltinMesh::Triangle => (TRIANGLE_VERTS, TRIANGLE_INDICES),
        BuiltinMesh::Rect => (RECT_VERTS, RECT_INDICES),
        BuiltinMesh::Pentagon => (PENTAGON_VERTS, PENTAGON_INDICES),
        BuiltinMesh::Cube => (CUBE_VERTS, CUBE_INDICES),
    }
}

#[allow(dead_code)]
pub const TRIANGLE_VERTS: &[Vertex] = &[
    Vertex {
        position: [0.0, 1.0, 0.0],
        normal: [0.0, 0.0, 1.0],
        tex_coords: [0.5, 0.0],
    },
    Vertex {
        position: [-1.0, -1.0, 0.0],
        normal: [0.0, 0.0, 1.0],
        tex_coords: [0.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0],
        normal: [0.0, 0.0, 1.0],
        tex_coords: [1.0, 1.0],
    },
];

#[allow(dead_code)]
pub const TRIANGLE_INDICES: &[u16] = &[0, 1, 2];

#[allow(dead_code)]
pub const RECT_VERTS: &[Vertex] = &[
    Vertex {
        position: [1.0, 1.0, 0.0],
        normal: [0.0, 0.0, 1.0],
        tex_coords: [1.0, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0, 0.0],
        normal: [0.0, 0.0, 1.0],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0],
        normal: [0.0, 0.0, 1.0],
        tex_coords: [1.0, 1.0],
    },
    Vertex {
        position: [-1.0, -1.0, 0.0],
        normal: [0.0, 0.0, 1.0],
        tex_coords: [0.0, 1.0],
    },
];

#[allow(dead_code)]
pub const RECT_INDICES: &[u16] = &[0, 1, 2, 2, 1, 3];

#[allow(dead_code)]
pub const PENTAGON_VERTS: &[Vertex] = &[
    Vertex {
        position: [-0.1736482, 0.984_807_7, 0.0],
        normal: [0.0, 0.0, 1.0],
        tex_coords: [0.4131759, 0.99240386],
    }, // A
    Vertex {
        position: [-0.990_268_1, 0.13917294, 0.0],
        normal: [0.0, 0.0, 1.0],
        tex_coords: [0.0048659444, 0.56958647],
    }, // B
    Vertex {
        position: [-0.43837098, -0.898_794_1, 0.0],
        normal: [0.0, 0.0, 1.0],
        tex_coords: [0.28081453, 0.05060294],
    }, // C
    Vertex {
        position: [0.71933996, -0.6946582, 0.0],
        normal: [0.0, 0.0, 1.0],
        tex_coords: [0.85967, 0.1526709],
    }, // D
    Vertex {
        position: [0.88294744, 0.4694718, 0.0],
        normal: [0.0, 0.0, 1.0],
        tex_coords: [0.9414737, 0.7347359],
    }, // E
];

#[allow(dead_code)]
pub const PENTAGON_INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

#[allow(dead_code)]
pub const CUBE_VERTS: &[Vertex] = &[
    Vertex {
        position: [0.5, 0.5, -0.5],
        normal: [0.0, 0.0, -1.0],
        tex_coords: [1.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, -0.5],
        normal: [0.0, 0.0, -1.0],
        tex_coords: [1.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, -0.5],
        normal: [0.0, 0.0, -1.0],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, -0.5],
        normal: [0.0, 0.0, -1.0],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, -0.5],
        normal: [0.0, 0.0, -1.0],
        tex_coords: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, -0.5],
        normal: [0.0, 0.0, -1.0],
        tex_coords: [1.0, 1.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.5],
        normal: [0.0, 0.0, 1.0],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        normal: [0.0, 0.0, 1.0],
        tex_coords: [1.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.5],
        normal: [0.0, 0.0, 1.0],
        tex_coords: [1.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.5],
        normal: [0.0, 0.0, 1.0],
        tex_coords: [1.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.5],
        normal: [0.0, 0.0, 1.0],
        tex_coords: [0.0, 1.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.5],
        normal: [0.0, 0.0, 1.0],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.5],
        normal: [-1.0, 0.0, 0.0],
        tex_coords: [1.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, -0.5],
        normal: [-1.0, 0.0, 0.0],
        tex_coords: [1.0, 1.0],
    },
    Vertex {
        position: [-0.5, -0.5, -0.5],
        normal: [-1.0, 0.0, 0.0],
        tex_coords: [0.0, 1.0],
    },
    Vertex {
        position: [-0.5, -0.5, -0.5],
        normal: [-1.0, 0.0, 0.0],
        tex_coords: [0.0, 1.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.5],
        normal: [-1.0, 0.0, 0.0],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.5],
        normal: [-1.0, 0.0, 0.0],
        tex_coords: [1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, -0.5],
        normal: [1.0, 0.0, 0.0],
        tex_coords: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, -0.5],
        normal: [1.0, 0.0, 0.0],
        tex_coords: [1.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.5],
        normal: [1.0, 0.0, 0.0],
        tex_coords: [1.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.5],
        normal: [1.0, 0.0, 0.0],
        tex_coords: [1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        normal: [1.0, 0.0, 0.0],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, -0.5],
        normal: [1.0, 0.0, 0.0],
        tex_coords: [0.0, 1.0],
    },
    Vertex {
        position: [-0.5, -0.5, -0.5],
        normal: [0.0, -1.0, 0.0],
        tex_coords: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, -0.5],
        normal: [0.0, -1.0, 0.0],
        tex_coords: [1.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        normal: [0.0, -1.0, 0.0],
        tex_coords: [1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        normal: [0.0, -1.0, 0.0],
        tex_coords: [1.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.5],
        normal: [0.0, -1.0, 0.0],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, -0.5],
        normal: [0.0, -1.0, 0.0],
        tex_coords: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.5],
        normal: [0.0, 1.0, 0.0],
        tex_coords: [1.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, -0.5],
        normal: [0.0, 1.0, 0.0],
        tex_coords: [1.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5, -0.5],
        normal: [0.0, 1.0, 0.0],
        tex_coords: [0.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5, -0.5],
        normal: [0.0, 1.0, 0.0],
        tex_coords: [0.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.5],
        normal: [0.0, 1.0, 0.0],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.5],
        normal: [0.0, 1.0, 0.0],
        tex_coords: [1.0, 0.0],
    },
];

pub const CUBE_INDICES: &[u16] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35,
];
