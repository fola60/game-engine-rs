//! Run with `cargo run --example paddle_game`.
//! Move with A/D or the arrow keys, reset the ball with R, and quit with Escape.
//! Swipe left/right to move, up to reset the ball, or down to center the paddle.

use game_engine_rs::{
    Circle, Color, Gesture, Point2D, Rectangle, TwoD,
    engine::{Engine, GameLoop},
    engine_context::EngineContext,
};
use winit::{
    event::WindowEvent,
    keyboard::{KeyCode, PhysicalKey},
};

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
    paddle_x: f32,
    ball_position: Point2D,
    ball_velocity: Point2D,
    moving_left: bool,
    moving_right: bool,
    score: u32,
    misses: u32,
}

impl PaddleGame {
    fn new() -> Self {
        Self {
            paddle_x: -PADDLE_WIDTH * 0.5,
            ball_position: Point2D { x: 0.0, y: 5.0 },
            ball_velocity: Point2D { x: 4.5, y: -6.0 },
            moving_left: false,
            moving_right: false,
            score: 0,
            misses: 0,
        }
    }

    fn reset_ball(&mut self) {
        self.ball_position = Point2D { x: 0.0, y: 5.0 };
        self.ball_velocity = Point2D {
            x: if self.misses % 2 == 0 { 4.5 } else { -4.5 },
            y: -6.0,
        };
    }
}

impl GameLoop<TwoD> for PaddleGame {
    fn startup(&mut self, ctx: &mut EngineContext<TwoD>) {
        ctx.set_target_fps(60);
        let mut paddle = Rectangle::new(PADDLE_WIDTH, PADDLE_HEIGHT);
        paddle.color = Color::Blue;
        ctx.spawn(PADDLE_ID, paddle);
        let mut ball = Circle::new(BALL_RADIUS);
        ball.color = Color::Magenta;
        ctx.spawn(BALL_ID, ball);
    }

    fn event(&mut self, ctx: &mut EngineContext<TwoD>, event: &WindowEvent) {
        if let WindowEvent::KeyboardInput { event, .. } = event {
            let pressed = event.state.is_pressed();

            match event.physical_key {
                PhysicalKey::Code(KeyCode::ArrowLeft | KeyCode::KeyA) => {
                    self.moving_left = pressed;
                }
                PhysicalKey::Code(KeyCode::ArrowRight | KeyCode::KeyD) => {
                    self.moving_right = pressed;
                }
                PhysicalKey::Code(KeyCode::KeyR) if pressed => self.reset_ball(),
                _ => {}
            }
        }

        if let Some(gesture) = ctx.take_gesture() {
            match gesture {
                Gesture::SwipeLeft(_) => self.paddle_x -= 2.5,
                Gesture::SwipeRight(_) => self.paddle_x += 2.5,
                Gesture::SwipeUp(_) => self.reset_ball(),
                Gesture::SwipeDown(_) => self.paddle_x = -PADDLE_WIDTH * 0.5,
            }
        }
    }

    fn update(&mut self, _ctx: &mut EngineContext<TwoD>, dt: f32) {
        // Limit large time steps after pausing or dragging the window.
        let dt = dt.min(0.05);
        let movement = self.moving_right as i8 - self.moving_left as i8;
        self.paddle_x += movement as f32 * PADDLE_SPEED * dt;
        self.paddle_x = self.paddle_x.clamp(LEFT_WALL, RIGHT_WALL - PADDLE_WIDTH);

        self.ball_position.x += self.ball_velocity.x * dt;
        self.ball_position.y += self.ball_velocity.y * dt;

        if self.ball_position.x - BALL_RADIUS <= LEFT_WALL {
            self.ball_position.x = LEFT_WALL + BALL_RADIUS;
            self.ball_velocity.x = self.ball_velocity.x.abs();
        } else if self.ball_position.x + BALL_RADIUS >= RIGHT_WALL {
            self.ball_position.x = RIGHT_WALL - BALL_RADIUS;
            self.ball_velocity.x = -self.ball_velocity.x.abs();
        }

        if self.ball_position.y + BALL_RADIUS >= TOP_WALL {
            self.ball_position.y = TOP_WALL - BALL_RADIUS;
            self.ball_velocity.y = -self.ball_velocity.y.abs();
        }

        let ball_over_paddle = self.ball_position.x + BALL_RADIUS >= self.paddle_x
            && self.ball_position.x - BALL_RADIUS <= self.paddle_x + PADDLE_WIDTH;
        let ball_at_paddle = self.ball_position.y - BALL_RADIUS <= PADDLE_TOP
            && self.ball_position.y - BALL_RADIUS >= PADDLE_TOP - PADDLE_HEIGHT;

        if self.ball_velocity.y < 0.0 && ball_over_paddle && ball_at_paddle {
            self.ball_position.y = PADDLE_TOP + BALL_RADIUS;
            self.ball_velocity.y = self.ball_velocity.y.abs();

            let paddle_center = self.paddle_x + PADDLE_WIDTH * 0.5;
            let hit_offset = (self.ball_position.x - paddle_center) / (PADDLE_WIDTH * 0.5);
            self.ball_velocity.x += hit_offset * 2.0;
            self.score += 1;
        }

        if self.ball_position.y + BALL_RADIUS < BOTTOM_WALL {
            self.misses += 1;
            self.reset_ball();
        }
    }

    fn render(&mut self, ctx: &mut EngineContext<TwoD>) {
        ctx.clear_background(Color::LightGray);
        let paddle = ctx
            .entity_mut::<Rectangle>(PADDLE_ID)
            .expect("paddle exists");
        paddle.transform.position.x = self.paddle_x;
        paddle.transform.position.y = PADDLE_TOP;
        let ball = ctx.entity_mut::<Circle>(BALL_ID).expect("ball exists");
        ball.transform.position.x = self.ball_position.x;
        ball.transform.position.y = self.ball_position.y;

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
