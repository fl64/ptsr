//! Pads screen and pad edit drawing.

use super::*;

mod source;
pub use source::{BigView, SourceRange, SourceSample, SourceStrip, SourceVoice};

/// The current command labels for the SMC-PAD second bank.
pub struct CommandPanel {
    pub labels: [Option<String>; 16],
    /// 0 is the screens view; 1 is the actions of the current view (127).
    pub page: usize,
    pub view: &'static str,
    /// A target view (Copy, Assign and others) instead of the screens or action pages.
    pub target: bool,
    /// The cell index that a pad hit lights now (131).
    pub flash: Option<usize>,
    /// Cells whose action cannot run now, drawn grey; a press explains why (135, 138).
    pub dim: [bool; 16],
    /// Cells whose action replaces or removes the owner's work, drawn orange (138).
    pub warn: [bool; 16],
    /// The cell of the current choice among its kind, drawn brighter: the pad edit page (138).
    pub current: [bool; 16],
    /// The current state of a state cell, shown under its label (138).
    pub states: [Option<&'static str>; 16],
}

/// The orange of a cell that replaces or removes work (138): apart from the amber bound
/// and the yellow marks.
const ORANGE: Color = Color::Rgb(0xFF, 0x8A, 0x3D);

/// The number that the device prints on the pad of top-down `cell` 1–16.
pub fn device_number(cell: usize) -> usize {
    let (row, column) = ((cell - 1) / 4, (cell - 1) % 4);
    (3 - row) * 4 + column + 1
}

/// The pad keys in grid order, top left first, as the program maps them.
const KEYS: [char; 16] = [
    '1', '2', '3', '4', 'q', 'w', 'e', 'r', 'a', 's', 'd', 'f', 'z', 'x', 'c', 'v',
];

thread_local! {
    /// With an SMC-PAD connection, pads read by the numbers printed on the device (134).
    static DEVICE_NUMBERS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub fn set_device_numbers(on: bool) {
    DEVICE_NUMBERS.set(on);
}

pub fn device_numbers() -> bool {
    DEVICE_NUMBERS.get()
}

thread_local! {
    /// The last input came from the SMC-PAD: texts name its controls, not keys (134).
    static SMC_WORDS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub fn set_smc_words(on: bool) {
    SMC_WORDS.set(on);
}

pub fn smc_words() -> bool {
    SMC_WORDS.get()
}

thread_local! {
    /// The knobs that turn the rows of a pad edit page, top down; `None` counts 1, 2, 3.
    static ROW_KNOBS: std::cell::Cell<Option<[u8; 4]>> = const { std::cell::Cell::new(None) };
}

pub fn set_row_knobs(knobs: Option<[u8; 4]>) {
    ROW_KNOBS.set(knobs);
}

thread_local! {
    /// The knobs that turn the rows of the selected FX slot; `None` counts 1, 2, 3.
    static FX_KNOBS: std::cell::Cell<Option<[u8; 3]>> = const { std::cell::Cell::new(None) };
}

pub fn set_fx_knobs(knobs: Option<[u8; 3]>) {
    FX_KNOBS.set(knobs);
}

/// The number on FX row `index`: with SMC-PAD knobs only the selected slot has one.
pub(super) fn fx_row_knob(index: usize, selected: bool) -> Option<usize> {
    match FX_KNOBS.get() {
        Some(knobs) => selected
            .then(|| knobs.get(index).copied().map(usize::from))
            .flatten(),
        None => Some(index + 1),
    }
}

/// The knob number shown on page row `index`.
fn row_knob(index: usize) -> usize {
    ROW_KNOBS
        .get()
        .and_then(|knobs| knobs.get(index).copied())
        .map_or(index + 1, usize::from)
}

/// The name of the pad at grid `index` (0 is top left): its device number with an
/// SMC-PAD connection, otherwise its key.
pub fn pad_tag(index: usize) -> String {
    let index = index % 16;
    if device_numbers() {
        device_number(index + 1).to_string()
    } else {
        KEYS[index].to_ascii_uppercase().to_string()
    }
}

/// [`pad_tag`] for a pad known by its key.
pub fn key_tag(key: char) -> String {
    match KEYS.iter().position(|k| *k == key.to_ascii_lowercase()) {
        Some(index) => pad_tag(index),
        None => key.to_uppercase().to_string(),
    }
}

/// The header: the shown view in the middle, its neighbors on the left and right.
fn panel_header(panel: &CommandPanel) -> String {
    if panel.target {
        return format!(" PANEL │ {}", panel.view);
    }
    if panel.page == 0 {
        " · │ SCREENS │ actions ›".to_string()
    } else {
        format!(" ‹ screens │ {} actions │ ·", panel.view)
    }
}

pub fn command_panel_area(area: Rect) -> Rect {
    let [_, body, _, _] = frame_areas(area);
    pad_frame(body, false).grid
}

pub fn command_panel(frame: &mut Frame, panel: &CommandPanel, question_view: Option<&Question>) {
    command_panel_in(frame, frame.area(), panel, question_view);
}

pub fn command_panel_in(
    frame: &mut Frame,
    content: Rect,
    panel: &CommandPanel,
    question_view: Option<&Question>,
) {
    let [_, body, _, _] = frame_areas(content);
    let pad = pad_frame(body, false);
    // The screen title and its badges stay; the header below names the panel (134).
    // The bank strip under the header is wider than the header text.
    frame.render_widget(Clear, pad.bank);
    let accent = if panel.page == 0 && !panel.target {
        MAGENTA
    } else {
        CYAN
    };
    frame.render_widget(
        Line::from(panel_header(panel)).fg(accent).bg(BG).bold(),
        pad.bank,
    );
    let rows = Layout::vertical([Constraint::Length(PAD_H + pad.waves); 4]).split(pad.grid);
    for (row, area) in rows.iter().enumerate() {
        let cells = Layout::horizontal([Constraint::Length(pad.grid.width / 4); 4]).split(*area);
        for (column, area) in cells.iter().enumerate() {
            let index = row * 4 + column;
            let label = panel.labels[index].as_deref().unwrap_or("");
            let active = panel.labels[index].is_some()
                && !panel.dim[index]
                && !matches!(label, "mixer" | "redo");
            // Three zones: screens, the actions of the current view, the fixed bottom row.
            // An action that removes work is orange, the current page brighter (138).
            let zone = if row == 3 {
                TEXT
            } else if panel.warn[index] {
                ORANGE
            } else if panel.current[index] {
                WHITE
            } else if panel.page == 0 && !panel.target {
                MAGENTA
            } else {
                CYAN
            };
            let lit = panel.flash == Some(index);
            let color = match (lit, active) {
                (true, _) => BG,
                (false, true) => zone,
                (false, false) => DIM,
            };
            // A hit lights the cell: its own color when active, grey when dim.
            let fill = match (lit, active) {
                (true, true) => zone,
                (true, false) => DIM,
                (false, _) => PANEL,
            };
            let cell = Block::bordered()
                .title(Line::from(device_number(index + 1).to_string()).fg(if lit {
                    BG
                } else {
                    DIM
                }))
                .border_style(Style::new().fg(if active { zone } else { BORDER }))
                .style(Style::new().bg(fill));
            let inner = cell.inner(*area);
            frame.render_widget(Clear, *area);
            frame.render_widget(cell, *area);
            let lines = cell_lines(label, panel.states[index], inner.height);
            let top = inner.y + inner.height.saturating_sub(lines.len() as u16) / 2;
            for (i, (line, state)) in lines.into_iter().enumerate() {
                let at = Rect {
                    y: top + i as u16,
                    height: 1,
                    ..inner
                };
                // the state reads apart from the label: its own colour, not bold (138)
                let line = if state {
                    Line::from(line).fg(match (lit, active) {
                        (true, _) => BG,
                        (false, true) => METER,
                        (false, false) => DIM,
                    })
                } else {
                    Line::from(line).fg(color).bold()
                };
                if at.y < inner.y + inner.height {
                    frame.render_widget(line.centered(), at);
                }
            }
        }
    }
    if let Some(question_view) = question_view {
        question(frame, body, question_view);
    }
}

/// The lines of a panel cell in `height` rows, each with `true` for the state line (138).
/// Two words take two lines; a state takes the line under them, or under the whole label
/// when only two rows fit.
fn cell_lines<'a>(label: &'a str, state: Option<&'a str>, height: u16) -> Vec<(&'a str, bool)> {
    let words = label.split_once(' ');
    let mut lines = match (words, state) {
        (Some((first, second)), None) if height > 1 => vec![first, second],
        (Some((first, second)), Some(_)) if height > 2 => vec![first, second],
        _ => vec![label],
    };
    let count = lines.len();
    let mut out: Vec<(&str, bool)> = lines.drain(..).map(|line| (line, false)).collect();
    if let Some(state) = state.filter(|_| usize::from(height) > count) {
        out.push((state, true));
    }
    out
}

pub(super) fn draw(frame: &mut Frame, area: Rect, p: &Pads, s: &Status) {
    let f = pad_frame(area, p.hidden);
    let (badge, badge_style) = if p.playing {
        (" PLAY ▶   ", Style::new().fg(BG).bg(CYAN))
    } else {
        ("          ", Style::new().fg(DIM).bg(BG))
    };
    frame.render_widget(
        title(
            vec![
                Span::styled(" F2 PADS ", Style::new().fg(BG).bg(TEXT)),
                " ".into(),
                Span::styled(badge, badge_style),
            ],
            &s.badges,
            f.title.width,
            0,
        ),
        f.title,
    );
    if p.hidden {
        frame.render_widget(pad_line(p, Vec::new(), true), f.bank);
    } else {
        frame.render_widget(Line::from(banks(p)), f.bank);
    }
    draw_strip(frame, f.strip, p, s);
    let [meters_area, _, info] = Layout::vertical([
        Constraint::Length(METER_ROWS),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(f.side);
    meters(frame, meters_area, s);
    if !p.hidden {
        pad_grid(frame, f.grid, p, f.waves);
    }

    draw_side(frame, info, p);
}

fn draw_strip(frame: &mut Frame, strip: Rect, p: &Pads, s: &Status) {
    if !s.recording
        && p.edit.is_none()
        && let Some(source) = &p.source
    {
        source::draw(frame, strip, source);
        return;
    }
    // a three-row strip holds only the rhythm; the L/R meters still show the level
    let wave = if let Some(pattern) = &p.pattern
        && p.edit.is_none()
        && !s.recording
        && strip.height >= 3
    {
        let tracks = pattern
            .tracks
            .len()
            .min(usize::from(strip.height.saturating_sub(4))) as u16;
        let events_h = 3 + tracks;
        let wave = Rect {
            height: strip.height - events_h,
            ..strip
        };
        let events = Rect {
            y: wave.bottom(),
            height: events_h,
            ..strip
        };
        pattern_strip(frame, events, pattern);
        wave
    } else {
        strip
    };
    match (p.edit, &p.big) {
        _ if s.recording => envelope_strip(frame, strip, &p.out_wave, MAGENTA),
        (Some(_), _) if p.curve.is_some() => filter_strip(
            frame,
            strip,
            p.curve.as_ref().unwrap_or(&FilterCurve::default()),
        ),
        (Some(_), Some(big)) => {
            let gate = p.pads.get(p.selected).is_some_and(|v| v.gate);
            pad_envelope_strip(frame, strip, big, p.param, gate)
        }
        _ if wave.height == 0 => {}
        _ if p.splash.is_some() => splash_strip(frame, wave, p.splash.as_ref().unwrap()),
        _ => envelope_strip(frame, wave, &p.out_wave, DIM),
    }
}

fn draw_side(frame: &mut Frame, info: Rect, p: &Pads) {
    if let Some(sel) = p.pads.get(p.selected) {
        let [panel, _] =
            Layout::vertical([Constraint::Length(PANEL_ROWS), Constraint::Fill(1)]).areas(info);
        let lines = vec![
            Line::from(vec![
                "pad ".fg(DIM),
                Span::styled(key_tag(sel.key), Style::new().fg(CYAN).bold()),
                format!("  bank {}", p.bank + 1).fg(DIM),
                // the closed panel says how to open the edit; the edit has its frame (70)
                if p.edit.is_none() && !p.edit_key.is_empty() {
                    format!("   {}: edit", p.edit_key).fg(DIM)
                } else {
                    "".into()
                },
            ]),
            // what the next change hits, on top: under the blocks it read as a part of the
            // last one (70); the same word as the help line (68)
            Line::from(vec!["  scope   ".fg(DIM), p.scope.clone().fg(CYAN)]),
            Line::from({
                let detail = sel.detail.clone().unwrap_or_else(|| sel.name.clone());
                let width = panel.width.saturating_sub(2) as usize;
                if let Some((name, source)) = detail.rsplit_once(" · ") {
                    let source = format!(" · {source}");
                    format!(
                        "{}{}",
                        clip_name(name, width.saturating_sub(Span::raw(&source).width())),
                        source
                    )
                } else {
                    clip_name(&detail, width)
                }
            }),
            Line::from(vec![
                if panel.width < 28 {
                    "  mode   "
                } else {
                    "  mode    "
                }
                .fg(DIM),
                mode(sel.gate).into(),
                " ".fg(DIM),
                Span::styled(
                    if sel.pitch == 0 {
                        ""
                    } else {
                        sel.pitch_gate.label()
                    },
                    Style::new().fg(if sel.pitch_gate == PitchGate::A1 {
                        TEXT
                    } else {
                        DIM
                    }),
                ),
            ]),
            Line::from(format!(
                "  {}",
                if panel.width < 30 && sel.pitch_gate == PitchGate::NeedsSlice {
                    "PG on: needs slice"
                } else {
                    sel.pitch_gate.description()
                }
            ))
            .fg(if sel.pitch_gate == PitchGate::Off {
                DIM
            } else {
                TEXT
            }),
            param_line(
                p,
                PadParam::Volume,
                "volume  ",
                format!("{:+.0} dB", sel.volume_db),
            ),
            param_line(
                p,
                PadParam::Pitch,
                "pitch   ",
                format!("{:+} st", sel.pitch),
            ),
            param_line(
                p,
                PadParam::Group,
                "mute    ",
                if sel.group == 0 {
                    "off".to_string()
                } else {
                    sel.group.to_string()
                },
            ),
        ];
        // every page at once (43), in blocks under their names (48): a page change moves
        // only the knob numbers and the lit name
        let width = panel.width.saturating_sub(2) as usize;
        let block = |name: &str, folded: bool| -> Line<'static> {
            // a block with a change carries a dot (55)
            let params: &[PadParam] = match name {
                "TONE" => &[PadParam::Volume, PadParam::Pitch, PadParam::Group],
                "ENV" => &[PadParam::Attack, PadParam::Decay, PadParam::Release],
                "FILTER" => &[PadParam::Filter, PadParam::Cutoff, PadParam::Reso],
                "FX" => &[PadParam::Drive, PadParam::Send1, PadParam::Send2],
                _ => &[PadParam::Source, PadParam::Start, PadParam::End],
            };
            let mut name = if params.iter().any(|w| sel.changed.contains(w)) {
                format!("{name} ●")
            } else {
                name.to_string()
            };
            if folded {
                name.push_str(" ▸");
            }
            let rule = "─".repeat(width.saturating_sub(name.chars().count() + 3));
            let lit = p.edit.is_some_and(|e| name.starts_with(e));
            let style = if lit {
                Style::new().fg(MAGENTA).bold()
            } else {
                Style::new().fg(DIM)
            };
            Line::from(Span::styled(format!("─ {name} {rule}"), style))
        };
        // the blocks under the pad rows: (name, its parameters, its rows)
        let mut blocks: Vec<(&str, Vec<PadParam>, Vec<Line>)> = vec![
            (
                "ENV",
                vec![PadParam::Attack, PadParam::Decay, PadParam::Release],
                vec![
                    param_line(
                        p,
                        PadParam::Attack,
                        "attack  ",
                        format!("{} ms", sel.attack_ms),
                    ),
                    param_line(
                        p,
                        PadParam::Decay,
                        "decay   ",
                        format!("{} ms", sel.decay_ms),
                    ),
                    release_line(p, sel),
                ],
            ),
            (
                "FILTER",
                vec![PadParam::Filter, PadParam::Cutoff, PadParam::Reso],
                vec![
                    param_line(p, PadParam::Filter, "filter  ", sel.filter[0].clone()),
                    param_line(p, PadParam::Cutoff, "cutoff  ", sel.filter[1].clone()),
                    param_line(p, PadParam::Reso, "reso    ", sel.filter[2].clone()),
                ],
            ),
            (
                "SOURCE",
                vec![PadParam::Source, PadParam::Start, PadParam::End],
                vec![
                    param_line(p, PadParam::Source, "source  ", sel.source.clone()),
                    param_line(
                        p,
                        PadParam::Start,
                        "start   ",
                        format!("{:.3} s", sel.bounds.0),
                    ),
                    param_line(
                        p,
                        PadParam::End,
                        "end     ",
                        format!("{:.3} s", sel.bounds.1),
                    ),
                ],
            ),
        ];
        // Only drive and sends belong to the pad. Shared slots have their own screen (82).
        if !p.fx.is_empty() {
            blocks.push((
                "FX",
                p.fx.iter().map(|r| r.0).collect(),
                p.fx.iter()
                    .map(|(w, label, value)| param_line(p, *w, label, value.clone()))
                    .collect(),
            ));
        }
        // the pad rows: TONE stands under the mode
        let mut lines = lines;
        let tone = lines.split_off(5);
        // the open page stays unfolded; with the edit closed, the block of the picked one
        let open = |name: &str, params: &[PadParam]| match p.edit {
            Some(e) => e == name,
            None => params.contains(&p.param),
        };
        let tone_params = [PadParam::Volume, PadParam::Pitch, PadParam::Group];
        // the pickup hint only while there is one, it is the last row
        let mut tail = Vec::new();
        if let Some(hint) = &p.pickup {
            tail.push(Line::from(hint.clone()).fg(MAGENTA));
        }
        let full = lines.len()
            + 1
            + tone.len()
            + blocks.iter().map(|b| 1 + b.2.len()).sum::<usize>()
            + tail.len();
        // a short window folds the blocks to their titles, the lowest first (70); the open
        // page and the pad rows stay
        let room = framed_inner_height(panel) as usize;
        let mut over = full.saturating_sub(room);
        let mut folded = vec![false; blocks.len()];
        for i in (0..blocks.len()).rev() {
            if over == 0 {
                break;
            }
            if !open(blocks[i].0, &blocks[i].1) {
                folded[i] = true;
                over = over.saturating_sub(blocks[i].2.len());
            }
        }
        let tone_folded = over > 0 && !open("TONE", &tone_params);
        // the blocks, each row with the parameter it shows: the scroll follows the pick
        let mut body: Vec<(Option<PadParam>, Line)> = vec![(None, block("TONE", tone_folded))];
        if !tone_folded {
            body.extend(tone_params.iter().map(|w| Some(*w)).zip(tone));
        }
        for ((name, params, rows), folded) in blocks.into_iter().zip(folded) {
            body.push((None, block(name, folded)));
            if !folded {
                body.extend(params.into_iter().map(Some).zip(rows));
            }
        }
        // still too tall: the blocks scroll with the picked parameter in sight, and say how
        // many rows are above and below (70)
        let space = room.saturating_sub(lines.len() + tail.len());
        if body.len() > space && space >= 3 {
            let at = body
                .iter()
                .position(|(w, _)| *w == Some(p.param))
                .unwrap_or(0);
            let start = at.saturating_sub(space / 2).min(body.len() - space);
            let end = start + space;
            let mut seen: Vec<Line> = body[start..end].iter().map(|(_, l)| l.clone()).collect();
            // a marker takes the edge row; the pick sits in the middle, away from the edges
            if start > 0 {
                seen[0] = Line::from(format!("  ▲ {} more", start + 1)).fg(DIM);
            }
            if end < body.len() {
                let n = seen.len();
                seen[n - 1] = Line::from(format!("  ▼ {} more", body.len() - end + 1)).fg(DIM);
            }
            lines.extend(seen);
        } else {
            lines.extend(body.into_iter().map(|(_, l)| l));
        }
        lines.extend(tail);
        // a narrow panel shortens the labels, so the values stay whole (70)
        if width < NARROW_PANEL {
            for line in &mut lines {
                for span in &mut line.spans {
                    if let Some(short) = short_label(&span.content) {
                        span.content = short.into();
                    }
                }
            }
        }
        // the edit mode frames the panel and names its page; closed, the frame's place
        // stays empty, so the text does not move when the mode opens
        let framed = Block::bordered();
        let text_area = framed.inner(panel);
        if let Some(page) = p.edit {
            // every page named, the current one lit: Tab and I K lead through them; a title
            // wider than the panel drops "EDIT ·", the frame colour still tells the mode (56)
            let pages_len: usize = p.pages.iter().map(|n| n.chars().count() + 1).sum::<usize>() + 1;
            let fits = pages_len + 7 <= panel.width.saturating_sub(2) as usize;
            // the space after the last page may be cut by the frame
            let all_fit = pages_len <= panel.width.saturating_sub(2) as usize + 1;
            let mut title = if fits {
                vec![Span::styled(" EDIT ·", Style::new().fg(MAGENTA).bold())]
            } else {
                Vec::new()
            };
            // too narrow for every page: the open one and its place, `TONE 1/7` (70)
            let at = p.pages.iter().position(|&n| n == page).unwrap_or(0);
            if !all_fit {
                title.push(" ".into());
                title.push(Span::styled(
                    format!("{page} {}/{}", at + 1, p.pages.len()),
                    Style::new().fg(BG).bg(MAGENTA).bold(),
                ));
            }
            for &name in p.pages.iter().filter(|_| all_fit) {
                title.push(" ".into());
                title.push(if name == page {
                    Span::styled(name, Style::new().fg(BG).bg(MAGENTA).bold())
                } else {
                    name.fg(DIM)
                });
            }
            title.push(" ".into());
            frame.render_widget(
                framed
                    .border_style(Style::new().fg(MAGENTA))
                    .title(Line::from(title)),
                panel,
            );
        }
        // no wrap: a long name is cut at the edge instead of pushing the last rows out
        frame.render_widget(Paragraph::new(lines), text_area);
    }
}

/// Below this many columns inside the frame the panel labels shorten (70): a label takes
/// 10 columns, a value up to 12.
const NARROW_PANEL: usize = 22;

/// The short label of a narrow panel (70).
fn short_label(label: &str) -> Option<&'static str> {
    Some(match label {
        "  mode    " => "  mode ",
        "  scope   " => "  scope ",
        "volume  " => "vol  ",
        "pitch   " => "pit  ",
        "mute    " => "mute ",
        "attack  " => "atk  ",
        "decay   " => "dec  ",
        "release " => "rel  ",
        "filter  " => "flt  ",
        "cutoff  " => "cut  ",
        "reso    " => "res  ",
        "source  " => "src  ",
        "start   " => "in   ",
        "end     " => "out  ",
        "drive   " => "drv  ",
        "send 1  " => "s1   ",
        "send 2  " => "s2   ",
        "type    " => "type ",
        "size    " => "size ",
        "tone    " => "tone ",
        "time    " => "time ",
        "feedback" => "fb   ",
        _ => return None,
    })
}

/// The rows inside the panel frame: the frame is drawn in the edit mode, its place is
/// kept when closed.
fn framed_inner_height(panel: Rect) -> u16 {
    Block::bordered().inner(panel).height
}

/// The release row: a one-shot pad keeps the value but plays to its end, so the release
/// acts in gate only — dimmed with "(gate)" until the pad is gate.
fn release_line(p: &Pads, sel: &PadView) -> Line<'static> {
    let mut line = param_line(
        p,
        PadParam::Release,
        "release ",
        format!("{} ms", sel.release_ms),
    );
    if !sel.gate {
        for span in line.spans.iter_mut().skip(1) {
            if span.style.fg != Some(CYAN) && span.style.bg.is_none() {
                span.style = span.style.fg(BORDER);
            }
        }
        line.spans.push(" (gate)".fg(BORDER));
    }
    line
}

fn param_line(p: &Pads, which: PadParam, label: &'static str, value: String) -> Line<'static> {
    let knob = match p.knobs.iter().position(|&k| k == which) {
        Some(i) => Span::styled(format!("{} ", row_knob(i)), Style::new().fg(MAGENTA).bold()),
        None => "  ".into(),
    };
    let style = if p.lit == Some(which) {
        Style::new().fg(BG).bg(CYAN).bold()
    } else if p.param == which {
        Style::new().fg(CYAN).bold()
    } else {
        Style::new()
    };
    // a value off its default stands out, a default one steps back (55)
    let changed = p
        .pads
        .get(p.selected)
        .is_some_and(|s| s.changed.contains(&which));
    let style = match (style == Style::new(), changed) {
        (true, true) => Style::new().fg(WHITE),
        (true, false) => Style::new().fg(DIM),
        (false, _) => style,
    };
    let mut spans = vec![knob, label.fg(DIM), Span::styled(value, style)];
    if let Some((_, control)) = p.learn.iter().find(|(w, _)| *w == which) {
        spans.extend(learn_spans(&format!("  ◀ {control}")));
    }
    Line::from(spans)
}

/// The live strip above the grid: the last seconds of the master output as a wave,
/// the newest sound at the right; silence is a thin middle line.
fn envelope_strip(frame: &mut Frame, area: Rect, out_wave: &[f32], color: Color) {
    let rows = Layout::vertical(vec![Constraint::Length(1); area.height as usize]).split(area);
    let silence = [0.0f32];
    let wave = if out_wave.is_empty() {
        &silence[..]
    } else {
        out_wave
    };
    // the meters' decibel scale: -60 dB is the middle line, 0 dBFS is full height,
    // so a peak stands as tall on the strip as its bar on the meters
    let db: Vec<f32> = wave.iter().map(|&p| db_height(p)).collect();
    for (r, line) in braille_abs(&db, area.width as usize, rows.len())
        .iter()
        .enumerate()
    {
        frame.render_widget(
            Line::from(Span::styled(line.clone(), Style::new().fg(color))),
            rows[r],
        );
    }
}

/// The strip on the FILTER page (44): the response as a line of Braille dots, the
/// frequency along the octaves from the left, the level upwards from −36 to +12 dB; the
/// caption on the top row and a few frequencies on the bottom one.
fn filter_strip(frame: &mut Frame, area: Rect, curve: &FilterCurve) {
    let (cols, rows) = (area.width as usize, area.height as usize);
    if cols == 0 || rows == 0 || curve.levels.is_empty() {
        return;
    }
    let (dx, dy) = (cols * 2, rows * 4);
    const BITS: [[u8; 4]; 2] = [[0x01, 0x02, 0x04, 0x40], [0x08, 0x10, 0x20, 0x80]];
    let mut dots = vec![vec![0u8; cols]; rows];
    let db = |l: f32| 20.0 * l.max(1e-6).log10();
    // the hump's top stays inside: the scale rises with it
    let peak = curve.levels.iter().copied().fold(0.0, f32::max);
    let top = CURVE_TOP_DB.max(db(peak) + 3.0);
    for x in 0..dx {
        let i = (x * curve.levels.len() / dx).min(curve.levels.len() - 1);
        let t = (top - db(curve.levels[i]).clamp(CURVE_BOTTOM_DB, top)) / (top - CURVE_BOTTOM_DB);
        let y = (t * (dy - 1) as f32).round() as usize;
        dots[y / 4][x / 2] |= BITS[x % 2][y % 4];
    }
    let mut text: Vec<Vec<Span>> = dots
        .iter()
        .map(|row| {
            row.iter()
                .map(|&b| {
                    let c = char::from_u32(0x2800 + b as u32).unwrap_or(' ');
                    Span::styled(c.to_string(), Style::new().fg(CYAN))
                })
                .collect()
        })
        .collect();
    let mut put = |row: usize, at: usize, s: &str, style: Style| {
        for (k, ch) in s.chars().enumerate() {
            if let Some(cell) = text.get_mut(row).and_then(|r| r.get_mut(at + k)) {
                *cell = Span::styled(ch.to_string(), style);
            }
        }
    };
    put(0, 1, &curve.caption, Style::new().fg(TEXT).bold());
    if rows > 1 {
        let (lo, hi) = curve.span;
        let decades = (hi / lo).log10();
        for (hz, name) in [
            (20.0f32, "20"),
            (200.0, "200"),
            (2_000.0, "2k"),
            (20_000.0, "20k"),
        ] {
            let x = ((hz / lo).log10() / decades * (cols - 1) as f32).round() as usize;
            let at = x.min(cols.saturating_sub(name.len()));
            put(rows - 1, at, name, Style::new().fg(DIM));
        }
    }
    let lines: Vec<Line> = text.into_iter().map(Line::from).collect();
    frame.render_widget(Paragraph::new(lines), area);
}

/// The strip in the pad edit: the piece's wave dimmed and the envelope over it as a bright
/// line of Braille dots — the attack rises from zero, the decay falls to zero at the end of
/// the piece; the picked part is cyan. The release acts only in gate mode, from the key's
/// release, so a gate pad shows it apart from the wave as a dotted "key up" fall; a
/// one-shot pad shows none.
fn pad_envelope_strip(frame: &mut Frame, area: Rect, big: &BigView, picked: PadParam, gate: bool) {
    let (cols, rows) = (area.width as usize, area.height as usize);
    if cols == 0 || rows == 0 {
        return;
    }
    // the release takes its share of the width, at most a third, at least room for its
    // name, after a gap that keeps it apart from the wave
    const LABEL: &str = "key up";
    let tail = if gate && big.release > 0.0 {
        ((cols as f32 * (big.release / (1.0 + big.release)).min(1.0 / 3.0)).round() as usize)
            .max(LABEL.len() + 2)
            .min(cols / 2)
    } else {
        0
    };
    let piece = cols - tail;
    let gap = usize::from(tail > 0);
    let wave = braille_abs(&big.wave, piece, rows);
    // the line: one dot per dot column, at the envelope's height
    let (dx, dy) = (cols * 2, rows * 4);
    const BITS: [[u8; 4]; 2] = [[0x01, 0x02, 0x04, 0x40], [0x08, 0x10, 0x20, 0x80]];
    let mut line = vec![vec![0u8; cols]; rows];
    let mut part = vec![PadParam::Volume; cols];
    let piece_dots = piece * 2;
    for x in 0..dx {
        let (level, which) = if x < piece_dots {
            let t = (x as f32 + 0.5) / piece_dots.max(1) as f32;
            let i = ((t * big.env.len() as f32) as usize).min(big.env.len().saturating_sub(1));
            let which = if t < big.attack {
                PadParam::Attack
            } else if t > 1.0 - big.decay {
                PadParam::Decay
            } else {
                PadParam::Volume
            };
            (big.env.get(i).copied().unwrap_or(1.0), which)
        } else {
            // the gap column stays empty; then dotted: every other dot column, from full
            // to silence
            let from = piece_dots + gap * 2;
            if x < from || x % 2 == 1 {
                continue;
            }
            let t = (x - from) as f32 / (dx - from).max(1) as f32;
            (1.0 - t, PadParam::Release)
        };
        let y = ((1.0 - level.clamp(0.0, 1.0)) * (dy - 1) as f32).round() as usize;
        line[y / 4][x / 2] |= BITS[x % 2][y % 4];
        part[x / 2] = which;
    }
    // the release's name on the top row, at the right end where the fall is already low
    let label_from = (tail > 0).then(|| cols - LABEL.len());
    for (r, row) in line.iter().enumerate() {
        let wave_row: Vec<char> = wave[r].chars().collect();
        if r == 0
            && let Some(from) = label_from
        {
            let mut spans: Vec<Span> = Vec::new();
            for (x, &bits) in row.iter().enumerate().take(from) {
                spans.push(envelope_cell(bits, x, &wave_row, &part, picked));
            }
            let color = if picked == PadParam::Release {
                CYAN
            } else {
                DIM
            };
            spans.push(Span::styled(LABEL, Style::new().fg(color)));
            frame.render_widget(
                Line::from(spans),
                Rect {
                    y: area.y,
                    height: 1,
                    ..area
                },
            );
            continue;
        }
        let spans: Vec<Span> = row
            .iter()
            .enumerate()
            .map(|(x, &bits)| envelope_cell(bits, x, &wave_row, &part, picked))
            .collect();
        frame.render_widget(
            Line::from(spans),
            Rect {
                y: area.y + r as u16,
                height: 1,
                ..area
            },
        );
    }
}

/// One cell of the envelope strip: a dot of the line (cyan on the picked part) or the dim
/// wave under it.
fn envelope_cell(
    bits: u8,
    x: usize,
    wave_row: &[char],
    part: &[PadParam],
    picked: PadParam,
) -> Span<'static> {
    if bits != 0 {
        let ch = char::from_u32(0x2800 + u32::from(bits)).unwrap_or(' ');
        let color = if part[x] == picked { CYAN } else { WHITE };
        Span::styled(ch.to_string(), Style::new().fg(color))
    } else {
        let ch = wave_row.get(x).copied().unwrap_or(' ');
        Span::styled(ch.to_string(), Style::new().fg(BORDER))
    }
}

fn mode(gate: bool) -> &'static str {
    if gate { "gate" } else { "one-shot" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    fn render(
        panel: &CommandPanel,
        width: u16,
        height: u16,
    ) -> (Vec<String>, Terminal<TestBackend>) {
        let mut term = Terminal::new(TestBackend::new(width, height)).unwrap();
        term.draw(|frame| {
            let area = frame.area();
            let filler = "x".repeat(usize::from(area.width));
            let lines = vec![Line::from(filler); usize::from(area.height)];
            frame.render_widget(Paragraph::new(lines), area);
            command_panel_in(frame, area, panel, None);
        })
        .unwrap();
        let rows = term
            .backend()
            .buffer()
            .content()
            .chunks(usize::from(width))
            .map(|row| row.iter().map(|cell| cell.symbol()).collect())
            .collect();
        (rows, term)
    }

    /// Two words take two lines; a state goes under them, or under the whole label when
    /// only two rows fit; one row keeps the label (138).
    #[test]
    fn cell_lines_put_the_state_under_the_label() {
        assert_eq!(
            cell_lines("copy pad", None, 2),
            [("copy", false), ("pad", false)]
        );
        assert_eq!(cell_lines("copy pad", None, 1), [("copy pad", false)]);
        assert_eq!(
            cell_lines("loop play", Some("on"), 3),
            [("loop", false), ("play", false), ("on", true)]
        );
        assert_eq!(
            cell_lines("loop play", Some("on"), 2),
            [("loop play", false), ("on", true)]
        );
        assert_eq!(
            cell_lines("loop play", Some("on"), 1),
            [("loop play", false)]
        );
        assert_eq!(cell_lines("undo", None, 2), [("undo", false)]);
    }

    fn panel(page: usize) -> CommandPanel {
        let mut labels: [Option<String>; 16] = std::array::from_fn(|_| None);
        labels[0] = Some("cut".into());
        labels[12] = Some("back".into());
        CommandPanel {
            labels,
            page,
            view: "CHOP",
            target: false,
            flash: None,
            dim: [false; 16],
            warn: [false; 16],
            current: [false; 16],
            states: [None; 16],
        }
    }

    /// Scenario "The header names the view and its neighbors" (screen-layout).
    #[test]
    fn the_header_names_the_view_and_its_neighbors() {
        for (width, height) in [(80, 26), (120, 40)] {
            let [_, body, _, _] = frame_areas(Rect::new(0, 0, width, height));
            let frame = pad_frame(body, false);
            let (rows, actions) = render(&panel(1), width, height);
            let header: String = rows[usize::from(frame.bank.y)]
                .chars()
                .skip(usize::from(frame.bank.x))
                .take(usize::from(frame.bank.width))
                .collect();
            assert!(header.contains("‹ screens │ CHOP actions │ ·"), "{header}");
            assert!(!header.contains('x'), "{width}x{height}: {header}");
            let (rows, screens) = render(&panel(0), width, height);
            assert!(rows[usize::from(frame.bank.y)].contains("· │ SCREENS │ actions ›"));
            // The first cell border is cyan for actions and magenta for screens.
            let corner = (frame.grid.x, frame.grid.y);
            assert_eq!(actions.backend().buffer()[corner].fg, CYAN);
            assert_eq!(screens.backend().buffer()[corner].fg, MAGENTA);
        }
    }

    /// Scenario "A cell hit lights the cell" (screen-layout).
    #[test]
    fn a_cell_hit_lights_the_cell() {
        for (width, height) in [(80, 26), (120, 40)] {
            let [_, body, _, _] = frame_areas(Rect::new(0, 0, width, height));
            let grid = pad_frame(body, false).grid;
            // Inside the first cell (active "cut") and the second cell (empty, dim).
            let first = (grid.x + 2, grid.y + 1);
            let second = (grid.x + grid.width / 4 + 2, grid.y + 1);
            let mut lit = panel(1);
            lit.flash = Some(0);
            let (_, term) = render(&lit, width, height);
            assert_eq!(term.backend().buffer()[first].bg, CYAN, "{width}x{height}");
            assert_eq!(term.backend().buffer()[second].bg, PANEL);
            lit.flash = Some(1);
            let (_, term) = render(&lit, width, height);
            assert_eq!(term.backend().buffer()[first].bg, PANEL);
            assert_eq!(term.backend().buffer()[second].bg, DIM, "{width}x{height}");
        }
    }

    /// Scenario "Grey, orange and empty cells look different" (screen-layout).
    #[test]
    fn grey_orange_and_empty_cells_look_different() {
        for (width, height) in [(80, 26), (120, 40)] {
            let [_, body, _, _] = frame_areas(Rect::new(0, 0, width, height));
            let grid = pad_frame(body, false).grid;
            let corner = |cell: u16| (grid.x + cell * (grid.width / 4), grid.y);
            let mut looks = panel(1);
            looks.labels[1] = Some("copy pad".into());
            looks.dim[1] = true;
            looks.labels[2] = Some("clear pad".into());
            looks.warn[2] = true;
            let (rows, term) = render(&looks, width, height);
            let buffer = term.backend().buffer();
            assert_eq!(buffer[corner(0)].fg, CYAN, "a bright cell");
            assert_eq!(buffer[corner(1)].fg, BORDER, "a grey cell");
            assert_eq!(buffer[corner(2)].fg, ORANGE, "an orange cell");
            assert_eq!(buffer[corner(3)].fg, BORDER, "an empty cell");
            let top: String = rows
                .iter()
                .skip(usize::from(grid.y))
                .take(usize::from(grid.height / 4))
                .cloned()
                .collect();
            assert!(top.contains("copy") && top.contains("clear"), "{top}");
            let row = rows
                .iter()
                .skip(usize::from(grid.y))
                .take(usize::from(grid.height / 4))
                .map(|row| {
                    row.chars()
                        .skip(usize::from(corner(3).0))
                        .take(usize::from(grid.width / 4))
                        .collect::<String>()
                })
                .collect::<String>();
            assert!(
                !row.chars().any(char::is_alphabetic),
                "an empty cell shows no label: {row}"
            );
        }
    }

    /// The current page of the pad edit is brighter than the other page cells (138).
    #[test]
    fn the_current_page_cell_is_brighter() {
        let [_, body, _, _] = frame_areas(Rect::new(0, 0, 80, 26));
        let grid = pad_frame(body, false).grid;
        let mut looks = panel(1);
        looks.labels[1] = Some("fx page".into());
        looks.current[1] = true;
        let (_, term) = render(&looks, 80, 26);
        let buffer = term.backend().buffer();
        assert_eq!(buffer[(grid.x + grid.width / 4, grid.y)].fg, WHITE);
        assert_eq!(buffer[(grid.x, grid.y)].fg, CYAN);
    }

    /// Scenario "Cells show the device numbers" (command-panel).
    #[test]
    fn cells_show_the_device_numbers() {
        assert_eq!((device_number(13), device_number(4)), (1, 16));
        assert_eq!((device_number(1), device_number(16)), (13, 4));
        for (width, height) in [(80, 26), (120, 40)] {
            let [_, body, _, _] = frame_areas(Rect::new(0, 0, width, height));
            let grid = pad_frame(body, false).grid;
            let (rows, _) = render(&panel(1), width, height);
            let top = &rows[usize::from(grid.y)];
            let bottom = rows
                .iter()
                .rev()
                .find(|row| row.contains("─1─") || row.contains("┌1─"))
                .unwrap();
            assert!(top.contains("┌13"), "{top}");
            assert!(top.contains("┌16"), "{top}");
            assert!(bottom.contains("┌1─"), "{bottom}");
        }
    }
}
