//! Chop screen drawing.

use super::*;

// Paragraph can put a wide grapheme in its last single cell. Pre-wrap only wide side text.
pub(super) fn side_rows(text: &str, width: u16) -> Vec<String> {
    let width = usize::from(width);
    if width == 0 {
        return Vec::new();
    }
    if !Span::raw(text)
        .styled_graphemes(Style::default())
        .any(|g| Span::raw(g.symbol).width() > 1)
    {
        return vec![text.into()];
    }
    let mut rows = Vec::new();
    let mut line = String::new();
    let mut gap = String::new();
    let mut used = 0;
    for token in text.split_inclusive(char::is_whitespace) {
        let word = token.trim_end_matches(char::is_whitespace);
        if !word.is_empty() {
            if !line.is_empty()
                && used + Span::raw(gap.as_str()).width() + Span::raw(word).width() > width
            {
                rows.push(std::mem::take(&mut line));
                used = 0;
                gap.clear();
            }
            for part in [gap.as_str(), word] {
                for grapheme in Span::raw(part).styled_graphemes(Style::default()) {
                    let cells = Span::raw(grapheme.symbol).width();
                    if cells > width {
                        continue;
                    }
                    if used + cells > width {
                        rows.push(std::mem::take(&mut line));
                        used = 0;
                    }
                    line.push_str(grapheme.symbol);
                    used += cells;
                }
            }
            gap.clear();
        }
        gap.push_str(&token[word.len()..]);
    }
    if !line.is_empty() {
        rows.push(line);
    }
    rows
}

