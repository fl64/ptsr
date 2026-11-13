//! Output meters: what the bars and the overload light show, driven by the engine's
//! per-block output peaks. Pure state, no audio thread involved.

use std::time::{Duration, Instant};

/// Display floor: −60 dB reads as silence.
pub const FLOOR_DB: f32 = -60.0;
/// How fast a bar falls when the sound gets quieter, dB per second.
const FALL_DB: f32 = 40.0;
/// How long the peak mark stays at the top before following the bar down.
const HOLD: Duration = Duration::from_millis(1500);
/// A hit that stays silent this long counts as a clean hit for the overload light.
const PROBE_TIMEOUT: Duration = Duration::from_millis(100);

#[derive(Debug)]
pub struct Meters {
    /// Bar level per channel, dB.
    level: [f32; 2],
    /// Peak-hold level per channel, dB.
    peak: [f32; 2],
    peak_at: [Instant; 2],
    /// The UI clock starts when the application starts.
    start: Instant,
    /// Latest clipped output observed since the last clean hit.
    clip_at: Option<Instant>,
    /// A pad hit happened; the first read with sound (or after the timeout) decides the light.
    probing: Option<Instant>,
    last: Instant,
}

impl Default for Meters {
    fn default() -> Self {
        Self::new(Instant::now())
    }
}

fn to_db(p: f32) -> f32 {
    if p > 0.0 {
        (20.0 * p.log10()).clamp(FLOOR_DB, 0.0)
    } else {
        FLOOR_DB
    }
}

impl Meters {
    pub fn new(now: Instant) -> Self {
        Self {
            level: [FLOOR_DB; 2],
            peak: [FLOOR_DB; 2],
            peak_at: [now; 2],
            start: now,
            clip_at: None,
            probing: None,
            last: now,
        }
    }

    /// Bar levels per channel, dB.
    pub fn level_db(&self) -> [f32; 2] {
        self.level
    }

    /// Peak-hold levels per channel, dB.
    pub fn peak_db(&self) -> [f32; 2] {
        self.peak
    }

    /// Time of the latest clipped output since this application started.
    pub fn clip_elapsed(&self) -> Option<Duration> {
        self.clip_at
            .map(|at| at.saturating_duration_since(self.start))
    }

    /// A pad was hit: the next blocks it produces decide the overload light.
    pub fn hit(&mut self, now: Instant) {
        self.probing = Some(now);
    }

