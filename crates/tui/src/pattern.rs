//! Pattern strip and overview drawing.

use super::*;

pub(super) fn pattern_strip(frame: &mut Frame, area: Rect, pattern: &PatternStrip) {
    let total = pattern.cells.len().min(pattern.all.len());
    let beat_cells = pattern.beat_cells.max(1);
    let page = pattern_page_cells(area.width as usize, beat_cells, total.max(1));
    let first = pattern.playhead.filter(|&cell| cell < total).unwrap_or(0) / page * page;
    let range = first..(first + page).min(total);
    // the legend for a count in a cell comes last, so a narrow strip cuts it before the
    // pad name (98b)
    let header = if pattern.cells[range.clone()]
        .iter()
        .any(|cell| cell.count > 1)
    {
        format!("{} · 2-9 = hits in a cell, one plays", pattern.header)
    } else {
        pattern.header.clone()
    };
    frame.render_widget(
        Line::from(clip_label(&header, area.width as usize)).fg(CYAN),
        Rect { height: 1, ..area },
    );
    if total == 0 {
        return;
    }
    let pad_glyph = |cell: &PatternCell| match cell.count {
        0 => '·',
        1 if cell.off_grid => '~',
        1 => '■',
        count @ 2..=9 => char::from(b'0' + count as u8),
        _ => '+',
    };
    let all: Vec<char> = pattern
        .all
        .iter()
        .map(|&hit| if hit { '■' } else { '·' })
        .collect();
    let pad: Vec<char> = pattern.cells.iter().map(pad_glyph).collect();
    let pad_label = clip_name(&pattern.label, PATTERN_LABEL);
    let pad_label = format!(
        "{pad_label}{}",
        " ".repeat(PATTERN_LABEL - Span::raw(&pad_label).width())
    );
    let rows = [
        (
            Span::styled(format!("{:<5}", "all"), Style::new().fg(DIM)),
            all,
            TEXT,
        ),
        (Span::styled(pad_label, Style::new().fg(TEXT)), pad, WHITE),
    ];
    for (row, (label, glyphs, hit_color)) in rows.into_iter().enumerate() {
        // a beat is " g g g g ": every glyph has a space on both sides, so the playback
        // plate over a glyph and its two neighbour spaces stays centred
        let mut columns: Vec<(char, Style)> = Vec::new();
        let mut plate = None;
        for index in range.clone() {
            let shade = if (index / beat_cells) % 2 == 1 {
                Style::new().bg(BEAT_BG)
            } else {
                Style::new()
            };
            if index != first && index.is_multiple_of(4 * beat_cells) {
                columns.push(('│', Style::new().fg(DIM)));
            }
            if index.is_multiple_of(beat_cells) {
                columns.push((' ', shade));
            }
            let glyph = glyphs[index];
            if pattern.playhead == Some(index) {
                plate = Some(columns.len());
            }
            let color = if glyph == '·' { STEP_EMPTY } else { hit_color };
            columns.push((glyph, shade.fg(color)));
            columns.push((' ', shade));
        }
        if let Some(at) = plate {
            for column in &mut columns[at - 1..=at + 1] {
                column.1 = Style::new().fg(BG).bg(PLATE).bold();
            }
        }
        let mut spans = vec![label];
        spans.extend(
            columns
                .into_iter()
                .map(|(glyph, style)| Span::styled(glyph.to_string(), style)),
        );
        frame.render_widget(
            Line::from(spans),
            Rect {
                y: area.y + 1 + row as u16,
                height: 1,
                ..area
            },
        );
    }
}

