use winit::{
    event_loop::{ControlFlow, EventLoop},
};

mod app;
mod renderer;
mod ui;

use app::App;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new("App", 1024, 800);
    event_loop.run_app(&mut app).unwrap();
}