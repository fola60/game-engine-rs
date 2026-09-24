use crate::{Dimension, TwoD, engine_context::EngineContext, state::State};
use std::{
    marker::PhantomData,
    sync::Arc,
    time::{Duration, Instant},
};

use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalSize, Size};
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowAttributes, WindowId};

pub struct Engine<D: Dimension, G = ()> {
    game: G,
    pub scene: crate::scene::Scene<D>,
    dimension: PhantomData<D>,
    initialized: bool,
    screen_width: u32,
    screen_height: u32,
    title: String,
    last_frame_time: Option<Instant>,
    frame_counter: u32,
    pub window: Option<Arc<dyn Window>>,
    pub state: Option<State>,
    pub fps: u32,
}

impl<D: Dimension> Engine<D> {
    pub fn init<G: GameLoop<D> + 'static>(
        game: G,
        screen_width: u32,
        screen_height: u32,
        title: &str,
    ) -> Engine<D, G> {
        Engine {
            game,
            scene: crate::scene::Scene::default(),
            dimension: PhantomData,
            initialized: false,
            screen_width,
            screen_height,
            title: String::from(title),
            last_frame_time: None,
            frame_counter: 0,
            window: None,
            state: None,
            fps: 60,
        }
    }
}

impl<D: Dimension, G: GameLoop<D> + 'static> Engine<D, G> {
    pub fn run(self) -> anyhow::Result<()> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Wait);
        event_loop.run_app(self)?;

        Ok(())
    }

    fn run_frame(&mut self) {
        let now = Instant::now();
        let dt = self
            .last_frame_time
            .map(|last| now.duration_since(last).as_secs_f32())
            .unwrap_or(0.0);
        self.last_frame_time = Some(now);

        let state = match self.state.as_mut() {
            Some(state) => state,
            None => return,
        };

        {
            let mut ctx = EngineContext::<D>::new(state, &mut self.fps, &mut self.scene);
            self.game.update(&mut ctx, dt);
        }
        self.scene.update(dt);
        {
            let mut ctx = EngineContext::<D>::new(state, &mut self.fps, &mut self.scene);
            self.game.render(&mut ctx);
        }

        state.sync_scene(&mut self.scene);
        state.update();
        match state.render() {
            Ok(()) => {}
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                let size = state.window.surface_size();
                state.resize(size.width, size.height);
            }
            Err(error) => println!("Unable to render {error}"),
        }

        self.frame_counter = self.frame_counter.wrapping_add(1);
    }
}

impl<D: Dimension, G: GameLoop<D> + 'static> ApplicationHandler for Engine<D, G> {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        let mut window_attributes = WindowAttributes::default();
        window_attributes.surface_size = Some(Size::Physical(PhysicalSize {
            width: self.screen_width,
            height: self.screen_height,
        }));
        window_attributes.title = self.title.clone();

        let window: Arc<dyn Window> =
            Arc::from(event_loop.create_window(window_attributes).unwrap());

        let mut state = pollster::block_on(State::new_with_mode(window.clone(), D::MODE))
            .expect("Failed to create State");

        if !self.initialized {
            let mut ctx = EngineContext::<D>::new(&mut state, &mut self.fps, &mut self.scene);
            self.game.startup(&mut ctx);
            self.initialized = true;
        }

        self.window = Some(window.clone());
        self.state = Some(state);
        self.last_frame_time = Some(Instant::now());
        window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        _id: WindowId,
        event: WindowEvent,
    ) {
        if matches!(event, WindowEvent::CloseRequested) {
            event_loop.exit();
            return;
        }

        if matches!(event, WindowEvent::RedrawRequested) {
            self.run_frame();
            return;
        }

        let state = match self.state.as_mut() {
            Some(state) => state,
            None => return,
        };

        match &event {
            WindowEvent::RedrawRequested => {
                unreachable!("redraw events are handled before input events")
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
                if *primary {
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
            }
            _ => {}
        }

        let mut ctx = EngineContext::<D>::new(state, &mut self.fps, &mut self.scene);
        self.game.event(&mut ctx, &event);
        self.scene.event(&event);
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        let Some(window) = &self.window else {
            return;
        };

        let frame_duration = Duration::from_secs_f64(1.0 / self.fps.max(1) as f64);
        let next_frame = self
            .last_frame_time
            .map(|last| last + frame_duration)
            .unwrap_or_else(Instant::now);
        let now = Instant::now();

        if now >= next_frame {
            window.request_redraw();
        } else {
            event_loop.set_control_flow(ControlFlow::WaitUntil(next_frame));
        }
    }
}

pub trait GameLoop<D: Dimension = TwoD> {
    fn startup(&mut self, _ctx: &mut EngineContext<D>) {}

    fn event(&mut self, _ctx: &mut EngineContext<D>, _event: &WindowEvent) {}

    fn update(&mut self, _ctx: &mut EngineContext<D>, _dt: f32) {}

    fn render(&mut self, _ctx: &mut EngineContext<D>) {}
}
