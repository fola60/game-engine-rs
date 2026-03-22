use cgmath::Point3;
use game_engine_rs::{
    Point2D,
    camera, 
    engine::{Engine, EngineContext, GameLoop},
    renderer::{EntityType},
    VERTICES, INDICES
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
        let width = 2.0;
        let height = 2.0;
        ctx.draw_rectangle(Point2D {x: -0.5, y: 0.5}, width, height);
        ctx.draw_rectangle(Point2D {x: 3.5, y: 0.5}, width, height);

        // ctx.add_entity_vertex_data(VERTICES.to_vec(), INDICES.to_vec(), EntityType::VertIndicie);
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
    let my_game = MyGame {camera_eye: Point3 { x: 0.0, y: 0.0, z: -2.0 }, camera_target: Point3 { x: 0.5, y: 0.5, z: 0.0 }};
    let app = Engine::init(my_game, 700, 600, "100x200 app window title.");
    app.run()

}

