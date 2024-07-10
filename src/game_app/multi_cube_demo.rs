use std::rc::Rc;

use glam::{Quat, Vec2, Vec3};

use crate::{
    gameplay::{ArcballCameraController, CameraController},
    math_utils::rotate_around_pivot,
    renderer::{
        meshes::{builtin_mesh, BuiltinMesh},
        models::Model,
        shading::{DirectionalLight, LightAttenuation, Material, PointLight, SpotLight},
        textures, Renderer,
    },
};

use super::GameApp;

pub struct MultiCubeDemo {
    camera_controller: ArcballCameraController,
    sim_time_elapsed: std::time::Duration,
}

impl MultiCubeDemo {
    const POINT_LIGHTS: &'static [PointLight] = &[
        PointLight {
            position: Vec3::new(1.2, 1.0, 2.0),
            attenuation: LightAttenuation {
                constant: 1.0,
                linear: 0.09,
                quadratic: 0.032,
            },
            color: Vec3::new(0.8, 0.8, 0.8),
            ambient: 0.0425,
            specular: 1.0,
        },
        PointLight {
            position: Vec3::new(-4.0, 2.0, -12.0),
            attenuation: LightAttenuation {
                constant: 1.0,
                linear: 0.09,
                quadratic: 0.032,
            },
            color: Vec3::new(1.0, 0.0, 0.0),
            ambient: 0.0,
            specular: 1.0,
        },
        PointLight {
            position: Vec3::new(0.7, 0.2, 2.0),
            attenuation: LightAttenuation {
                constant: 1.0,
                linear: 0.09,
                quadratic: 0.032,
            },
            color: Vec3::new(1.0, 0.5, 0.0),
            ambient: 0.0,
            specular: 1.0,
        },
        PointLight {
            position: Vec3::new(2.3, -3.3, -4.0),
            attenuation: LightAttenuation {
                constant: 1.0,
                linear: 0.09,
                quadratic: 0.032,
            },
            color: Vec3::new(0.0, 0.0, 1.0),
            ambient: 0.0,
            specular: 1.0,
        },
    ];
    const DIRECTIONAL_LIGHT: DirectionalLight = DirectionalLight {
        direction: Vec3::new(0.0, -1.0, 0.0),
        color: Vec3::new(0.3, 0.3, 0.3),
        ambient: 0.01,
        specular: 0.2,
    };
    const SPOT_LIGHT: SpotLight = SpotLight {
        position: Vec3::ZERO,
        direction: Vec3::ZERO,
        cutoff_radians: 0.2181662,       // 12.5 degree.
        outer_cutoff_radians: 0.3054326, // 17.5 degree.
        color: Vec3::new(0.8, 0.8, 0.8),
        attenuation: LightAttenuation {
            constant: 1.0,
            linear: 0.09,
            quadratic: 0.032,
        },
        ambient: 0.01,
        specular: 1.0,
    };

    const INITIAL_CUBE_POS: &'static [Vec3] = &[
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(2.0, 5.0, -15.0),
        Vec3::new(-1.5, -2.2, -2.5),
        Vec3::new(-3.8, -2.0, -12.3),
        Vec3::new(2.4, -0.4, -3.5),
        Vec3::new(-1.7, 3.0, 7.5),
        Vec3::new(1.3, -2.0, -2.5),
        Vec3::new(1.5, 2.0, -2.5),
        Vec3::new(1.5, 0.2, -1.5),
        Vec3::new(-1.3, 1.0, -1.5),
    ];

    pub fn new(renderer: &mut Renderer) -> Self {
        // Create the crate model.
        let crate_material = Material {
            ambient_color: Vec3::new(1.0, 1.0, 1.0),
            diffuse_color: Vec3::new(1.0, 1.0, 1.0),
            diffuse_map: Rc::new(
                textures::from_image_bytes(
                    &renderer.device,
                    &renderer.queue,
                    include_bytes!("../../content/crate_diffuse.dds"),
                    Some("crate diffuse texture"),
                )
                .unwrap(),
            ),
            specular_color: Vec3::new(1.0, 1.0, 1.0),
            specular_map: Rc::new(
                textures::from_image_bytes(
                    &renderer.device,
                    &renderer.queue,
                    include_bytes!("../../content/crate_specular.dds"),
                    Some("crate specular texture"),
                )
                .unwrap(),
            ),
            specular_power: 64.0,
            emissive_map: Rc::new(textures::new_1x1(
                &renderer.device,
                &renderer.queue,
                [0, 0, 0],
                Some("default emission texture map"),
            )),
        };

        let cube_mesh = Rc::new(builtin_mesh(
            &renderer.device,
            &renderer.bind_group_layouts,
            BuiltinMesh::Cube,
            &crate_material,
        ));

        // Spawn a buch of copies of the crate model.

        // Set up scene.
        renderer.models.reserve(Self::INITIAL_CUBE_POS.len());

        for initial_pos in Self::INITIAL_CUBE_POS {
            renderer.models.push(Model::new(
                &renderer.device,
                &renderer.bind_group_layouts,
                *initial_pos,
                Quat::IDENTITY,
                Vec3::ONE,
                cube_mesh.clone(),
            ));
        }

        // This demo has one directional, one spot and three point lights.
        renderer.directional_lights.push(Self::DIRECTIONAL_LIGHT);
        renderer.spot_lights.push(Self::SPOT_LIGHT);

        for light in Self::POINT_LIGHTS.iter() {
            renderer.point_lights.push(light.clone());
        }

        Self {
            camera_controller: ArcballCameraController::new(),
            sim_time_elapsed: Default::default(),
        }
    }
}

impl GameApp for MultiCubeDemo {
    fn input(&mut self, _event: &winit::event::WindowEvent) -> bool {
        false
    }

    fn update_sim(&mut self, delta: std::time::Duration) {
        self.sim_time_elapsed += delta;
    }

    fn prepare_render(&mut self, renderer: &mut Renderer, delta: std::time::Duration) {
        // Allow camera controller to control the scene's camera.
        self.camera_controller
            .update_camera(&mut renderer.camera, delta);

        // Spot light follows the camera.
        renderer.spot_lights[0].position = renderer.camera.eye();
        renderer.spot_lights[0].direction = renderer.camera.forward();

        // Make the primary light orbit around the scene.
        let sys_time_secs: f32 = self.sim_time_elapsed.as_secs_f32();

        let light_xy = rotate_around_pivot(
            Vec2::new(0.0, 0.0),
            1.0,
            (sys_time_secs * 24.0).to_radians(),
        );

        renderer.point_lights[0].position = Vec3::new(light_xy.x, light_xy.y, light_xy.y);
    }

    fn mouse_motion(&mut self, delta_x: f64, delta_y: f64) {
        self.camera_controller.process_mouse_motion(Vec2 {
            x: delta_x as f32,
            y: delta_y as f32,
        })
    }

    fn mouse_scroll_wheel(&mut self, delta_x: f64, delta_y: f64) {
        self.camera_controller.process_mouse_wheel(Vec2 {
            x: delta_y as f32,
            y: delta_x as f32,
        })
    }
}
