use cgmath::Point3;
use game_engine_rs::{
    Color, 
    Point2D, 
    engine::{Engine, EngineContext, GameLoop}, 
    Mode
};

use winit::{event::{WindowEvent}, keyboard::{KeyCode, PhysicalKey}};



struct MyGame {
    camera_eye: Point3<f32>,
    camera_target: Point3<f32>,
    ball_pos: Point2D
}

impl GameLoop for MyGame {

    fn startup(
        &mut self,
        ctx: &mut EngineContext,
    ) {
        ctx.add_circle(1, self.ball_pos.clone(), 0.5);
    }

    fn game_loop(
        &mut self,
        ctx: &mut EngineContext,
        event: WindowEvent,
    ) {
        ctx.set_mode(Mode::MODE2D);
        // let width = 0.5;
        // let height = 0.5;
        // ctx.draw_rectangle(Point2D {x: -0.5, y: 0.5}, width, height);
        let _ = ctx.draw_circle(1, Point2D::default());
        ctx.clear_background(Color { r: 255.0, g: 255.0, b: 255.0, a: 255.0 });
        match event {
            WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _ } => {
                match event.physical_key {
                    PhysicalKey::Code(code) => {
                        match code {
                            KeyCode::ArrowLeft => { self.ball_pos.x -= 0.1 },
                            KeyCode::ArrowRight => { self.ball_pos.x += 0.1 },
                            KeyCode::ArrowDown => { self.ball_pos.y -= 0.1 },
                            KeyCode::ArrowUp => { self.ball_pos.y += 0.1 },
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
        ctx.set_location(1, cgmath::Vector3 { x: self.ball_pos.x, y: self.ball_pos.y, z: 0.0 });
        ctx.set_camera_eye(self.camera_eye);
        ctx.set_camera_target(self.camera_target);
    }
}

fn main() -> anyhow::Result<()> {
    let my_game = MyGame {camera_eye: Point3 { x: 0.0, y: -0.5, z: -5.0 }, camera_target: Point3 { x: 0.5, y: 0.5, z: 0.0 }, ball_pos: Point2D::default()};
    let app = Engine::init(my_game, 700, 600, "Game engine");
    app.run()

}