    /// Feeds the output peaks of the blocks since the last call.
    pub fn update(&mut self, out: (f32, f32), now: Instant) {
        let dt = now.saturating_duration_since(self.last).as_secs_f32();
        self.last = now;

        let clipped = out.0 >= 1.0 || out.1 >= 1.0;
        let sounding = out.0 > 0.0 || out.1 > 0.0;
        if clipped {
            self.clip_at = Some(now);
            self.probing = None;
        } else if let Some(at) = self.probing
            && (sounding || now.duration_since(at) >= PROBE_TIMEOUT)
        {
            // the hit's blocks arrived (or it stayed silent): a clean hit clears the light
            self.clip_at = None;
            self.probing = None;
        }

        for (ch, p) in [out.0, out.1].into_iter().enumerate() {
            let heard = to_db(p);
            let falling = self.level[ch] - FALL_DB * dt;
            self.level[ch] = heard.max(falling).clamp(FLOOR_DB, 0.0);
            if self.level[ch] >= self.peak[ch] {
                self.peak[ch] = self.level[ch];
                self.peak_at[ch] = now;
            } else if now.duration_since(self.peak_at[ch]) >= HOLD {
                // after the hold the mark follows the bar down
                self.peak[ch] = self.peak[ch].min(falling).max(self.level[ch]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(plus_ms: u64) -> Instant {
        Instant::now()
            .checked_sub(Duration::from_millis(10_000))
            .unwrap()
            .checked_add(Duration::from_millis(plus_ms))
            .unwrap()
    }

    #[test]
    fn meters_follow_the_sound() {
        let mut m = Meters::new(t(0));
        m.update((0.5, 0.25), t(0));
        assert!((m.level_db()[0] + 6.02).abs() < 0.01, "L at −6 dB");
        assert!((m.level_db()[1] + 12.04).abs() < 0.01, "R at −12 dB");
        // a quieter sound does not jump down: the bar falls at 40 dB/s
        m.update((0.0, 0.0), t(500));
        assert!((m.level_db()[0] + 6.02 + 20.0).abs() < 1.0, "L fell ~20 dB");
        assert!(
            (m.level_db()[1] + 12.04 + 20.0).abs() < 1.0,
            "R fell ~20 dB"
        );
        // the peak mark held at the top during the hold time
        assert!(
            (m.peak_db()[0] + 6.02).abs() < 0.01,
            "the L peak stays 1.5 s"
        );
        // and follows the bar down afterwards
        m.update((0.0, 0.0), t(2500));
        assert!(m.peak_db()[0] < -6.0, "the peak mark follows the bar down");
    }

    #[test]
    fn overload_latches_until_a_clean_hit() {
        let mut m = Meters::new(t(0));
        m.update((1.0, 0.5), t(0));
        assert!(m.clip_elapsed().is_some(), "full scale latches the light");
        m.update((0.1, 0.1), t(50));
        assert!(
            m.clip_elapsed().is_some(),
            "silence alone does not clear it"
        );
        m.hit(t(60));
        m.update((0.0, 0.0), t(70));
        assert!(
            m.clip_elapsed().is_some(),
            "the probe waits for the hit's sound"
        );
        m.update((0.3, 0.3), t(100));
        assert!(
            m.clip_elapsed().is_none(),
            "a hit that does not clip clears the light"
        );
        // a clipping hit keeps it on
        m.hit(t(200));
        m.update((1.0, 0.9), t(220));
        assert!(m.clip_elapsed().is_some());
    }

    #[test]
    fn an_empty_hit_clears_the_light_after_a_beat() {
        let mut m = Meters::new(t(0));
        m.update((1.2, 1.0), t(0));
        assert!(m.clip_elapsed().is_some());
        m.hit(t(10));
        // the empty pad stays silent: the probe times out and the light goes out
        m.update((0.0, 0.0), t(60));
        assert!(
            m.clip_elapsed().is_some(),
            "not yet: inside the probe timeout"
        );
        m.update((0.0, 0.0), t(200));
        assert!(
            m.clip_elapsed().is_none(),
            "a silent hit is a clean hit too"
        );
    }

    /// Scenario "Clip feedback appears only for a real clipped block".
    #[test]
    fn clip_feedback_appears_only_for_a_real_clipped_block() {
        let start = Instant::now();
        let mut meter = Meters::new(start);
        meter.update((0.0, 0.0), start + Duration::from_secs(1));
        meter.hit(start + Duration::from_secs(2));
        meter.update((0.8, 0.7), start + Duration::from_secs(2));
        assert_eq!(meter.clip_elapsed(), None);
        meter.update((1.0, 0.7), start + Duration::from_secs(3));
        assert_eq!(meter.clip_elapsed(), Some(Duration::from_secs(3)));
        meter.update((1.2, 0.9), start + Duration::from_secs(3_723));
        assert_eq!(meter.clip_elapsed(), Some(Duration::from_secs(3_723)));
    }

    /// Scenario "A clean hit hides clip feedback".
    #[test]
    fn a_clean_hit_hides_clip_feedback() {
        let start = Instant::now();
        let mut meter = Meters::new(start);
        meter.update((1.0, 0.2), start + Duration::from_secs(4));
        meter.update((0.0, 0.0), start + Duration::from_secs(5));
        assert_eq!(meter.clip_elapsed(), Some(Duration::from_secs(4)));
        meter.hit(start + Duration::from_secs(6));
        meter.update((0.3, 0.2), start + Duration::from_secs(6));
        assert_eq!(meter.clip_elapsed(), None);
        let mut restarted = Meters::new(start + Duration::from_secs(10));
        assert_eq!(restarted.clip_elapsed(), None);
        restarted.update((1.0, 0.1), start + Duration::from_secs(11));
        assert_eq!(restarted.clip_elapsed(), Some(Duration::from_secs(1)));
    }
}
