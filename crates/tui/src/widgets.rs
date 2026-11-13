use super::*;

/// Pad width bounds: narrow terminals keep the sketch size, wide ones show longer names.
pub(super) const PAD_W_MIN: u16 = 13;
pub(super) const PAD_W_MAX: u16 = 40;

/// Pad width that leaves `reserve` columns for what stands next to the grid.
pub(super) fn pad_width(total: u16, reserve: u16) -> u16 {
    (total.saturating_sub(reserve) / 4).clamp(PAD_W_MIN, PAD_W_MAX)
}
pub(super) const PAD_H: u16 = 4;
/// Rows the info panel reserves under the selected pad: the parameter lines and the
/// edit frame around them, kept even while the frame is not drawn so nothing jumps.
pub(super) const PANEL_ROWS: u16 = 30;
/// Rows the help line always keeps at the bottom, so the screen above never moves.
pub(super) const HELP_ROWS: u16 = 3;

/// "bank [1 ⣿⣿] 2 ⣰⣄ …" line: each bank shows its fill as two braille glyphs,
/// every bank in its own color, the open one bold in brackets.
pub(super) fn banks(p: &Pads) -> Vec<Span<'static>> {
    // every bank brings its own leading space
    let mut spans = vec!["bank".fg(DIM)];
    // the pads of the open bank lit right now: a hit flashes in the row too (62)
    let hits = p
        .pads
        .iter()
        .take(16)
        .enumerate()
        .filter(|(_, pad)| pad.flash)
        .fold(0u16, |m, (i, _)| m | 1 << i);
    for (i, &color) in BANK_COLORS.iter().enumerate() {
        let b = i + 1;
        // one glyph cell has one colour, so a bank shows one thing at a time: the pads
        // held or flashing (white, 62), else its marks (yellow, 76), else its fill
        // a closed bank lights the pads its A1 onsets hit now (98); the open bank keeps
        // its own flashes
        let (mask, dots) = if i == p.bank && hits != 0 {
            (hits, Some(WHITE))
        } else if i != p.bank && p.bank_pattern.hits[i] != 0 {
            (p.bank_pattern.hits[i], Some(WHITE))
        } else if p.bank_marks[i] != 0 {
            (p.bank_marks[i], Some(MARK))
        } else {
            (p.bank_fill[i], None)
        };
        let glyphs = bank_glyphs(mask);
        let dots = dots.map(|c| Style::new().fg(c).bold());
        // the closed banks keep their number and glyphs together: the row fits a narrow
        // window
        let open = i == p.bank;
        let style = if open {
            Style::new().fg(color).bold()
        } else {
            Style::new().fg(color)
        };
        if p.bank_pattern.recorded[i] {
            // an underlined number: the bank holds A1 onsets (98)
            spans.push(Span::styled(if open { " [" } else { " " }, style));
            spans.push(Span::styled(b.to_string(), style.underlined()));
            if open {
                spans.push(Span::styled(" ", style));
            }
            spans.push(Span::styled(glyphs, dots.unwrap_or(style)));
            if open {
                spans.push(Span::styled("]", style));
            }
        } else if open {
            match dots {
                // the space before the bracket, as before a closed bank: no `1⣿⣿[2`
                Some(dots) => {
                    spans.push(Span::styled(format!(" [{b} "), style));
                    spans.push(Span::styled(glyphs, dots));
                    spans.push(Span::styled("]", style));
                }
                None => spans.push(Span::styled(format!(" [{b} {glyphs}]"), style)),
            }
        } else {
            match dots {
                Some(dots) => {
                    spans.push(Span::styled(format!(" {b}"), style));
                    spans.push(Span::styled(glyphs, dots));
                }
                None => spans.push(Span::styled(format!(" {b}{glyphs}"), style)),
            }
        }
    }
    spans
}

