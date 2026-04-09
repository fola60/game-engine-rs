use game_engine_rs::{
    engine::{Engine, GameLoop},
    engine_context::EngineContext,
    Color, Gesture, Mode, Point2D,
};
use winit::event::WindowEvent;

struct GestureDemo {
    latest: String,
}

impl GestureDemo {
    fn format_gesture(gesture: Gesture) -> String {
        match gesture {
            Gesture::SwipeUp(angle) => format!("Latest: SwipeUp ({angle:.1} deg)"),
            Gesture::SwipeDown(angle) => format!("Latest: SwipeDown ({angle:.1} deg)"),
            Gesture::SwipeLeft(angle) => format!("Latest: SwipeLeft ({angle:.1} deg)"),
            Gesture::SwipeRight(angle) => format!("Latest: SwipeRight ({angle:.1} deg)"),
        }
    }
}

impl GameLoop for GestureDemo {
    fn startup(&mut self, ctx: &mut EngineContext) {
        ctx.set_mode(Mode::Mode2D);
        ctx.set_target_fps(60);
    }

    fn game_loop(&mut self, ctx: &mut EngineContext, _event: WindowEvent) {
        if let Some(gesture) = ctx.get_gesture() {
            self.latest = Self::format_gesture(gesture);
        }

        ctx.clear_background(Color::White);
        ctx.draw_text(Point2D { x: 20.0, y: 20.0 }, "Swipe and release", 34);
        ctx.draw_text(Point2D { x: 20.0, y: 60.0 }, &self.latest, 28);
    }
}

fn main() -> anyhow::Result<()> {
    let game = GestureDemo {
        latest: String::from("Latest: none"),
    };
    Engine::init(game, 1280, 720, "Gesture Demo").run()
}
