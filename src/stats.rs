// Unlicense — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.6

//! Live stats tracker — request count, bytes served, uptime, power estimate.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

pub struct Stats {
    pub start: Instant,
    pub requests: AtomicU64,
    pub bytes_served: AtomicU64,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            requests: AtomicU64::new(0),
            bytes_served: AtomicU64::new(0),
        }
    }

    pub fn record_request(&self, bytes: u64) {
        self.requests.fetch_add(1, Ordering::Relaxed);
        self.bytes_served.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn uptime_secs(&self) -> u64 {
        self.start.elapsed().as_secs()
    }

    pub fn uptime_display(&self) -> String {
        let s = self.uptime_secs();
        let h = s / 3600;
        let m = (s % 3600) / 60;
        format!("{}h {}m", h, m)
    }

    pub fn requests_total(&self) -> u64 {
        self.requests.load(Ordering::Relaxed)
    }

    pub fn bytes_total(&self) -> u64 {
        self.bytes_served.load(Ordering::Relaxed)
    }

    pub fn bytes_display(&self) -> String {
        let b = self.bytes_total();
        if b < 1024 {
            format!("{} B", b)
        } else if b < 1024 * 1024 {
            format!("{:.1} KB", b as f64 / 1024.0)
        } else if b < 1024 * 1024 * 1024 {
            format!("{:.1} MB", b as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", b as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }

    /// Estimated power draw in watts based on request rate.
    pub fn power_estimate_w(&self) -> f64 {
        let rps = if self.uptime_secs() > 0 {
            self.requests_total() as f64 / self.uptime_secs() as f64
        } else {
            0.0
        };
        // Base: 0.5W idle, +0.1W per request/sec
        0.5 + (rps * 0.1).min(1.0)
    }

    /// Estimated monthly electricity cost at $0.15/kWh.
    pub fn monthly_cost_display(&self) -> String {
        let watts = self.power_estimate_w();
        let kwh_month = watts * 24.0 * 30.0 / 1000.0;
        let cost = kwh_month * 0.15;
        format!("${:.2}", cost)
    }

    /// JSON snapshot for the dashboard or API.
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"uptime":"{}","requests":{},"bytes_served":"{}","power_w":{:.1},"monthly_cost":"{}"}}"#,
            self.uptime_display(),
            self.requests_total(),
            self.bytes_display(),
            self.power_estimate_w(),
            self.monthly_cost_display()
        )
    }
}