pub(super) fn draw(
    frame: &mut Frame,
    area: Rect,
    c: &Chop,
    s: &Status,
    mut images: Option<&mut Vec<WaveImage>>,
) {
    let pf = pad_frame(area, c.pads.hidden);
    let header = pf.title;
    // the marks over the wave are the top two rows of the strip (31)
    let [numbers, play_bar, wave] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(pf.strip);
    let slices = c.bounds.len().saturating_sub(1);
    let playing = c.mode == "LISTEN ▶";
    let badge = if playing {
        " LISTEN ▶ "
    } else {
        " LISTEN   "
    };
    let badge_style = if playing {
        Style::new().fg(BG).bg(CYAN)
    } else {
        Style::new().fg(DIM).bg(BG)
    };
    let mut slice_count = format!(" · {slices} slice{}", if slices == 1 { "" } else { "s" });
    let mut variant = c
        .which
        .map_or_else(String::new, |(i, n)| format!(" · chop {i} of {n}"));
    let fixed = vec![
        Span::styled(" F3 CHOP ", Style::new().fg(BG).bg(TEXT)),
        " ".into(),
        Span::styled(badge, badge_style),
    ];
    let fixed_width = fixed.iter().map(Span::width).sum::<usize>();
    let mode_width: usize = s
        .badges
        .iter()
        .take(2)
        .map(|b| Line::from(b.compact.as_str()).width() + 3)
        .sum();
    if fixed_width + mode_width + Line::from(format!("{variant}{slice_count}")).width() + 3
        > header.width as usize
    {
        variant = c
            .which
            .map_or_else(String::new, |(i, n)| format!(" #{i}/{n}"));
        slice_count = format!(" · {slices}sl");
    }
    let suffix_width = Line::from(format!("{variant}{slice_count}")).width();
    // Keep variant and count identities. File text yields before mode and question badges.
    let file_reserve = (header.width as usize)
        .saturating_sub(fixed_width + mode_width + suffix_width + 3)
        .min(12);
    let mut heading = title(
        fixed,
        &s.badges,
        header.width,
        suffix_width + 3 + file_reserve,
    );
    let details = Line::from(vec![
        format!("   cuts of {}", c.file).into(),
        variant.clone().into(),
        format!("   {:.2} s{slice_count}", c.seconds).fg(DIM),
    ]);
    let room = (header.width as usize).saturating_sub(heading.width());
    if details.width() <= room {
        heading.spans.extend(details.spans);
    } else {
        let file_width = room.saturating_sub(suffix_width + 3);
        heading
            .spans
            .push(format!("   {}", title_detail(c.file, file_width)).into());
        heading.spans.push(variant.into());
        heading.spans.push(slice_count.fg(DIM));
    }
    frame.render_widget(heading, header);
    let width = wave.width as usize;
    let col = |at: f32| ((at * width as f32) as usize).min(width.saturating_sub(1));
    let (first, last) = (
        c.bounds.first().copied().unwrap_or(0.0),
        c.bounds.last().copied().unwrap_or(1.0),
    );

    // where every slice lies (36): a slice on a pad wears a plate in its bank color across
    // its width with the number and the pads (`3 2-Q`), a gap of one cell before the next;
    // a slice on no pad shows its number and a dim `○`; the cursor is the "!" above it all
    let mut nums: Vec<(char, Color, Option<Color>)> = vec![(' ', DIM, None); width];
    for (i, &start) in c.bounds.iter().take(slices).enumerate() {
        let (from, to) = (col(start), col(c.bounds[i + 1]));
        let room = to.saturating_sub(from);
        let number = (i + 1).to_string();
        match c.owner_bank.get(i).copied().flatten() {
            Some(b) => {
                let plate = BANK_COLORS[b % BANK_COLORS.len()];
                let pads = c.owners.get(i).cloned().unwrap_or_default();
                let text: Vec<char> = format!("{number} {pads}").chars().collect();
                let end = (from + room.saturating_sub(1)).max(from + 1).min(width);
                for (k, cell) in nums.iter_mut().take(end).skip(from).enumerate() {
                    *cell = (text.get(k).copied().unwrap_or(' '), BG, Some(plate));
                }
            }
            None => {
                let label: Vec<(char, Color, Option<Color>)> = number
                    .chars()
                    .map(|ch| (ch, DIM, None))
                    .chain((number.len() < room).then_some(('○', BORDER, None)))
                    .collect();
                for (k, cell) in label.into_iter().enumerate() {
                    if let Some(at) = nums.get_mut(from + k) {
                        *at = cell;
                    }
                }
            }
        }
    }
    if let Some(at) = c.cursor
        && let Some(cell) = nums.get_mut(((at * width as f32) as usize).min(width - 1))
    {
        cell.0 = '!';
        if cell.2.is_none() {
            cell.1 = DIM;
        }
    }
    frame.render_widget(
        Line::from(
            nums.into_iter()
                .map(|(ch, fg, bg)| {
                    let style = Style::new().fg(fg);
                    Span::styled(ch.to_string(), bg.map_or(style, |b| style.bg(b)))
                })
                .collect::<Vec<_>>(),
        ),
        numbers,
    );

    // what play covers now: a green bar right above the wave; the hits of the hits mode
    // stand on the same row as ▾ (51)
    let mut bar: Vec<Span> = (0..width).map(|_| Span::raw(" ")).collect();
    if let Some((from, to)) = c.play {
        // a span narrower than a cell still shows one
        let (first, last) = (col(from), col(to.max(from)).max(col(from)));
        for (x, cell) in bar.iter_mut().enumerate() {
            if (first..=last).contains(&x) && (x < last || last == first || to >= 1.0) {
                *cell = Span::styled("━", Style::new().fg(METER));
            }
        }
    }
    for &h in &c.hits {
        if let Some(cell) = bar.get_mut(col(h)) {
            *cell = Span::styled("▾", Style::new().fg(TEXT).bold());
        }
    }
    frame.render_widget(Line::from(bar), play_bar);

    let sel = (
        c.bounds.get(c.selected).copied().unwrap_or(0.0),
        c.bounds.get(c.selected + 1).copied().unwrap_or(1.0),
    );
    // the wave: Braille cells, or blank cells with the picture described for the
    // terminal to draw under the marks (04b)
    // the color tells what plays (38): the band above its average takes the column; every
    // slice of the region is at full color, the selection is a lighter background below
    let at_color = |x: usize| {
        let at = (x as f32 + 0.5) / width.max(1) as f32;
        let (plain, bright) = if at < first || at > last {
            (BORDER, 0.35)
        } else if at >= sel.0 && at < sel.1 {
            (TEXT, 1.0)
        } else {
            (DIM, 1.0)
        };
        if c.bands.is_empty() {
            return plain;
        }
        let b = c.bands[(x * c.bands.len() / width.max(1)).min(c.bands.len() - 1)];
        // silence has no color to tell: its flat line keeps the plain tone
        if b.iter().sum::<f32>() < 1e-4 {
            return plain;
        }
        let color = band_color(b, c.band_mean, bright);
        // the selected slice is the one in color; the rest is grey at its brightness (40)
        if at >= sel.0 && at < sel.1 && at >= first && at <= last {
            color
        } else {
            grey(color)
        }
    };
    let graphics = images.is_some();
    if let Some(out) = images.as_mut() {
        let top = c.wave.iter().fold(0f32, |m, x| m.max(x.abs())).max(1e-6);
        out.push(WaveImage {
            id: 1,
            area: wave,
            peaks: dot_peaks(c.wave, width * 2, top),
            colors: (0..width).map(at_color).collect(),
        });
    }
    let rows = if graphics {
        vec![" ".repeat(width); wave.height as usize]
    } else {
        braille(c.wave, width, wave.height as usize)
    };
    let mut lines: Vec<Vec<Span>> = rows
        .iter()
        .map(|r| {
            r.chars()
                .enumerate()
                .map(|(x, ch)| Span::from(ch.to_string()).fg(at_color(x)))
                .collect()
        })
        .collect();
    let mut mark = |at: f32, ch: &'static str, color: Color| {
        for line in lines.iter_mut() {
            if let Some(span) = line.get_mut(col(at)) {
                *span = Span::styled(ch, Style::new().fg(color).bold());
            }
        }
    };
    for (k, &b) in c.bounds.iter().enumerate() {
        let edge = k == 0 || k == slices;
        // the range edges are drawn only when trimmed
        if edge && (b <= 0.0 || b >= 1.0) && k != c.selected && k != c.selected + 1 {
            continue;
        }
        if k == c.selected || k == c.selected + 1 {
            // the bound in play differs by colour, not by weight
            let hot = c.hot == Some(k == c.selected + 1);
            mark(b, "│", if hot { AMBER } else { MAGENTA });
        } else {
            mark(b, "│", CYAN);
        }
    }
    // the playhead is a thin line of its own; the owner's cursor is drawn over it
    if let Some(at) = c.playhead {
        mark(at, "│", METER);
    }
    // the cursor on the lit bound wears its colour: the bound must not hide under it
    let hot_at = c
        .hot
        .and_then(|end| c.bounds.get(c.selected + usize::from(end)).copied());
    if let Some(at) = c.cursor {
        let on_hot = hot_at.is_some_and(|h| col(h) == col(at));
        mark(at, "┃", if on_hot { AMBER } else { WHITE });
    }
    frame.render_widget(
        Paragraph::new(lines.into_iter().map(Line::from).collect::<Vec<_>>()),
        wave,
    );
    if let Some(marks) = c.banks {
        let mut spans = vec!["bank ".fg(DIM)];
        for (i, (taken, planned)) in marks.iter().enumerate() {
            let glyphs = bank_glyphs(c.pads.bank_fill[i]);
            // the bank of the source wears a ◆ (37)
            let src = if c.source_banks[i] { "◆" } else { "" };
            let text = if i == c.pads.bank {
                format!("[{}{src} {glyphs}]", i + 1)
            } else {
                format!(" {}{src} {glyphs} ", i + 1)
            };
            let color = match (planned, taken) {
                (true, _) => MAGENTA,
                (false, true) => TEXT,
                (false, false) => DIM,
            };
            let style = Style::new().fg(color);
            spans.push(Span::styled(
                text,
                if i == c.pads.bank {
                    style.bold()
                } else {
                    style
                },
            ));
        }
        frame.render_widget(Line::from(spans), pf.bank);
    }
    if let Some((a, b)) = c.slice_keys {
        frame.render_widget(
            Line::from(vec![
                Span::styled(format!(" slices {a}–{b} "), Style::new().fg(BG).bg(CYAN)),
                "  the keys play the slices; once one lies on a pad, the grid shows the pads"
                    .fg(DIM),
            ]),
            pf.bank,
        );
    }
    if c.pads.hidden {
        // the listen mode has no bank row: the line names the pad and its slice
        frame.render_widget(pad_line(&c.pads, Vec::new(), false), pf.bank);
    } else {
        pad_grid(frame, pf.grid, &c.pads, pf.waves);
    }

    // the slice edit takes the right column in the frame of the pad edit (98f)
    if c.edit {
        slice_edit(frame, pf.side, c);
        return;
    }
    // the right column: the slice line on top, wrapped with the side list, not cut
    let mut slice_line = Vec::new();
    if c.edit {
        slice_line.push(Span::styled(
            " EDIT · BOUNDS ",
            Style::new().fg(BG).bg(MAGENTA).bold(),
        ));
    }
    let lines: Vec<Line> = slice_rows(slice_line, &c.info, pf.side.width)
        .into_iter()
        .chain(c.side.iter().enumerate().flat_map(|(i, line)| {
            side_rows(line, pf.side.width)
                .into_iter()
                .map(move |line| Line::from(line).fg(if i == 0 { CYAN } else { DIM }))
        }))
        .collect();
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), pf.side);
}