pub(super) fn pattern_overview(frame: &mut Frame, area: Rect, pads: &Pads, view: &PatternOverview) {
    if area.width < 40 || area.height == 0 {
        return;
    }
    let width = area.width as usize;
    let count = view.rows.first().map_or(0, Vec::len);
    let beat_cells = view.beat_cells.max(1);
    let label_width = overview_page_layout(width, beat_cells, view.total_cells).1;
    let grid_width = width - label_width - 2;
    let bar_cells = 4 * beat_cells;
    let first = view.first_cell;
    let end = (first + count).min(view.total_cells);
    let mut title = if let Some(cursor) = view.edit_cursor {
        let choice = match view.event_choice {
            Some((index, count)) => format!("event {index}/{count}"),
            None => "empty cell".into(),
        };
        format!(
            "STEP EDIT · A1 · B{}/8 · step {}/{} · {choice} · {}",
            pads.bank + 1,
            cursor + 1,
            view.total_cells,
            view.header
        )
    } else {
        format!("PATTERN · A1 · B{}/8 · {}", pads.bank + 1, view.header)
    };
    if view.total_cells > 0 && view.edit_cursor.is_none() {
        title.push_str(&format!(
            " · steps {}-{}/{}",
            first + 1,
            end,
            view.total_cells
        ));
    }
    let selected = pads.bank * 16 + pads.selected;
    if let (Some(name), Some(pad)) = (view.names.get(selected), pads.pads.get(pads.selected)) {
        let detail = format!(" · {} {name}", pad.key.to_ascii_uppercase());
        if name != "empty" && Span::raw(format!("{title}{detail}")).width() <= width {
            title.push_str(&detail);
        }
    }
    frame.render_widget(
        Line::from(clip_label(&title, width)).fg(CYAN).bold(),
        Rect::new(area.x, area.y, area.width, 1),
    );
    if area.height < 2 {
        return;
    }

    let mut index = vec![Span::styled(
        "BANKS · pads with A1 hits ",
        Style::new().fg(DIM),
    )];
    for (bank, &color) in BANK_COLORS.iter().enumerate() {
        let hits = view.active_pads[bank].count_ones();
        let label = if bank == pads.bank {
            format!("[{}:{}] ", bank + 1, hits)
        } else {
            format!("{}:{} ", bank + 1, hits)
        };
        let style = if bank == pads.bank {
            Style::new().fg(color).bold().underlined()
        } else {
            Style::new().fg(color)
        };
        index.push(Span::styled(label, style));
    }
    index.push(Span::styled(
        if view.hide_empty {
            "empty hidden"
        } else {
            "empty shown"
        },
        Style::new().fg(DIM),
    ));
    frame.render_widget(
        Line::from(index),
        Rect::new(area.x, area.y + 1, area.width, 1),
    );
    if area.height < 3 {
        return;
    }

    let separators = (1..count)
        .filter(|offset| (first + offset).is_multiple_of(beat_cells))
        .count();
    let usable = grid_width.saturating_sub(separators);
    if count > 0 && usable < 2 * count {
        frame.render_widget(
            Line::from("Widen the terminal to show this view division.").fg(DIM),
            Rect::new(area.x, area.y + 2, area.width, 1),
        );
        return;
    }
    let base = usable.checked_div(count).unwrap_or(0);
    let extra = usable.checked_rem(count).unwrap_or(0);
    let widths = (0..count)
        .map(|offset| base + ((offset + 1) * extra / count - offset * extra / count))
        .collect::<Vec<_>>();
    let mut numbers = vec![Span::raw(" ".repeat(label_width + 1))];
    for (offset, &cell_width) in widths.iter().enumerate() {
        let cell = first + offset;
        if offset > 0 && cell.is_multiple_of(beat_cells) {
            numbers.push(Span::styled(
                if cell.is_multiple_of(bar_cells) {
                    "│"
                } else {
                    " "
                },
                Style::new().fg(DIM),
            ));
        }
        numbers.push(Span::styled(
            format!("{:^cell_width$}", cell % bar_cells + 1),
            Style::new().fg(DIM),
        ));
    }
    frame.render_widget(
        Line::from(numbers),
        Rect::new(area.x, area.y + 2, area.width, 1),
    );

    let visible = (0..BANKS)
        .filter(|&bank| !view.hide_empty || view.active_pads[bank] != 0 || bank == pads.bank)
        .collect::<Vec<_>>();
    let selected_pos = visible
        .iter()
        .position(|&bank| bank == pads.bank)
        .unwrap_or(0);
    let full_sections = (area.height as usize).saturating_sub(3) / 18;
    let start = selected_pos.saturating_sub(full_sections.saturating_sub(1));
    let mut y = area.y + 3;
    for &bank in visible.iter().skip(start) {
        if y >= area.bottom() {
            break;
        }
        let color = BANK_COLORS[bank];
        let current = bank == pads.bank;
        let style = if current {
            Style::new().fg(color).bold()
        } else {
            Style::new().fg(color)
        };
        let active_pads = view.active_pads[bank].count_ones();
        let section_name = format!(
            " BANK {} / 8 · {} {} with hits ",
            bank + 1,
            active_pads,
            if active_pads == 1 { "pad" } else { "pads" }
        );
        let fill = width.saturating_sub(2 + Span::raw(section_name.as_str()).width());
        let top = format!(
            "{}{}{}{}",
            if current { '┏' } else { '╭' },
            section_name,
            "━".repeat(fill),
            if current { '┓' } else { '╮' }
        );
        frame.render_widget(
            Line::from(top).style(style),
            Rect::new(area.x, y, area.width, 1),
        );
        y += 1;

        for row in 0..16 {
            if y >= area.bottom() {
                break;
            }
            let abs = bank * 16 + row;
            let key = pads
                .pads
                .get(row)
                .map_or('?', |pad| pad.key.to_ascii_uppercase());
            let name = view.names.get(abs).map_or("empty", String::as_str);
            let prefix = format!(" {key} ");
            let text = format!(
                "{prefix}{}",
                clip_name(name, label_width.saturating_sub(Span::raw(&prefix).width()))
            );
            let text_width = Span::raw(text.as_str()).width();
            let text = format!("{text}{}", " ".repeat(label_width - text_width));
            let selected = current && row == pads.selected;
            let mut spans = vec![
                Span::styled(if current { "┃" } else { "│" }, style),
                Span::styled(text, Style::new().fg(if selected { CYAN } else { TEXT })),
            ];
            let cells = view.rows.get(abs);
            for (offset, &cell_width) in widths.iter().enumerate() {
                let cell = first + offset;
                if offset > 0 && cell.is_multiple_of(beat_cells) {
                    spans.push(Span::styled(
                        if cell.is_multiple_of(bar_cells) {
                            "│"
                        } else {
                            " "
                        },
                        Style::new().fg(DIM),
                    ));
                }
                let mark = cells.and_then(|cells| cells.get(offset));
                let glyph = match mark.map_or(0, |mark| mark.count) {
                    0 => '·',
                    1 if mark.is_some_and(|mark| mark.off_grid) => '~',
                    1 => '■',
                    count @ 2..=9 => char::from(b'0' + count as u8),
                    _ => '+',
                };
                let shade = if (cell / beat_cells) % 2 == 1 {
                    Style::new().bg(BEAT_BG)
                } else {
                    Style::new()
                };
                let cell_style = if view.edit_cursor == Some(cell) && selected {
                    Style::new().fg(BG).bg(CYAN).bold()
                } else if view.edit_cursor == Some(cell) {
                    Style::new().fg(WHITE).bg(BEAT_BG).bold()
                } else if view.playhead == Some(cell) {
                    Style::new().fg(BG).bg(PLATE).bold()
                } else {
                    shade.fg(if glyph == '·' { STEP_EMPTY } else { WHITE })
                };
                let left = (cell_width - 1) / 2;
                let cursor = view.edit_cursor == Some(cell) && selected;
                let mut before = " ".repeat(left);
                let mut after = " ".repeat(cell_width - left - 1);
                if cursor && cell_width >= 3 {
                    before.replace_range(..1, "[");
                    after.replace_range(after.len() - 1.., "]");
                }
                spans.push(Span::styled(before, cell_style));
                spans.push(Span::styled(glyph.to_string(), cell_style));
                spans.push(Span::styled(after, cell_style));
            }
            spans.push(Span::styled(if current { "┃" } else { "│" }, style));
            frame.render_widget(Line::from(spans), Rect::new(area.x, y, area.width, 1));
            y += 1;
        }
        if y < area.bottom() {
            let bottom = format!(
                "{}{}{}",
                if current { '┗' } else { '╰' },
                "━".repeat(width - 2),
                if current { '┛' } else { '╯' }
            );
            frame.render_widget(
                Line::from(bottom).style(style),
                Rect::new(area.x, y, area.width, 1),
            );
            y += 1;
        }
    }
}
