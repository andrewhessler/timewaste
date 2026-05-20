use std::num::NonZeroU64;

use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferUsages, Device, Queue,
    RenderPipeline, SurfaceConfiguration,
    util::{BufferInitDescriptor, DeviceExt},
};
use winit::{
    event::KeyEvent,
    keyboard::{Key, NamedKey},
};

use crate::{
    Direction,
    shape_util::{create_cube_vertices, create_practice_vertices},
    world::projection,
};

const TRANSLATION_SPEED: f32 = 100.;
const ROTATION_SPEED: f32 = 1.;
const SCALE_SPEED: f32 = 20.;

pub struct Translation {
    x: f32,
    y: f32,
    z: f32,
    x_speed: f32,
    y_speed: f32,
    z_speed: f32,
    x_direction: Direction,
    y_direction: Direction,
    z_direction: Direction,
}

#[derive(Debug)]
pub struct Rotation {
    angle: f32,
    direction: Direction,
}

#[derive(Debug)]
pub struct Scale {
    x: f32,
    y: f32,
    z: f32,
    x_direction: Direction,
    y_direction: Direction,
    z_direction: Direction,
}

pub struct Cube {
    pub translation: Translation,
    pub rotation: Rotation,
    pub scale: Scale,
    pub num_vertices: u32,
    pub uniform_buf: Buffer,
    pub uniform_bind_group: BindGroup,
    pub vertex_buf: Buffer,
    pub index_buf: Buffer,
}

impl Cube {
    pub fn new(device: &Device, pipeline: &RenderPipeline) -> Self {
        let translation = Translation {
            x: 0.,
            y: 0.,
            z: 100.,
            x_speed: 0.,
            y_speed: 0.,
            z_speed: 0.,
            x_direction: Direction::None,
            y_direction: Direction::None,
            z_direction: Direction::None,
        };

        let rotation = Rotation {
            angle: 0.,
            direction: Direction::None,
        };

        let scale = Scale {
            x: 1.,
            y: 1.,
            z: 1.,
            x_direction: Direction::None,
            y_direction: Direction::None,
            z_direction: Direction::None,
        };

        let mut contents: Vec<f32> = vec![
            1.0, 0.2, 0.2, 1.0, //color
        ];

        contents.extend([0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.]);
        contents.extend([0., 0., 0., 0., 0., 0., 0., 0.]);

        let uniform_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("uniform buffer"),
            contents: bytemuck::cast_slice(&contents),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("bind group"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
        });

        let (vertex_data, index_data, num_vertices) = create_cube_vertices();

        let vertex_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let index_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("index buffer"),
            contents: bytemuck::cast_slice(&index_data),
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
        });

        Self {
            translation,
            rotation,
            scale,
            num_vertices,
            uniform_buf,
            uniform_bind_group,
            index_buf,
            vertex_buf,
        }
    }

    pub fn animate(&mut self, queue: &Queue, config: &SurfaceConfiguration, delta_time: f32) {
        let x_speed_step = self.translation.x_speed * delta_time;

        self.translation.x += x_speed_step;
        self.translation.y += self.translation.y_speed * delta_time;
        self.translation.z += self.translation.z_speed * delta_time;

        self.rotation.angle += match self.rotation.direction {
            Direction::Inc => ROTATION_SPEED * delta_time,
            Direction::Dec => -ROTATION_SPEED * delta_time,
            _ => 0.,
        };

        self.scale.x += match self.scale.x_direction {
            Direction::Inc => SCALE_SPEED * delta_time,
            Direction::Dec => -SCALE_SPEED * delta_time,
            _ => 0.,
        };

        self.scale.y += match self.scale.y_direction {
            Direction::Inc => SCALE_SPEED * delta_time,
            Direction::Dec => -SCALE_SPEED * delta_time,
            _ => 0.,
        };

        self.scale.z += match self.scale.z_direction {
            Direction::Inc => SCALE_SPEED * delta_time,
            Direction::Dec => -SCALE_SPEED * delta_time,
            _ => 0.,
        };

        {
            let mut temp_buf = queue
                .write_buffer_with(
                    &self.uniform_buf,
                    std::mem::size_of::<[f32; 4]>() as u64,
                    NonZeroU64::new(std::mem::size_of::<[f32; 16]>() as u64).unwrap(),
                )
                .expect("temp buf to be creatable");

            let projection = projection(
                config.width as f32,
                config.height as f32,
                config.width as f32,
            );
            let trans_matrix = na::Matrix4::new_translation(&na::Vector3::new(
                self.translation.x,
                self.translation.y,
                self.translation.z,
            ));
            let scale_matrix = na::Matrix4::new_nonuniform_scaling(&na::Vector3::new(
                self.scale.x,
                self.scale.y,
                self.scale.z,
            ));
            let rotat_matrix = na::Matrix4::from_euler_angles(0., self.rotation.angle, 0.);
            let move_origin = na::Matrix4::new_translation(&na::Vector3::new(-50., -50., -50.));

            let mut matrix = projection * trans_matrix * scale_matrix * rotat_matrix * move_origin;

            let uniform_vals: [[f32; 4]; 4] = matrix.into();
            let mut res: Vec<f32> = vec![];

            for row in uniform_vals {
                res.extend(row.iter());
                // res.push(0.);
            }

            temp_buf.copy_from_slice(bytemuck::cast_slice(&res));
        }
    }

    pub fn handle_input(&mut self, event: &KeyEvent) {
        handle_translation(event, self);
        handle_rotation(event, self);
        handle_scale_x(event, self);
        handle_scale_y(event, self);
    }
}

