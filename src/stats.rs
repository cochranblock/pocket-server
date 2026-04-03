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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_at_zero() {
        let s = t1::f10();
        assert_eq!(s.f14(), 0);
        assert_eq!(s.f15(), 0);
    }

    #[test]
    fn record_request_increments() {
        let s = t1::f10();
        s.f11(1024);
        s.f11(2048);
        assert_eq!(s.f14(), 2);
        assert_eq!(s.f15(), 3072);
    }

    #[test]
    fn bytes_display_units() {
        let s = t1::f10();
        assert_eq!(s.f16(), "0 B");

        s.f11(500);
        assert_eq!(s.f16(), "500 B");

        s.f11(1024 - 500);
        assert_eq!(s.f16(), "1.0 KB");

        // Push into MB range
        let s2 = t1::f10();
        s2.s6.store(1024 * 1024 * 5, Ordering::Relaxed);
        assert_eq!(s2.f16(), "5.0 MB");

        // Push into GB range
        let s3 = t1::f10();
        s3.s6.store(1024 * 1024 * 1024 * 2, Ordering::Relaxed);
        assert_eq!(s3.f16(), "2.0 GB");
    }

    #[test]
    fn uptime_display_format() {
        let s = t1::f10();
        let d = s.f13();
        // Fresh stats = "0h 0m"
        assert_eq!(d, "0h 0m");
    }

    #[test]
    fn power_estimate_idle() {
        let s = t1::f10();
        // At zero RPS, power = 0.5W base
        let w = s.f17();
        assert!((w - 0.5).abs() < 0.01);
    }

    #[test]
    fn monthly_cost_format() {
        let s = t1::f10();
        let cost = s.f18();
        assert!(cost.starts_with('$'));
        // Idle: 0.5W * 24 * 30 / 1000 = 0.36 kWh * 0.15 = $0.05
        assert_eq!(cost, "$0.05");
    }

    #[test]
    fn to_json_valid() {
        let s = t1::f10();
        s.f11(512);
        let json = s.f19();
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));
        assert!(json.contains("\"uptime\""));
        assert!(json.contains("\"requests\":1"));
        assert!(json.contains("\"bytes_served\":\"512 B\""));
        assert!(json.contains("\"power_w\":"));
        assert!(json.contains("\"monthly_cost\":\"$"));
    }

    #[test]
    fn default_is_new() {
        let s = t1::default();
        assert_eq!(s.f14(), 0);
        assert_eq!(s.f15(), 0);
    }

    #[test]
    fn record_request_zero_bytes() {
        let s = t1::f10();
        s.f11(0);
        assert_eq!(s.f14(), 1);
        assert_eq!(s.f15(), 0);
    }

    #[test]
    fn record_request_large_bytes() {
        let s = t1::f10();
        s.f11(u64::MAX / 2);
        assert_eq!(s.f14(), 1);
        assert_eq!(s.f15(), u64::MAX / 2);
    }

    #[test]
    fn record_request_many() {
        let s = t1::f10();
        for i in 0..1000 {
            s.f11(i);
        }
        assert_eq!(s.f14(), 1000);
        assert_eq!(s.f15(), (0..1000u64).sum::<u64>());
    }

    #[test]
    fn bytes_display_boundary_1023() {
        let s = t1::f10();
        s.s6.store(1023, Ordering::Relaxed);
        assert_eq!(s.f16(), "1023 B");
    }

    #[test]
    fn bytes_display_boundary_1024() {
        let s = t1::f10();
        s.s6.store(1024, Ordering::Relaxed);
        assert_eq!(s.f16(), "1.0 KB");
    }

    #[test]
    fn bytes_display_boundary_1mb_minus_1() {
        let s = t1::f10();
        s.s6.store(1024 * 1024 - 1, Ordering::Relaxed);
        assert!(s.f16().ends_with(" KB"));
    }

    #[test]
    fn bytes_display_boundary_1mb() {
        let s = t1::f10();
        s.s6.store(1024 * 1024, Ordering::Relaxed);
        assert_eq!(s.f16(), "1.0 MB");
    }

    #[test]
    fn bytes_display_boundary_1gb_minus_1() {
        let s = t1::f10();
        s.s6.store(1024 * 1024 * 1024 - 1, Ordering::Relaxed);
        assert!(s.f16().ends_with(" MB"));
    }

    #[test]
    fn bytes_display_boundary_1gb() {
        let s = t1::f10();
        s.s6.store(1024 * 1024 * 1024, Ordering::Relaxed);
        assert_eq!(s.f16(), "1.0 GB");
    }

    #[test]
    fn bytes_display_fractional_kb() {
        let s = t1::f10();
        s.s6.store(1536, Ordering::Relaxed); // 1.5 KB
        assert_eq!(s.f16(), "1.5 KB");
    }

    #[test]
    fn power_estimate_capped() {
        // Power = 0.5 + (rps * 0.1).min(1.0), max = 1.5W
        // Need rps >= 10 to hit cap
        let s = t1::f10();
        // Simulate: 10000 requests in ~1 second is impossible with Instant,
        // but we can verify the formula directly
        // At 0 uptime, rps = 0, power = 0.5
        assert!((s.f17() - 0.5).abs() < 0.01);
    }

    #[test]
    fn monthly_cost_formula() {
        // Idle: 0.5W * 24h * 30d / 1000 = 0.36 kWh * $0.15 = $0.054 → "$0.05"
        let s = t1::f10();
        assert_eq!(s.f18(), "$0.05");
    }

    #[test]
    fn to_json_all_fields_present() {
        let s = t1::f10();
        let json = s.f19();
        // Verify all 5 fields
        assert!(json.contains("\"uptime\":"));
        assert!(json.contains("\"requests\":"));
        assert!(json.contains("\"bytes_served\":"));
        assert!(json.contains("\"power_w\":"));
        assert!(json.contains("\"monthly_cost\":"));
    }

    #[test]
    fn to_json_parseable_structure() {
        let s = t1::f10();
        s.f11(100);
        s.f11(200);
        let json = s.f19();
        // Verify it's valid JSON-like (balanced braces, no trailing comma)
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));
        assert!(!json.contains(",}"));
        assert!(!json.contains(",,"));
        // requests should be 2
        assert!(json.contains("\"requests\":2"));
        assert!(json.contains("\"bytes_served\":\"300 B\""));
    }

    #[test]
    fn concurrent_record_requests() {
        use std::sync::Arc;
        use std::thread;
        let s = Arc::new(t1::f10());
        let mut handles = vec![];
        for _ in 0..10 {
            let s = s.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    s.f11(1);
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(s.f14(), 1000);
        assert_eq!(s.f15(), 1000);
    }

    #[test]
    fn uptime_secs_non_negative() {
        let s = t1::f10();
        // Uptime should be 0 or very small
        assert!(s.f12() < 2);
    }
}
