use crate::{game_event_handle::FRAME_DURATION, Game};
use std::{cell::RefCell, rc::Rc, time};

struct StandardTimeWorld {
    time_step: f32,
    frame_counter: FrameCounter,
}
impl StandardTimeWorld {
    fn new(time_step: f32) -> Self {
        Self {
            time_step,
            frame_counter: FrameCounter::new(),
        }
    }
}

pub struct FrameCounter {
    pub last_printed_instant: time::Instant,
    pub frame_count: u32,
    pub fps: Rc<RefCell<f32>>,
    pub frame_time: f32,
}
impl FrameCounter {
    pub fn new() -> Self {
        Self {
            last_printed_instant: time::Instant::now(),
            frame_count: 0,
            fps: Rc::new(RefCell::new(0.)),
            frame_time: 0.,
        }
    }

    pub fn update(&mut self, game: &Game) {
        println!("FrameCounter::update");
        self.frame_count += 1;
        let new_instant = game.last_update;
        let elapsed_secs = (new_instant - self.last_printed_instant).as_secs_f32();
        if elapsed_secs > 1.0 {
            let elapsed_ms = elapsed_secs * 1000.0;
            let frame_time = elapsed_ms / self.frame_count as f32;
            let fps = self.frame_count as f32 / elapsed_secs;
            *self.fps.borrow_mut() = fps;
            self.frame_time = frame_time;

            self.last_printed_instant = new_instant;
            self.frame_count = 0;
        }
    }
}
