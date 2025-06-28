use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PomodoroState {
    Work,
    ShortBreak,
    LongBreak,
    Idle,
}

#[derive(Debug, Clone)]
pub struct PomodoroTimer {
    pub state: PomodoroState,
    pub work_duration: Duration,
    pub short_break_duration: Duration,
    pub long_break_duration: Duration,
    pub cycles: u32,
    pub cycles_before_long_break: u32,
    pub last_transition: Instant,
    pub elapsed: Duration,
}

impl PomodoroTimer {
    pub fn new(
        work: Duration,
        short_break: Duration,
        long_break: Duration,
        cycles_before_long_break: u32,
    ) -> Self {
        Self {
            state: PomodoroState::Idle,
            work_duration: work,
            short_break_duration: short_break,
            long_break_duration: long_break,
            cycles: 0,
            cycles_before_long_break,
            last_transition: Instant::now(),
            elapsed: Duration::ZERO,
        }
    }

    pub fn start(&mut self) {
        self.state = PomodoroState::Work;
        self.last_transition = Instant::now();
        self.elapsed = Duration::ZERO;
    }

    pub fn reset(&mut self) {
        self.state = PomodoroState::Idle;
        self.cycles = 0;
        self.elapsed = Duration::ZERO;
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let delta = now.duration_since(self.last_transition);
        self.elapsed += delta;
        self.last_transition = now;
        match self.state {
            PomodoroState::Work => {
                if self.elapsed >= self.work_duration {
                    self.cycles += 1;
                    self.elapsed = Duration::ZERO;
                    if self.cycles % self.cycles_before_long_break == 0 {
                        self.state = PomodoroState::LongBreak;
                    } else {
                        self.state = PomodoroState::ShortBreak;
                    }
                }
            }
            PomodoroState::ShortBreak => {
                if self.elapsed >= self.short_break_duration {
                    self.state = PomodoroState::Work;
                    self.elapsed = Duration::ZERO;
                }
            }
            PomodoroState::LongBreak => {
                if self.elapsed >= self.long_break_duration {
                    self.state = PomodoroState::Work;
                    self.elapsed = Duration::ZERO;
                }
            }
            PomodoroState::Idle => {}
        }
    }

    pub fn is_running(&self) -> bool {
        self.state != PomodoroState::Idle
    }

    pub fn time_left(&self) -> Duration {
        match self.state {
            PomodoroState::Work => self.work_duration.saturating_sub(self.elapsed),
            PomodoroState::ShortBreak => self.short_break_duration.saturating_sub(self.elapsed),
            PomodoroState::LongBreak => self.long_break_duration.saturating_sub(self.elapsed),
            PomodoroState::Idle => Duration::ZERO,
        }
    }
}

mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    fn make_timer() -> PomodoroTimer {
        PomodoroTimer::new(
            Duration::from_millis(100), // work
            Duration::from_millis(50),  // short break
            Duration::from_millis(200), // long break
            4,                          // cycles before long break
        )
    }

    #[test]
    fn test_initial_state() {
        let timer = make_timer();
        assert_eq!(timer.state, PomodoroState::Idle);
        assert_eq!(timer.cycles, 0);
        assert_eq!(timer.elapsed, Duration::ZERO);
        assert_eq!(timer.time_left(), Duration::ZERO);
        assert!(!timer.is_running());
    }

    #[test]
    fn test_start_and_work_transition() {
        let mut timer = make_timer();
        timer.start();
        assert_eq!(timer.state, PomodoroState::Work);
        assert!(timer.is_running());
        assert_eq!(timer.cycles, 0);
        // Simulate work period elapsed
        timer.elapsed = timer.work_duration;
        timer.update();
        assert_eq!(timer.state, PomodoroState::ShortBreak);
        assert_eq!(timer.cycles, 1);
        assert_eq!(timer.elapsed, Duration::ZERO);
    }

    #[test]
    fn test_short_break_to_work() {
        let mut timer = make_timer();
        timer.state = PomodoroState::ShortBreak;
        timer.elapsed = timer.short_break_duration;
        timer.update();
        assert_eq!(timer.state, PomodoroState::Work);
        assert_eq!(timer.elapsed, Duration::ZERO);
    }

    #[test]
    fn test_long_break_to_work() {
        let mut timer = make_timer();
        timer.state = PomodoroState::LongBreak;
        timer.elapsed = timer.long_break_duration;
        timer.update();
        assert_eq!(timer.state, PomodoroState::Work);
        assert_eq!(timer.elapsed, Duration::ZERO);
    }

    #[test]
    fn test_cycles_and_long_break() {
        let mut timer = make_timer();
        timer.start();
        for i in 1..=4 {
            timer.elapsed = timer.work_duration;
            timer.update();
            if i < 4 {
                assert_eq!(timer.state, PomodoroState::ShortBreak);
                timer.elapsed = timer.short_break_duration;
                timer.update();
                assert_eq!(timer.state, PomodoroState::Work);
            } else {
                assert_eq!(timer.state, PomodoroState::LongBreak);
                timer.elapsed = timer.long_break_duration;
                timer.update();
                assert_eq!(timer.state, PomodoroState::Work);
            }
        }
        assert_eq!(timer.cycles, 4);
    }

    #[test]
    fn test_reset() {
        let mut timer = make_timer();
        timer.start();
        timer.elapsed = timer.work_duration;
        timer.update();
        timer.reset();
        assert_eq!(timer.state, PomodoroState::Idle);
        assert_eq!(timer.cycles, 0);
        assert_eq!(timer.elapsed, Duration::ZERO);
        assert!(!timer.is_running());
    }

    #[test]
    fn test_time_left() {
        let mut timer = make_timer();
        timer.start();
        timer.elapsed = Duration::from_millis(30);
        assert_eq!(timer.time_left(), Duration::from_millis(70));
        timer.state = PomodoroState::ShortBreak;
        timer.elapsed = Duration::from_millis(10);
        assert_eq!(timer.time_left(), Duration::from_millis(40));
        timer.state = PomodoroState::LongBreak;
        timer.elapsed = Duration::from_millis(100);
        assert_eq!(timer.time_left(), Duration::from_millis(100));
        timer.state = PomodoroState::Idle;
        assert_eq!(timer.time_left(), Duration::ZERO);
    }
}
