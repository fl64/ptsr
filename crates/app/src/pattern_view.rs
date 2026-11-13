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

pub(super) struct State {
    pub division: Division,
    pub overview: bool,
    pub hide_empty_banks: bool,
    pub edit: bool,
    pub cursor_cell: usize,
    pub cursor_visited: bool,
    pub selected_event: Option<SelectedEvent>,
    pub midi_presses: std::collections::HashSet<(usize, u64)>,
    pub step_generations: Vec<u64>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            division: Division::default(),
            overview: false,
            hide_empty_banks: true,
            edit: false,
            cursor_cell: 0,
            cursor_visited: false,
            selected_event: None,
            midi_presses: std::collections::HashSet::new(),
            step_generations: vec![0; MAX_HITS],
        }
    }
}

pub(super) struct SelectedEvent {
    pub event: Event,
    pub pattern: crate::session::PatternState,
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

    fn events_in_cell(&self, pad: usize, cell: usize) -> Vec<Event> {
        self.events
            .iter()
            .copied()
            .filter(|event| {
                event.played.pad() == pad
                    && (event.played.tick() / self.division.spacing()) as usize == cell
            })
            .collect()
    }

    #[cfg(test)]
    pub(super) fn overview_cells(&self, bank: usize) -> Vec<Vec<tui::PatternCell>> {
        let count = (u32::from(self.bars) * self.division.cells_per_bar()) as usize;
        let (rows, _) = self.overview_page_cells(0, count);
        let start = bank * super::PADS_PER_BANK;
        rows.into_iter()
            .skip(start)
            .take(super::PADS_PER_BANK)
            .collect()
    }

