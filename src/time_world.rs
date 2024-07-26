use std::time;

struct TimeWorld {
    time_step: f32,
    frame_counter: FrameCounter,
}

struct FrameCounter {
    last_time_stamp: f32,
    frame_count: u32,
}

impl FrameCounter {
    fn start() -> Self {
        let now = time::Instant::now();
        Self {
            last_time_stamp: now.elapsed().as_secs_f32(),
            frame_count: 0,
        }
    }
    fn update(&mut self) {
        self.frame_count += 1;
    }
}

impl TimeWorld {
    fn new(time_step: f32) -> Self {
        Self {
            time_step,
            frame_counter: FrameCounter::start(),
        }
    }
} 