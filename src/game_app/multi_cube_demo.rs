use std::rc::Rc;

use glam::{Quat, Vec2, Vec3};

use crate::{
    gameplay::{ArcballCameraController, CameraController, FreeLookCameraController},
    math_utils::rotate_around_pivot,
    renderer::{
        lighting::{DirectionalLight, LightAttenuation, PointLight, SpotLight},
        materials::MaterialBuilder,
        meshes::{builtin_mesh, BuiltinMesh},
        scene::Scene,
        textures::{self, ColorSpace},
        Renderer,
    },
};

use super::GameApp;

enum CameraControllerType {
    Arcball,
    Freelook,
}

pub struct MultiCubeDemo {
    arcball: ArcballCameraController,
    freelook: FreeLookCameraController,
    camera_type: CameraControllerType,
    sim_time_elapsed: std::time::Duration,
    scene: Scene,
}

impl MultiCubeDemo {
    const POINT_LIGHTS: &'static [PointLight] = &[
        PointLight {
            position: Vec3::new(1.2, 1.0, 2.0),
            attenuation: LightAttenuation {
                constant: 1.0,
                linear: 0.2,
                quadratic: 0.01,
            },
            color: Vec3::new(0.8, 0.8, 0.8),
            ambient: 0.0425,
            specular: 1.0,
        },
        PointLight {
            position: Vec3::new(-4.0, 2.0, -12.0),
            attenuation: LightAttenuation {
                constant: 1.0,
                linear: 0.2,
                quadratic: 0.03,
            },
            color: Vec3::new(1.0, 0.0, 0.0),
            ambient: 0.0,
            specular: 1.0,
        },
        PointLight {
            position: Vec3::new(0.7, 0.2, 2.0),
            attenuation: LightAttenuation {
                constant: 1.0,
                linear: 0.2,
                quadratic: 0.03,
            },
            color: Vec3::new(1.0, 0.5, 0.0),
            ambient: 0.0,
            specular: 1.0,
        },
        PointLight {
            position: Vec3::new(2.3, -3.3, -4.0),
            attenuation: LightAttenuation {
                constant: 1.0,
                linear: 0.2,
                quadratic: 0.03,
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

    pub fn new() -> Self {
        Self {
            arcball: ArcballCameraController::new(),
            freelook: FreeLookCameraController::new(),
            camera_type: CameraControllerType::Arcball,
            sim_time_elapsed: Default::default(),
            scene: Default::default(),
        }
    }
}

impl GameApp for MultiCubeDemo {
    fn load_content(&mut self, renderer: &mut Renderer) -> anyhow::Result<()> {
        // TODO: Pass these values as raw parameters.
        let device = &renderer.device;
        let queue = &renderer.queue;
        let default_textures = &renderer.default_textures;

        // Create the crate model.
        let diffuse_map = Rc::new(textures::from_image_bytes(
            device,
            queue,
            include_bytes!("../../content/crate_diffuse.dds"),
            ColorSpace::Srgb,
            Some("crate diffuse map"),
        )?);

        let specular_map = Rc::new(textures::from_image_bytes(
            device,
            queue,
            include_bytes!("../../content/crate_specular.dds"),
            ColorSpace::Srgb,
            Some("crate specular map"),
        )?);

        let crate_material = MaterialBuilder::new()
            .specular_color(Vec3::new(1.0, 1.0, 1.0))
            .specular_power(64.0)
            .diffuse_map(diffuse_map)
            .specular_map(specular_map)
            .build(default_textures);

        let cube_mesh = Rc::new(builtin_mesh(
            &renderer.device,
            &renderer.bind_group_layouts,
            BuiltinMesh::Cube,
            &crate_material,
        ));

        // Spawn a buch of copies of the crate model.

        // Set up scene.
        self.scene.models.reserve(Self::INITIAL_CUBE_POS.len());

        for initial_pos in Self::INITIAL_CUBE_POS {
            self.scene.models.push(renderer.create_model(
                cube_mesh.clone(),
                *initial_pos,
                Quat::IDENTITY,
                Vec3::ONE,
            ));
        }

        // This demo has one directional, one spot and three point lights.
        self.scene.directional_lights.push(Self::DIRECTIONAL_LIGHT);
        self.scene.spot_lights.push(Self::SPOT_LIGHT);

        for light in Self::POINT_LIGHTS.iter() {
            self.scene.point_lights.push(light.clone());
        }

        Ok(())
    }

    fn input(&mut self, event: &winit::event::WindowEvent) -> bool {
        use winit::{
            event::{ElementState, WindowEvent},
            keyboard::{KeyCode, PhysicalKey},
        };

        // Handle keyboard input events specific to this demo scene:
        //  `c` -> Toggle between arcball and freelook camera.
        if let WindowEvent::KeyboardInput {
            event: keyboard_input_event,
            ..
        } = event
        {
            match keyboard_input_event.physical_key {
                PhysicalKey::Code(KeyCode::KeyC)
                    if keyboard_input_event.state == ElementState::Released =>
                {
                    self.camera_type = match self.camera_type {
                        CameraControllerType::Arcball => CameraControllerType::Freelook,
                        CameraControllerType::Freelook => CameraControllerType::Arcball,
                    };
                }
                _ => {}
            }
        }

        // Forward input to the active camera controller.
        match self.camera_type {
            CameraControllerType::Arcball => self.arcball.process_input(event),
            CameraControllerType::Freelook => self.freelook.process_input(event),
        }
    }

    fn update_sim(&mut self, delta: std::time::Duration) {
        self.sim_time_elapsed += delta;
    }

    fn prepare_render(&mut self, renderer: &mut Renderer, delta: std::time::Duration) {
        // Allow camera controller to control the scene's camera.
        match self.camera_type {
            CameraControllerType::Arcball => {
                self.arcball.update_camera(&mut renderer.camera, delta)
            }
            CameraControllerType::Freelook => {
                self.freelook.update_camera(&mut renderer.camera, delta)
            }
        }

        // Spot light follows the camera.
        self.scene.spot_lights[0].position = renderer.camera.eye();
        self.scene.spot_lights[0].direction = renderer.camera.forward();

        // Make the primary light orbit around the scene.
        let sys_time_secs: f32 = self.sim_time_elapsed.as_secs_f32();

        let light_xy = rotate_around_pivot(
            Vec2::new(0.0, 0.0),
            1.0,
            (sys_time_secs * 24.0).to_radians(),
        );

        self.scene.point_lights[0].position = Vec3::new(light_xy.x, light_xy.y, light_xy.y);
    }

    fn mouse_motion(&mut self, delta_x: f64, delta_y: f64) {
        match self.camera_type {
            CameraControllerType::Arcball => self.arcball.process_mouse_motion(Vec2 {
                x: delta_x as f32,
                y: delta_y as f32,
            }),
            CameraControllerType::Freelook => self.freelook.process_mouse_motion(Vec2 {
                x: delta_x as f32,
                y: delta_y as f32,
            }),
        }
    }

    fn mouse_scroll_wheel(&mut self, delta_x: f64, delta_y: f64) {
        match self.camera_type {
            CameraControllerType::Arcball => self.arcball.process_mouse_wheel(Vec2 {
                x: delta_y as f32,
                y: delta_x as f32,
            }),
            CameraControllerType::Freelook => self.freelook.process_mouse_wheel(Vec2 {
                x: delta_y as f32,
                y: delta_x as f32,
            }),
        }
    }

    fn render_scene(&self) -> &Scene {
        &self.scene
    }
}
