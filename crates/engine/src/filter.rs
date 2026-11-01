//! The pad filter (44): a state variable filter (topology-preserving transform) per voice.
//! Low-pass, high-pass and band come out of the same two integrators; the kind is a mix of
//! them, so a change of kind, cutoff or resonance glides and does not click. Off is a dry
//! mix that skips the filter, sample for sample the sound without it.

/// What the filter keeps.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FilterKind {
    #[default]
    Off,
    LowPass,
    HighPass,
    Band,
}

pub const CUTOFF_MIN: f32 = 20.0;
pub const CUTOFF_MAX: f32 = 20_000.0;
/// Time a sounding voice takes to follow a changed filter.
const GLIDE_SECONDS: f32 = 0.010;

/// The filter of a pad as the owner sets it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FilterSettings {
    pub kind: FilterKind,
    pub cutoff_hz: f32,
    /// 0..1: 0 is flat (Butterworth), 1 a strong hump that still never whistles alone.
    pub resonance: f32,
}

impl Default for FilterSettings {
    fn default() -> Self {
        Self {
            kind: FilterKind::Off,
            cutoff_hz: 1_000.0,
            resonance: 0.0,
        }
    }
}

/// Damping of the filter: √2 (flat) at no resonance down to 0.064 (Q about 16) at full;
/// it stays above zero, so the filter never oscillates by itself.
pub fn damping(resonance: f32) -> f32 {
    std::f32::consts::SQRT_2 - 1.35 * resonance.clamp(0.0, 1.0)
}

/// Mix of (dry, low, band, high) that makes a kind; the band is normalised to 0 dB.
fn mix(kind: FilterKind) -> [f32; 4] {
    match kind {
        FilterKind::Off => [1.0, 0.0, 0.0, 0.0],
        FilterKind::LowPass => [0.0, 1.0, 0.0, 0.0],
        FilterKind::Band => [0.0, 0.0, 1.0, 0.0],
        FilterKind::HighPass => [0.0, 0.0, 0.0, 1.0],
    }
}

/// How much of a sine at `f` Hz passes, 0..(above 1 at a resonance hump): the response
/// the curve on the screen draws.
pub fn response(s: FilterSettings, f: f32) -> f32 {
    let k = damping(s.resonance);
    let w = f / s.cutoff_hz.clamp(CUTOFF_MIN, CUTOFF_MAX);
    let den = ((1.0 - w * w).powi(2) + (k * w).powi(2)).sqrt();
    match s.kind {
        FilterKind::Off => 1.0,
        FilterKind::LowPass => 1.0 / den,
        FilterKind::HighPass => w * w / den,
        FilterKind::Band => k * w / den,
    }
}

/// What a pad's voices glide to: ready-made for the audio thread.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Target {
    /// log2 of the cutoff in Hz, kept under 0.45 of the sample rate.
    oct: f32,
    k: f32,
    mix: [f32; 4],
}

impl Target {
    pub const BYPASS: Self = Self {
        oct: 10.0,
        k: std::f32::consts::SQRT_2,
        mix: [1.0, 0.0, 0.0, 0.0],
    };

    pub fn new(s: FilterSettings, sample_rate: f32) -> Self {
        let top = (0.45 * sample_rate).min(CUTOFF_MAX);
        Self {
            oct: s.cutoff_hz.clamp(CUTOFF_MIN, top).log2(),
            k: damping(s.resonance),
            mix: mix(s.kind),
        }
    }

    fn bypass(&self) -> bool {
        self.mix == Self::BYPASS.mix
    }
}

impl Default for Target {
    fn default() -> Self {
        Self::BYPASS
    }
}

/// One voice's filter: two integrators per channel and the settings it glides with.
#[derive(Clone, Copy, Debug, Default)]
pub struct VoiceFilter {
    ic1: [f32; 2],
    ic2: [f32; 2],
    now: Target,
    a: [f32; 3],
}

/// The per-sample glide factor for `sample_rate`.
pub fn glide_factor(sample_rate: f32) -> f32 {
    1.0 - (-1.0 / (GLIDE_SECONDS * sample_rate)).exp()
}

impl VoiceFilter {
    /// A new voice starts right at the pad's filter, no glide.
    pub fn start(t: Target, sample_rate: f32) -> Self {
        let mut f = Self {
            now: t,
            ..Self::default()
        };
        f.coeffs(sample_rate);
        f
    }

    fn coeffs(&mut self, sample_rate: f32) {
        let g = (std::f32::consts::PI * self.now.oct.exp2() / sample_rate).tan();
        let a1 = 1.0 / (1.0 + g * (g + self.now.k));
        let a2 = g * a1;
        self.a = [a1, a2, g * a2];
    }

    /// One frame through the filter, gliding towards `t` by `glide` of the way.
    #[inline]
    pub fn tick(&mut self, t: &Target, x: [f32; 2], glide: f32, sample_rate: f32) -> [f32; 2] {
        if self.now.bypass() && t.bypass() {
            return x;
        }
        if self.now != *t {
            let step = |now: &mut f32, to: f32, eps: f32| {
                *now += (to - *now) * glide;
                if (to - *now).abs() < eps {
                    *now = to;
                }
            };
            step(&mut self.now.oct, t.oct, 1e-4);
            step(&mut self.now.k, t.k, 1e-5);
            for (now, to) in self.now.mix.iter_mut().zip(t.mix) {
                step(now, to, 1e-5);
            }
            self.coeffs(sample_rate);
        }
        let [a1, a2, a3] = self.a;
        let k = self.now.k;
        let [dry, low, band, high] = self.now.mix;
        let mut out = [0.0; 2];
        for c in 0..2 {
            let v3 = x[c] - self.ic2[c];
            let v1 = a1 * self.ic1[c] + a2 * v3;
            let v2 = self.ic2[c] + a2 * self.ic1[c] + a3 * v3;
            self.ic1[c] = 2.0 * v1 - self.ic1[c];
            self.ic2[c] = 2.0 * v2 - self.ic2[c];
            let hp = x[c] - k * v1 - v2;
            out[c] = dry * x[c] + low * v2 + band * k * v1 + high * hp;
        }
        out
    }
}