/// Two braille glyphs mirroring the 4×4 pad grid of a bank: a lit dot is a pad with a sound
/// (bit `r*4+c` of `mask` is pad at row `r`, column `c`).
pub(super) fn bank_glyphs(mask: u16) -> String {
    // Braille dot bits: left dot column rows 0..3 and right dot column rows 0..3.
    const LEFT: [u32; 4] = [0x01, 0x02, 0x04, 0x40];
    const RIGHT: [u32; 4] = [0x08, 0x10, 0x20, 0x80];
    let mut out = String::new();
    for half in 0..2 {
        let mut bits = 0u32;
        for row in 0..4 {
            for col in 0..2 {
                if mask & (1 << (row * 4 + half * 2 + col)) != 0 {
                    bits |= if col == 0 { LEFT[row] } else { RIGHT[row] };
                }
            }
        }
        out.push(char::from_u32(0x2800 + bits).unwrap_or('\u{2800}'));
    }
    out
}

/// Rows of the pad frame besides the strip and the pad waves: the title, the bank row, the grid.
pub(super) const FRAME_ROWS: u16 = 2 + PAD_H * 4;
/// The strip keeps this many rows before the pad waves get any; below it only on a low
/// terminal, down to [`STRIP_MIN`].
pub(super) const STRIP_FULL: u16 = 8;
pub(super) const STRIP_MIN: u16 = 3;
/// Columns the right column of the pad frame keeps next to the grid.
pub(super) const SIDE_W: u16 = 40;

/// The frame of the screens with pads (31): the same rows and columns on F2, F6 and F3.
pub(super) struct PadFrame {
    pub(super) title: Rect,
    /// Over the full width, all the height the grid leaves.
    pub(super) strip: Rect,
    pub(super) bank: Rect,
    pub(super) grid: Rect,
    /// Wave rows in each pad cell (29).
    pub(super) waves: u16,
    /// The mode column, from the bank row down.
    pub(super) side: Rect,
}

/// The pad waves give way first, then the strip shrinks; a tall terminal gives the rest to the strip.
/// `hidden` (32): the strip stands in the grid's place, the left column from the title down
/// with the pad line under it; the right column keeps the full height, it needs its rows.
pub(super) fn pad_frame(area: Rect, hidden: bool) -> PadFrame {
    if hidden {
        let [title, body] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);
        let [left, _, side] = Layout::horizontal([
            Constraint::Length(pad_width(area.width, SIDE_W) * 4),
            Constraint::Length(2),
            Constraint::Fill(1),
        ])
        .areas(body);
        let [strip, bank] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(left);
        return PadFrame {
            title,
            strip,
            bank,
            grid: Rect { height: 0, ..bank },
            waves: 0,
            side,
        };
    }
    let free = area.height.saturating_sub(FRAME_ROWS);
    let waves = (free.saturating_sub(STRIP_FULL) / PAD_H).min(3);
    let strip_h = (free - waves * PAD_H).max(STRIP_MIN);
    let [title, strip, body] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(strip_h),
        Constraint::Fill(1),
    ])
    .areas(area);
    let [left, _, side] = Layout::horizontal([
        Constraint::Length(pad_width(area.width, SIDE_W) * 4),
        Constraint::Length(2),
        Constraint::Fill(1),
    ])
    .areas(body);
    let [bank, grid] = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(left);
    PadFrame {
        title,
        strip,
        bank,
        grid,
        waves,
        side,
    }
}

/// The 4×4 grid of the current bank; `waves` extra Braille rows in each pad cell (29),
/// 0 keeps the cells exactly as they were.
pub(super) fn pad_grid(frame: &mut Frame, area: Rect, p: &Pads, waves: u16) {
    let bank = BANK_COLORS[p.bank % BANK_COLORS.len()];
    let rows = Layout::vertical([Constraint::Length(PAD_H + waves); 4]).split(area);
    for (r, row) in rows.iter().enumerate() {
        let cells = Layout::horizontal([Constraint::Length(area.width / 4); 4]).split(*row);
        for (c, cell) in cells.iter().enumerate() {
            let i = r * 4 + c;
            if let Some(pad) = p.pads.get(i) {
                pad_cell(
                    frame,
                    *cell,
                    pad,
                    i == p.selected,
                    bank,
                    waves,
                    (p.feedback.outline, p.feedback.corners[i]),
                );
            }
        }
    }
}

