use crate::sphere::{DrawLight, Entity, Sphere};
use crate::{camera, render, sphere, texture, DrawSphere};
use cgmath::{Rotation3, Vector3};
use wgpu::*;
use winit::event::WindowEvent;
use winit::window::Window;
use winit::*;

/// The struct State holds the the current state of the program.
///
pub struct State {
    /// A handle to our surface and adapter(GPU)
    pub instance: wgpu::Instance,
    /// The window we will draw to
    pub surface: wgpu::Surface,
    /// The connection to our GPU
    pub device: wgpu::Device,
    /// The command queue for our device
    pub queue: wgpu::Queue,
    /// The configuration for our surface
    pub config: wgpu::SurfaceConfiguration,
    /// The size of our surface
    pub size: winit::dpi::PhysicalSize<u32>,
    /// Our renderer from render.rs
    pub renderer: render::Render,
}

impl State {
    /// Initializes a new state.
    /// Takes a winit::window parameter
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // An instance is a handle to surface and adapter
        // We can target all backends (Vulkan, Metal, DX12, WebGPU)
        let instance = wgpu::Instance::new(wgpu::Backends::all());

        // Where we will draw to
        let surface = unsafe { instance.create_surface(window) };

        // Our GPU
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        // Creating our connection to the GPU and its command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    // The features we want from our GPU
                    // Currently only calling the line draw mode
                    features: wgpu::Features::POLYGON_MODE_LINE,
                    // The limits an adapter supports.
                    // default() will support all modern backends
                    limits: wgpu::Limits::default(),
                    // Debug Label
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        // Definding our surface's configuration
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        // Initializing our surface using the above config
        surface.configure(&device, &config);

        // Initializing our render
        let renderer = render::Render::new(&device, &config);

        Self {
            size,
            instance,
            surface,
            device,
            queue,
            config,
            renderer,
        }
    }

    /// Recalculates window size whenever the user resizes the window.
    /// Takes in the state itself as well as the new size of the window.
    /// new_size is a winit::PhysicalSize struct that contains a width and height of the specificed type,
    /// in this case a u32.
    /// Also recreates our depth texture.
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            // Rebuilding our depth texture and then reconfiguring the surface
            self.renderer.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            self.surface.configure(&self.device, &self.config);
        }
    }

    // A lot of the following functions (input, update, render) can probably also
    // be refactored out to render.rs
    // Will do in future update

    /// Catches window events such as keyboard and mouse clicks
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.renderer.camera_controller.process_events(event)
    }

    /// Updates our camera position and light uniform
    pub fn update(&mut self) {
        self.renderer
            .camera_controller
            .update_camera(&mut self.renderer.camera);
        self.renderer
            .camera_uniform
            .update_view_proj(&self.renderer.camera);
        self.queue.write_buffer(
            &self.renderer.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.renderer.camera_uniform]),
        );
        let old_position: cgmath::Vector3<_> = self.renderer.light_uniform.position.into();
        self.renderer.light_uniform.position =
            (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(1.0))
                * old_position)
                .into();
        self.queue.write_buffer(
            &self.renderer.light_buffer,
            0,
            bytemuck::cast_slice(&[self.renderer.light_uniform]),
        );
    }

    /// Calls all of the necessary rendering commands
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // Store a surface texture to Render to
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // A command encoder to create commands for the gpu
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            // Where we will draw our color to. In this case we will draw to view, our TextureView
            color_attachments: &[
                // [[location(0)]] in our fragment shader
                wgpu::RenderPassColorAttachment {
                    view: &view,
                    // The texture to receive the output. Don't need to specify, so left a None
                    resolve_target: None,
                    // Telling wgpu what to do with the colors
                    ops: wgpu::Operations {
                        // Loading the stored colors after clearing with a bluish color
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        // Store the results to the texture in TextureView
                        store: true,
                    },
                },
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.renderer.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        render_pass.set_vertex_buffer(1, self.renderer.instance_buffer.slice(..));

        use crate::sphere::DrawLight;
        render_pass.set_pipeline(&self.renderer.light_render_pipeline);
        render_pass.draw_light_model(
            &self.renderer.sphere,
            &self.renderer.camera_bind_group,
            &self.renderer.light_bind_group,
        );

        render_pass.set_pipeline(&self.renderer.render_pipeline);
        render_pass.draw_sphere(
            &self.renderer.sphere,
            &self.renderer.camera_bind_group,
            &self.renderer.light_bind_group,
        );

        // Releasing the borrow on 'encoder'
        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}