fn handle_translation(event: &KeyEvent, state: &mut Cube) {
    handle_direction(
        event,
        Key::Named(NamedKey::ArrowLeft),
        Direction::Dec,
        &mut state.translation.x_direction,
    );
    handle_direction(
        event,
        Key::Named(NamedKey::ArrowRight),
        Direction::Inc,
        &mut state.translation.x_direction,
    );
    handle_direction(
        event,
        Key::Named(NamedKey::ArrowDown),
        Direction::Inc,
        &mut state.translation.y_direction,
    );
    handle_direction(
        event,
        Key::Named(NamedKey::ArrowUp),
        Direction::Dec,
        &mut state.translation.y_direction,
    );

    if state.translation.x_direction == Direction::Dec {
        state.translation.x_speed = -TRANSLATION_SPEED;
    }
    if state.translation.x_direction == Direction::Inc {
        state.translation.x_speed = TRANSLATION_SPEED;
    }
    if state.translation.x_direction == Direction::None {
        state.translation.x_speed = 0.;
    }

    if state.translation.y_direction == Direction::Dec {
        state.translation.y_speed = -TRANSLATION_SPEED;
    }
    if state.translation.y_direction == Direction::Inc {
        state.translation.y_speed = TRANSLATION_SPEED;
    }
    if state.translation.y_direction == Direction::None {
        state.translation.y_speed = 0.;
    }
}

fn handle_rotation(event: &KeyEvent, state: &mut Cube) {
    handle_direction(
        event,
        Key::Character("q".into()),
        Direction::Inc,
        &mut state.rotation.direction,
    );
    handle_direction(
        event,
        Key::Character("f".into()),
        Direction::Dec,
        &mut state.rotation.direction,
    );
}

fn handle_scale_x(event: &KeyEvent, state: &mut Cube) {
    handle_direction(
        event,
        Key::Character("s".into()),
        Direction::Inc,
        &mut state.scale.x_direction,
    );
    handle_direction(
        event,
        Key::Character("a".into()),
        Direction::Dec,
        &mut state.scale.x_direction,
    );
}

fn handle_scale_y(event: &KeyEvent, state: &mut Cube) {
    handle_direction(
        event,
        Key::Character("w".into()),
        Direction::Inc,
        &mut state.scale.y_direction,
    );
    handle_direction(
        event,
        Key::Character("r".into()),
        Direction::Dec,
        &mut state.scale.y_direction,
    );
}

fn handle_direction(
    event: &KeyEvent,
    logical_key: Key,
    pressed_direction: Direction,
    value_ref: &mut Direction,
) {
    if event.logical_key == logical_key {
        if event.state.is_pressed() {
            *value_ref = pressed_direction;
        } else if *value_ref == pressed_direction {
            *value_ref = Direction::None;
        }
    }
}
