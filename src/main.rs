use cgmath::Point3;
use game_engine_rs::{
    Color, 
    Point2D, 
    engine::{Engine, EngineContext, GameLoop}, 
    Mode
};

use winit::{event::{KeyEvent, WindowEvent}, keyboard::{KeyCode, PhysicalKey}};



struct MyGame {
    camera_eye: Point3<f32>,
    camera_target: Point3<f32>
}
impl GameLoop for MyGame {
    fn game_loop(
        &mut self,
        ctx: &mut EngineContext,
        event: WindowEvent,
    ) {
        ctx.set_mode(Mode::MODE2D);
        ctx.clear_background(Color {r: 255.0, g: 255.0, b: 255.0, a: 0.0 });
        let width = 0.5;
        let height = 0.5;
        // ctx.draw_rectangle(Point2D {x: -0.5, y: 0.5}, width, height);
        ctx.draw_circle(Point2D {x: -0.5, y: 0.5}, 0.5);

        match event {
            WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _ } => {
                match event.physical_key {
                    PhysicalKey::Code(code) => {
                        match code {
                            KeyCode::ArrowLeft => { self.camera_target.x -= 0.1 },
                            KeyCode::ArrowRight => { self.camera_target.x += 0.1 },
                            KeyCode::ArrowDown => { self.camera_target.y -= 0.1 },
                            KeyCode::ArrowUp => { self.camera_target.y += 0.1 },
                            KeyCode::KeyA => { self.camera_eye.x -= 0.1 },
                            KeyCode::KeyD => { self.camera_eye.x += 0.1 },
                            KeyCode::KeyS => { self.camera_eye.z -= 0.1 },
                            KeyCode::KeyW => { self.camera_eye.z += 0.1 },
                            _ => {}
                        }
                    },
                    _ => {}
                }
            },
            _ => {}
        }
        ctx.set_camera_eye(self.camera_eye);
        ctx.set_camera_target(self.camera_target);
    }
}

fn main() -> anyhow::Result<()> {
    let my_game = MyGame {camera_eye: Point3 { x: 0.0, y: -0.5, z: -5.0 }, camera_target: Point3 { x: 0.5, y: 0.5, z: 0.0 }};
    let app = Engine::init(my_game, 700, 600, "Game engine");
    app.run()

}

