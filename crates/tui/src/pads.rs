//! Pads screen and pad edit drawing.

use super::*;

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
    // a recording takes the strip in the rec colour (25), `out_wave` then holds what goes
    // into the file; the pad edit draws the pad's envelope over its piece; else the output
    // the splash keeps only the wave rows: the rhythm shows from the start (98b)
    let show_pattern =
        p.pattern.is_some() && p.edit.is_none() && !s.recording && f.strip.height >= 3;
    // a three-row strip holds only the rhythm; the L/R meters still show the level
    let wave = if show_pattern {
        let wave = Rect {
            height: f.strip.height - 3,
            ..f.strip
        };
        let events = Rect {
            y: wave.bottom(),
            height: 3,
            ..f.strip
        };
        pattern_strip(frame, events, p.pattern.as_ref().unwrap());
        wave
    } else {
        f.strip
    };
    match (p.edit, &p.big) {
        _ if s.recording => envelope_strip(frame, f.strip, &p.out_wave, MAGENTA),
        (Some(_), _) if p.curve.is_some() => filter_strip(
            frame,
            f.strip,
            p.curve.as_ref().unwrap_or(&FilterCurve::default()),
        ),
        (Some(_), Some(big)) => {
            let gate = p.pads.get(p.selected).is_some_and(|v| v.gate);
            pad_envelope_strip(frame, f.strip, big, p.param, gate)
        }
        _ if wave.height == 0 => {}
        _ if p.splash.is_some() => splash_strip(frame, wave, p.splash.as_ref().unwrap()),
        _ => envelope_strip(frame, wave, &p.out_wave, DIM),
    }
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

    if let Some(sel) = p.pads.get(p.selected) {
        let [panel, _] =
            Layout::vertical([Constraint::Length(PANEL_ROWS), Constraint::Fill(1)]).areas(info);
        let lines = vec![
            Line::from(vec![
                "pad ".fg(DIM),
                Span::styled(
                    sel.key.to_uppercase().to_string(),
                    Style::new().fg(CYAN).bold(),
                ),
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
        Some(i) => Span::styled(format!("{} ", i + 1), Style::new().fg(MAGENTA).bold()),
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
