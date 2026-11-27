use super::*;

/// Audio settings screen (F12).
pub struct Settings<'a> {
    /// Selected audio row: output, input, buffer, wave or click level. Paths has 0 projects,
    /// 1 samples.
    pub row: usize,
    /// The waves are drawn as kitty pictures from the next start (04b); Braille otherwise.
    pub graphics: bool,
    /// Device the audio actually goes to.
    pub output: Option<&'a str>,
    /// Saved choice that is not among the devices now.
    pub output_missing: Option<&'a str>,
    pub input: Option<&'a str>,
    pub input_missing: Option<&'a str>,
    /// Input peak, 0..1.
    pub input_level: f32,
    /// The actual buffer size the server runs.
    pub buffer_size: u32,
    /// The saved choice: 0 = auto (follow the server), else a fixed size.
    pub buffer_choice: u32,
    pub click_level: u8,
    pub click_encoder: bool,
    /// Device choice is available (JACK); not in cpal mode.
    pub routing: bool,
    /// The config asks for JACK: no routing then means JACK is down, not a wrong backend.
    pub backend_jack: bool,
    /// 0 audio, 1 input (MIDI devices and the input debug log), 2 MIDI, 3 paths (11).
    pub tab: usize,
    /// Connected MIDI devices: "port (profile)".
    pub midi_devices: &'a [String],
    /// Latest input events, newest last.
    pub input_log: &'a [String],
    /// MIDI tab: the groups with their actions under the open ones.
    pub midi_rows: &'a [MidiRow],
    pub midi_row: usize,
    /// The ready line about waiting for a control: a single learn or a run (28).
    pub learning: Option<&'a str>,
    /// Paths tab (11): the two roots as shown, `~`-shortened.
    pub paths: Option<(&'a str, &'a str)>,
    /// Paths tab (11): the open folder picker replaces the tab rows while it is open.
    pub picker: Option<PathPickerView>,
}

pub(super) fn draw(frame: &mut Frame, area: Rect, st: &Settings, s: &Status) {
    let [header, body] = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);
    let heading = title(
        vec![Span::styled(" F12 SETTINGS ", Style::new().fg(BG).bg(TEXT))],
        &s.badges,
        header.width,
        0,
    );
    frame.render_widget(heading, header);
    // the body in the edit frame, the tabs as its pages, the open one lit (98g)
    let names = ["AUDIO", "INPUT", "MIDI", "PATHS"];
    let page = |name: &'static str, open: bool| {
        if open {
            Span::styled(name, Style::new().fg(BG).bg(MAGENTA).bold())
        } else {
            name.fg(DIM)
        }
    };
    let mut tabs = vec![Span::styled(" SETTINGS ·", Style::new().fg(MAGENTA).bold())];
    for (i, name) in names.iter().enumerate() {
        tabs.push(" ".into());
        tabs.push(page(name, st.tab == i));
    }
    tabs.push(" ".into());
    if tabs.iter().map(Span::width).sum::<usize>() + 2 > usize::from(body.width) {
        tabs = vec![
            " ".into(),
            page(names.get(st.tab).copied().unwrap_or("SETTINGS"), true),
            " ".into(),
        ];
    }
    let block = Block::bordered()
        .border_style(Style::new().fg(MAGENTA))
        .title(Line::from(tabs));
    let inner = block.inner(body);
    frame.render_widget(block, body);
    let inner = if st.tab == 0 && inner.height > 0 {
        let [diagnostics, rest] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(inner);
        let text = format!(" CPU {:.0} % · xruns {}", s.load_peak, s.xruns);
        frame.render_widget(Line::from(text).fg(DIM), diagnostics);
        rest
    } else {
        inner
    };
    frame.render_widget(settings_body(st, inner.height), inner);
}

