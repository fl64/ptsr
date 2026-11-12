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
    coeff_inputs: Option<[u32; 3]>,
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
        let inputs = [
            self.now.oct.to_bits(),
            self.now.k.to_bits(),
            sample_rate.to_bits(),
        ];
        // Rounding can stall a glide short of its target. Keep its sound, but reuse
        // coefficients when only the mix changes or these inputs remain identical.
        if self.coeff_inputs == Some(inputs) {
            return;
        }
        let g = (std::f32::consts::PI * self.now.oct.exp2() / sample_rate).tan();
        let a1 = 1.0 / (1.0 + g * (g + self.now.k));
        let a2 = g * a1;
        self.a = [a1, a2, g * a2];
        self.coeff_inputs = Some(inputs);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn previous_coeffs(filter: &mut VoiceFilter, sample_rate: f32) {
        let g = (std::f32::consts::PI * filter.now.oct.exp2() / sample_rate).tan();
        let a1 = 1.0 / (1.0 + g * (g + filter.now.k));
        let a2 = g * a1;
        filter.a = [a1, a2, g * a2];
    }

    // Retain the pre-optimization frame calculation as an independent reference.
    fn previous_tick(
        filter: &mut VoiceFilter,
        target: &Target,
        input: [f32; 2],
        glide: f32,
        sample_rate: f32,
    ) -> [f32; 2] {
        if filter.now.bypass() && target.bypass() {
            return input;
        }
        if filter.now != *target {
            let step = |now: &mut f32, to: f32, eps: f32| {
                *now += (to - *now) * glide;
                if (to - *now).abs() < eps {
                    *now = to;
                }
            };
            step(&mut filter.now.oct, target.oct, 1e-4);
            step(&mut filter.now.k, target.k, 1e-5);
            for (now, to) in filter.now.mix.iter_mut().zip(target.mix) {
                step(now, to, 1e-5);
            }
            previous_coeffs(filter, sample_rate);
        }
        let [a1, a2, a3] = filter.a;
        let k = filter.now.k;
        let [dry, low, band, high] = filter.now.mix;
        let mut out = [0.0; 2];
        for c in 0..2 {
            let v3 = input[c] - filter.ic2[c];
            let v1 = a1 * filter.ic1[c] + a2 * v3;
            let v2 = filter.ic2[c] + a2 * filter.ic1[c] + a3 * v3;
            filter.ic1[c] = 2.0 * v1 - filter.ic1[c];
            filter.ic2[c] = 2.0 * v2 - filter.ic2[c];
            let hp = input[c] - k * v1 - v2;
            out[c] = dry * input[c] + low * v2 + band * k * v1 + high * hp;
        }
        out
    }

    fn compare_frame(
        actual: &mut VoiceFilter,
        previous: &mut VoiceFilter,
        target: &Target,
        frame: usize,
        sample_rate: f32,
    ) {
        let input = [
            (frame as f32 * 0.071).sin() * 0.001,
            (frame as f32 * 0.113).cos() * 0.002,
        ];
        let glide = glide_factor(sample_rate);
        let expected = previous_tick(previous, target, input, glide, sample_rate);
        let output = actual.tick(target, input, glide, sample_rate);
        assert_eq!(output.map(f32::to_bits), expected.map(f32::to_bits));
        assert!(output.iter().all(|x| x.is_finite() && x.abs() < 1.0));
        assert_eq!(actual.ic1.map(f32::to_bits), previous.ic1.map(f32::to_bits));
        assert_eq!(actual.ic2.map(f32::to_bits), previous.ic2.map(f32::to_bits));
        assert_eq!(actual.a.map(f32::to_bits), previous.a.map(f32::to_bits));
        assert_eq!(actual.now.oct.to_bits(), previous.now.oct.to_bits());
        assert_eq!(actual.now.k.to_bits(), previous.now.k.to_bits());
        assert_eq!(
            actual.now.mix.map(f32::to_bits),
            previous.now.mix.map(f32::to_bits)
        );
    }

    #[test]
    fn filter_reuses_only_identical_coefficient_inputs() {
        let mut filter = VoiceFilter::default();
        assert_eq!(filter.coeff_inputs, None);
        filter.coeffs(48_000.0);
        assert!(filter.a.iter().all(|x| x.is_finite()));
        assert_ne!(filter.a, [0.0; 3]);

        // Sentinel coefficients distinguish reuse from another identical calculation.
        filter.a = [0.125, 0.25, 0.5];
        filter.coeffs(48_000.0);
        assert_eq!(filter.a, [0.125, 0.25, 0.5]);
        filter.now.mix = mix(FilterKind::Band);
        filter.coeffs(48_000.0);
        assert_eq!(filter.a, [0.125, 0.25, 0.5]);

        for (oct, k, rate) in [
            (9.0, 0.7, 48_000.0),
            (10.0, 0.7, 48_000.0),
            (10.0, 0.9, 48_000.0),
            (10.0, 0.9, 96_000.0),
            (9.0, 0.7, 48_000.0),
        ] {
            filter.now.oct = oct;
            filter.now.k = k;
            let mut expected = filter;
            previous_coeffs(&mut expected, rate);
            filter.coeffs(rate);
            assert_eq!(filter.a.map(f32::to_bits), expected.a.map(f32::to_bits));
            assert_eq!(
                filter.coeff_inputs,
                Some([oct.to_bits(), k.to_bits(), rate.to_bits()])
            );
        }
    }

    #[test]
    fn filter_matches_previous_formula_through_target_changes() {
        let kinds = [
            FilterKind::Off,
            FilterKind::LowPass,
            FilterKind::HighPass,
            FilterKind::Band,
        ];
        for rate in [44_100.0, 48_000.0, 96_000.0] {
            for from in kinds {
                for to in kinds {
                    let settings = FilterSettings {
                        kind: from,
                        cutoff_hz: 900.0,
                        resonance: 0.5,
                    };
                    let initial = Target::new(settings, rate);
                    let targets = [
                        initial,
                        Target::new(
                            FilterSettings {
                                kind: to,
                                ..settings
                            },
                            rate,
                        ),
                        Target::new(
                            FilterSettings {
                                kind: to,
                                cutoff_hz: CUTOFF_MAX,
                                resonance: 1.0,
                            },
                            rate,
                        ),
                        Target::new(
                            FilterSettings {
                                kind: from,
                                cutoff_hz: CUTOFF_MIN,
                                resonance: 0.0,
                            },
                            rate,
                        ),
                        Target::BYPASS,
                        initial,
                    ];
                    let mut actual = VoiceFilter::start(initial, rate);
                    let mut previous = VoiceFilter {
                        now: initial,
                        ..VoiceFilter::default()
                    };
                    previous_coeffs(&mut previous, rate);
                    for frame in 0..12_000 {
                        let sample_rate = if frame >= 11_000 { rate * 2.0 } else { rate };
                        compare_frame(
                            &mut actual,
                            &mut previous,
                            &targets[frame / 2_000],
                            frame,
                            sample_rate,
                        );
                    }
                    actual = VoiceFilter::start(initial, rate);
                    previous = VoiceFilter {
                        now: initial,
                        ..VoiceFilter::default()
                    };
                    previous_coeffs(&mut previous, rate);
                    compare_frame(&mut actual, &mut previous, &initial, 0, rate);
                }
            }
        }
    }

    #[test]
    fn filter_stalled_glide_keeps_previous_sound_and_residual_state() {
        let sample_rate = 48_000.0;
        let target = Target::new(
            FilterSettings {
                kind: FilterKind::Band,
                cutoff_hz: 900.0,
                resonance: 0.5,
            },
            sample_rate,
        );
        let mut actual = VoiceFilter::start(Target::BYPASS, sample_rate);
        let mut previous = VoiceFilter {
            now: Target::BYPASS,
            ..VoiceFilter::default()
        };
        previous_coeffs(&mut previous, sample_rate);
        for frame in 0..48_000 {
            compare_frame(&mut actual, &mut previous, &target, frame, sample_rate);
        }
        let residual = actual.now;
        let coefficients = actual.a;
        assert_ne!(residual, target);
        for frame in 48_000..528_000 {
            compare_frame(&mut actual, &mut previous, &target, frame, sample_rate);
            assert_eq!(actual.now, residual);
            assert_eq!(actual.a, coefficients);
        }
        let inputs = actual.coeff_inputs;
        actual.a = [0.125, 0.25, 0.5];
        actual.tick(&target, [0.0; 2], glide_factor(sample_rate), sample_rate);
        assert_eq!(actual.a, [0.125, 0.25, 0.5]);
        assert_eq!(actual.coeff_inputs, inputs);
    }

    #[test]
    fn filter_cache_preserves_previous_update_conditions() {
        for kind in [FilterKind::Off, FilterKind::Band] {
            let initial = Target::new(
                FilterSettings {
                    kind,
                    cutoff_hz: 900.0,
                    resonance: 0.5,
                },
                48_000.0,
            );
            let mut actual = VoiceFilter::start(initial, 48_000.0);
            let mut previous = VoiceFilter {
                now: initial,
                ..VoiceFilter::default()
            };
            previous_coeffs(&mut previous, 48_000.0);
            let inputs = actual.coeff_inputs;
            compare_frame(&mut actual, &mut previous, &initial, 0, 96_000.0);
            assert_eq!(actual.coeff_inputs, inputs);
            if kind == FilterKind::Off {
                compare_frame(&mut actual, &mut previous, &Target::BYPASS, 1, 44_100.0);
                assert_eq!(actual.coeff_inputs, inputs);
            } else {
                let mut target = initial;
                target.mix = mix(FilterKind::HighPass);
                compare_frame(&mut actual, &mut previous, &target, 1, 96_000.0);
                assert_eq!(
                    actual.coeff_inputs,
                    Some([
                        initial.oct.to_bits(),
                        initial.k.to_bits(),
                        96_000.0_f32.to_bits()
                    ])
                );
            }
        }
        let mut actual = VoiceFilter::default();
        let mut previous = VoiceFilter::default();
        let mut target = Target::BYPASS;
        target.mix = mix(FilterKind::LowPass);
        compare_frame(&mut actual, &mut previous, &target, 0, 48_000.0);
        assert!(actual.coeff_inputs.is_some());
    }
}
