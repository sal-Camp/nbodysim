use crate::{camera, render};
use wgpu::*;
use winit::window::Window;
use winit::*;

pub struct State {
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub gfx: render::Render,
}

impl State {
    async fn new(window: &Window) -> Self {
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

        let gfx = render::Render::new(&device, &config);

        Self {
            size,
            instance,
            surface,
            device,
            queue,
            config,
            gfx,
        }
    }
}
