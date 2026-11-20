//! File browser and project picker drawing.

use super::*;

/// The project picker of Ctrl+P (11), drawn over the whole body in place of the screen
/// (as [`PathPickerView`] takes the rows of the Paths tab in place of theirs).
pub(super) fn project_picker(p: &ProjectPickerView, s: &Status, width: u16) -> Paragraph<'static> {
    let mut lines = vec![title(
        vec![Span::styled(" PROJECTS ", Style::new().fg(BG).bg(TEXT))],
        &s.badges,
        width,
        0,
    )];
    if p.entries.is_empty() {
        lines.push("no projects yet — type a name".fg(DIM).into());
    }
    let empty = p.entries.is_empty();
    for (i, name) in p.entries.iter().enumerate() {
        let row = format!("   {}", clip_name(name, width.saturating_sub(3) as usize));
        lines.push(Line::from(if i == p.selected {
            Span::styled(row, Style::new().fg(BG).bg(CYAN))
        } else {
            row.into()
        }));
    }
    lines.push(Line::default());
    match &p.naming {
        Some(name) => {
            lines.push(format!("name: {name}").into());
            lines.push(" Enter/P4 create · Esc/P1 back to list".fg(DIM).into());
        }
        // after SMC-PAD input the hint names the held button with the pads (F32)
        // with no project, pad 4 makes one with a date name (138)
        None if crate::pads::smc_words() && empty => {
            lines.push(
                " PAD BANK + 4 new project (date name) · PAD BANK + 1 close"
                    .fg(DIM)
                    .into(),
            );
        }
        None if crate::pads::smc_words() => {
            lines.push(
                " knob 1 select · PAD BANK + 4 open · PAD BANK + 1 close"
                    .fg(DIM)
                    .into(),
            );
        }
        None if empty => {
            lines.push(
                " type a name · Enter/P4 date name · Esc/P1 close"
                    .fg(DIM)
                    .into(),
            );
        }
        None => {
            lines.push(
                " knob 1/↑↓ select · Enter/P4 open · type a name · Esc/P1 close"
                    .fg(DIM)
                    .into(),
            );
        }
    }
    Paragraph::new(lines)
}

