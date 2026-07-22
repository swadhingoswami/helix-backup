use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

pub struct ProgressTracker {
    total: u64,
    completed: Arc<AtomicU64>,
    start_time: Instant,
    label: String,
}

impl ProgressTracker {
    pub fn new(total: u64, label: &str) -> Self {
        Self {
            total,
            completed: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
            label: label.to_string(),
        }
    }

    pub fn increment(&self, amount: u64) {
        self.completed.fetch_add(amount, Ordering::SeqCst);
    }

    pub fn get_completed(&self) -> u64 {
        self.completed.load(Ordering::SeqCst)
    }

    pub fn get_total(&self) -> u64 {
        self.total
    }

    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            return 100.0;
        }
        (self.get_completed() as f64 / self.total as f64) * 100.0
    }

    pub fn elapsed_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    pub fn rate_per_second(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed < 0.001 {
            return 0.0;
        }
        self.get_completed() as f64 / elapsed
    }

    pub fn eta_seconds(&self) -> u64 {
        let rate = self.rate_per_second();
        if rate < 0.001 {
            return 0;
        }
        let remaining = self.total - self.get_completed();
        (remaining as f64 / rate) as u64
    }

    pub fn summary(&self) -> String {
        format!(
            "{}: {}/{} blocks ({:.1}%) - {:.0} blocks/s - ETA: {}s",
            self.label,
            self.get_completed(),
            self.total,
            self.percentage(),
            self.rate_per_second(),
            self.eta_seconds()
        )
    }

    pub fn shared_completed(&self) -> Arc<AtomicU64> {
        self.completed.clone()
    }
}

pub struct ProgressBar {
    tracker: ProgressTracker,
    width: usize,
}

impl ProgressBar {
    pub fn new(total: u64, label: &str) -> Self {
        Self {
            tracker: ProgressTracker::new(total, label),
            width: 40,
        }
    }

    pub fn render(&self) -> String {
        let pct = self.tracker.percentage();
        let filled = (pct / 100.0 * self.width as f64) as usize;
        let empty = self.width - filled;

        let bar: String = format!(
            "\r[{}>{}] {:.1}% | {}",
            "#".repeat(filled),
            " ".repeat(empty),
            pct,
            self.tracker.summary()
        );

        if pct >= 100.0 {
            format!("{}\n", bar)
        } else {
            bar
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_tracker() {
        let tracker = ProgressTracker::new(100, "test");
        assert_eq!(tracker.percentage(), 0.0);

        tracker.increment(50);
        assert_eq!(tracker.percentage(), 50.0);

        tracker.increment(50);
        assert_eq!(tracker.percentage(), 100.0);
    }

    #[test]
    fn test_progress_eta() {
        let tracker = ProgressTracker::new(1000, "test");
        tracker.increment(100);
        // ETA should be non-zero since we started
        assert!(tracker.eta_seconds() > 0 || tracker.rate_per_second() == 0.0);
    }
}