/// The slice edit in the right column: the frame and title of the pad edit, the slice as
/// its block, the two bounds that I/K selects and two dim information rows.
fn slice_edit(frame: &mut Frame, area: Rect, c: &Chop) {
    let area = Rect {
        height: area.height.min(7),
        ..area
    };
    if area.width < 3 || area.height < 3 {
        return;
    }
    let mut title = Vec::new();
    if usize::from(area.width.saturating_sub(2)) >= " EDIT · BOUNDS ".len() {
        title.push(Span::styled(" EDIT ·", Style::new().fg(MAGENTA).bold()));
    }
    title.push(" ".into());
    title.push(Span::styled(
        "BOUNDS",
        Style::new().fg(BG).bg(MAGENTA).bold(),
    ));
    title.push(" ".into());
    let block = Block::bordered()
        .border_style(Style::new().fg(MAGENTA))
        .title(Line::from(title));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let name = format!("SLICE {}", c.bounds_slice);
    let rule = "─".repeat(usize::from(inner.width).saturating_sub(name.chars().count() + 3));
    let mut lines =
        vec![Line::from(format!("─ {name} {rule}")).style(Style::new().fg(MAGENTA).bold())];
    let end = c.hot == Some(true);
    for (row, (label, value)) in ["start", "end", "length", "cursor"]
        .into_iter()
        .zip(&c.edit_rows)
        .enumerate()
    {
        let selected = row == usize::from(end);
        let style = if selected {
            Style::new().fg(CYAN).bold()
        } else if row < 2 {
            Style::new().fg(TEXT)
        } else {
            Style::new().fg(DIM)
        };
        lines.push(Line::from(vec![
            Span::raw(if selected { "▸ " } else { "  " }),
            Span::styled(format!("{label:<8}"), Style::new().fg(DIM)),
            Span::styled(value.clone(), style),
        ]));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

/// The Chop slice line in rows of `width`: it wraps between its ` · ` parts, so a number
/// stays with its unit (98d). A part wider than the column wraps by words later.
fn slice_rows(first: Vec<Span<'static>>, info: &str, width: u16) -> Vec<Line<'static>> {
    let width = usize::from(width);
    let mut rows = Vec::new();
    let mut row = first;
    let mut used: usize = row.iter().map(Span::width).sum();
    for (index, part) in info.split(" · ").enumerate() {
        let text = if index == 0 || used == 0 {
            format!(" {part}")
        } else {
            format!(" · {part}")
        };
        let cells = Span::raw(text.as_str()).width();
        if used > 0 && used + cells > width && index > 0 {
            // the separator opens the next row, so no text of the line goes away
            rows.push(Line::from(std::mem::take(&mut row)));
            used = cells;
            row.push(text.fg(CYAN));
            continue;
        }
        used += cells;
        row.push(text.fg(CYAN));
    }
    rows.push(Line::from(row));
    rows
}
