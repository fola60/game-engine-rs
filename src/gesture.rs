use std::time::Instant;

use crate::{Gesture, Point2D};

pub struct SwipeTracker {
    start_pos: Option<Point2D>,
    last_pos: Option<Point2D>,
    start_time: Option<Instant>,
    min_distance: f32,
    max_duration_secs: f32,
}

impl SwipeTracker {
    pub fn new() -> Self {
        Self {
            start_pos: None,
            last_pos: None,
            start_time: None,
            min_distance: 30.0,
            max_duration_secs: 0.35,
        }
    }

    pub fn with_thresholds(min_distance: f32, max_duration_secs: f32) -> Self {
        Self {
            start_pos: None,
            last_pos: None,
            start_time: None,
            min_distance,
            max_duration_secs,
        }
    }

    pub fn pointer_down(&mut self, pos: Point2D, now: Instant) {
        self.start_pos = Some(pos.clone());
        self.last_pos = Some(pos);
        self.start_time = Some(now);
    }

    pub fn pointer_move(&mut self, pos: Point2D) {
        self.last_pos = Some(pos);
    }

    pub fn pointer_up(&mut self, pos: Point2D, now: Instant) -> Option<Gesture> {
        self.last_pos = Some(pos);

        let start = self.start_pos.as_ref()?;
        let end = self.last_pos.as_ref()?;
        let started_at = self.start_time?;
        let duration_secs = now.duration_since(started_at).as_secs_f32();

        let gesture = Gesture::get_gesture(
            start,
            end,
            duration_secs,
            self.min_distance,
            self.max_duration_secs,
        );

        self.reset();
        gesture
    }

    pub fn reset(&mut self) {
        self.start_pos = None;
        self.last_pos = None;
        self.start_time = None;
    }
}
