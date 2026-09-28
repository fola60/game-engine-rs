//! Run with `cargo run --example paddle_game`.
//! Move with A/D or arrows, reset with R, and quit with Escape.
//! Swipe left/right to move, up to reset, or down to center the paddle.

use cgmath::Vector3;
use game_engine_rs::{
    Circle, CollisionPhase, Color, Gesture, Point2D, Rectangle, SceneCollisionEvent, TwoD,
    engine::{Engine, GameLoop},
    engine_context::EngineContext,
    scene::Scene,
};
use winit::keyboard::KeyCode;

const PADDLE_ID: &str = "paddle";
const BALL_ID: &str = "ball";
const PADDLE_WIDTH: f32 = 3.5;
const PADDLE_HEIGHT: f32 = 0.5;
const PADDLE_TOP: f32 = -7.5;
const PADDLE_SPEED: f32 = 12.0;
const BALL_RADIUS: f32 = 0.45;
const LEFT_WALL: f32 = -17.0;
const RIGHT_WALL: f32 = 17.0;
const TOP_WALL: f32 = 9.5;
const BOTTOM_WALL: f32 = -10.0;

struct PaddleGame {
    ball_position: Vector3<f32>,
    velocity: Point2D,
    previous_bottom: f32,
    paddle_x: f32,
    score: u32,
    misses: u32,
}

impl PaddleGame {
    fn new() -> Self {
        Self {
            ball_position: Vector3::new(0.0, 5.0, 0.0),
            velocity: Point2D { x: 4.5, y: -6.0 },
            previous_bottom: 5.0 - BALL_RADIUS,
            paddle_x: -PADDLE_WIDTH * 0.5,
            score: 0,
            misses: 0,
        }
    }

    fn reset(&mut self) {
        self.ball_position = Vector3::new(0.0, 5.0, 0.0);
        self.previous_bottom = 5.0 - BALL_RADIUS;
        self.velocity = Point2D {
            x: if self.misses % 2 == 0 { 4.5 } else { -4.5 },
            y: -6.0,
        };
    }

    fn sync(&self, scene: &mut Scene<TwoD>) {
        scene
            .get_mut::<Rectangle>(PADDLE_ID)
            .unwrap()
            .transform
            .position
            .x = self.paddle_x;
        scene.get_mut::<Circle>(BALL_ID).unwrap().transform.position = self.ball_position;
    }

    fn collide(&mut self, event: &SceneCollisionEvent) {
        if event.phase == CollisionPhase::Ended {
            return;
        }
        let position = &mut self.ball_position;
        match event.other_id(BALL_ID).unwrap_or("") {
            "left-wall" if self.velocity.x < 0.0 => {
                position.x = LEFT_WALL + BALL_RADIUS;
                self.velocity.x = self.velocity.x.abs();
            }
            "right-wall" if self.velocity.x > 0.0 => {
                position.x = RIGHT_WALL - BALL_RADIUS;
                self.velocity.x = -self.velocity.x.abs();
            }
            "top-wall" if self.velocity.y > 0.0 => {
                position.y = TOP_WALL - BALL_RADIUS;
                self.velocity.y = -self.velocity.y.abs();
            }
            PADDLE_ID if self.velocity.y < 0.0 && self.previous_bottom >= PADDLE_TOP => {
                position.y = PADDLE_TOP + BALL_RADIUS;
                self.velocity.y = self.velocity.y.abs();
                let center = self.paddle_x + PADDLE_WIDTH * 0.5;
                let offset = (position.x - center) / (PADDLE_WIDTH * 0.5);
                self.velocity.x = (self.velocity.x + offset * 2.0).clamp(-12.0, 12.0);
                self.score += 1;
            }
            "bottom-wall" if self.velocity.y < 0.0 => {
                self.misses += 1;
                self.reset();
            }
            _ => {}
        }
    }
}

impl GameLoop<TwoD> for PaddleGame {
    fn startup(&mut self, ctx: &mut EngineContext<TwoD>) {
        ctx.set_target_fps(60);
        ctx.spawn(
            PADDLE_ID,
            Rectangle::new(PADDLE_WIDTH, PADDLE_HEIGHT)
                .at(self.paddle_x, PADDLE_TOP)
                .with_color(Color::Blue),
        );
        ctx.spawn(
            BALL_ID,
            Circle::new(BALL_RADIUS)
                .at(0.0, 5.0)
                .with_color(Color::Magenta),
        );
        let width = RIGHT_WALL - LEFT_WALL;
        let height = TOP_WALL - BOTTOM_WALL + 3.0;
        for (id, wall) in [
            (
                "left-wall",
                Rectangle::new(1.0, height).at(LEFT_WALL - 1.0, TOP_WALL + 1.0),
            ),
            (
                "right-wall",
                Rectangle::new(1.0, height).at(RIGHT_WALL, TOP_WALL + 1.0),
            ),
            (
                "top-wall",
                Rectangle::new(width, 1.0).at(LEFT_WALL, TOP_WALL + 1.0),
            ),
            (
                "bottom-wall",
                Rectangle::new(width, 1.0).at(LEFT_WALL, BOTTOM_WALL - 2.0 * BALL_RADIUS),
            ),
        ] {
            ctx.spawn(id, wall.with_color(Color::LightGray));
        }
    }

    fn update(&mut self, ctx: &mut EngineContext<TwoD>, dt: f32) {
        let dt = dt.min(0.05);
        let left = ctx.key_down(KeyCode::KeyA) || ctx.key_down(KeyCode::ArrowLeft);
        let right = ctx.key_down(KeyCode::KeyD) || ctx.key_down(KeyCode::ArrowRight);
        self.paddle_x += (right as i8 - left as i8) as f32 * PADDLE_SPEED * dt;
        if ctx.key_pressed(KeyCode::KeyR) {
            self.reset();
        }
        if let Some(gesture) = ctx.take_gesture() {
            match gesture {
                Gesture::SwipeLeft(_) => self.paddle_x -= 2.5,
                Gesture::SwipeRight(_) => self.paddle_x += 2.5,
                Gesture::SwipeUp(_) => self.reset(),
                Gesture::SwipeDown(_) => self.paddle_x = -PADDLE_WIDTH * 0.5,
            }
        }
        self.paddle_x = self.paddle_x.clamp(LEFT_WALL, RIGHT_WALL - PADDLE_WIDTH);
        self.previous_bottom = self.ball_position.y - BALL_RADIUS;
        self.ball_position.x += self.velocity.x * dt;
        self.ball_position.y += self.velocity.y * dt;
        self.sync(ctx.scene);
    }

    fn on_collision(&mut self, ctx: &mut EngineContext<TwoD>, event: &SceneCollisionEvent) {
        self.collide(event);
        self.sync(ctx.scene);
    }

    fn render(&mut self, ctx: &mut EngineContext<TwoD>) {
        ctx.clear_background(Color::LightGray);
        ctx.draw_text(
            Point2D { x: 20.0, y: 20.0 },
            &format!("Score: {}  Misses: {}", self.score, self.misses),
            28,
        );
        ctx.draw_text(
            Point2D { x: 20.0, y: 55.0 },
            "Move: A/D or arrows | Swipe: move/reset | R: reset ball",
            20,
        );
    }
}

fn main() -> anyhow::Result<()> {
    Engine::<TwoD>::init(PaddleGame::new(), 1280, 720, "Paddle Game").run()
}

