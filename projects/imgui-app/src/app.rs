use std::{cell::RefCell, sync::Arc};
use std::time::Instant;

use imgui::{Context};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use imgui_wgpu::Renderer;

use wgpu::hal::SurfaceError;
use winit::{
    application::ApplicationHandler, event::{Event, WindowEvent}, event_loop::{ActiveEventLoop, ControlFlow}, window::{Window, WindowAttributes, WindowId},
};

use crate::renderer::WgpuState;
use crate::ui;

pub struct App {
    title: String,
    width: u32,
    height: u32,
    state: Option<AppState>
}

pub struct AppState {
    window: Arc<Window>,
    wgpu: WgpuState,
    imgui: Context,
    platform: WinitPlatform,
    renderer: Renderer,
    last_frame: Instant,
}

impl App {
    pub fn new(title: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            title: title.into(),
            width,
            height,
            state: None,
        }
    }

    fn state(&mut self) -> &mut AppState {
        self.state.as_mut().expect("AppState not initialized")
    }
}

impl App {
    fn create_window(&self, event_loop: &ActiveEventLoop) -> Arc<Window> {
        Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title(&self.title)
                        .with_inner_size(winit::dpi::LogicalSize::new(self.width, self.height)),
                )
                .unwrap(),
        )
    }

    fn setup_imgui() -> (Context, WinitPlatform) {
        let mut imgui = Context::create();
        imgui.set_ini_filename(None);

        let fonts = imgui.fonts();
        fonts.add_font(&[imgui::FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                size_pixels: 16.0,
                oversample_h: 4,
                oversample_v: 4,
                ..Default::default()
            }),
        }]);

        let mut platform = WinitPlatform::new(&mut imgui);
        (imgui, platform)
    }

    fn setup_imgui_platform(platform: &mut WinitPlatform, imgui: &mut Context, window: &Window) {
        platform.attach_window(imgui.io_mut(), window, HiDpiMode::Default);
    }

    fn setup_renderer(imgui: &mut Context, wgpu: &WgpuState) -> Renderer {
        Renderer::new(
            imgui,
            &wgpu.device,
            &wgpu.queue,
            imgui_wgpu::RendererConfig {
                texture_format: wgpu.config.format,
                ..imgui_wgpu::RendererConfig::new()
            },
        )
    }
}

impl App {
    fn get_surface_texture(&mut self) -> Option<wgpu::SurfaceTexture> {
        let state = self.state();
        
        match state.wgpu.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => Some(texture),
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => Some(texture),
            wgpu::CurrentSurfaceTexture::Occluded => {
                self.end_frame();
                None
            }
            wgpu::CurrentSurfaceTexture::Timeout => {
                self.end_frame();
                None
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                state.wgpu.surface.configure(&state.wgpu.device, &state.wgpu.config);
                self.end_frame();
                None
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                self.end_frame();
                None
            }
            _ => {
                self.end_frame();
                None
            }
        }
    }

    fn begin_frame(&mut self) -> bool {
        let state = self.state();
        
        if state.wgpu.config.width == 0 || state.wgpu.config.height == 0 {
            return false;
        }

        let io = state.imgui.io_mut();
        state.platform.prepare_frame(io, &state.window).unwrap();

        let ui = state.imgui.frame();
        ui::render_main_window(&ui);

        true
    }

    fn end_frame(&mut self) -> &imgui::DrawData {
        self.state().imgui.render()
    }

    fn get_surface_frame(&mut self) -> Option<(wgpu::SurfaceTexture, &imgui::DrawData)> {
        let state = self.state();
        
        let frame = match state.wgpu.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
            wgpu::CurrentSurfaceTexture::Occluded => {
                self.end_frame();
                return None;
            }
            wgpu::CurrentSurfaceTexture::Timeout => {
                self.end_frame();
                return None;
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                state.wgpu.surface.configure(&state.wgpu.device, &state.wgpu.config);
                self.end_frame();
                return None;
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                self.end_frame();
                return None;
            }
            _ => {
                self.end_frame();
                return None;
            }
        };

        let draw_data = self.end_frame();
        Some((frame, &draw_data))
    }

    fn render_pipeline(&mut self) -> Result<(), SurfaceError> {
        let Some(frame) = self.get_surface_texture() else {
            return Ok(());
        };

        let state = self.state();
        
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let descriptor = &wgpu::CommandEncoderDescriptor { label: Some("encoder") };
        let mut encoder = state.wgpu.device.create_command_encoder(descriptor);

        {
            let descriptor = &wgpu::RenderPassDescriptor {
                multiview_mask: None,
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    depth_slice: None,
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            };
            let mut rpass = encoder.begin_render_pass(descriptor);

            let draw_data = state.imgui.render();
            state.renderer.render(
                &draw_data,
                &state.wgpu.queue,
                &state.wgpu.device,
                &mut rpass
            ).unwrap();
        }

        state.wgpu.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }
}

impl App {
    fn handle_resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            let state = self.state();
            state.wgpu.resize(width, height);
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = self.create_window(event_loop);
        let wgpu = pollster::block_on(WgpuState::new(window.clone()));
        let (mut imgui, mut platform) = Self::setup_imgui();
        Self::setup_imgui_platform(&mut platform, &mut imgui, &window);
        let renderer = Self::setup_renderer(&mut imgui, &wgpu);

        self.state = Some(AppState {
            window: window.clone(),
            wgpu,
            imgui,
            platform,
            renderer,
            last_frame: Instant::now(),
        });

        window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let state = self.state();
        
        state.platform.handle_event::<WindowEvent>(
            state.imgui.io_mut(),
            &state.window,
            &Event::WindowEvent { window_id: _window_id, event: event.clone() }
        );

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                self.handle_resize(new_size.width, new_size.height);
            }
            WindowEvent::RedrawRequested => {
                self.render();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = &self.state {
            state.window.request_redraw();
        }
    }
}

impl App {
    fn render(&mut self) {
        if !self.begin_frame() {
            return;
        }

        self.render_pipeline();
    }
}