fn settings_body(st: &Settings, height: u16) -> Paragraph<'static> {
    // the selected row of an edit frame: bright with a marker, not inverse (98g)
    let label = |row: usize, name: &str| {
        let text = format!(
            "{} {:<8}",
            if st.row == row { "▸" } else { " " },
            name.to_lowercase()
        );
        if st.row == row {
            Span::styled(text, Style::new().fg(CYAN).bold())
        } else {
            text.fg(DIM)
        }
    };
    let device = |actual: Option<&str>, missing: Option<&str>| {
        let mut spans = vec![Span::from(format!(" ◀ {} ▶", actual.unwrap_or("none")))];
        if let Some(m) = missing {
            spans.push(format!("   {m} not found").fg(MAGENTA));
        }
        spans
    };

    let mut lines = Vec::new();
    if st.tab == 3 {
        if let Some(p) = &st.picker {
            lines.push(Line::from(Span::styled(
                p.title,
                Style::new().fg(BG).bg(TEXT),
            )));
            let visible = usize::from(height.saturating_sub(4));
            let start = (p.selected + 1).saturating_sub(visible);
            for (i, name) in p.entries.iter().enumerate().skip(start).take(visible) {
                let row = format!("   {name}");
                lines.push(Line::from(if i == p.selected {
                    Span::styled(row, Style::new().fg(BG).bg(CYAN))
                } else {
                    row.into()
                }));
            }
            lines.push(Line::default());
            lines.push(" knob 1/↑↓ select · LEFT/RIGHT folder".fg(DIM).into());
            lines.push(" Enter/P4 pick · Esc/P1 cancel".fg(DIM).into());
            return Paragraph::new(lines);
        }
        if let Some((projects, samples)) = st.paths {
            lines.push(Line::from(vec![
                label(0, "Projects root"),
                Span::from(format!(" {projects}")),
            ]));
            lines.push(Line::from(vec![
                label(1, "Samples root"),
                Span::from(format!(" {samples}")),
            ]));
        }
        return Paragraph::new(lines);
    }
    if st.tab == 2 {
        lines.push(match st.learning {
            Some(l) => Span::styled(l.to_string(), Style::new().fg(MAGENTA)).into(),
            None => {
                let hint = match st.midi_rows.get(st.midi_row) {
                    Some(MidiRow::Action { label, .. }) if label == "hold time" => {
                        " J/L: change hold time by 50 ms. Saves at once."
                    }
                    Some(MidiRow::Action { label, .. }) if label == "SMC-PAD PAD BANK" => {
                        " J/L: select Off or Hold panel. Saves at once."
                    }
                    Some(MidiRow::Action { label, .. }) if label == "knob response" => {
                        " J/L: change knob response. Saves at once."
                    }
                    _ => {
                        // Bindings change from the keyboard only (138).
                        " Bindings: keyboard only (F10, Enter) · Bksp clears"
                    }
                };
                hint.fg(DIM).into()
            }
        });
        let width = st
            .midi_rows
            .iter()
            .map(|r| match r {
                MidiRow::Action { label, .. } => label.len(),
                MidiRow::Group { .. } => 0,
            })
            .max()
            .unwrap_or(0);
        let mut selected_line = 0;
        let groups = st
            .midi_rows
            .iter()
            .filter(|row| matches!(row, MidiRow::Group { .. }))
            .count();
        let needed = lines.len() + st.midi_rows.len() + groups;
        let omit_from = groups.saturating_sub(needed.saturating_sub(height as usize));
        let mut group_at = 0;
        for (i, row) in st.midi_rows.iter().enumerate() {
            match row {
                MidiRow::Group { name, counts } => {
                    if group_at < omit_from {
                        lines.push(Line::default());
                    }
                    group_at += 1;
                    lines.push(Line::from(if i == st.midi_row {
                        selected_line = lines.len();
                        Span::styled(format!("▸{name} · {counts}"), Style::new().fg(CYAN).bold())
                    } else {
                        format!(" {name} · {counts}").bold()
                    }));
                }
                MidiRow::Action {
                    label,
                    who,
                    taken,
                    stays,
                } => {
                    let label = format!(
                        "{}  {label:<width$} ",
                        if i == st.midi_row { "▸" } else { " " }
                    );
                    let label = match (i == st.midi_row, taken) {
                        (true, _) => {
                            selected_line = lines.len();
                            Span::styled(label, Style::new().fg(CYAN).bold())
                        }
                        (false, true) => label.fg(MAGENTA),
                        (false, false) => label.fg(DIM),
                    };
                    let who = format!("  {who}");
                    let mut spans = vec![label];
                    match (taken, who.contains(" → ")) {
                        (true, _) => spans.push(Span::styled(who, Style::new().fg(MAGENTA).bold())),
                        (false, true) => spans.extend(learn_spans(&who)),
                        (false, false) => spans.push(who.into()),
                    }
                    lines.push(Line::from(spans));
                    let pad = " ".repeat(width + 6);
                    lines.extend(
                        stays
                            .iter()
                            .map(|s| Line::from(format!("{pad}{s}").fg(DIM))),
                    );
                }
            }
        }
        let offset = (selected_line + 2).saturating_sub(usize::from(height));
        if st.learning.is_some() {
            // Keep the pending control and its next action above the scrolled list.
            let scrolled = offset.min(lines.len() - 1);
            lines.drain(1..1 + scrolled);
            return Paragraph::new(lines);
        }
        return Paragraph::new(lines).scroll((offset as u16, 0));
    }
    if st.tab == 1 {
        lines.push(" MIDI devices".fg(DIM).into());
        if st.midi_devices.is_empty() {
            lines.push("   none connected".into());
        }
        lines.extend(
            st.midi_devices
                .iter()
                .map(|d| Line::from(format!("   {d}"))),
        );
        lines.push(Line::default());
        lines.push(" Latest input".fg(DIM).into());
        lines.extend(st.input_log.iter().map(|l| Line::from(format!("   {l}"))));
        return Paragraph::new(lines);
    }
    if st.routing {
        let mut out = vec![label(0, "Output")];
        out.extend(device(st.output, st.output_missing));
        let mut inp = vec![label(1, "Input")];
        inp.extend(device(st.input, st.input_missing));
        lines.push(Line::from(out));
        lines.push(Line::from(inp));
        lines.push(Line::from(vec![
            " ".repeat(10).into(),
            meter(st.input_level),
        ]));
    } else {
        let msg = if st.backend_jack {
            " JACK is not available right now."
        } else {
            " Device choice needs JACK: set backend = \"jack\" in the config."
        };
        lines.push(msg.fg(MAGENTA).into());
        lines.push(Line::default());
        lines.push(Line::default());
    }
    let shown = if st.buffer_choice == 0 {
        format!("auto ({})", st.buffer_size)
    } else {
        format!("{}", st.buffer_choice)
    };
    lines.push(Line::from(vec![
        label(2, "Buffer"),
        Span::from(format!(" ◀ {shown} ▶")),
    ]));
    lines.push(
        " auto: follow the server — smaller if pads feel late, bigger if clicks"
            .fg(DIM)
            .into(),
    );
    lines.push(Line::from(vec![
        label(3, "Wave"),
        Span::from(format!(
            " ◀ {} ▶",
            if st.graphics { "picture" } else { "Braille" }
        )),
    ]));
    lines.push(
        " picture: kitty graphics, from the next start; Braille is lighter"
            .fg(DIM)
            .into(),
    );
    lines.push(Line::from(vec![
        label(4, "Click level"),
        Span::from(format!(" ◀ {} % ▶", st.click_level)),
    ]));
    if st.row == 4 && st.click_encoder {
        lines.push(" SMC-PAD E2 changes the level".fg(DIM).into());
    }
    Paragraph::new(lines)
}
