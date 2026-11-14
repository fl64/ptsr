//! Overview geometry at the supported terminal sizes.

use super::*;

/// Scenario "Overview fixed cells align every division".
#[test]
fn overview_fixed_cells_align_every_division() {
    for (width, height) in [(80u16, 26u16), (120, 40)] {
        for beat_cells in [1, 2, 3, 4, 6, 8, 12, 16] {
            let total = 4 * beat_cells;
            let (page, label_width) = overview_page_layout(width as usize, beat_cells, total);
            let mut rows = vec![vec![PatternCell::default(); page]; 128];
            rows[0][0].count = 1;
            let view = PatternOverview {
                header: "view 1/16".into(),
                rows,
                names: vec!["sample".into(); 128],
                active_pads: [1, 0, 0, 0, 0, 0, 0, 0],
                hide_empty: true,
                beat_cells,
                playhead: Some(beat_cells.min(page - 1)),
                first_cell: 0,
                total_cells: total,
                edit_cursor: Some(0),
                event_choice: None,
                event_velocity: None,
            };
            let mut pads = idle_pads();
            pads.selected = 0;
            let mut term = Terminal::new(TestBackend::new(width, height)).unwrap();
            term.draw(|frame| {
                pattern_overview(frame, Rect::new(0, 0, width, height), &pads, &view)
            })
            .unwrap();
            let buffer = term.backend().buffer();
            let mut x = label_width + 1;
            for cell in 0..page {
                if cell > 0 && cell.is_multiple_of(beat_cells) {
                    x += 1;
                }
                let number = format!("{:^3}", cell % (4 * beat_cells) + 1);
                let actual = (x..x + 3)
                    .map(|column| buffer[(column as u16, 2)].symbol())
                    .collect::<String>();
                assert_eq!(
                    actual, number,
                    "{width}x{height}, beat {beat_cells}, cell {cell}"
                );
                assert_eq!(
                    buffer[((x + 1) as u16, 4)].symbol(),
                    if cell == 0 { "■" } else { "·" },
                    "{width}x{height}, beat {beat_cells}, cell {cell}"
                );
                if cell == 0 {
                    assert_eq!(buffer[((x + 1) as u16, 4)].bg, CYAN);
                } else if view.playhead == Some(cell) {
                    assert_eq!(buffer[((x + 1) as u16, 4)].bg, PLATE);
                }
                x += 3;
            }
            assert!(x < width as usize);
            assert_eq!(buffer[(width - 1, 4)].symbol(), "┃");
        }
    }
}

/// Scenario "Overview velocity levels keep event marks".
#[test]
fn overview_velocity_levels_keep_event_marks() {
    for (width, height) in [(80u16, 26u16), (120, 40)] {
        let (page, label_width) = overview_page_layout(width as usize, 4, 16);
        let mut rows = vec![vec![PatternCell::default(); page]; 128];
        for (index, velocity) in [1, 42, 43, 84, 85, 127].into_iter().enumerate() {
            rows[0][index] = PatternCell {
                count: if index == 5 { 2 } else { 1 },
                off_grid: index == 4,
                max_velocity: velocity,
            };
        }
        let view = PatternOverview {
            header: "view 1/16".into(),
            rows,
            names: vec!["sample".into(); 128],
            active_pads: [1, 0, 0, 0, 0, 0, 0, 0],
            hide_empty: true,
            beat_cells: 4,
            playhead: None,
            first_cell: 0,
            total_cells: 16,
            edit_cursor: Some(6),
            event_choice: Some((1, 1)),
            event_velocity: Some(85),
        };
        let pads = idle_pads();
        let mut term = Terminal::new(TestBackend::new(width, height)).unwrap();
        term.draw(|frame| pattern_overview(frame, Rect::new(0, 0, width, height), &pads, &view))
            .unwrap();
        let buffer = term.backend().buffer();
        let title = (0..width)
            .map(|x| buffer[(x, 0)].symbol())
            .collect::<String>();
        assert!(title.contains("vel 85"), "{width}x{height}: {title}");
        for (index, (glyph, color)) in [
            ("■", DIM),
            ("■", DIM),
            ("■", TEXT),
            ("■", TEXT),
            ("~", WHITE),
            ("2", WHITE),
        ]
        .into_iter()
        .enumerate()
        {
            let x = label_width + 2 + 3 * index + usize::from(index >= 4);
            let cell = &buffer[(x as u16, 4)];
            assert_eq!(cell.symbol(), glyph, "{width}x{height}, cell {index}");
            assert_eq!(cell.fg, color, "{width}x{height}, cell {index}");
        }
    }
}

/// Scenario "Step cursor shows one time across pad rows".
#[test]
fn step_cursor_shows_one_time_across_pad_rows() {
    for (width, height) in [(80u16, 26u16), (120, 40)] {
        let (page, label_width) = overview_page_layout(width as usize, 4, 16);
        let mut rows = vec![vec![PatternCell::default(); page]; 128];
        rows[0][1].count = 1;
        rows[1][2].count = 1;
        let view = PatternOverview {
            header: "view 1/16".into(),
            rows,
            names: vec!["sample".into(); 128],
            active_pads: [0b11, 0, 0, 0, 0, 0, 0, 0],
            hide_empty: true,
            beat_cells: 4,
            playhead: Some(2),
            first_cell: 0,
            total_cells: 16,
            edit_cursor: Some(1),
            event_choice: None,
            event_velocity: None,
        };
        let mut pads = idle_pads();
        pads.selected = 1;
        let mut term = Terminal::new(TestBackend::new(width, height)).unwrap();
        term.draw(|frame| pattern_overview(frame, Rect::new(0, 0, width, height), &pads, &view))
            .unwrap();
        let buffer = term.backend().buffer();
        let cursor_x = (label_width + 1 + 3) as u16;
        for y in [2, 4, 5] {
            assert_eq!(buffer[(cursor_x, y)].bg, CYAN, "{width}x{height}, row {y}");
            assert_eq!(buffer[(cursor_x + 2, y)].bg, CYAN);
        }
        assert_eq!(buffer[(label_width as u16, 5)].symbol(), "▶");
        assert_eq!(buffer[(cursor_x, 5)].symbol(), " ");
        assert_eq!(buffer[(cursor_x + 2, 5)].symbol(), " ");
        assert_eq!(buffer[(cursor_x + 4, 4)].bg, PLATE);

        let mut overlap = view.clone();
        overlap.playhead = Some(1);
        term.draw(|frame| {
            pattern_overview(frame, Rect::new(0, 0, width, height), &pads, &overlap)
        })
        .unwrap();
        let buffer = term.backend().buffer();
        for y in [2, 4, 5] {
            assert_eq!(buffer[(cursor_x + 1, y)].bg, CYAN);
            assert_eq!(buffer[(cursor_x + 1, y)].fg, PLATE);
        }
    }
}
