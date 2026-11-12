//! Checked pattern data and playback compilation, outside the audio callback.
//!
//! Format 5 stores raw rows as `[tick, absolute pad, velocity, gate ticks]`:
//! ```toml
//! version = 5
//! pattern = "A1"
//!
//! [[patterns]]
//! slot = "A1"
//! bars = 2
//! quantize = "1/16"
//! strength = 100
//! hits = [[0, 0, 100, 0], [1920, 127, 110, 960]]
//! ```
//! This example supplies pattern fields, not sample assignments. Slot and selection
//! validation belongs to the session loader. Compilation never edits the raw rows.

use std::cmp::Reverse;
use std::fmt;
use std::str::FromStr;

pub const TICKS_PER_BAR: u32 = 3840;
pub const MAX_BARS: u8 = 64;
pub const MAX_HITS: usize = 4096;
pub const MAX_TICK: u32 = TICKS_PER_BAR * MAX_BARS as u32 - 1;
pub const MAX_GATE_TICKS: u32 = MAX_TICK + 1;
pub const DEFAULT_STRENGTH: u8 = 100;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Grid {
    Off,
    Quarter,
    Eighth,
    #[default]
    Sixteenth,
    ThirtySecond,
    SixtyFourth,
    EighthTriplet,
    SixteenthTriplet,
    ThirtySecondTriplet,
}

impl Grid {
    pub fn spacing(self) -> Option<u32> {
        match self {
            Self::Off => None,
            Self::Quarter => Some(960),
            Self::Eighth => Some(480),
            Self::Sixteenth => Some(240),
            Self::ThirtySecond => Some(120),
            Self::SixtyFourth => Some(60),
            Self::EighthTriplet => Some(320),
            Self::SixteenthTriplet => Some(160),
            Self::ThirtySecondTriplet => Some(80),
        }
    }

    /// A visible tick moved `strength` percent toward its nearest line; the loop end wraps
    /// to zero. The nearest midpoint takes the later line, and a half tick rounds later.
    pub fn place(self, strength: u8, tick: u32, loop_ticks: u32) -> u32 {
        let Some(spacing) = self.spacing() else {
            return tick;
        };
        let nearest = (tick + spacing / 2) / spacing * spacing;
        let placed = (tick * u32::from(100 - strength) + nearest * u32::from(strength) + 50) / 100;
        if placed == loop_ticks { 0 } else { placed }
    }

    pub fn gate_spacing(self) -> u32 {
        self.spacing().unwrap_or(240)
    }
}

impl FromStr for Grid {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Error> {
        match value {
            "off" => Ok(Self::Off),
            "1/4" => Ok(Self::Quarter),
            "1/8" => Ok(Self::Eighth),
            "1/16" => Ok(Self::Sixteenth),
            "1/32" => Ok(Self::ThirtySecond),
            "1/64" => Ok(Self::SixtyFourth),
            "1/8T" => Ok(Self::EighthTriplet),
            "1/16T" => Ok(Self::SixteenthTriplet),
            "1/32T" => Ok(Self::ThirtySecondTriplet),
            _ => Err(Error::Grid),
        }
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Off => "off",
            Self::Quarter => "1/4",
            Self::Eighth => "1/8",
            Self::Sixteenth => "1/16",
            Self::ThirtySecond => "1/32",
            Self::SixtyFourth => "1/64",
            Self::EighthTriplet => "1/8T",
            Self::SixteenthTriplet => "1/16T",
            Self::ThirtySecondTriplet => "1/32T",
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Bars,
    Grid,
    Strength,
    Tick,
    Pad,
    Velocity,
    GateTicks,
    HitCount,
    PlaybackRows,
    RowIdentity,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bars => write!(f, "Pattern length must be within 1..{MAX_BARS} bars."),
            Self::Grid => f.write_str("Unsupported pattern grid."),
            Self::Strength => f.write_str("Pattern strength must be within 0..100 in steps of 10."),
            Self::Tick => write!(f, "Pattern tick must be within 0..{MAX_TICK}."),
            Self::Pad => write!(f, "Pattern pad must be within 0..{}.", crate::PADS - 1),
            Self::Velocity => f.write_str("Pattern velocity must be within 1..127."),
            Self::GateTicks => {
                write!(
                    f,
                    "Pattern gate length must be within 0..{MAX_GATE_TICKS} ticks."
                )
            }
            Self::HitCount => write!(f, "A pattern must not exceed {MAX_HITS} hits."),
            Self::PlaybackRows => f.write_str("Playback descriptors must match the raw rows."),
            Self::RowIdentity => {
                f.write_str("Playback rows need distinct slots and nonzero generations.")
            }
        }
    }
}

