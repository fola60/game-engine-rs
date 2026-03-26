use std::collections::{HashMap, HashSet};
use std::sync::Arc;


use crate::camera::Camera;
use crate::renderer::{EntityType, Renderer, VertexIndicie};
use crate::{Color, Mode, Vertex};
use crate::entity::{Entity};
use crate::{
    state::{State},
    Point2D
};


use cgmath::{Point3, Vector3};
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalSize, Size};
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId, WindowAttributes};



pub struct EngineContext<'a> {
    entities: &'a mut HashMap<u32, Entity>,
    entity_ids: &'a mut HashSet<u32>,
    camera: &'a mut Camera,
    renderer: &'a mut Renderer,
    background: &'a mut Color,
    mode: &'a mut Mode,
    text: &'a mut Vec<(String, f32, f32, u8)>
}

impl<'a> EngineContext<'a> {
    pub fn clear_background(&mut self, color: Color) {
        *self.background = color;      
    }

    pub fn draw_circle(&mut self, id: u32, position: Point2D) -> bool {
        self.entity_ids.insert(id)
    }
    
    pub fn add_circle(&mut self, id: u32, location: Point2D, radius: f32) {

        let segments = 32; // increase for smoother circle

        let mut vertices = vec![];
        let mut indices = vec![];

        // center vertex
        vertices.push(Vertex {
            position: [location.x, location.y, 0.0],
            tex_coords: [0.5, 0.5],
        });

        // outer ring
        for i in 0..=segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = location.x + radius * angle.cos();
            let y = location.y + radius * angle.sin();

            vertices.push(Vertex {
                position: [x, y, 0.0],
                tex_coords: [0.0, 0.0],
            });
        }

        // indices (triangle fan)
        for i in 1..=segments {
            indices.push(i as u16);
            indices.push(0);
            indices.push((i + 1) as u16);
        }

        let data = VertexIndicie {
            vertexes: vertices,
            indicies: indices,
            entity_type: EntityType::Circle,
        };


        self.entities.insert(id, Entity::new(id, data, 1, Vector3 {x: 0.0, y: 0.0, z: 0.0}));
    }
    pub fn set_mode(&mut self, mode: Mode) {
        *self.mode = mode;
    }

    pub fn draw_entity(&mut self, id: u32, entity: Entity) {
        self.entities.insert(id, entity);
    }

    pub fn add_entity(&mut self, id: u32) -> bool {
        self.entity_ids.insert(id)
    }

    pub fn set_camera_eye(&mut self, eye: Point3<f32>) {
        self.camera.eye = eye;
    }

    pub fn set_camera_target(&mut self, target: Point3<f32>) {
        self.camera.target = target;
    }

    pub fn get_camera_eye(&self) -> Point3<f32> {
        self.camera.eye
    }

    pub fn get_camera_target(&self) -> Point3<f32> {
        self.camera.target
    }


    pub fn draw_rectangle(&mut self, id: u32, location: Point2D, width: f32, height: f32) {
        let z = -1.0;
        let top_left = Vertex { 
            position: [location.x, location.y, 0.0], 
            tex_coords: [0.0, 1.0]
        };

        let top_right = Vertex { 
            position: [location.x + width as f32, location.y, 0.0], 
            tex_coords: [1.0, 1.0]
        };

        let bottom_left = Vertex { 
            position: [location.x, location.y - height, 0.0], 
            tex_coords: [0.0, 0.0]
        };

        let bottom_right = Vertex { 
            position: [location.x +  width, location.y - height, 0.0], 
            tex_coords: [1.0, 0.0]
        };

        let entity_vertex_data = VertexIndicie { 
            vertexes: vec![top_left, top_right, bottom_left, bottom_right], indicies: vec![0, 1, 2, 2, 1, 3], entity_type: EntityType::Rectangle
        };

        
        self.entities.insert(id, Entity::new(id, entity_vertex_data, 1, Vector3 {x: 0.0, y: 0.0, z: 0.0}));
    }

    // draws text, relative to the camera position, (0.0, 0.0) is top right
    pub fn draw_text(&mut self, location: Point2D, text: &str, font_size: u8) {
        self.text.push((String::from(text), location.x, location.y, font_size));
    }
}


pub struct Engine<G: GameLoop + 'static > {
    game: G,
    initialized: bool,
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
            initialized: false,
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
    
    pub fn add_entity(&mut self, entity: Entity) {
        
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
        
        self.window = Some(window);
        self.state = Some(state);

        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };

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
            entities: &mut state.entities,
            entity_ids: &mut state.entity_ids,
            camera: &mut state.camera,
            renderer: &mut state.renderer,
            background: &mut state.background,
            mode: &mut state.mode,
            text: &mut state.text,
        };
        
        if !self.initialized {
            self.game.startup(&mut ctx);
            self.initialized = true;
        }
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

    fn startup (
        &mut self,
        ctx: &mut EngineContext,
    ) {

    }
}
