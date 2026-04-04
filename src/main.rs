use cgmath::{Point3, Vector3};
use game_engine_rs::{
    engine::{Engine, GameLoop},
    engine_context::EngineContext,
    Color, Mode, Point2D, Z,
};

use winit::{
    event::WindowEvent,
    keyboard::{KeyCode, PhysicalKey},
};

struct MyGame {
    camera_eye: Point3<f32>,
    camera_target: Point3<f32>,
    ball_pos: Point2D,
    rectangle_id: u32,
    cube_id: u32,
}

impl GameLoop for MyGame {
    fn startup(&mut self, ctx: &mut EngineContext) {
        ctx.add_circle(1, 1.0);
        ctx.add_rectangle(self.rectangle_id, 1.0, 1.0);
        let _res = ctx.add_entity_from_model(3, "res/cube.obj");
        let _ = ctx.draw_cube(
            self.cube_id,
            1.0,
            1.0,
            1.0,
            Vector3 {
                x: -2.0,
                y: 0.5,
                z: Z,
            },
            Color::Blue,
        );

        ctx.set_target_fps(60);
        ctx.set_mode(Mode::Mode3D);
    }

    fn game_loop(&mut self, ctx: &mut EngineContext, event: WindowEvent) {
        ctx.draw_rectangle(self.rectangle_id, &Point2D::default(), Color::Green);
        ctx.clear_background(Color::White);
        ctx.draw_text(Point2D { x: 20.0, y: 20.0 }, "HUD: Hello text", 28);
        let _ = ctx.draw_circle(1, &self.ball_pos, Color::Black);
        let _ = ctx.draw_entity(
            3,
            Vector3 {
                x: 5.0,
                y: 0.5,
                z: Z,
            },
        );
        ctx.draw_entity(
            self.cube_id,
            Vector3 {
                x: -2.0,
                y: 0.5,
                z: Z,
            },
        );
        match event {
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => match event.physical_key {
                PhysicalKey::Code(code) => match code {
                    KeyCode::ArrowLeft => self.ball_pos.x -= 0.1,
                    KeyCode::ArrowRight => self.ball_pos.x += 0.1,
                    KeyCode::ArrowDown => self.ball_pos.y -= 0.1,
                    KeyCode::ArrowUp => self.ball_pos.y += 0.1,
                    KeyCode::KeyA => self.camera_eye.x -= 0.1,
                    KeyCode::KeyD => self.camera_eye.x += 0.1,
                    KeyCode::KeyS => self.camera_eye.z -= 0.1,
                    KeyCode::KeyW => self.camera_eye.z += 0.1,
                    KeyCode::Digit2 => ctx.set_mode(Mode::Mode2D),
                    KeyCode::Digit3 => ctx.set_mode(Mode::Mode3D),
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }
        // ctx.set_location(1, cgmath::Vector3 { x: self.ball_pos.x, y: self.ball_pos.y, z: 0.0 });
        ctx.set_camera_eye(self.camera_eye);
        ctx.set_camera_target(self.camera_target);
    }
}

fn main() -> anyhow::Result<()> {
    let my_game = MyGame {
        camera_eye: Point3 {
            x: 0.0,
            y: -5.5,
            z: -5.0,
        },
        camera_target: Point3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        ball_pos: Point2D::default(),
        rectangle_id: 0,
        cube_id: 4,
    };
    let app = Engine::init(my_game, 2700, 1600, "Game engine");
    app.run()
}
