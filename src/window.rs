use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalSize, Size};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId, WindowAttributes};

#[derive(Default)]
struct App {
    window: Option<Box<dyn Window>>,
    screen_width: u32,
    screen_height: u32,
    title: String
}
impl ApplicationHandler for App {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        // The event loop has launched, and we can initialize our UI state.

        // Create a simple window with default attributes.
        let mut window_attributes = WindowAttributes::default();
        window_attributes.surface_size = Some(Size::Physical(PhysicalSize {width: self.screen_width, height: self.screen_height }));
        window_attributes.title = self.title.clone();
        self.window = Some(event_loop.create_window(window_attributes).unwrap());
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        id: WindowId,
        event: WindowEvent,
    ) {
        // Called by `EventLoop::run_app` when a new event happens on the window.
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                self.window.as_ref().unwrap().request_redraw();
            },
            WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _ } => {
                println!("Event: {:?}", event); 
            },
            _ => (),
        }
    }
}

impl App {
    fn new(screen_width: u32, screen_height: u32, title: String) -> Self {
        Self { window: None, screen_height, screen_width, title }
    }

    fn init_window(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Create a new event loop.
        let event_loop = EventLoop::new()?;

        // Configure settings before launching.

        // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
        // dispatched any events. This is ideal for games and similar applications.
        event_loop.set_control_flow(ControlFlow::Poll);

        // Launch and begin running the event loop.
        event_loop.run_app(App::default())?;

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new event loop.
    let event_loop = EventLoop::new()?;

    // Configure settings before launching.

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    event_loop.set_control_flow(ControlFlow::Wait);

    // Launch and begin running the event loop.
    event_loop.run_app(App::default())?;

    Ok(())
}