/// The line in place of a hidden grid (32): the bank row (`with_bank`, 62), then the
/// selected pad and what it plays; it flashes as the pad does on a hit. A narrow window
/// cuts the pad part at the end first.
pub(super) fn pad_line(p: &Pads, mut spans: Vec<Span<'static>>, with_bank: bool) -> Line<'static> {
    if with_bank {
        // every bank, as above the grid (62); the pad part after it is cut first
        spans.extend(banks(p));
        spans.push(" · ".fg(DIM));
    }
    if let Some(sel) = p.pads.get(p.selected) {
        let style = if sel.flash {
            Style::new().fg(BG).bg(WHITE)
        } else {
            Style::new().fg(CYAN)
        };
        spans.push(Span::styled(
            format!("pad {} {}", sel.key.to_uppercase(), sel.name),
            style,
        ));
    }
    Line::from(spans)
}

/// Keep fixed slots first. Compact labels before dropping lower-priority badges.
pub(super) fn title<'a>(
    mut fixed: Vec<Span<'a>>,
    badges: &[Badge],
    width: u16,
    reserve: usize,
) -> Line<'a> {
    let available =
        (width as usize).saturating_sub(fixed.iter().map(Span::width).sum::<usize>() + reserve);
    let mut count = badges.len().min(3);
    let badge_width = |n: usize, compact: bool| {
        badges[..n]
            .iter()
            .map(|b| {
                Line::from(if compact {
                    b.compact.as_str()
                } else {
                    b.normal.as_str()
                })
                .width()
                    + 3
            })
            .sum::<usize>()
    };
    while count > 1 && badge_width(count, true) > available {
        count -= 1;
    }
    let compact = badge_width(count, false) > available;
    let used = badges[..count]
        .iter()
        .map(|b| {
            Line::from(if compact {
                b.compact.as_str()
            } else {
                b.normal.as_str()
            })
            .width()
                + 3
        })
        .sum::<usize>();
    if used <= available {
        for b in &badges[..count] {
            let color = match b.kind {
                BadgeKind::PadMode => MARK,
                BadgeKind::On => CYAN,
                BadgeKind::Attention => MAGENTA,
            };
            let text = if compact { &b.compact } else { &b.normal };
            fixed.push(" ".into());
            fixed.push(Span::styled(
                format!(" {text} "),
                Style::new().fg(BG).bg(color).bold(),
            ));
        }
    }
    Line::from(fixed)
}

/// Shorten title names in display cells at word or grapheme boundaries.
pub(super) fn title_detail(text: &str, width: usize) -> String {
    clip_name(text, width)
}

