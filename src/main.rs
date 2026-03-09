use game_engine_rs::app_window::AppWindow;
use winit::event_loop::{ControlFlow, EventLoop};



fn main() -> anyhow::Result<()> {
    let app = AppWindow::new(700, 600, String::from("100x200 app window title."));

    app.init_window()
}