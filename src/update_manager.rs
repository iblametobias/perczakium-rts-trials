use std::{fmt::Display, time::{Duration, Instant}};

use macroquad::color::colors;

pub const DEFAULT_TPS: f32 = 20.0;

pub struct UpdateManager {
    profiler: Profiler,

    last_update: Instant,
    accumulator: Duration,
    tick_time:   Duration,
}

impl UpdateManager {
    pub fn new(tps: f32) -> Self {
        Self {
            profiler: Profiler::new(),
            last_update: Instant::now(),
            accumulator: Duration::ZERO,
            tick_time: Self::tick_time_from_tps(tps),
        }
    }

    pub fn update(&mut self, mut f: impl FnMut(Duration)) {
        self.accumulator += self.last_update.elapsed();
        self.last_update = Instant::now();

        let n = (self.accumulator.as_nanos() / self.tick_time.as_nanos())
            .min(3) as u32;

        for _ in 0..n {
            self.profiler.measure_tick(|| {
                f(self.tick_time);
            });
        }

        self.accumulator -= self.tick_time * n;
    }

    pub fn draw_profiler(&mut self) {
        self.profiler.draw();
    }

    pub fn take_profiler(&mut self) -> Profiler {
        std::mem::replace(&mut self.profiler, Profiler::new())
    }

    pub fn reset_accumulator(&mut self) {
        self.last_update = Instant::now();
        self.accumulator = Duration::ZERO;
    }

    pub fn get_alpha(&self) -> f32 {
        self.accumulator.as_secs_f32() / self.tick_time.as_secs_f32()
    }

    fn tick_time_from_tps(tps: f32) -> Duration{
        Duration::from_secs_f32(tps.recip())
    }
}

impl Default for UpdateManager {
    fn default() -> Self {
        Self::new(DEFAULT_TPS)
    }
}

pub struct Profiler {
    tick_times: Vec<Duration>,
    text_update: Instant,
    text_value: f32,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            tick_times: Vec::new(),
            text_update: Instant::now(),
            text_value: 0.0
        }
    }

    pub fn draw(&mut self) {
        if self.text_update.elapsed().as_millis() > 200 {
            self.text_update = Instant::now();
            self.text_value = self.tick_times.last()
                .map(|d| d.as_secs_f32() * 1000.0).unwrap_or_default();
        }

        macroquad::text::draw_text(
            format!("{} ms", self.text_value),
            12.0, 22.0, 20.0, colors::RED
        );
    }

    fn measure_tick(&mut self, mut tick: impl FnMut()) {
        let t = Instant::now();
        tick();
        self.tick_times.push(t.elapsed());
    }

    pub fn get_stats(self) -> PerformanceStats {
        let mut stats = PerformanceStats::default();
        let mut t = self.tick_times;
        if t.is_empty() { return stats; }

        t.sort_unstable();

        let n = t.len();
        stats.avg = t.iter().sum::<Duration>() / n as u32;
        stats.med = t[n / 2];
        stats.p99 = t[n * 99 / 100];

        stats
    }
}

#[derive(Default)]
pub struct PerformanceStats {
    pub avg: Duration,
    pub med: Duration,
    pub p99: Duration,
}

impl Display for PerformanceStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Average: {:?}, Median: {:?}, Worst 1%: {:?}",
            self.avg, self.med, self.p99
        )
    }
}
