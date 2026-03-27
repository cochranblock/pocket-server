// Unlicense — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.6

//! Live stats tracker — request count, bytes served, uptime, power estimate.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// t1=Stats — live request/byte/uptime tracker
#[allow(non_camel_case_types)]
pub struct t1 {
    /// s4=start
    pub s4: Instant,
    /// s5=requests
    pub s5: AtomicU64,
    /// s6=bytes_served
    pub s6: AtomicU64,
}

impl Default for t1 {
    fn default() -> Self {
        Self::f10()
    }
}

impl t1 {
    /// f10=new
    pub fn f10() -> Self {
        Self {
            s4: Instant::now(),
            s5: AtomicU64::new(0),
            s6: AtomicU64::new(0),
        }
    }

    /// f11=record_request
    pub fn f11(&self, bytes: u64) {
        self.s5.fetch_add(1, Ordering::Relaxed);
        self.s6.fetch_add(bytes, Ordering::Relaxed);
    }

    /// f12=uptime_secs
    pub fn f12(&self) -> u64 {
        self.s4.elapsed().as_secs()
    }

    /// f13=uptime_display
    pub fn f13(&self) -> String {
        let s = self.f12();
        let h = s / 3600;
        let m = (s % 3600) / 60;
        format!("{}h {}m", h, m)
    }

    /// f14=requests_total
    pub fn f14(&self) -> u64 {
        self.s5.load(Ordering::Relaxed)
    }

    /// f15=bytes_total
    pub fn f15(&self) -> u64 {
        self.s6.load(Ordering::Relaxed)
    }

    /// f16=bytes_display
    pub fn f16(&self) -> String {
        let b = self.f15();
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

    /// f17=power_estimate_w
    pub fn f17(&self) -> f64 {
        let rps = if self.f12() > 0 {
            self.f14() as f64 / self.f12() as f64
        } else {
            0.0
        };
        0.5 + (rps * 0.1).min(1.0)
    }

    /// f18=monthly_cost_display
    pub fn f18(&self) -> String {
        let watts = self.f17();
        let kwh_month = watts * 24.0 * 30.0 / 1000.0;
        let cost = kwh_month * 0.15;
        format!("${:.2}", cost)
    }

    /// f19=to_json
    pub fn f19(&self) -> String {
        format!(
            r#"{{"uptime":"{}","requests":{},"bytes_served":"{}","power_w":{:.1},"monthly_cost":"{}"}}"#,
            self.f13(),
            self.f14(),
            self.f16(),
            self.f17(),
            self.f18()
        )
    }
}
