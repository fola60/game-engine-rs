use cgmath::{Quaternion, Rotation3, Vector3};
use game_engine_rs::{
    Color, Cube, Entity, EntityContext, RenderData, ThreeD,
    engine::{Engine, GameLoop},
    engine_context::EngineContext,
};

struct SpinningCube {
    cube: Cube,
    angle: f32,
}

impl Entity<ThreeD> for SpinningCube {
    fn ready(&mut self, _ctx: &mut EntityContext<ThreeD>) {
        self.cube.color = Color::Cyan;
    }

    fn update(&mut self, _ctx: &mut EntityContext<ThreeD>, dt: f32) {
        self.angle = (self.angle + dt) % std::f32::consts::TAU;
        self.cube.transform.rotation =
            Quaternion::from_axis_angle(Vector3::unit_y(), cgmath::Rad(self.angle));
    }

    fn render_data(&self) -> Option<RenderData<ThreeD>> {
        self.cube.render_data()
    }
}

struct Demo;

impl GameLoop<ThreeD> for Demo {
    fn startup(&mut self, ctx: &mut EngineContext<ThreeD>) {
        ctx.clear_background(Color::Black);
        ctx.spawn(
            "spinning-cube",
            SpinningCube {
                cube: Cube::new(1.5, 1.5, 1.5),
                angle: 0.0,
            },
        );
    }
}

fn main() -> anyhow::Result<()> {
    Engine::<ThreeD>::init(Demo, 960, 640, "Entity: spinning cube").run()
}
