//! Read-only pattern projection. Recorded timing and playback settings stay unchanged.

use engine::pattern::{Grid, Hit, MAX_HITS, PlaybackRow, RawPattern, RowId, TICKS_PER_BAR};

/// In order of cell size: a finer step always zooms in.
const GRIDS: [Grid; 8] = [
    Grid::Quarter,
    Grid::Eighth,
    Grid::EighthTriplet,
    Grid::Sixteenth,
    Grid::SixteenthTriplet,
    Grid::ThirtySecond,
    Grid::ThirtySecondTriplet,
    Grid::SixtyFourth,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Division(usize);

impl Default for Division {
    fn default() -> Self {
        Self(3)
    }
}

impl Division {
    pub(super) fn grid(self) -> Grid {
        GRIDS[self.0]
    }

    pub(super) fn spacing(self) -> u32 {
        self.grid().gate_spacing()
    }

    pub(super) fn cells_per_bar(self) -> u32 {
        TICKS_PER_BAR / self.spacing()
    }

    /// `view 1/16`, `view 1/16 triplet`: the header names the view apart from the quantize.
    pub(super) fn label(self) -> String {
        let grid = self.grid().to_string();
        match grid.strip_suffix('T') {
            Some(straight) => format!("view {straight} triplet"),
            None => format!("view {grid}"),
        }
    }

    pub(super) fn change(&mut self, next: bool) {
        self.0 = if next {
            (self.0 + 1).min(GRIDS.len() - 1)
        } else {
            self.0.saturating_sub(1)
        };
    }
}

#[derive(Default)]
pub(super) struct State {
    pub division: Division,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Event {
    pub id: RowId,
    pub source: usize,
    pub raw: Hit,
    pub played: Hit,
}

pub(super) struct Model {
    pub division: Division,
    pub bars: u8,
    events: Vec<Event>,
}

/// The whole loop at the view division; the drawing picks the page that fits.
pub(super) struct Cells {
    /// The selected pad's events per cell.
    pub pad: Vec<Vec<Event>>,
    /// Any onset of a pad in the bank, per cell.
    pub bank: Vec<bool>,
}

impl Model {
    pub(super) fn new(
        raw: &RawPattern,
        rows: Option<&[PlaybackRow]>,
        division: Division,
    ) -> Result<Self, engine::pattern::Error> {
        // Loaded patterns can lack capture identities. Projection does not install new ones.
        let temporary;
        let rows = match rows {
            Some(rows) => rows,
            None => {
                temporary = (0..raw.hits().len())
                    .map(|slot| RowId::new(slot, 1).map(PlaybackRow::existing))
                    .collect::<Result<Vec<_>, _>>()?;
                &temporary
            }
        };
        let compiled = raw.compile_recorded(rows)?;
        let mut source_by_slot = [0; MAX_HITS];
        for (source, row) in rows.iter().enumerate() {
            source_by_slot[row.id().slot()] = source;
        }
        let mut events = compiled
            .events()
            .iter()
            .zip(compiled.playback_rows().unwrap_or_default())
            .map(|(&played, row)| {
                let source = source_by_slot[row.id().slot()];
                Event {
                    id: row.id(),
                    source,
                    raw: raw.hits()[source],
                    played,
                }
            })
            .collect::<Vec<_>>();
        events.sort_by_key(|event| (event.played.tick(), event.raw.tick(), event.source));
        Ok(Self {
            division,
            bars: raw.bars(),
            events,
        })
    }

    pub(super) fn cells(&self, pad: usize, bank: std::ops::Range<usize>) -> Cells {
        let count = (u32::from(self.bars) * self.division.cells_per_bar()) as usize;
        let mut cells = Cells {
            pad: vec![Vec::new(); count],
            bank: vec![false; count],
        };
        for event in &self.events {
            let cell = (event.played.tick() / self.division.spacing()) as usize;
            if cell >= count {
                continue;
            }
            if event.played.pad() == pad {
                cells.pad[cell].push(*event);
            }
            if bank.contains(&event.played.pad()) {
                cells.bank[cell] = true;
            }
        }
        cells
    }
}

/// No UI timer or cached transport fallback can fabricate a moving position.
pub(super) fn position(
    snapshot: Option<engine::transport::Snapshot>,
    loop_ticks: u32,
) -> Option<u32> {
    let snapshot = snapshot?;
    (snapshot.playing
        && snapshot.pattern_ticks == loop_ticks
        && loop_ticks > 0
        && snapshot.ticks.is_finite()
        && snapshot.ticks >= 0.0)
        .then(|| (snapshot.ticks % f64::from(loop_ticks)).floor() as u32)
}

impl super::App {
    pub(super) fn change_pattern_view(&mut self, next: bool) {
        if self.transport_input_blocked() || self.help_open {
            return;
        }
        if self.screen != input::Screen::Pads {
            self.post_notice(
                super::Level::Info,
                (
                    "Open Pads to change the pattern view.".into(),
                    std::time::Instant::now(),
                ),
            );
            return;
        }
        self.pattern_view.division.change(next);
    }

    /// The current ready project's transport, or nothing while a project changes.
    fn pattern_snapshot(&self) -> Option<engine::transport::Snapshot> {
        self.audio_alive().and_then(|audio| {
            use std::sync::atomic::Ordering::Acquire;
            let epoch = audio.project.epoch.load(Acquire);
            if audio.project.ready.load(Acquire) != epoch {
                return None;
            }
            let snapshot = audio.stats.transport(epoch);
            (audio.project.epoch.load(Acquire) == epoch
                && audio.project.ready.load(Acquire) == epoch)
                .then_some(snapshot)
                .flatten()
        })
    }

    /// Banks with visible A1 onsets, and per bank the pads whose onset falls in the
    /// current 1/16 playback step: a fixed step keeps a flash visible at any view.
    pub(super) fn bank_pattern(&self) -> tui::BankPattern {
        let mut banks = tui::BankPattern::default();
        let Some(raw) = &self.pads.pattern else {
            return banks;
        };
        let Ok(model) = raw.checked().and_then(|raw| {
            Model::new(&raw, self.pads.pattern_rows.as_deref(), Division::default())
                .map_err(|error| error.to_string())
        }) else {
            return banks;
        };
        let step = Grid::Sixteenth.gate_spacing();
        let now = position(
            self.pattern_snapshot(),
            u32::from(model.bars) * TICKS_PER_BAR,
        )
        .map(|tick| tick / step);
        for event in &model.events {
            let pad = event.played.pad();
            let bank = pad / super::PADS_PER_BANK;
            if bank >= tui::BANKS {
                continue;
            }
            banks.recorded[bank] = true;
            if now == Some(event.played.tick() / step) {
                banks.hits[bank] |= 1 << (pad % super::PADS_PER_BANK);
            }
        }
        banks
    }

    pub(super) fn pattern_strip(&self, keymap: &input::Keymap) -> tui::PatternStrip {
        let mut strip = tui::PatternStrip {
            header: String::new(),
            label: String::new(),
            cells: Vec::new(),
            all: Vec::new(),
            beat_cells: 1,
            playhead: None,
        };
        let Some(raw) = &self.pads.pattern else {
            let keys = self
                .help_bindings(keymap, input::Action::PatternRecord)
                .join("/");
            strip.header = format!(
                "No A1 · {} record A1",
                if keys.is_empty() { "none" } else { &keys }
            );
            return strip;
        };
        let model = raw.checked().and_then(|raw| {
            Model::new(
                &raw,
                self.pads.pattern_rows.as_deref(),
                self.pattern_view.division,
            )
            .map_err(|error| error.to_string())
        });
        let model = match model {
            Ok(model) => model,
            Err(error) => {
                strip.header = format!("A1 unavailable: {error}");
                return strip;
            }
        };
        let tick = position(
            self.pattern_snapshot(),
            u32::from(model.bars) * TICKS_PER_BAR,
        );
        let pad = self.pads.selected_abs();
        let bank = self.pads.bank * super::PADS_PER_BANK;
        let cells = model.cells(pad, bank..bank + super::PADS_PER_BANK);
        let spacing = model.division.spacing();
        let position = match tick {
            Some(tick) => format!(
                "bar {}/{} beat {}",
                tick / TICKS_PER_BAR + 1,
                model.bars,
                tick % TICKS_PER_BAR / 960 + 1
            ),
            None if model.bars == 1 => "1 bar".into(),
            None => format!("{} bars", model.bars),
        };
        // the quantize of the next hits, apart from the view (98b); the file name stays in
        // the pad panel, it does not fit the strip
        let quantize = if raw.quantize == "off" {
            "quantize off".to_string()
        } else {
            format!("quantize {} {} %", raw.quantize, raw.strength)
        };
        strip.header = format!(
            "A1 · {position} · {} · {quantize} · bank {}",
            model.division.label(),
            self.pads.bank + 1,
        );
        strip.label = format!(
            "pad {}",
            super::PAD_KEYS[self.pads.selected].to_ascii_uppercase()
        );
        strip.beat_cells = (960 / spacing) as usize;
        strip.playhead = tick.map(|tick| (tick / spacing) as usize);
        strip.cells = cells
            .pad
            .iter()
            .map(|events| tui::PatternCell {
                count: events.len(),
                off_grid: events
                    .iter()
                    .any(|event| !event.played.tick().is_multiple_of(spacing)),
            })
            .collect();
        strip.all = cells.bank;
        strip
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_raw(bars: u8, grid: Grid, strength: u8, ticks: &[u32]) -> RawPattern {
        RawPattern::new(
            bars,
            grid,
            strength,
            ticks
                .iter()
                .map(|&t| Hit::new(t, 0, 80, 480).unwrap())
                .collect(),
        )
        .unwrap()
    }

    #[test]
    fn pattern_view_divisions_preserve_the_take() {
        for grid in std::iter::once(Grid::Off).chain(GRIDS) {
            for strength in [0, 50, 100] {
                let raw = make_raw(2, grid, strength, &[0, 1, 120, 121, 3839, 7680]);
                let before = raw.clone();
                for index in 0..GRIDS.len() {
                    let model = Model::new(&raw, None, Division(index)).unwrap();
                    for event in &model.events {
                        assert_eq!(event.raw, raw.hits()[event.source]);
                        assert_eq!(event.id.slot(), event.source);
                        assert_eq!(event.raw.velocity(), 80);
                        assert_eq!(event.raw.gate_ticks(), 480);
                    }
                    assert_eq!(raw, before);
                    assert_eq!(model.bars, 2);
                }
            }
        }
    }

    /// Scenario "Pattern cells show off-grid events and collisions".
    #[test]
    fn pattern_cells_keep_off_grid_positions_and_colliding_rows() {
        let raw = make_raw(1, Grid::Off, 100, &[239, 0, 1, 240, 1]);
        let before = raw.clone();
        let rows = [9, 7, 1, 4, 0].map(|slot| PlaybackRow::existing(RowId::new(slot, 42).unwrap()));
        let model = Model::new(&raw, Some(&rows), Division::default()).unwrap();
        assert_eq!(raw, before, "grouping changes no raw row");
        let cells = model.cells(0, 0..16);
        assert_eq!(
            cells.pad[0].iter().map(|e| e.source).collect::<Vec<_>>(),
            [1, 2, 4, 0]
        );
        assert_eq!(cells.pad[1][0].source, 3);
        for event in &model.events {
            assert_eq!(event.id, rows[event.source].id());
        }
        assert!(model.cells(127, 112..128).pad.iter().all(Vec::is_empty));
        let raw = make_raw(1, Grid::Sixteenth, 100, &[121, 120, 122]);
        let model = Model::new(&raw, None, Division::default()).unwrap();
        assert_eq!(
            model.cells(0, 0..16).pad[1]
                .iter()
                .map(|e| e.source)
                .collect::<Vec<_>>(),
            [1, 0, 2]
        );
        assert!(Model::new(&raw, Some(&rows[..2]), Division::default()).is_err());
    }

    #[test]
    fn pattern_view_preserves_hidden_rows_and_the_loop_seam() {
        for bars in [1, 2] {
            let raw = make_raw(bars, Grid::Sixteenth, 100, &[3839, 4000]);
            let model = Model::new(&raw, None, Division::default()).unwrap();
            if bars == 1 {
                assert_eq!(model.events.len(), 1);
                assert_eq!(model.events[0].played.tick(), 0);
                assert_eq!(model.cells(0, 0..16).pad[0].len(), 1);
            } else {
                assert_eq!(model.events.len(), 2);
                assert_eq!(model.events[0].played.tick(), 3840);
                assert_eq!(model.events[1].raw.tick(), 4000);
            }
            assert_eq!(raw.hits().len(), 2);
        }
    }

    #[test]
    fn pattern_cells_cover_the_whole_loop_and_mark_the_bank() {
        for (index, total) in [4, 8, 12, 16, 24, 32, 48, 64].into_iter().enumerate() {
            let division = Division(index);
            assert_eq!(division.cells_per_bar(), total);
            assert_eq!(960 % division.spacing(), 0, "whole cells per beat");
            for bars in [1u8, 2, 64] {
                // pad 0 and pad 20 (bank 2) hit the last cell of the loop
                let last = u32::from(bars) * TICKS_PER_BAR - division.spacing();
                let raw = RawPattern::new(
                    bars,
                    Grid::Off,
                    100,
                    vec![
                        Hit::new(last, 0, 80, 0).unwrap(),
                        Hit::new(0, 20, 80, 0).unwrap(),
                    ],
                )
                .unwrap();
                let model = Model::new(&raw, None, division).unwrap();
                let cells = model.cells(0, 0..16);
                let count = (u32::from(bars) * total) as usize;
                assert_eq!(cells.pad.len(), count);
                assert_eq!(cells.bank.len(), count);
                assert_eq!(cells.pad[count - 1].len(), 1);
                assert!(cells.bank[count - 1]);
                // pad 20 is in bank 2: not in this bank's row
                assert!(!cells.bank[0]);
                assert!(model.cells(20, 16..32).bank[0]);
                assert!(!model.cells(20, 16..32).bank[count - 1]);
            }
        }
        let mut division = Division::default();
        for _ in 0..20 {
            division.change(false);
        }
        assert_eq!(division.grid(), Grid::Quarter);
        assert_eq!(division.label(), "view 1/4");
        // every finer step has a smaller cell, so Alt+Up always zooms in
        let mut labels = vec![division.label()];
        for _ in 0..20 {
            let spacing = division.spacing();
            division.change(true);
            assert!(division.spacing() < spacing || division.grid() == Grid::SixtyFourth);
            if labels.last() != Some(&division.label()) {
                labels.push(division.label());
            }
        }
        assert_eq!(division.grid(), Grid::SixtyFourth);
        assert_eq!(
            labels,
            [
                "view 1/4",
                "view 1/8",
                "view 1/8 triplet",
                "view 1/16",
                "view 1/16 triplet",
                "view 1/32",
                "view 1/32 triplet",
                "view 1/64"
            ]
        );
        assert_eq!(Division::default().label(), "view 1/16");
    }

    #[test]
    fn missing_invalid_or_mismatched_audio_feedback_has_no_playhead() {
        use engine::transport::Snapshot;
        assert_eq!(position(None, 3840), None);
        for ticks in [f64::NAN, f64::INFINITY, -1.0] {
            assert_eq!(
                position(
                    Some(Snapshot {
                        playing: true,
                        pattern_ticks: 3840,
                        ticks,
                        ..Default::default()
                    }),
                    3840
                ),
                None
            );
        }
        let state = Snapshot {
            playing: true,
            pattern_ticks: 3840,
            ticks: 4000.5,
            ..Default::default()
        };
        assert_eq!(position(Some(state), 3840), Some(160));
        assert_eq!(position(Some(state), 7680), None);
        assert_eq!(
            position(
                Some(Snapshot {
                    playing: false,
                    ..state
                }),
                3840
            ),
            None
        );
        assert_eq!(position(Some(state), 0), None);
    }
}