    pub(super) fn overview_page_cells(
        &self,
        first: usize,
        end: usize,
    ) -> (Vec<Vec<tui::PatternCell>>, [u16; tui::BANKS]) {
        let mut rows =
            vec![vec![tui::PatternCell::default(); end.saturating_sub(first)]; super::PADS];
        let mut active = [0u16; tui::BANKS];
        let spacing = self.division.spacing();
        for event in &self.events {
            let pad = event.played.pad();
            if pad >= super::PADS {
                continue;
            }
            active[pad / super::PADS_PER_BANK] |= 1 << (pad % super::PADS_PER_BANK);
            let cell = (event.played.tick() / spacing) as usize;
            if !(first..end).contains(&cell) {
                continue;
            }
            if let Some(mark) = rows[pad].get_mut(cell - first) {
                mark.count += 1;
                mark.off_grid |= !event.played.tick().is_multiple_of(spacing);
                mark.max_velocity = mark.max_velocity.max(event.raw.velocity());
            }
        }
        (rows, active)
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
    fn step_model(&self) -> Result<Model, String> {
        let raw = self
            .pads
            .pattern
            .as_ref()
            .ok_or("A1 is absent.")?
            .checked()?;
        Model::new(
            &raw,
            self.pads.pattern_rows.as_deref(),
            self.pattern_view.division,
        )
        .map_err(|error| error.to_string())
    }

    pub(super) fn toggle_step_edit(&mut self) {
        if self.screen != input::Screen::Pads || !self.pattern_view.overview || self.help_open {
            return;
        }
        if self.pattern_view.edit {
            self.pattern_view.edit = false;
            self.pattern_view.selected_event = None;
            return;
        }
        self.flush_audio();
        let refusal = if self.pads.pattern.is_none() {
            Some("No A1: record A1 before step correction.")
        } else if self.transport_input_blocked()
            || self.edit
            || self.pads.marking
            || self.pad_in_hand.is_some()
            || self.copy_in_hand.is_some()
            || self.browser.in_hand.is_some()
            || self.erase_pads
            || self.pattern_panel.is_some()
            || self.recording.is_some()
        {
            Some("Finish the current edit or recording before step correction.")
        } else {
            self.pattern_tidy_refusal()
        };
        if let Some(text) = refusal {
            self.post_notice(super::Level::Warn, (text.into(), std::time::Instant::now()));
            return;
        }
        let Ok(model) = self.step_model() else {
            self.post_notice(
                super::Level::Error,
                (
                    "A1 cannot open for correction.".into(),
                    std::time::Instant::now(),
                ),
            );
            return;
        };
        if !self.pattern_view.cursor_visited {
            let tick = position(
                self.pattern_snapshot(),
                u32::from(model.bars) * TICKS_PER_BAR,
            );
            self.pattern_view.cursor_cell =
                tick.map_or(0, |tick| tick / model.division.spacing()) as usize;
            self.pattern_view.cursor_visited = true;
        }
        self.pattern_view.edit = true;
        self.pattern_view.selected_event = None;
    }

    pub(super) fn move_step_cursor(&mut self, right: bool) {
        if !self.pattern_view.edit || self.transport_input_blocked() || self.help_open {
            return;
        }
        let Ok(model) = self.step_model() else {
            return;
        };
        let count = (u32::from(model.bars) * model.division.cells_per_bar()) as usize;
        self.pattern_view.cursor_cell = if right {
            (self.pattern_view.cursor_cell + 1).min(count.saturating_sub(1))
        } else {
            self.pattern_view.cursor_cell.saturating_sub(1)
        };
        self.pattern_view.selected_event = None;
    }

    pub(super) fn choose_step_event(&mut self, next: bool) {
        if !self.pattern_view.edit || self.transport_input_blocked() || self.help_open {
            return;
        }
        let Ok(model) = self.step_model() else {
            return;
        };
        let abs = self.pads.selected_abs();
        let events = model.events_in_cell(abs, self.pattern_view.cursor_cell);
        if events.is_empty() {
            self.pattern_view.selected_event = None;
            return;
        }
        let index = self
            .pattern_view
            .selected_event
            .as_ref()
            .and_then(|selected| events.iter().position(|event| *event == selected.event));
        let index = match index {
            Some(index) if next => (index + 1).min(events.len() - 1),
            Some(index) => index.saturating_sub(1),
            None if next => 0,
            None => events.len() - 1,
        };
        self.pattern_view.selected_event = self.pads.pattern.clone().map(|pattern| SelectedEvent {
            event: events[index],
            pattern,
        });
    }

    pub(super) fn step_pad_hit(&mut self, abs: usize, velocity: f32) {
        if self.edit
            || self.pads.marking
            || self.pad_in_hand.is_some()
            || self.copy_in_hand.is_some()
            || self.browser.in_hand.is_some()
        {
            self.post_notice(
                super::Level::Warn,
                (
                    "Finish the current edit before step correction.".into(),
                    std::time::Instant::now(),
                ),
            );
            return;
        }
        if !self.pattern_view.edit
            || self.transport_input_blocked()
            || self.help_open
            || self.pattern_tidy_refusal().is_some()
            || self.pattern_panel.is_some()
            || self.erase_pads
            || self.recording.is_some()
        {
            return;
        }
        self.pads.selected = abs % super::PADS_PER_BANK;
        let Ok(model) = self.step_model() else {
            return;
        };
        let events = model.events_in_cell(abs, self.pattern_view.cursor_cell);
        if self
            .pattern_view
            .selected_event
            .as_ref()
            .is_some_and(|choice| {
                choice.event.played.pad() == abs
                    && (self.pads.pattern.as_ref() != Some(&choice.pattern)
                        || !events.contains(&choice.event))
            })
        {
            self.pattern_view.selected_event = self.pads.pattern.clone().and_then(|pattern| {
                events
                    .first()
                    .copied()
                    .map(|event| SelectedEvent { event, pattern })
            });
            return;
        }
        let selected = self
            .pattern_view
            .selected_event
            .as_ref()
            .and_then(|choice| {
                (choice.pattern == *self.pads.pattern.as_ref()?)
                    .then(|| events.iter().find(|event| **event == choice.event).copied())
                    .flatten()
            });
        let target = match events.as_slice() {
            [] => {
                self.pattern_view.selected_event = None;
                let expected = self.pads.pattern.clone().unwrap();
                let tick =
                    self.pattern_view.cursor_cell as u32 * self.pattern_view.division.spacing();
                let velocity = (velocity * 127.0).round().clamp(1.0, 127.0) as u8;
                let gate_ticks = if self.pads.gate[abs] {
                    self.pattern_view.division.spacing()
                } else {
                    0
                };
                let result = self.add_step_event(abs, tick, velocity, gate_ticks, &expected);
                match result {
                    Ok(true) => self.post_notice(
                        super::Level::Done,
                        (
                            format!(
                                "A1: added one hit on pad {}. Undo removes it.",
                                super::pads::PAD_KEYS[abs % super::PADS_PER_BANK]
                                    .to_ascii_uppercase()
                            ),
                            std::time::Instant::now(),
                        ),
                    ),
                    Ok(false) => self.post_notice(
                        super::Level::Info,
                        (
                            "A1 changed: select the cell again.".into(),
                            std::time::Instant::now(),
                        ),
                    ),
                    Err(error) => {
                        self.post_notice(super::Level::Error, (error, std::time::Instant::now()))
                    }
                }
                return;
            }
            [event] => Some(*event),
            _ => selected,
        };
        let Some(target) = target else {
            self.pattern_view.selected_event =
                self.pads.pattern.clone().map(|pattern| SelectedEvent {
                    event: events[0],
                    pattern,
                });
            return;
        };
        let expected = self.pads.pattern.clone().unwrap();
        let result = self.remove_step_event(target, &expected);
        self.pattern_view.selected_event = None;
        match result {
            Ok(true) => self.post_notice(
                super::Level::Done,
                (
                    format!(
                        "A1: removed one hit from pad {}. Undo restores it.",
                        super::pads::PAD_KEYS[abs % super::PADS_PER_BANK].to_ascii_uppercase()
                    ),
                    std::time::Instant::now(),
                ),
            ),
            Ok(false) => self.post_notice(
                super::Level::Info,
                (
                    "A1 changed: select the hit again.".into(),
                    std::time::Instant::now(),
                ),
            ),
            Err(error) => self.post_notice(super::Level::Error, (error, std::time::Instant::now())),
        }
    }

    #[cfg(test)]
    pub(super) fn pattern_overview(&self, keymap: &input::Keymap) -> tui::PatternOverview {
        self.pattern_overview_at(keymap, 80)
    }

    pub(super) fn pattern_overview_at(
        &self,
        keymap: &input::Keymap,
        width: u16,
    ) -> tui::PatternOverview {
        let mut view = tui::PatternOverview {
            header: String::new(),
            rows: vec![Vec::new(); super::PADS],
            names: (0..super::PADS)
                .map(|abs| {
                    if self.pads.file[abs].is_some() {
                        super::pad_name(&self.pads, abs)
                    } else {
                        "empty".into()
                    }
                })
                .collect(),
            active_pads: [0; tui::BANKS],
            hide_empty: self.pattern_view.hide_empty_banks,
            beat_cells: 1,
            playhead: None,
            first_cell: 0,
            total_cells: 0,
            edit_cursor: self
                .pattern_view
                .edit
                .then_some(self.pattern_view.cursor_cell),
            event_choice: None,
            event_velocity: None,
        };
        let Some(raw) = &self.pads.pattern else {
            let keys = self
                .help_bindings(keymap, input::Action::PatternRecord)
                .join("/");
            view.header = format!(
                "pattern A1 is empty · {} record A1",
                if keys.is_empty() { "none" } else { &keys }
            );
            return view;
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
                view.header = format!("A1 unavailable: {error}");
                return view;
            }
        };
        let spacing = model.division.spacing();
        let tick = position(
            self.pattern_snapshot(),
            u32::from(model.bars) * TICKS_PER_BAR,
        );
        let location = tick.map_or_else(
            || format!("{} bars", model.bars),
            |tick| {
                format!(
                    "bar {}/{} beat {}",
                    tick / TICKS_PER_BAR + 1,
                    model.bars,
                    tick % TICKS_PER_BAR / 960 + 1
                )
            },
        );
        let quantize = if raw.quantize == "off" {
            "quantize off".to_string()
        } else {
            format!("quantize {} {} %", raw.quantize, raw.strength)
        };
        view.header = format!("{} · {location} · {quantize}", model.division.label());
        view.beat_cells = (960 / spacing) as usize;
        view.playhead = tick.map(|tick| (tick / spacing) as usize);
        view.total_cells = (u32::from(model.bars) * model.division.cells_per_bar()) as usize;
        let page =
            tui::pattern_overview_page_cells(width as usize, view.beat_cells, view.total_cells);
        view.first_cell = if self.pattern_view.edit {
            self.pattern_view.cursor_cell
        } else {
            view.playhead.unwrap_or(0)
        } / page
            * page;
        let end = (view.first_cell + page).min(view.total_cells);
        (view.rows, view.active_pads) = model.overview_page_cells(view.first_cell, end);
        if self.pattern_view.edit {
            let events =
                model.events_in_cell(self.pads.selected_abs(), self.pattern_view.cursor_cell);
            if !events.is_empty() {
                let index = self
                    .pattern_view
                    .selected_event
                    .as_ref()
                    .and_then(|choice| events.iter().position(|event| *event == choice.event))
                    .map_or(usize::from(events.len() == 1), |index| index + 1);
                view.event_choice = Some((index, events.len()));
                view.event_velocity = (index > 0).then(|| events[index - 1].raw.velocity());
            }
        }
        view
    }

