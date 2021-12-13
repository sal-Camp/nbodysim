use crate::{camera, render, texture, DrawSphere, celestial_body};
use wgpu::*;
use winit::event::WindowEvent;
use winit::window::Window;
use winit::*;
use crate::celestial_body::Entity;

pub struct State {
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub renderer: render::Render,
    pub entities: Vec<celestial_body::Entity>,
}

impl State {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // An instance is a handle to our GPU
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

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::POLYGON_MODE_LINE,
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        let renderer = render::Render::new(&device, &config);

        let entities: Vec<celestial_body::Entity> = Vec::new();
        let entity1 =

        Self {
            size,
            instance,
            surface,
            device,
            queue,
            config,
            renderer,
            entities
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.renderer.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            self.surface.configure(&self.device, &self.config);
        }
    }

    // A lot of the following functions (input, update, render) can probably also
    // be refactored out to render.rs
    // Will do in future update
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.renderer.camera_controller.process_events(event)
    }

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
    }

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
        render_pass.set_pipeline(&self.renderer.render_pipeline);
        render_pass.draw_sphere(
            &self.renderer.sphere,
            &self.renderer.camera_bind_group,
        );
        render_pass.draw_sphere_instanced(
            &self.renderer.sphere2,
            0..self.renderer.instances.len() as u32,
            &self.renderer.camera_bind_group,
        );

        // Releasing the borrow on 'encoder'
        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}
