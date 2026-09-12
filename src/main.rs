use game_engine_rs::{
    Color, Gesture, Mode, Point2D,
    engine::{Engine, GameLoop},
    engine_context::EngineContext,
};
use winit::{
    event::WindowEvent,
    keyboard::{KeyCode, PhysicalKey},
};

const PADDLE_ID: u32 = 1;
const BALL_ID: u32 = 2;

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

impl GameLoop for PaddleGame {
    fn startup(&mut self, ctx: &mut EngineContext) {
        ctx.set_mode(Mode::Mode2D);
        ctx.set_target_fps(60);
        ctx.add_rectangle(PADDLE_ID, PADDLE_WIDTH, PADDLE_HEIGHT);
        ctx.add_circle(BALL_ID, BALL_RADIUS);
    }

    fn event(&mut self, ctx: &mut EngineContext, event: &WindowEvent) {
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

    fn update(&mut self, _ctx: &mut EngineContext, dt: f32) {
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

    fn render(&mut self, ctx: &mut EngineContext) {
        ctx.clear_background(Color::LightGray);
        ctx.draw_rectangle(
            PADDLE_ID,
            &Point2D {
                x: self.paddle_x,
                y: PADDLE_TOP,
            },
            Color::Blue,
        );
        ctx.draw_circle(BALL_ID, &self.ball_position, Color::Magenta);

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
    Engine::init(PaddleGame::new(), 1280, 720, "Paddle Game").run()
}