    pub(super) fn change_pattern_view(&mut self, next: bool) {
        if self.transport_input_blocked() || self.help_open {
            return;
        }
        if self.screen != input::Screen::Pads {
            self.post_notice(
                super::Level::Info,
                (
                    "Open Pads or the A1 overview to change the pattern view.".into(),
                    std::time::Instant::now(),
                ),
            );
            return;
        }
        let old_tick = self.pattern_view.cursor_cell as u32 * self.pattern_view.division.spacing();
        self.pattern_view.division.change(next);
        if self.pattern_view.edit {
            self.pattern_view.cursor_cell =
                (old_tick / self.pattern_view.division.spacing()) as usize;
            self.pattern_view.selected_event = None;
        }
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
                max_velocity: events
                    .iter()
                    .map(|event| event.raw.velocity())
                    .max()
                    .unwrap_or(0),
            })
            .collect();
        strip.all = cells.bank;
        strip
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario "Overview velocity levels keep event marks".
    #[test]
    fn overview_velocity_levels_keep_event_marks() {
        let hits = [
            (0, 1),
            (240, 42),
            (480, 43),
            (720, 84),
            (961, 85),
            (1200, 20),
            (1201, 127),
        ]
        .map(|(tick, velocity)| Hit::new(tick, 0, velocity, 0).unwrap());
        let raw = RawPattern::new(1, Grid::Off, 100, hits.to_vec()).unwrap();
        let before = raw.clone();
        let model = Model::new(&raw, None, Division::default()).unwrap();
        let (rows, _) = model.overview_page_cells(0, 6);
        assert_eq!(
            rows[0]
                .iter()
                .map(|cell| cell.max_velocity)
                .collect::<Vec<_>>(),
            [1, 42, 43, 84, 85, 127]
        );
        assert_eq!(
            rows[0].iter().map(|cell| cell.count).collect::<Vec<_>>(),
            [1, 1, 1, 1, 1, 2]
        );
        assert!(rows[0][4].off_grid);
        assert!(rows[0][5].off_grid);
        assert_eq!(raw, before);
    }

    /// Scenario "Overview shows all pad rows and source events".
    #[test]
    fn overview_shows_all_pad_rows_and_source_events() {
        let raw = RawPattern::new(
            1,
            Grid::Off,
            100,
            vec![
                Hit::new(0, 0, 100, 0).unwrap(),
                Hit::new(1, 0, 80, 0).unwrap(),
                Hit::new(120, 4, 90, 0).unwrap(),
                Hit::new(0, 20, 70, 0).unwrap(),
            ],
        )
        .unwrap();
        let before = raw.clone();
        let model = Model::new(&raw, None, Division::default()).unwrap();
        let bank1 = model.overview_cells(0);
        assert_eq!(bank1.len(), 16);
        assert!(bank1.iter().all(|row| row.len() == 16));
        assert_eq!(bank1[0][0].count, 2);
        assert!(bank1[0][0].off_grid);
        assert_eq!(bank1[4][0].count, 1);
        assert!(bank1[4][0].off_grid);
        assert!(bank1[1].iter().all(|cell| cell.count == 0));
        let bank2 = model.overview_cells(1);
        assert_eq!(bank2[4][0].count, 1);
        assert!(bank2[0].iter().all(|cell| cell.count == 0));
        let (page, active) = model.overview_page_cells(0, 4);
        assert_eq!(page.len(), 128);
        assert!(page.iter().all(|row| row.len() == 4));
        assert_eq!(page[20][0].count, 1);
        assert_eq!(active[0].count_ones(), 2);
        assert_eq!(active[1].count_ones(), 1);
        let (later, later_active) = model.overview_page_cells(4, 8);
        assert!(later.iter().flatten().all(|cell| cell.count == 0));
        assert_eq!(later_active, active, "bank counts cover the whole loop");
        assert_eq!(raw, before);
    }

    /// Scenario "Overview filters hidden hits before seam wrapping".
    #[test]
    fn overview_filters_hidden_hits_before_seam_wrapping() {
        let raw = RawPattern::new(
            1,
            Grid::Sixteenth,
            100,
            vec![
                Hit::new(3839, 0, 100, 0).unwrap(),
                Hit::new(4000, 1, 90, 0).unwrap(),
            ],
        )
        .unwrap();
        let model = Model::new(&raw, None, Division::default()).unwrap();
        let rows = model.overview_cells(0);
        assert_eq!(rows[0][0].count, 1);
        assert_eq!(rows[0].iter().map(|cell| cell.count).sum::<usize>(), 1);
        assert!(rows[1].iter().all(|cell| cell.count == 0));
        assert_eq!(raw.hits().len(), 2, "projection keeps hidden raw rows");
    }

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
