//! User-defined scene entities and built-in shapes.
use crate::scene::Command;
use crate::{Color, Dimension, Mesh, ThreeD, Transform, TwoD};
use winit::event::WindowEvent;

/// Implement this trait to register a user-owned type with the engine.
/// Hooks run on the engine thread. Each registration receives start, then ready,
/// once before its first update. Invisible entities continue updating.
///
/// ```compile_fail
/// use game_engine_rs::{Cube, TwoD, engine_context::EngineContext};
/// fn wrong_dimension(ctx: &mut EngineContext<TwoD>) {
///     ctx.spawn("cube", Cube::new(1.0, 1.0, 1.0));
/// }
/// ```
pub trait Entity<D: Dimension>: std::any::Any {
    fn start(&mut self, _ctx: &mut EntityContext<D>) {}
    fn ready(&mut self, _ctx: &mut EntityContext<D>) {}
    fn event(&mut self, _ctx: &mut EntityContext<D>, _event: &WindowEvent) {}
    fn update(&mut self, _ctx: &mut EntityContext<D>, _dt: f32) {}

    /// Return a cheap mesh clone plus current appearance, or None to hide.
    /// The engine caches GPU geometry until the mesh identity changes.
    fn render_data(&self) -> Option<RenderData<D>>;
}

/// Rendering description independent of GPU resources.
/// Transform uses the engine's existing world-space representation in both modes.
pub struct RenderData<D: Dimension> {
    pub mesh: Mesh<D>,
    pub transform: Transform,
    pub color: Color,
}

impl<D: Dimension> RenderData<D> {
    pub fn new(mesh: Mesh<D>) -> Self {
        Self {
            mesh,
            transform: Transform::default(),
            color: Color::White,
        }
    }
}

/// Commands requested during hooks are processed in order at the next frame.
/// Spawn uses unique string IDs; a queued duplicate is ignored without replacing
/// the existing entity. Despawn followed by spawn explicitly replaces an entity.
pub struct EntityContext<'a, D: Dimension> {
    pub(crate) id: &'a str,
    pub(crate) commands: &'a mut Vec<Command<D>>,
}

impl<D: Dimension> EntityContext<'_, D> {
    pub fn id(&self) -> &str {
        self.id
    }

    pub fn spawn<E: Entity<D>>(&mut self, id: impl Into<String>, entity: E) {
        self.commands
            .push(Command::Spawn(id.into(), Box::new(entity)));
    }

    pub fn despawn(&mut self, id: impl Into<String>) {
        self.commands.push(Command::Despawn(id.into()));
    }

    pub fn despawn_self(&mut self) {
        self.despawn(self.id.to_owned());
    }
}

macro_rules! shape {
    ($name:ident, $dimension:ty, $constructor:ident, $($arg:ident),+) => {
        pub struct $name {
            mesh: Mesh<$dimension>,
            pub transform: Transform,
            pub color: Color,
            pub visible: bool,
        }

        impl $name {
            /// Shape dimensions are in meters.
            pub fn new($($arg: f32),+) -> Self {
                Self {
                    mesh: Mesh::<$dimension>::$constructor($($arg),+),
                    transform: Transform::default(),
                    color: Color::White,
                    visible: true,
                }
            }
        }

        impl Entity<$dimension> for $name {
            fn render_data(&self) -> Option<RenderData<$dimension>> {
                self.visible.then(|| RenderData {
                    mesh: self.mesh.clone(),
                    transform: self.transform,
                    color: self.color,
                })
            }
        }
    };
}

shape!(Circle, TwoD, circle, radius);
shape!(Rectangle, TwoD, rectangle, width, height);
shape!(Cube, ThreeD, cube, width, height, length);
