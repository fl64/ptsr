//! Pattern screen geometry at the supported terminal sizes.

use super::*;
use crate::pattern_screen::overview_page_layout;
use ratatui::buffer::Buffer;

fn view(rows: Vec<Vec<PatternCell>>, beat_cells: usize, total: usize) -> PatternOverview {
    PatternOverview {
        header: String::new(),
        status: "Stopped".into(),
        length: "1 bar  120 BPM".into(),
        mode: None,
        state_label: None,
        next: None,
        rows,
        names: vec!["sample".into(); 128],
        active_pads: [1, 0, 0, 0, 0, 0, 0, 0],
        hide_empty: true,
        beat_cells,
        playhead: None,
        first_cell: 0,
        total_cells: total,
        editing: false,
        cursor: Some(0),
        cursor_hint: None,
        event_choice: None,
        event_velocity: None,
        detail: Some("step 1.1 · sample · no hit".into()),
        setup: "bars 1 · tempo 120".into(),
        empty: None,
    }
}

fn draw(width: u16, height: u16, pads: &Pads, view: &PatternOverview) -> Buffer {
    let mut term = Terminal::new(TestBackend::new(width, height)).unwrap();
    term.draw(|frame| pattern_overview(frame, Rect::new(0, 0, width, height), pads, view))
        .unwrap();
    term.backend().buffer().clone()
}

fn row(buffer: &Buffer, y: u16) -> String {
    (0..buffer.area.width)
        .map(|x| buffer[(x, y)].symbol())
        .collect()
}

/// The center column of a cell on the page.
fn center(label: usize, width: usize, page: usize, cell: usize) -> u16 {
    let cell_width = (width - label - 1) / page;
    (label + 1 + cell * cell_width + (cell_width - 1) / 2) as u16
}

/// Scenario "Overview fixed cells align every division".
#[test]
fn overview_fixed_cells_align_every_division() {
    for (width, height) in [(80u16, 26u16), (120, 40)] {
        for beat_cells in [1, 2, 3, 4, 6, 8, 12, 16] {
            let total = 4 * beat_cells;
            let (page, label) = overview_page_layout(width as usize, beat_cells, total);
            let cell_width = (width as usize - label - 1) / page;
            assert!(cell_width >= 3, "{width}: beat {beat_cells}");
            let mut rows = vec![vec![PatternCell::default(); page]; 128];
            rows[0][0] = PatternCell {
                count: 1,
                off_grid: false,
                max_velocity: 127,
            };
            let mut shown = view(rows, beat_cells, total);
            shown.playhead = Some(beat_cells.min(page - 1));
            let buffer = draw(width, height, &idle_pads(), &shown);
            assert!(row(&buffer, 0).contains("F5 PATTERN"));
            for cell in 0..page {
                let x = center(label, width as usize, page, cell);
                let number = if cell.is_multiple_of(beat_cells) {
                    (cell / beat_cells + 1).to_string()
                } else {
                    "·".into()
                };
                assert_eq!(
                    buffer[(x, 1)].symbol(),
                    number,
                    "{width}, beat {beat_cells}, {cell}"
                );
                let glyph = if cell == 0 { "█" } else { "·" };
                assert_eq!(
                    buffer[(x, 2)].symbol(),
                    glyph,
                    "{width}, beat {beat_cells}, {cell}"
                );
                if cell == 0 {
                    assert_eq!(buffer[(x, 2)].bg, CYAN);
                } else if shown.playhead == Some(cell) {
                    assert_eq!(buffer[(x, 2)].bg, PLATE);
                }
            }
        }
    }
}

/// Scenario "Pattern screen pages hold whole beats".
#[test]
fn pattern_screen_pages_hold_whole_beats() {
    for width in [80, 120] {
        // 1/16 shows one bar.
        assert_eq!(overview_page_layout(width, 4, 64).0, 16);
        for beat_cells in [1, 2, 3, 4, 6, 8, 12, 16] {
            let bar = 4 * beat_cells;
            let (page, label) = overview_page_layout(width, beat_cells, 2 * bar);
            assert!(
                page == bar || page == 2 * beat_cells || page == beat_cells,
                "{width}: {beat_cells} → {page}"
            );
            assert!(page.is_multiple_of(beat_cells));
            assert!(3 * page < width - label || page == beat_cells);
            if 3 * bar < width - label {
                assert_eq!(page, bar, "a bar that fits is one page");
            }
        }
    }
    // 1/32 is half a bar at 80 columns and one bar at 120 columns.
    assert_eq!(overview_page_layout(80, 8, 64).0, 16);
    assert_eq!(overview_page_layout(120, 8, 64).0, 32);
}

