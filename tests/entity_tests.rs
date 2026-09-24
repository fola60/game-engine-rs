use game_engine_rs::{
    Circle, Color, Cube, Entity, EntityContext, Mesh, Rectangle, RenderData, ThreeD, TwoD,
    model::ModelVertex, scene::Scene,
};
use std::{cell::RefCell, rc::Rc};
use winit::event::WindowEvent;

struct Probe {
    log: Rc<RefCell<Vec<String>>>,
    elapsed: f32,
    spawn_child: bool,
}

impl Probe {
    fn new(log: &Rc<RefCell<Vec<String>>>) -> Self {
        Self {
            log: log.clone(),
            elapsed: 0.0,
            spawn_child: false,
        }
    }
    fn record(&self, ctx: &EntityContext<TwoD>, hook: &str) {
        self.log.borrow_mut().push(format!("{}:{hook}", ctx.id()));
    }
}

impl Entity<TwoD> for Probe {
    fn start(&mut self, ctx: &mut EntityContext<TwoD>) {
        self.record(ctx, "start");
    }
    fn ready(&mut self, ctx: &mut EntityContext<TwoD>) {
        self.record(ctx, "ready");
    }
    fn event(&mut self, ctx: &mut EntityContext<TwoD>, _: &WindowEvent) {
        self.record(ctx, "event");
    }
    fn update(&mut self, ctx: &mut EntityContext<TwoD>, dt: f32) {
        self.record(ctx, "update");
        self.elapsed += dt;
        if self.spawn_child {
            ctx.spawn("child", Probe::new(&self.log));
            ctx.despawn_self();
        }
    }
    fn render_data(&self) -> Option<RenderData<TwoD>> {
        None
    }
}

#[test]
fn lifecycle_initializes_once_before_updates_and_events() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut scene = Scene::<TwoD>::default();
    scene.spawn("b", Probe::new(&log));
    scene.spawn("a", Probe::new(&log));
    scene.event(&WindowEvent::RedrawRequested);
    assert!(log.borrow().is_empty());
    scene.update(0.25);
    scene.event(&WindowEvent::RedrawRequested);
    scene.update(0.5);
    assert_eq!(
        *log.borrow(),
        [
            "a:start", "b:start", "a:ready", "b:ready", "a:update", "b:update", "a:event",
            "b:event", "a:update", "b:update",
        ]
    );
    assert_eq!(scene.get::<Probe>("a").unwrap().elapsed, 0.75);
}

#[test]
fn commands_from_callbacks_wait_until_next_frame() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut scene = Scene::<TwoD>::default();
    let mut parent = Probe::new(&log);
    parent.spawn_child = true;
    scene.spawn("parent", parent);
    scene.update(0.1);
    assert!(scene.contains("parent"));
    assert!(!scene.contains("child"));
    scene.update(0.1);
    assert!(!scene.contains("parent"));
    assert!(scene.contains("child"));
    assert_eq!(
        *log.borrow(),
        [
            "parent:start",
            "parent:ready",
            "parent:update",
            "child:start",
            "child:ready",
            "child:update",
        ]
    );
}

#[test]
fn duplicate_ids_preserve_existing_objects_and_despawn_allows_reuse() {
    let mut scene = Scene::<TwoD>::default();
    assert!(scene.spawn(String::from("shape"), Circle::new(1.0)));
    assert!(!scene.spawn("shape", Rectangle::new(2.0, 2.0)));
    assert!(scene.get::<Circle>("shape").is_some());
    assert!(scene.get::<Rectangle>("shape").is_none());
    assert!(scene.get_mut::<Circle>("missing").is_none());
    assert!(scene.despawn("shape"));
    assert!(!scene.despawn("shape"));
    assert!(scene.spawn("shape", Rectangle::new(2.0, 2.0)));
}

#[test]
fn shapes_return_current_appearance_and_support_visibility() {
    let mut scene = Scene::<ThreeD>::default();
    scene.spawn("cube", Cube::new(2.0, 4.0, 6.0));
    let cube = scene.get_mut::<Cube>("cube").unwrap();
    cube.color = Color::Blue;
    cube.transform.position.x = 5.0;
    let data = cube.render_data().unwrap();
    assert_eq!(data.color, Color::Blue);
    assert_eq!(data.transform.position.x, 5.0);
    assert_eq!(data.mesh.vertices().len(), 24);
    assert_eq!(data.mesh.indices().len(), 36);
    assert_eq!(data.mesh.vertices()[0].position, [-1.0, -2.0, 3.0]);
    assert_eq!(
        data.mesh.vertices().as_ptr(),
        cube.render_data().unwrap().mesh.vertices().as_ptr()
    );
    cube.visible = false;
    assert!(cube.render_data().is_none());
}

#[test]
fn custom_mesh_rejects_invalid_geometry() {
    let vertex = ModelVertex {
        position: [0.0; 3],
        tex_coords: [0.0; 2],
        normal: [0.0; 3],
    };
    assert!(Mesh::<TwoD>::new(vec![], vec![0, 0, 0]).is_err());
    assert!(Mesh::<TwoD>::new(vec![vertex], vec![0, 0]).is_err());
    assert!(Mesh::<TwoD>::new(vec![vertex], vec![0, 1, 0]).is_err());
    let mut invalid = vertex;
    invalid.position[0] = f32::NAN;
    assert!(Mesh::<TwoD>::new(vec![invalid], vec![0, 0, 0]).is_err());
    assert!(Mesh::<TwoD>::new(vec![vertex], vec![0, 0, 0]).is_ok());
}