pub(super) fn draw(frame: &mut Frame, area: Rect, f: &Files, s: &Status) {
    let pf = pad_frame(area, f.pads.hidden);
    // the title carries what the screen says: the file in hand or the error (31)
    let note = match (f.in_hand, f.error) {
        (Some(name), _) if pads::smc_words() => {
            format!("   In hand: {name}. Hit a pad. PAD BANK + 1 puts it back.")
                .fg(MAGENTA)
        }
        (Some(name), _) => format!(
            "   In hand: {name}. Arrows choose a pad, Enter puts it there (or hit a pad). Esc puts it back."
        )
        .fg(MAGENTA),
        (None, Some(e)) => format!("   {e}").fg(MAGENTA),
        (None, None) => "".into(),
    };
    let reserve = note.width().min(20);
    let mut heading = title(
        vec![Span::styled(" F6 FILES ", Style::new().fg(BG).bg(TEXT))],
        &s.badges,
        pf.title.width,
        reserve,
    );
    heading.spans.push(note);
    frame.render_widget(heading, pf.title);
    // the wave of the selected file over the whole strip, its length on the first row
    if let Some((secs, wave)) = f.preview {
        // the file the wave is of, next to its length (66)
        let suffix = format!(" · {secs:.2} s ");
        let label = match f.entries.get(f.selected) {
            Some((name, false, _)) => format!(
                " {}{suffix}",
                clip_name(
                    name,
                    (pf.strip.width.saturating_sub(1) as usize).saturating_sub(suffix.len())
                )
            ),
            _ => format!(" {secs:.2} s "),
        };
        // the wave over the whole strip, the label over the left end of its top row: a
        // long file name beside the wave pushed every row right
        let area = pf.strip;
        let cols = area.width as usize;
        if cols >= 2 && area.height > 0 {
            let rows = braille(wave, cols, area.height as usize);
            let lines: Vec<Line> = rows
                .iter()
                .map(|r| Line::from(r.clone().fg(CYAN)))
                .collect();
            frame.render_widget(Paragraph::new(lines), area);
            let width = Span::raw(label.as_str()).width().min(area.width as usize) as u16;
            frame.render_widget(
                Line::from(Span::styled(label, Style::new().fg(CYAN).bg(BG))),
                Rect {
                    width,
                    height: 1,
                    ..area
                },
            );
            // the thin green line of the chop playhead, over every row of the wave
            if let Some(at) = f.playhead {
                let x = area.x + ((at.clamp(0.0, 1.0) * cols as f32) as u16).min(cols as u16 - 1);
                for y in area.y..area.bottom() {
                    if let Some(cell) = frame.buffer_mut().cell_mut((x, y)) {
                        cell.set_symbol("│").set_fg(METER);
                    }
                }
            }
        }
    }

    let bank_spans = if f.in_hand.is_some() {
        vec![
            Span::styled(" hit a pad ↓ ", Style::new().fg(BG).bg(MAGENTA)),
            " ".into(),
        ]
    } else {
        Vec::new()
    };
    if f.pads.hidden {
        frame.render_widget(pad_line(&f.pads, bank_spans, true), pf.bank);
    } else {
        frame.render_widget(Line::from([bank_spans, banks(&f.pads)].concat()), pf.bank);
        pad_grid(frame, pf.grid, &f.pads, pf.waves);
    }

    // the folder and the filter stand right above the list they name
    let [path, list] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(pf.side);
    let filter = match (f.editing, f.filter.is_empty()) {
        (true, _) => Span::styled(
            format!(" filter: {}▌ ", f.filter),
            Style::new().fg(BG).bg(CYAN),
        ),
        (false, false) => format!(" filter: {} · Esc clears  ", f.filter).fg(CYAN),
        (false, true) => "".into(),
    };
    let folder_room = (path.width as usize).saturating_sub(filter.width() + 1);
    frame.render_widget(
        // the filter before the folder: a long path does not push it out of sight (66)
        Line::from(vec![
            filter,
            format!(" {}", clip_name(f.dir, folder_room)).fg(DIM),
        ]),
        path,
    );

    let mut lines = Vec::new();
    if f.entries.is_empty() {
        let why = if f.filter.is_empty() {
            "here"
        } else {
            "matching the filter"
        };
        lines.push(Line::from(format!(" no folders or audio files {why}")).fg(DIM));
    }
    for (i, (name, is_dir, favorite)) in f.entries.iter().enumerate() {
        let in_hand = !is_dir && f.in_hand == Some(*name);
        let (prefix, suffix) = match (*is_dir, in_hand) {
            (true, _) if *favorite => (" ★ ", "/ "),
            (true, _) => (" ", "/ "),
            (false, true) => (" ", "  ← in hand "),
            (false, false) => (" ", " "),
        };
        let room = (list.width as usize)
            .saturating_sub(Span::raw(prefix).width() + Span::raw(suffix).width());
        let text = format!("{prefix}{}{suffix}", clip_name(name, room));
        lines.push(match (i == f.selected, in_hand) {
            (_, true) => Line::from(Span::styled(text, Style::new().fg(BG).bg(MAGENTA))),
            (true, false) => Line::from(Span::styled(text, Style::new().fg(BG).bg(CYAN))),
            (false, false) if *is_dir => Line::from(text).fg(TEXT),
            (false, false) => Line::from(text).fg(DIM),
        });
    }
    // keep the selection on screen
    let offset = (f.selected as u16 + 1).saturating_sub(list.height);
    frame.render_widget(Paragraph::new(lines).scroll((offset, 0)), list);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    fn picker_text(p: &ProjectPickerView) -> String {
        let status = Status {
            badges: Vec::new(),
            save: SaveStatus::Clean,
            backend: None,
            recording: false,
            buffer_size: 0,
            sample_rate: 48_000,
            load_peak: 0.0,
            xruns: 0,
            master_db: 0,
            meters: ([-60.0, -60.0], [-60.0, -60.0], None),
            no_releases: false,
            notice: None,
            project: None,
        };
        let mut term = Terminal::new(TestBackend::new(80, 10)).unwrap();
        term.draw(|frame| {
            frame.render_widget(project_picker(p, &status, 80), frame.area());
        })
        .unwrap();
        term.backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect()
    }

    /// Scenario "The picker hint names PAD BANK" (controller-reach): after SMC-PAD input the
    /// picker names PAD BANK with pads 4 and 1 (F32); with no project, pad 4 makes one (F03).
    #[test]
    fn the_picker_hint_names_pad_bank() {
        let view = ProjectPickerView {
            entries: vec!["MyBeat".to_string()],
            selected: 0,
            naming: None,
        };
        crate::pads::set_smc_words(true);
        let smc = picker_text(&view);
        crate::pads::set_smc_words(false);
        let keys = picker_text(&view);
        assert!(
            smc.contains("PAD BANK + 4 open · PAD BANK + 1 close"),
            "{smc}"
        );
        assert!(keys.contains("Enter/P4 open"), "{keys}");
        let empty = ProjectPickerView {
            entries: Vec::new(),
            selected: 0,
            naming: None,
        };
        crate::pads::set_smc_words(true);
        let smc = picker_text(&empty);
        crate::pads::set_smc_words(false);
        assert!(smc.contains("PAD BANK + 4 new project"), "{smc}");
        assert!(smc.contains("PAD BANK + 1 close"), "{smc}");
    }
}
