use crate::{engine_context::EngineContext, state::State};
use std::{sync::Arc, time::Instant};

use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalSize, Size};
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowAttributes, WindowId};

pub struct Engine<G: GameLoop + 'static> {
    game: G,
    initialized: bool,
    screen_width: u32,
    screen_height: u32,
    title: String,
    last_frame_time: Option<Instant>,
    frame_counter: u32,
    last_tap: Option<u32>,
    last_hold: Option<u32>,
    pub window: Option<Arc<dyn Window>>,
    pub state: Option<State>,
    pub fps: u32,
}

impl<G: GameLoop> Engine<G> {
    pub fn init(game: G, screen_width: u32, screen_height: u32, title: &str) -> Engine<G> {
        Self {
            game,
            initialized: false,
            screen_width,
            screen_height,
            title: String::from(title),
            last_frame_time: None,
            frame_counter: 0,
            last_tap: None,
            last_hold: None,
            window: None,
            state: None,
            fps: 60,
        }
    }

    pub fn run(self) -> anyhow::Result<()> {
        let event_loop = EventLoop::new()?;

        // Configure settings before launching.

        // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
        // dispatched any events. This is ideal for games and similar applications.
        event_loop.set_control_flow(ControlFlow::Poll);

        // Launch and begin running the event loop.
        event_loop.run_app(self)?;

        Ok(())
    }
}

impl<G: GameLoop> ApplicationHandler for Engine<G> {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        let mut window_attributes = WindowAttributes::default();
        window_attributes.surface_size = Some(Size::Physical(PhysicalSize {
            width: self.screen_width,
            height: self.screen_height,
        }));
        window_attributes.title = self.title.clone();

        let window: Arc<dyn Window> =
            Arc::from(event_loop.create_window(window_attributes).unwrap());

        let state = pollster::block_on(State::new(window.clone())).expect("Failed to create State");

        self.window = Some(window);
        self.state = Some(state);
        self.last_frame_time = Some(Instant::now());
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        _id: WindowId,
        event: WindowEvent,
    ) {
        // Called by `EventLoop::run_app` when a new event happens on the window.
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };

        match &event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = state.window.surface_size();
                        state.resize(size.width, size.height);
                    }
                    Err(e) => {
                        println!("Unable to render {}", e);
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => state.handle_key(event_loop, *code, key_state.is_pressed()),
            WindowEvent::SurfaceResized(size) => state.resize(size.width, size.height),
            WindowEvent::PointerMoved {
                position, primary, ..
            } => {
                state.pointer_position = crate::Point2D {
                    x: position.x as f32,
                    y: position.y as f32,
                };
                if *primary {
                    state
                        .swipe_tracker
                        .pointer_move(state.pointer_position.clone());
                }
            }
            WindowEvent::PointerButton {
                state: button_state,
                primary,
                ..
            } => {
                if !*primary {
                    return;
                }
                let now = Instant::now();
                if button_state.is_pressed() {
                    state
                        .swipe_tracker
                        .pointer_down(state.pointer_position.clone(), now);
                } else if let Some(gesture) = state
                    .swipe_tracker
                    .pointer_up(state.pointer_position.clone(), now)
                {
                    state.gesture = Some(gesture);
                }
            }
            _ => {}
        }

        let now = Instant::now();
        let elapsed = now
            .duration_since(self.last_frame_time.unwrap())
            .as_secs_f32();
        let dt = if let Some(last) = self.last_frame_time {
            now.duration_since(last).as_secs_f32() // delta time in seconds as f32
        } else {
            0.0
        };

        // sleep for target fps
        let target_frame_time = 1.0 / self.fps as f32;
        if elapsed < target_frame_time {
            let sleep_duration = target_frame_time - elapsed;
            std::thread::sleep(std::time::Duration::from_secs_f32(sleep_duration));
        }

        self.last_frame_time = Some(now);

        let mut ctx = EngineContext {
            entities: &mut state.entities,
            entity_ids: &mut state.entity_ids,
            device: &state.device,
            queue: &state.queue,
            texture_bind_group_layout: &state.texture_bind_group_layout,
            camera: &mut state.camera,
            background: &mut state.background,
            mode: &mut state.mode,
            text: &mut state.text,
            gesture: &mut state.gesture,
            fps: &mut self.fps,
            dt: dt,
        };

        if !self.initialized {
            self.game.startup(&mut ctx);
            self.initialized = true;
        }

        self.game.game_loop(&mut ctx, event);
    }
}

pub trait GameLoop {
    fn game_loop(&mut self, _ctx: &mut EngineContext, _event: WindowEvent) {}

    fn startup(&mut self, _ctx: &mut EngineContext) {}
}