pub(super) fn pad_cell(
    frame: &mut Frame,
    area: Rect,
    pad: &PadView,
    selected: bool,
    bank: Color,
    waves: u16,
    (outline, corner): (bool, Option<PadCorner>),
) {
    // The bank color owns the idle frame and key letter; state colors go above it.
    // A marked pad keeps its frame in every idle state, so the group stays visible.
    let (border, bg, fg) = match (pad.flash, pad.marked, selected) {
        _ if pad.in_hand => (MAGENTA, PANEL, TEXT),
        (true, _, _) => (WHITE, WHITE, BG),
        (false, true, _) => (MARK, MARK_BG, BRIGHT),
        (false, false, true) => (CYAN, PANEL, TEXT),
        (false, false, false) if outline => (MARK, PANEL, if pad.dim { DIM } else { TEXT }),
        // the chop screen: a pad with no slice of the open chop steps back (36)
        (false, false, false) if pad.dim => (BORDER, PANEL, DIM),
        (false, false, false) => (bank, PANEL, TEXT),
    };
    // the selected pad keeps its cyan frame over a hit or a mark (98k); its frame cells stay
    // dark, as cyan lines on a white fill are hard to see
    let keep = selected && !pad.in_hand;
    let border = if keep { CYAN } else { border };
    let key = Span::styled(
        pad.key.to_uppercase().to_string(),
        Style::new().fg(if keep {
            border
        } else if pad.flash {
            BG
        } else if outline && !pad.in_hand && !pad.marked && !selected {
            bank
        } else {
            border
        }),
    );
    let frame_style = if keep {
        Style::new().fg(border).bg(PANEL)
    } else {
        Style::new().fg(border)
    };
    let mut block = Block::bordered()
        .title(key)
        .border_style(frame_style)
        .style(Style::new().bg(bg).fg(fg));
    if let Some(corner) = corner {
        let text = match corner {
            PadCorner::Available => "◎".to_string(),
            PadCorner::Replace => "!".to_string(),
            PadCorner::Slice(n) => n.to_string(),
            PadCorner::Erase => "E".into(),
        };
        block = block.title_top(
            Line::from(text)
                .fg(if pad.flash && !keep { BG } else { MARK })
                .right_aligned(),
        );
    }
    if pad.dashed {
        block = block.border_set(ratatui::symbols::border::Set {
            vertical_left: "┆",
            vertical_right: "┆",
            horizontal_top: "┄",
            horizontal_bottom: "┄",
            ..ratatui::symbols::border::PLAIN
        });
    }
    // gate as a glyph, the volume, the mute group (77): `⇥ +1 mute 2`; no separator, so
    // `⇥ -24 mute 8` fits the 13 cells of the narrowest pad. One-shot is the default and
    // has no mark (98b): an arrow on every pad read as a position
    let mut parts: Vec<String> = Vec::new();
    if pad.gate {
        parts.push("⇥".into());
    }
    if pad.volume_db != 0.0 {
        parts.push(format!("{:+.0}", pad.volume_db));
    }
    if pad.group > 0 {
        parts.push(format!("mute {}", pad.group));
    }
    // `✱`: the pad has more settings, the panel names them (77); the frame stays clean
    if !pad.glyphs.is_empty() {
        parts.push(pad.glyphs.clone());
    }
    let second = parts.join(" ");
    let second = match (pad.missing, pad.loading, &pad.note) {
        (_, Some(p), _) => Line::from(format!(" load {p}%")).fg(CYAN),
        (true, None, _) => Line::from(" missing").fg(MAGENTA),
        (_, _, Some(n)) if n.contains(" → ") => Line::from(learn_spans(&format!(" {n}"))),
        (_, _, Some(n)) => Line::from(format!(" {n}")).fg(if pad.flash { BG } else { DIM }),
        _ => Line::from(format!(" {second}")).fg(if pad.flash { BG } else { DIM }),
    };
    let width = area.width.saturating_sub(3) as usize;
    let name = if pad.naming {
        let mut chars = Vec::new();
        let mut used = usize::from(width > 0);
        for c in pad.name.chars().rev() {
            let n = Span::raw(c.to_string()).width();
            if used + n > width {
                break;
            }
            chars.push(c);
            used += n;
        }
        let mut text = chars.into_iter().rev().collect::<String>();
        if width > 0 {
            text.push('▏');
        }
        text
    } else {
        clip_name(&pad.name, width)
    };
    let name = Line::from(format!(" {name}"));
    let name = if pad.empty { name.fg(DIM) } else { name };
    let text = vec![name, second];
    frame.render_widget(Paragraph::new(text).block(block), area);
    // the wave sits between the mode line and the bottom frame (29); the selected pad's
    // wave wears the pad's text colour, the rest is dim
    if waves > 0 && !pad.wave.is_empty() {
        let color = if selected || pad.flash { fg } else { DIM };
        let cols = area.width.saturating_sub(2) as usize;
        let rows = braille_abs(&pad.wave, cols, waves as usize);
        for (w, row) in rows.iter().enumerate() {
            frame.render_widget(
                Line::from(Span::styled(row.clone(), Style::new().fg(color))),
                Rect {
                    x: area.x + 1,
                    y: area.y + PAD_H - 1 + w as u16,
                    width: area.width - 2,
                    height: 1,
                },
            );
        }
    }
}

/// A parameter row of the pad panel: the knob number while its page is up, the label, and
/// the value — highlighted when picked, lit for a moment after a change.
/// A learn label: the control now in magenta, a waiting one after ` → ` in amber (57).
pub(super) fn learn_spans(text: &str) -> Vec<Span<'static>> {
    match text.split_once(" → ") {
        Some((now, new)) => vec![
            now.to_string().fg(MAGENTA),
            " → ".fg(DIM),
            Span::styled(new.to_string(), Style::new().fg(AMBER).bold()),
        ],
        None => vec![text.to_string().fg(MAGENTA)],
    }
}
