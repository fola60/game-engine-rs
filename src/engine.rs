use std::sync::Arc;


use crate::camera::Camera;
use crate::state::Renderer;
use crate::{Color, Entity, Vertex};
use crate::{
    state::{State},
    Point2D
};


use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalSize, Size};
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId, WindowAttributes};



pub struct EngineContext<'a> {
    pub entities: &'a mut Vec<Entity>,
    pub camera: &'a mut Camera,
    pub renderer: &'a Renderer,
    pub device: &'a wgpu::Device
}

impl<'a> EngineContext<'a> {
    pub fn clear_background(&mut self, color: Color) {
        
    }
    
    pub fn draw_circle(&mut self, location: Point2D, radius: u128) {

    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }


    pub fn draw_rectangle(&mut self, location: Point2D, width: u32, height: u32) {

    }

    pub fn draw_text(&mut self, location: Point2D, text: &str, font_size: u8) {
    
    }
}


pub struct Engine<G: GameLoop + 'static > {
    game: G,
    pub window: Option<Arc<dyn Window>>,
    screen_width: u32,
    screen_height: u32,
    title: String,
    pub state: Option<State>,
    entities: Vec<Entity>,
    camera_eye: Point2D,
    camera_target: Point2D,
    camera_rotation: f32,
    camera_offset: Point2D,
    camera_zoom: f32
}

impl<G: GameLoop> Engine<G> {
    pub fn init(game: G, screen_width: u32, screen_height: u32, title: &str) -> Engine<G> {
        Self { 
            game,
            window: None,
            screen_width, 
            screen_height, 
            title: String::from(title), 
            state: None, 
            entities: vec![], 
            camera_eye: Point2D::default(), 
            camera_rotation: 0.0, 
            camera_target: Point2D::default(), 
            camera_offset: Point2D::default(), 
            camera_zoom: 1.0
        }
    }

    
    pub fn run(mut self) -> anyhow::Result<()>  {
        let event_loop = EventLoop::new()?;

        // Configure settings before launching.

        // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
        // dispatched any events. This is ideal for games and similar applications.
        event_loop.set_control_flow(ControlFlow::Poll);

        // Launch and begin running the event loop.
        event_loop.run_app(self)?;

        Ok(())
    }
    
    pub fn close_window(&self) {
        
    }
    
    pub fn clear_background(&self, color: Color) {
        
    }
    
    pub fn draw_circle(&self, location: Point2D, radius: u128) {

    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }


    pub fn draw_rectangle(&self, location: Point2D, width: u32, height: u32) {

    }

    pub fn draw_text(&self, location: Point2D, text: &str, font_size: u8) {

    }

    pub fn set_camera_eye(&mut self, location: Point2D) {
        self.camera_eye = location;
    }

    pub fn set_camera_target(&mut self, location: Point2D) {
        self.camera_target = location;
    }

    pub fn set_camera_rotation(&mut self, angle: f32) {
        self.camera_rotation = angle;
    }

    pub fn set_camera_offset(&mut self, offset: Point2D) {
        self.camera_offset = offset;
    }

     pub fn get_camera_eye(&mut self) -> &Point2D {
        &self.camera_eye
    }

    pub fn get_camera_target(&mut self) -> &Point2D {
        &self.camera_target
    }

    pub fn get_camera_rotation(&mut self) -> f32 {
        self.camera_rotation
    }

    pub fn get_camera_offset(&mut self) -> &Point2D {
        &self.camera_offset
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

        let window: Arc<dyn Window> = Arc::from(
            event_loop.create_window(window_attributes).unwrap()
        );

        

        let mut state = pollster::block_on(State::new(window.clone()))
            .expect("Failed to create State");

        
        if let eye = self.camera_eye.clone() {
            state.camera_controller.set_camera_eye(eye);
        }
        if let target = self.camera_target.clone() {
            state.camera_controller.set_camera_target(target);
        }
        if let rot = self.camera_rotation {
            state.camera_controller.set_camera_rotation(rot);
        }
        
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
            WindowEvent::SurfaceResized(size) => { 
                state.resize(size.width, size.height)
            }
            _ => {}
        }
        let mut ctx = EngineContext {
            entities: &mut self.entities,
            camera: &mut state.camera,
            renderer: &mut state.renderer
        };
        self.game.game_loop(&mut ctx, event);
    }

    
}

pub trait GameLoop {
    fn game_loop(
        &mut self,
        ctx: &mut EngineContext,
        event: WindowEvent
    ) {

    }
}

