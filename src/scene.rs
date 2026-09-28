//! A dimension-checked collection of engine-owned game objects.
use crate::collision::Bounds;
use crate::{
    CollisionEvent, CollisionPhase, Dimension, Entity, EntityContext, SceneCollisionEvent,
};
use std::collections::{BTreeMap, BTreeSet, btree_map::Entry};
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

    #[test]
    fn collision_snapshot_reports_each_pair_once_and_preserves_entity_callbacks() {
        struct Probe {
            shape: Circle,
            events: Vec<CollisionEvent>,
        }
        impl Entity<TwoD> for Probe {
            fn render_data(&self) -> Option<crate::RenderData<TwoD>> {
                self.shape.render_data()
            }
            fn on_collision(&mut self, _: &mut EntityContext<TwoD>, event: &CollisionEvent) {
                self.events.push(event.clone());
            }
        }
        let mut scene = Scene::<TwoD>::default();
        for id in ["a", "b"] {
            scene.spawn(
                id,
                Probe {
                    shape: Circle::new(1.0).at(2.0, 3.0).with_color(crate::Color::Blue),
                    events: Vec::new(),
                },
            );
        }
        for phase in [
            CollisionPhase::Started,
            CollisionPhase::Stayed,
            CollisionPhase::Ended,
        ] {
            if phase == CollisionPhase::Ended {
                scene
                    .get_mut::<Probe>("b")
                    .unwrap()
                    .shape
                    .transform
                    .position
                    .x = 20.0;
            }
            scene.update(0.0);
            let events = scene.take_collision_events();
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].phase, phase);
            assert!(events[0].involves("a", "b") && events[0].involves("b", "a"));
            assert!(!events[0].involves("a", "missing"));
            assert_eq!(events[0].other_id("a"), Some("b"));
            assert_eq!(events[0].other_id("b"), Some("a"));
            assert_eq!(events[0].other_id("missing"), None);
            assert!(scene.take_collision_events().is_empty());
            for (id, other) in [("a", "b"), ("b", "a")] {
                let probe = scene.get::<Probe>(id).unwrap();
                assert_eq!(
                    probe.events.last().unwrap(),
                    &CollisionEvent {
                        other_id: other.into(),
                        phase
                    }
                );
            }
        }
        scene.update(0.0);
        assert!(scene.take_collision_events().is_empty());
        assert_eq!(scene.get::<Probe>("a").unwrap().events.len(), 3);
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
    contacts: BTreeSet<(String, String)>,
    collision_events: Vec<SceneCollisionEvent>,
}

impl<D: Dimension> Default for Scene<D> {
    fn default() -> Self {
        Self {
            entities: BTreeMap::new(),
            commands: Vec::new(),
            removed: Vec::new(),
            contacts: BTreeSet::new(),
            collision_events: Vec::new(),
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

    /// Removes the entity and clears its contacts
    pub fn despawn(&mut self, id: &str) -> bool {
        if self.entities.remove(id).is_some() {
            self.contacts.retain(|(a, b)| a != id && b != id);
            self.removed.push(id.to_owned());
            true
        } else {
            false
        }
    }

    /// Apply queued commands, initialize and update entities, then detect collisions.
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
        self.detect_collisions();
    }

    fn detect_collisions(&mut self) {
        let bounds: Vec<_> = self
            .entities
            .iter()
            .filter_map(|(id, entry)| {
                let data = entry.entity.render_data()?;
                Bounds::from_render_data(&data).map(|bounds| (id.clone(), bounds))
            })
            .collect();
        let mut contacts = BTreeSet::new();
        let mut events = Vec::new();
        for i in 0..bounds.len() {
            for j in (i + 1)..bounds.len() {
                let (a, a_bounds) = &bounds[i];
                let (b, b_bounds) = &bounds[j];
                if a_bounds.overlaps(b_bounds) {
                    let pair = (a.clone(), b.clone());
                    let phase = if self.contacts.contains(&pair) {
                        CollisionPhase::Stayed
                    } else {
                        CollisionPhase::Started
                    };
                    events.push((pair.clone(), phase));
                    contacts.insert(pair);
                }
            }
        }
        for pair in self.contacts.difference(&contacts) {
            events.push((pair.clone(), CollisionPhase::Ended));
        }
        self.contacts = contacts;
        self.collision_events = events
            .iter()
            .map(|((a, b), phase)| SceneCollisionEvent {
                a_id: a.clone(),
                b_id: b.clone(),
                phase: *phase,
            })
            .collect();
        for ((a, b), phase) in events {
            for (id, other_id) in [(&a, &b), (&b, &a)] {
                if let Some(entry) = self.entities.get_mut(id) {
                    entry.entity.on_collision(
                        &mut EntityContext {
                            id,
                            commands: &mut self.commands,
                        },
                        &CollisionEvent {
                            other_id: other_id.clone(),
                            phase,
                        },
                    );
                }
            }
        }
    }

    /// Takes the latest update's collision snapshot, once per pair.
    pub fn take_collision_events(&mut self) -> Vec<SceneCollisionEvent> {
        std::mem::take(&mut self.collision_events)
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
