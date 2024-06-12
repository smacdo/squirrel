use super::shaders::Vertex;

// CCW, bottom left to bottom right.
/*
pub const TRIANGLE_VERTS: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.5, 0.0],
        color: [1.0, 0.0, 0.0],
        tex_coords: [0.5, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        color: [0.0, 1.0, 0.0],
        tex_coords: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        color: [0.0, 0.0, 1.0],
        tex_coords: [1.0, 1.0],
    },
];
pub const TRIANGLE_INDICES: &[u16] = &[0, 1, 2];
*/
pub const RECT_VERTS: &[Vertex] = &[
    Vertex {
        position: [1.0, 1.0, 0.0],
        color: [1.0, 0.0, 0.0],
        tex_coords: [1.0, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0, 0.0],
        color: [0.0, 1.0, 0.0],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0],
        color: [0.0, 0.0, 1.0],
        tex_coords: [1.0, 1.0],
    },
    Vertex {
        position: [-1.0, -1.0, 0.0],
        color: [1.0, 0.0, 0.0],
        tex_coords: [0.0, 1.0],
    },
];
pub const RECT_INDICES: &[u16] = &[0, 1, 2, 2, 1, 3];

/*
pub const PENTAGON_VERTS: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        color: [0.5, 0.0, 0.5],
        tex_coords: [0.4131759, 0.99240386],
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        color: [0.5, 0.0, 0.5],
        tex_coords: [0.0048659444, 0.56958647],
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        color: [0.5, 0.0, 0.5],
        tex_coords: [0.28081453, 0.05060294],
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        color: [0.5, 0.0, 0.5],
        tex_coords: [0.85967, 0.1526709],
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        color: [0.5, 0.0, 0.5],
        tex_coords: [0.9414737, 0.7347359],
    }, // E
];

pub const PENTAGON_INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];
*/
