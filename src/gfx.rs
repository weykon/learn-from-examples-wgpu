use std::sync::Arc;
use wgpu::RequestAdapterOptions;
use winit::window::Window;

pub(crate) struct GfxContext {
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) surface_config: Option<wgpu::SurfaceConfiguration>,
}

impl GfxContext {
    pub(crate) async fn new(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::default();

        let adapter = instance
            .request_adapter(&RequestAdapterOptions::default())
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap();

        let surface = unsafe { instance.create_surface(window.clone()).unwrap() };

        GfxContext {
            device,
            queue,
            surface,
            adapter,
            surface_config: None,
        }
    }
}