/// Scenario "Overview velocity levels keep event marks".
#[test]
fn overview_velocity_levels_keep_event_marks() {
    for (width, height) in [(80u16, 26u16), (120, 40)] {
        let (page, label) = overview_page_layout(width as usize, 4, 16);
        let mut rows = vec![vec![PatternCell::default(); page]; 128];
        for (index, velocity) in [1, 42, 43, 84, 85, 127].into_iter().enumerate() {
            rows[0][index] = PatternCell {
                count: if index == 5 { 2 } else { 1 },
                off_grid: index == 4,
                max_velocity: velocity,
            };
        }
        let mut shown = view(rows, 4, 16);
        shown.cursor = Some(6);
        shown.detail = Some("step 1.7 · sample · loud (vel 85)".into());
        let buffer = draw(width, height, &idle_pads(), &shown);
        let text = (0..height)
            .map(|y| row(&buffer, y))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("vel 85"), "{width}x{height}: {text}");
        for (index, (glyph, color)) in [
            ("▪", DIM),
            ("▪", DIM),
            ("▓", TEXT),
            ("▓", TEXT),
            ("~", WHITE),
            ("2", WHITE),
        ]
        .into_iter()
        .enumerate()
        {
            let cell = &buffer[(center(label, width as usize, page, index), 2)];
            assert_eq!(cell.symbol(), glyph, "{width}x{height}, cell {index}");
            assert_eq!(cell.fg, color, "{width}x{height}, cell {index}");
        }
    }
}

/// Scenario "Velocity lane follows the selected row".
#[test]
fn velocity_lane_follows_the_selected_row() {
    for (width, height) in [(80u16, 26u16), (120, 40)] {
        let (page, label) = overview_page_layout(width as usize, 4, 16);
        let mut rows = vec![vec![PatternCell::default(); page]; 128];
        rows[0][0] = PatternCell {
            count: 1,
            off_grid: false,
            max_velocity: 10,
        };
        rows[0][1] = PatternCell {
            count: 1,
            off_grid: false,
            max_velocity: 127,
        };
        rows[1][2] = PatternCell {
            count: 1,
            off_grid: false,
            max_velocity: 127,
        };
        let mut shown = view(rows, 4, 16);
        shown.active_pads = [0b11, 0, 0, 0, 0, 0, 0, 0];
        let lane = |selected: usize| {
            let mut pads = idle_pads();
            pads.selected = selected;
            let buffer = draw(width, height, &pads, &shown);
            let y = (0..height)
                .find(|&y| row(&buffer, y).starts_with(" velocity"))
                .unwrap();
            (0..3)
                .map(|cell| {
                    buffer[(center(label, width as usize, page, cell), y)]
                        .symbol()
                        .to_string()
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(lane(0), ["▁", "█", "·"], "{width}x{height}");
        assert_eq!(lane(1), ["·", "·", "█"], "{width}x{height}");
    }
}

/// Scenario "Step cursor shows one time across pad rows".
#[test]
fn step_cursor_shows_one_time_across_pad_rows() {
    for (width, height) in [(80u16, 26u16), (120, 40)] {
        let (page, label) = overview_page_layout(width as usize, 4, 16);
        let mut rows = vec![vec![PatternCell::default(); page]; 128];
        rows[0][1].count = 1;
        rows[1][2].count = 1;
        let mut shown = view(rows, 4, 16);
        shown.active_pads = [0b11, 0, 0, 0, 0, 0, 0, 0];
        shown.editing = true;
        shown.cursor = Some(1);
        shown.playhead = Some(2);
        let mut pads = idle_pads();
        pads.selected = 1;
        let buffer = draw(width, height, &pads, &shown);
        let cursor = center(label, width as usize, page, 1);
        for y in [1, 2, 3] {
            assert_eq!(buffer[(cursor, y)].bg, CYAN, "{width}x{height}, row {y}");
        }
        assert_eq!(buffer[(0, 3)].symbol(), "▶");
        assert_eq!(
            buffer[(center(label, width as usize, page, 2), 2)].bg,
            PLATE
        );

        let mut overlap = shown.clone();
        overlap.playhead = Some(1);
        let buffer = draw(width, height, &pads, &overlap);
        for y in [1, 2, 3] {
            assert_eq!(buffer[(cursor, y)].bg, CYAN);
            assert_eq!(buffer[(cursor, y)].fg, PLATE);
        }
    }
}