impl std::error::Error for Error {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hit {
    tick: u32,
    pad: usize,
    velocity: u8,
    gate_ticks: u32,
}

impl Hit {
    pub fn new(tick: u32, pad: usize, velocity: u8, gate_ticks: u32) -> Result<Self, Error> {
        if tick > MAX_TICK {
            return Err(Error::Tick);
        }
        if pad >= crate::PADS {
            return Err(Error::Pad);
        }
        if !(1..=127).contains(&velocity) {
            return Err(Error::Velocity);
        }
        if gate_ticks > MAX_GATE_TICKS {
            return Err(Error::GateTicks);
        }
        Ok(Self {
            tick,
            pad,
            velocity,
            gate_ticks,
        })
    }

    pub fn tick(self) -> u32 {
        self.tick
    }

    pub fn pad(self) -> usize {
        self.pad
    }

    pub fn velocity(self) -> u8 {
        self.velocity
    }

    pub fn gain(self) -> f32 {
        self.velocity as f32 / 127.0
    }

    pub fn gate_ticks(self) -> u32 {
        self.gate_ticks
    }
}

/// Stable runtime identity. Reuse a slot only with a different generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RowId {
    pub(crate) slot: usize,
    pub(crate) generation: u64,
}

impl RowId {
    pub fn slot(self) -> usize {
        self.slot
    }

    pub fn generation(self) -> u64 {
        self.generation
    }

    pub fn new(slot: usize, generation: u64) -> Result<Self, Error> {
        if slot >= MAX_HITS || generation == 0 {
            return Err(Error::RowIdentity);
        }
        Ok(Self { slot, generation })
    }
}

/// Accepted capture position before rounding, valid only in its transport run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Origin {
    pub(crate) run: u64,
    pub(crate) lap: u64,
    pub(crate) ticks: u64,
    pub(crate) pass: u64,
    pub(crate) sequence: u64,
}

impl Origin {
    pub fn capture_lap(self) -> u64 {
        self.lap
    }
}

/// Nonpersistent playback data, in the same order as the raw rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlaybackRow {
    pub(crate) id: RowId,
    pub(crate) origin: Option<Origin>,
    pub(crate) pending: bool,
}

impl PlaybackRow {
    pub fn id(self) -> RowId {
        self.id
    }

    pub fn birth_in(self, run: u64) -> Option<f64> {
        self.origin
            .filter(|origin| origin.run == run)
            .map(|origin| f64::from_bits(origin.ticks))
    }

    pub fn born_after(self, pass: u64, sequence: u64) -> bool {
        self.origin
            .is_some_and(|origin| origin.pass == pass && origin.sequence > sequence)
    }

    pub fn existing(id: RowId) -> Self {
        Self {
            id,
            origin: None,
            pending: false,
        }
    }

