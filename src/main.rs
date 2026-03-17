use game_engine_rs::{Point2D, app_window::AppWindow, camera, engine::Engine};
use winit::event_loop::{ControlFlow, EventLoop};



fn main() -> anyhow::Result<()> {
    let mut app = Engine::init(700, 600, "100x200 app window title.");
    let camera_eye = Point2D {x: 0.0, y: 0.0};
    let camera_target = Point2D {x: 0.0, y: 0.0};
    app.set_camera_eye(camera_eye);
    app.set_camera_target(camera_target);
    app.run()
}