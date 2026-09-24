//! A dimension-checked collection of engine-owned game objects.
use crate::{Dimension, Entity, EntityContext};
use std::collections::{BTreeMap, btree_map::Entry};
use winit::event::WindowEvent;

pub(crate) enum Command<D: Dimension> {
    Spawn(String, Box<dyn Entity<D>>),
    Despawn(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Circle, TwoD};

    #[test]
    fn new_entities_are_not_rendered_before_initialization() {
        let mut scene = Scene::<TwoD>::default();
        scene.spawn("first", Circle::new(1.0));
        assert_eq!(scene.renderables().count(), 0);
        scene.update(0.0);
        assert_eq!(scene.renderables().count(), 1);
        scene.spawn("late", Circle::new(1.0));
        assert_eq!(scene.renderables().count(), 1);
        scene.update(0.0);
        assert_eq!(scene.renderables().count(), 2);
        scene.despawn("first");
        assert_eq!(scene.renderables().count(), 1);
        assert_eq!(scene.take_removed(), ["first"]);
        assert!(scene.take_removed().is_empty());
    }
}

struct Registered<D: Dimension> {
    entity: Box<dyn Entity<D>>,
    initialized: bool,
}

/// Entities execute in string-ID order for deterministic lifecycle ordering.
pub struct Scene<D: Dimension> {
    entities: BTreeMap<String, Registered<D>>,
    commands: Vec<Command<D>>,
    removed: Vec<String>,
}

impl<D: Dimension> Default for Scene<D> {
    fn default() -> Self {
        Self {
            entities: BTreeMap::new(),
            commands: Vec::new(),
            removed: Vec::new(),
        }
    }
}

impl<D: Dimension> Scene<D> {
    /// Returns false for an existing ID; never silently replaces an entity.
    pub fn spawn<E: Entity<D>>(&mut self, id: impl Into<String>, entity: E) -> bool {
        self.insert(id.into(), Box::new(entity))
    }

    fn insert(&mut self, id: String, entity: Box<dyn Entity<D>>) -> bool {
        if let Entry::Vacant(slot) = self.entities.entry(id) {
            slot.insert(Registered {
                entity,
                initialized: false,
            });
            true
        } else {
            false
        }
    }

    pub fn contains(&self, id: &str) -> bool {
        self.entities.contains_key(id)
    }

    pub fn get<E: Entity<D>>(&self, id: &str) -> Option<&E> {
        let entity = self.entities.get(id)?.entity.as_ref();
        (entity as &dyn std::any::Any).downcast_ref()
    }

    pub fn get_mut<E: Entity<D>>(&mut self, id: &str) -> Option<&mut E> {
        let entity = self.entities.get_mut(id)?.entity.as_mut();
        (entity as &mut dyn std::any::Any).downcast_mut()
    }

    pub fn despawn(&mut self, id: &str) -> bool {
        if self.entities.remove(id).is_some() {
            self.removed.push(id.to_owned());
            true
        } else {
            false
        }
    }

    /// Apply last frame's commands, initialize new objects, then update all objects.
    pub fn update(&mut self, dt: f32) {
        for command in std::mem::take(&mut self.commands) {
            match command {
                Command::Spawn(id, entity) => {
                    self.insert(id, entity);
                }
                Command::Despawn(id) => {
                    self.despawn(&id);
                }
            }
        }
        // All new entities start before any receives ready.
        for (id, entry) in &mut self.entities {
            if !entry.initialized {
                entry.entity.start(&mut EntityContext {
                    id,
                    commands: &mut self.commands,
                });
            }
        }
        for (id, entry) in &mut self.entities {
            if !entry.initialized {
                entry.entity.ready(&mut EntityContext {
                    id,
                    commands: &mut self.commands,
                });
                entry.initialized = true;
            }
        }
        for (id, entry) in &mut self.entities {
            entry.entity.update(
                &mut EntityContext {
                    id,
                    commands: &mut self.commands,
                },
                dt,
            );
        }
    }

    /// Input is delivered only after an entity's initialization hooks.
    pub fn event(&mut self, event: &WindowEvent) {
        for (id, entry) in &mut self.entities {
            if entry.initialized {
                entry.entity.event(
                    &mut EntityContext {
                        id,
                        commands: &mut self.commands,
                    },
                    event,
                );
            }
        }
    }

    pub(crate) fn take_removed(&mut self) -> Vec<String> {
        std::mem::take(&mut self.removed)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &dyn Entity<D>)> {
        self.entities
            .iter()
            .map(|(id, entry)| (id.as_str(), entry.entity.as_ref()))
    }

    pub(crate) fn renderables(&self) -> impl Iterator<Item = (&str, &dyn Entity<D>)> {
        self.entities
            .iter()
            .filter(|(_, entry)| entry.initialized)
            .map(|(id, entry)| (id.as_str(), entry.entity.as_ref()))
    }
}