    pub fn recorded(id: RowId, origin: Origin, pending: bool) -> Self {
        Self {
            id,
            origin: Some(origin),
            pending,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawPattern {
    bars: u8,
    grid: Grid,
    strength: u8,
    hits: Vec<Hit>,
    /// The ticks are already placed at capture (98c): compile plays them as saved, and the
    /// grid and strength only travel with the pattern for the next captured hits.
    saved: bool,
}

impl RawPattern {
    pub fn new(bars: u8, grid: Grid, strength: u8, hits: Vec<Hit>) -> Result<Self, Error> {
        if !(1..=MAX_BARS).contains(&bars) {
            return Err(Error::Bars);
        }
        if strength > 100 || !strength.is_multiple_of(10) {
            return Err(Error::Strength);
        }
        if hits.len() > MAX_HITS {
            return Err(Error::HitCount);
        }
        Ok(Self {
            bars,
            grid,
            strength,
            hits,
            saved: false,
        })
    }

    /// A pattern of saved ticks that carries the grid and strength of the next captured hits.
    pub fn saved(bars: u8, grid: Grid, strength: u8, hits: Vec<Hit>) -> Result<Self, Error> {
        Ok(Self {
            saved: true,
            ..Self::new(bars, grid, strength, hits)?
        })
    }

    pub fn bars(&self) -> u8 {
        self.bars
    }

    pub fn grid(&self) -> Grid {
        self.grid
    }

    pub fn strength(&self) -> u8 {
        self.strength
    }

    pub fn hits(&self) -> &[Hit] {
        &self.hits
    }

    fn candidates(&self) -> impl Iterator<Item = (usize, Hit)> + '_ {
        let loop_ticks = u32::from(self.bars) * TICKS_PER_BAR;
        self.hits
            .iter()
            .copied()
            .enumerate()
            .filter(move |(_, h)| h.tick < loop_ticks)
            .map(move |(index, mut h)| {
                if !self.saved {
                    h.tick = self.grid.place(self.strength, h.tick, loop_ticks);
                }
                (index, h)
            })
    }

    pub fn compile(&self) -> Pattern {
        let loop_ticks = u32::from(self.bars) * TICKS_PER_BAR;
        let mut events: Vec<_> = self.candidates().map(|(_, h)| h).collect();
        // Stable sorting keeps the earlier raw row when all winner keys match.
        events.sort_by_key(|h| (h.tick, h.pad, Reverse(h.velocity), Reverse(h.gate_ticks)));
        events.dedup_by_key(|h| (h.tick, h.pad));
        Pattern {
            loop_ticks,
            grid: self.grid,
            strength: self.strength,
            raw_rows: self.hits.len(),
            events: events.into_boxed_slice(),
            rows: None,
        }
    }

    /// Compile outside audio. Retain collisions until runtime eligibility selects a winner.
    pub fn compile_recorded(&self, rows: &[PlaybackRow]) -> Result<Pattern, Error> {
        if rows.len() != self.hits.len() {
            return Err(Error::PlaybackRows);
        }
        let mut slots = [false; MAX_HITS];
        for row in rows {
            if slots[row.id.slot] {
                return Err(Error::RowIdentity);
            }
            slots[row.id.slot] = true;
        }
        let mut candidates: Vec<_> = self.candidates().map(|(i, h)| (h, rows[i])).collect();
        candidates
            .sort_by_key(|(h, _)| (h.tick, h.pad, Reverse(h.velocity), Reverse(h.gate_ticks)));
        let (events, rows): (Vec<_>, Vec<_>) = candidates.into_iter().unzip();
        Ok(Pattern {
            loop_ticks: u32::from(self.bars) * TICKS_PER_BAR,
            grid: self.grid,
            strength: self.strength,
            raw_rows: self.hits.len(),
            events: events.into_boxed_slice(),
            rows: Some(rows.into_boxed_slice()),
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Pattern {
    loop_ticks: u32,
    grid: Grid,
    strength: u8,
    raw_rows: usize,
    events: Box<[Hit]>,
    pub(crate) rows: Option<Box<[PlaybackRow]>>,
}

impl Pattern {
    pub fn loop_ticks(&self) -> u32 {
        self.loop_ticks
    }

    pub fn grid(&self) -> Grid {
        self.grid
    }

    pub fn strength(&self) -> u8 {
        self.strength
    }

    /// Source rows before hidden hits and collisions are removed from playback.
    pub fn raw_rows(&self) -> usize {
        self.raw_rows
    }

    pub fn events(&self) -> &[Hit] {
        &self.events
    }

    /// Runtime descriptors in event order, including collisions but not hidden rows.
    pub fn playback_rows(&self) -> Option<&[PlaybackRow]> {
        self.rows.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    const GRIDS: [(Grid, &str, Option<u32>); 9] = [
        (Grid::Off, "off", None),
        (Grid::Quarter, "1/4", Some(960)),
        (Grid::Eighth, "1/8", Some(480)),
        (Grid::Sixteenth, "1/16", Some(240)),
        (Grid::ThirtySecond, "1/32", Some(120)),
        (Grid::SixtyFourth, "1/64", Some(60)),
        (Grid::EighthTriplet, "1/8T", Some(320)),
        (Grid::SixteenthTriplet, "1/16T", Some(160)),
        (Grid::ThirtySecondTriplet, "1/32T", Some(80)),
    ];

    fn hit(tick: u32, pad: usize, velocity: u8, gate_ticks: u32) -> Hit {
        Hit::new(tick, pad, velocity, gate_ticks).unwrap()
    }

    fn raw(bars: u8, grid: Grid, strength: u8, hits: Vec<Hit>) -> RawPattern {
        RawPattern::new(bars, grid, strength, hits).unwrap()
    }

    #[test]
    fn recorded_descriptors_follow_compiled_candidates_without_hidden_rows() {
        let data = raw(
            1,
            Grid::Sixteenth,
            100,
            vec![
                hit(5000, 0, 127, 0),
                hit(121, 0, 80, 0),
                hit(120, 0, 100, 0),
                hit(3839, 1, 70, 0),
            ],
        );
        let rows: Vec<_> = (0..4)
            .map(|slot| PlaybackRow::existing(RowId::new(slot, 1).unwrap()))
            .collect();
        let compiled = data.compile_recorded(&rows).unwrap();
        assert_eq!(
            compiled.events(),
            [hit(0, 1, 70, 0), hit(240, 0, 100, 0), hit(240, 0, 80, 0)]
        );
        assert_eq!(
            compiled.playback_rows(),
            Some([rows[3], rows[2], rows[1]].as_slice())
        );
        assert!(data.compile().playback_rows().is_none());
    }

    #[test]
    fn pattern_grid_labels_and_clock_units_are_exact() {
        assert_eq!(
            f64::from(TICKS_PER_BAR),
            4.0 * crate::transport::TICKS_PER_QUARTER
        );
        assert_eq!(Grid::default(), Grid::Sixteenth);
        assert_eq!(DEFAULT_STRENGTH, 100);
        for (grid, label, spacing) in GRIDS {
            assert_eq!(label.parse::<Grid>(), Ok(grid));
            assert_eq!(grid.to_string(), label);
            assert_eq!(grid.spacing(), spacing);
            assert_eq!(grid.gate_spacing(), spacing.unwrap_or(240));
            if let Some(spacing) = spacing {
                assert_eq!(TICKS_PER_BAR % spacing, 0);
            }
        }
        for label in ["", "Off", "OFF", "1/2", "1/64T", "1/16t", " 1/16", "1/16 "] {
            assert_eq!(label.parse::<Grid>(), Err(Error::Grid));
        }
    }

    #[test]
    fn pattern_numeric_limits_refuse_invalid_data_without_clamping() {
        for (tick, pad, velocity, length, error) in [
            (MAX_TICK + 1, 0, 1, 0, Error::Tick),
            (u32::MAX, 0, 1, 0, Error::Tick),
            (0, crate::PADS, 1, 0, Error::Pad),
            (0, usize::MAX, 1, 0, Error::Pad),
            (0, 0, 0, 0, Error::Velocity),
            (0, 0, 128, 0, Error::Velocity),
            (0, 0, u8::MAX, 0, Error::Velocity),
            (0, 0, 1, MAX_GATE_TICKS + 1, Error::GateTicks),
            (0, 0, 1, u32::MAX, Error::GateTicks),
        ] {
            assert_eq!(Hit::new(tick, pad, velocity, length), Err(error));
        }
        for bars in [0, MAX_BARS + 1, u8::MAX] {
            assert_eq!(
                RawPattern::new(bars, Grid::Off, 100, vec![]),
                Err(Error::Bars)
            );
        }
        for strength in [1, 9, 11, 99, 101, u8::MAX] {
            assert_eq!(
                RawPattern::new(1, Grid::Off, strength, vec![]),
                Err(Error::Strength)
            );
        }
        assert_eq!(
            RawPattern::new(1, Grid::Off, 100, vec![hit(0, 0, 1, 0); MAX_HITS + 1]),
            Err(Error::HitCount)
        );
        for bars in [1, MAX_BARS] {
            for strength in (0..=100).step_by(10) {
                let data = raw(bars, Grid::Off, strength, vec![]);
                assert_eq!(
                    (data.bars(), data.grid(), data.strength()),
                    (bars, Grid::Off, strength)
                );
                let compiled = data.compile();
                assert_eq!(compiled.loop_ticks(), u32::from(bars) * TICKS_PER_BAR);
                assert!(compiled.events().is_empty());
            }
        }
        assert_eq!(hit(0, 0, 1, 0).gain(), 1.0 / 127.0);
        let last = hit(MAX_TICK, crate::PADS - 1, 127, MAX_GATE_TICKS);
        assert_eq!(last.gain(), 1.0);
        assert_eq!(last.pad(), 127);
        assert_eq!(last.gate_ticks(), 245760);
        let data = raw(MAX_BARS, Grid::Off, 100, vec![last]);
        assert_eq!(data.compile().events(), &[last]);
    }

    /// Scenario "Pattern quantize and strength preserve the raw take".
    #[test]
    fn pattern_quantize_and_strength_preserve_the_raw_take() {
        let hits: Vec<_> = (0..1000)
            .map(|t| hit(t, t as usize % crate::PADS, (t % 127 + 1) as u8, t))
            .collect();
        for (grid, _, spacing) in GRIDS {
            for strength in (0..=100).step_by(10) {
                let data = raw(2, grid, strength, hits.clone());
                let before = data.clone();
                let compiled = data.compile();
                let mut expected: BTreeMap<(u32, usize), Hit> = BTreeMap::new();
                for h in &hits {
                    let tick = spacing.map_or(h.tick, |spacing| {
                        let source = f64::from(h.tick);
                        let nearest = (source / f64::from(spacing)).round() * f64::from(spacing);
                        (source + (nearest - source) * f64::from(strength) / 100.0).round() as u32
                    });
                    let value = Hit { tick, ..*h };
                    expected
                        .entry((tick, h.pad))
                        .and_modify(|old| {
                            if (h.velocity, h.gate_ticks) > (old.velocity, old.gate_ticks) {
                                *old = value;
                            }
                        })
                        .or_insert(value);
                }
                let expected: Vec<_> = expected.into_values().collect();
                assert_eq!(compiled.events(), expected, "{grid}, strength {strength}");
                assert_eq!(compiled.grid(), grid);
                assert_eq!(data, before);
                assert_eq!(data.hits(), hits);
            }
        }
    }

    /// Scenario "Pattern midpoint and collision rules are deterministic".
    #[test]
    fn pattern_midpoint_and_collision_rules_are_deterministic() {
        for (grid, _, spacing) in GRIDS {
            let Some(spacing) = spacing else { continue };
            let data = raw(
                1,
                grid,
                100,
                vec![
                    hit(spacing / 2 - 1, 0, 1, 0),
                    hit(spacing / 2, 1, 1, 0),
                    hit(spacing / 2 + 1, 2, 1, 0),
                ],
            );
            assert_eq!(
                data.compile()
                    .events()
                    .iter()
                    .map(|h| h.tick())
                    .collect::<Vec<_>>(),
                [0, spacing, spacing]
            );
        }
        let data = raw(
            1,
            Grid::Sixteenth,
            50,
            vec![hit(1, 0, 1, 0), hit(3, 1, 1, 0)],
        );
        assert_eq!(data.compile().events(), &[hit(1, 0, 1, 0), hit(2, 1, 1, 0)]);
        let hits = vec![
            hit(100, 0, 90, 900),
            hit(101, 0, 91, 1),
            hit(102, 0, 91, 2),
            hit(103, 0, 91, 2),
            hit(104, 127, 1, 0),
            hit(105, 1, 1, 0),
        ];
        let data = raw(1, Grid::Quarter, 100, hits.clone());
        assert_eq!(
            data.compile().events(),
            &[hit(0, 0, 91, 2), hit(0, 1, 1, 0), hit(0, 127, 1, 0)]
        );
        assert_eq!(data.hits(), hits);
        // With Off or zero strength, only exact raw tick/pad collisions merge.
        for (grid, strength) in [(Grid::Off, 100), (Grid::Sixteenth, 0)] {
            let data = raw(
                1,
                grid,
                strength,
                vec![hit(42, 0, 1, 0), hit(42, 0, 2, 0), hit(43, 0, 1, 0)],
            );
            assert_eq!(
                data.compile().events(),
                &[hit(42, 0, 2, 0), hit(43, 0, 1, 0)]
            );
            assert_eq!(data.hits().len(), 3);
        }
    }

    /// Scenario "Pattern shortening cannot wrap hidden hits into the start".
    #[test]
    fn pattern_shortening_cannot_wrap_hidden_hits_into_the_start() {
        let hits = vec![
            hit(3839, 0, 80, 5),
            hit(3840, 1, 90, 6),
            hit(5000, 127, 100, 7),
            hit(7679, 2, 110, 8),
            hit(100, 3, 120, 9),
        ];
        let long = raw(2, Grid::Sixteenth, 100, hits.clone());
        let original = long.compile();
        assert_eq!(
            original.events(),
            &[
                hit(0, 2, 110, 8),
                hit(0, 3, 120, 9),
                hit(3840, 0, 80, 5),
                hit(3840, 1, 90, 6),
                hit(5040, 127, 100, 7)
            ]
        );
        let short = raw(1, Grid::Sixteenth, 100, long.hits().to_vec());
        assert_eq!(
            short.compile().events(),
            &[hit(0, 0, 80, 5), hit(0, 3, 120, 9)]
        );
        assert_eq!(short.hits(), hits);
        let grown = raw(2, Grid::Sixteenth, 100, short.hits().to_vec());
        assert_eq!(grown.compile(), original);
        assert_eq!(grown.hits(), hits);
        let hidden = raw(
            1,
            Grid::Quarter,
            100,
            vec![
                hit(3840, 0, 127, 0),
                hit(MAX_TICK, 127, 127, MAX_GATE_TICKS),
            ],
        );
        assert!(hidden.compile().events().is_empty());
        assert_eq!(hidden.hits().len(), 2);
    }

    #[test]
    fn pattern_rounding_stays_in_bounds_at_every_loop_end() {
        for bars in 1..=MAX_BARS {
            let end = u32::from(bars) * TICKS_PER_BAR;
            let included = hit(end - 1, 127, 100, MAX_GATE_TICKS);
            let mut hits = vec![included];
            if end <= MAX_TICK {
                hits.push(hit(end, 0, 127, 0));
            }
            for (grid, _, spacing) in GRIDS {
                for strength in (0..=100).step_by(10) {
                    let data = raw(bars, grid, strength, hits.clone());
                    let compiled = data.compile();
                    let expected = spacing.map_or(end - 1, |spacing| {
                        let t = f64::from(end - 1);
                        let line = (t / f64::from(spacing)).round() * f64::from(spacing);
                        ((t + (line - t) * f64::from(strength) / 100.0).round() as u32) % end
                    });
                    assert_eq!(
                        compiled.events(),
                        &[hit(expected, 127, 100, MAX_GATE_TICKS)]
                    );
                    assert!(compiled.events()[0].tick() < end);
                    assert_eq!(data.hits(), hits);
                }
            }
        }
    }

    #[test]
    fn compiled_raw_row_count_includes_hidden_and_collision_rows() {
        for count in [0, MAX_HITS - 1, MAX_HITS] {
            let rows: Vec<_> = (0..count)
                .map(|i| hit(if i % 2 == 0 { 0 } else { TICKS_PER_BAR }, 0, 127, 0))
                .collect();
            for grid in [Grid::Off, Grid::Sixteenth] {
                let data = raw(1, grid, 100, rows.clone());
                let compiled = data.compile();
                assert_eq!(compiled.raw_rows(), count);
                assert_eq!(compiled.events().len(), usize::from(count > 0));
                assert_eq!(data.hits(), rows);
            }
        }
    }

    #[test]
    fn pattern_exact_hit_capacity_and_last_tick_survive_compilation() {
        let hits: Vec<_> = (0..MAX_HITS)
            .rev()
            .map(|i| hit(i as u32, i % crate::PADS, 1, MAX_GATE_TICKS))
            .collect();
        let data = raw(MAX_BARS, Grid::Off, 0, hits.clone());
        assert_eq!(data.compile().events().len(), MAX_HITS);
        assert_eq!(data.compile().events().first(), hits.last());
        assert_eq!(data.compile().events().last(), hits.first());
        assert_eq!(data.hits(), hits);
        for (grid, _, _) in GRIDS {
            let data = raw(
                MAX_BARS,
                grid,
                100,
                vec![hit(MAX_TICK, 127, 127, MAX_GATE_TICKS)],
            );
            let expected_tick = if grid == Grid::Off { MAX_TICK } else { 0 };
            assert_eq!(
                data.compile().events(),
                &[hit(expected_tick, 127, 127, MAX_GATE_TICKS)]
            );
            assert_eq!(data.hits()[0].tick(), MAX_TICK);
        }
    }
}
