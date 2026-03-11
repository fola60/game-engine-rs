use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalSize, Size};
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId, WindowAttributes};

use crate::state::State;

#[derive(Default)]
pub struct AppWindow {
    pub window: Option<Arc<dyn Window>>,
    pub screen_width: u32,
    pub screen_height: u32,
    title: String,
    state: Option<State>
}

impl ApplicationHandler for AppWindow {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        let mut window_attributes = WindowAttributes::default();
        window_attributes.surface_size = Some(Size::Physical(PhysicalSize {
            width: self.screen_width,
            height: self.screen_height,
        }));
        window_attributes.title = self.title.clone();

        let window: Arc<dyn Window> = Arc::from(
            event_loop.create_window(window_attributes).unwrap()
        );

        let state = pollster::block_on(State::new(window.clone()))
            .expect("Failed to create State");

        self.window = Some(window);
        self.state = Some(state);
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        id: WindowId,
        event: WindowEvent,
    ) {
        // Called by `EventLoop::run_app` when a new event happens on the window.
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };
        

        match event {
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
            },
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => state.handle_key(event_loop, code, key_state.is_pressed()),
            WindowEvent::SurfaceResized(size) => state.resize(size.width, size.height),
            WindowEvent::PointerMoved { device_id: _, position, primary, source } => {
                match state.handle_mouse_moved(position, primary, source) {
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
            },
            _ => {}
        }

    }

    
}

impl AppWindow {
    pub fn new(screen_width: u32, screen_height: u32, title: String) -> Self {
        Self { window: None, screen_height, screen_width, title, state: None }
    }

    pub fn init_window(mut self) -> anyhow::Result<()> {
        // Create a new event loop.
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

#[cfg(test)]
mod tests {
    use winit::event_loop::{ControlFlow, EventLoop};

    use crate::app_window::AppWindow;

    #[test]
    fn open_window() -> Result<(), Box<dyn std::error::Error>> {
        let app = AppWindow::new(700, 500, String::from("100x200 app window title."));
        // Create a new event loop.
        let event_loop = EventLoop::new()?;

        // Configure settings before launching.

        // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
        // dispatched any events. This is ideal for games and similar applications.
        event_loop.set_control_flow(ControlFlow::Poll);

        // Launch and begin running the event loop.
        event_loop.run_app(app)?;

        Ok(())
    }
}