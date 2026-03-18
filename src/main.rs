use game_engine_rs::{Point2D, engine::{Engine, EngineContext, GameLoop}};
use winit::{event::WindowEvent};



struct MyGame {

}
impl GameLoop for MyGame {
    fn game_loop(
        &mut self,
        ctx: &mut EngineContext,
        event: WindowEvent,
    ) {
        let width = 200;
        let height = 200;
        ctx.renderer.draw_rectangle(ctx.device, location: Point2D {x: 0.0, y: -1.0}, width, height);
    }
}

fn main() -> anyhow::Result<()> {
    let my_game = MyGame {};
    let mut app = Engine::init(my_game, 700, 600, "100x200 app window title.");
    let camera_eye = Point2D {x: 0.0, y: 0.0};
    let camera_target = Point2D {x: 0.0, y: 0.0};
    app.set_camera_eye(camera_eye);
    app.set_camera_target(camera_target);
    app.run()

}

