use anyhow::Result;
use cgmath::*;
use std::ops::Range;
use wgpu::util::DeviceExt;
use wgpu::BindGroup;

pub struct Entity {
    pub sphere: Sphere,
    pub position: Vector3<f32>,
}

impl Entity {
    pub fn new(new_position: Vector3<f32>, device: &wgpu::Device) -> Self {
        /*        let mut sphere;
        match Sphere::new(5, &device) {
            Ok(sp) => {
                sphere = sp;
            }
            Err(e) => {
                Panic!("Sphere failed to create!");
            }
        }*/

        let mut sphere = Sphere::new(5, &device);

        let position = new_position;

        Self { sphere, position }
    }
}

pub trait Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SphereMeshVertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex for SphereMeshVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SphereMeshVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct Mesh {
    resolution: u32,
    local_up: Vector3<f32>,
    axis_a: Vector3<f32>,
    axis_b: Vector3<f32>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_elements: u32,
}

impl Mesh {
    fn new(resolution: u32, local_up: Vector3<f32>, device: &wgpu::Device) -> Self {
        let axis_a = Vector3::new(local_up.y, local_up.z, local_up.x);
        let axis_b = Vector3::cross(local_up, axis_a);

        let mut vertices = Vec::new();
        let mut triangles = Vec::new();
        for x in 0..resolution {
            for y in 0..resolution {
                let i = x + y * resolution;
                let percent = Vector2::new(x as f32, y as f32) / (resolution - 1) as f32;
                let point_on_unit_cube =
                    local_up + (percent.x - 0.5) * 2.0 * axis_a + (percent.y - 0.5) * 2.0 * axis_b;
                let point_on_unit_sphere = point_on_unit_cube.normalize();
                vertices.push(SphereMeshVertex {
                    position: [
                        point_on_unit_sphere.x,
                        point_on_unit_sphere.y,
                        point_on_unit_sphere.z,
                    ],
                    color: [0.5, 0.5, 0.5],
                });

                if x != resolution - 1 && y != resolution - 1 {
                    // First Triangle
                    triangles.push(i);
                    triangles.push(i + resolution + 1);
                    triangles.push(i + resolution);

                    // Second Triangle
                    triangles.push(i);
                    triangles.push(i + 1);
                    triangles.push(i + resolution + 1);
                }
            }
        }
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Index Buffer"),
            contents: bytemuck::cast_slice(&triangles),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_elements = triangles.len() as u32;

        Self {
            resolution,
            local_up,
            axis_a,
            axis_b,
            vertex_buffer,
            index_buffer,
            num_elements,
        }
    }
}

const DIRECTIONS: [Vector3<f32>; 6] = [
    Vector3::new(0.0, 1.0, 0.0),  // up
    Vector3::new(0.0, -1.0, 0.0), // down
    Vector3::new(-1.0, 0.0, 0.0), // left
    Vector3::new(1.0, 0.0, 0.0),  // right
    Vector3::new(0.0, 0.0, 1.0),  // forward
    Vector3::new(0.0, 0.0, -1.0), // back
];

pub struct Sphere {
    meshes: Vec<Mesh>,
}

impl Sphere {
    pub fn new(resolution: u32, device: &wgpu::Device) -> Self {
        let mut meshes: Vec<Mesh> = Vec::with_capacity(6);
        // Creating our 6 faces of the cube/sphere
        for dir in DIRECTIONS {
            meshes.push(Mesh::new(resolution, dir, device));
        }

        Self { meshes }
    }
}

pub trait DrawSphere<'a> {
    fn draw_mesh(
        &mut self,
        mesh: &'a Mesh,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );

    fn draw_sphere(
        &mut self,
        sphere: &'a Sphere,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );

    fn draw_sphere_instanced(
        &mut self,
        sphere: &'a Sphere,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
}

impl<'a, 'b> DrawSphere<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(
        &mut self,
        mesh: &'b Mesh,
        camera_bind_group: &'b BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
    ) {
        self.draw_mesh_instanced(mesh, 0..1, camera_bind_group, light_bind_group);
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        instances: Range<u32>,
        camera_bind_group: &'b BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, &camera_bind_group, &[]);
        self.set_bind_group(1, &light_bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_sphere(
        &mut self,
        sphere: &'b Sphere,
        camera_bind_group: &'b BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
    ) {
        self.draw_sphere_instanced(sphere, 0..1, camera_bind_group, light_bind_group);
    }

    fn draw_sphere_instanced(
        &mut self,
        sphere: &'b Sphere,
        instances: Range<u32>,
        camera_bind_group: &'b BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
    ) {
        for mesh in &sphere.meshes {
            self.draw_mesh_instanced(mesh, instances.clone(), camera_bind_group, light_bind_group);
        }
    }
}

pub trait DrawLight<'a> {
    fn draw_light_mesh(
        &mut self,
        mesh: &'a Mesh,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
    fn draw_light_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );

    fn draw_light_model(
        &mut self,
        sphere: &'a Sphere,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
    fn draw_light_model_instanced(
        &mut self,
        sphere: &'a Sphere,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
}

impl<'a, 'b> DrawLight<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_light_mesh(
        &mut self,
        mesh: &'b Mesh,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
    ) {
        self.draw_light_mesh_instanced(mesh, 0..1, camera_bind_group, light_bind_group);
    }

    fn draw_light_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        instances: Range<u32>,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, camera_bind_group, &[]);
        self.set_bind_group(1, light_bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_light_model(
        &mut self,
        sphere: &'b Sphere,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
    ) {
        self.draw_light_model_instanced(sphere, 0..1, camera_bind_group, light_bind_group);
    }
    fn draw_light_model_instanced(
        &mut self,
        sphere: &'b Sphere,
        instances: Range<u32>,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
    ) {
        for mesh in &sphere.meshes {
            self.draw_light_mesh_instanced(
                mesh,
                instances.clone(),
                camera_bind_group,
                light_bind_group,
            );
        }
    }
}
