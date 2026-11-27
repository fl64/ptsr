//! Pattern strip and overview drawing.

use super::*;

/// The rhythm of the whole loop. The drawing picks the page that fits its width.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternStrip {
    pub header: String,
    pub label: String,
    pub cells: Vec<PatternCell>,
    pub all: Vec<bool>,
    pub tracks: Vec<(String, Vec<PatternCell>)>,
    pub beat_cells: usize,
    pub playhead: Option<usize>,
}

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
    let label = |name: &str| {
        let name = clip_name(name, PATTERN_LABEL);
        let padding = " ".repeat(PATTERN_LABEL - Span::raw(&name).width());
        Span::styled(format!("{name}{padding}"), Style::new().fg(TEXT))
    };
    let mut rows = vec![
        (
            Span::styled(format!("{:<5}", "all"), Style::new().fg(DIM)),
            all,
            TEXT,
        ),
        (label(&pattern.label), pad, WHITE),
    ];
    rows.extend(
        pattern
            .tracks
            .iter()
            .take(usize::from(area.height.saturating_sub(3)))
            .map(|(name, cells)| (label(name), cells.iter().map(pad_glyph).collect(), WHITE)),
    );
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

pub(super) fn compact_a1(area: Rect, screen: &Screen<'_>) -> bool {
    area.width <= 80 && a1_overview(screen)
}

pub(super) fn a1_overview(screen: &Screen<'_>) -> bool {
    matches!(screen, Screen::Pads(pads) if pads.overview.is_some())
}

pub(super) fn compact_a1_body(content: Rect) -> Rect {
    let [_, body, _, _] = frame_areas(content);
    Rect::new(
        body.x,
        body.y.saturating_add(1),
        body.width,
        body.height.saturating_sub(1),
    )
}

pub(super) fn controls_area(area: Rect, screen: &Screen) -> Rect {
    if compact_a1(area, screen) {
        return compact_a1_body(area);
    }
    let [_, body, _, _] = frame_areas(area);
    let side = match screen {
        Screen::Pads(pads) => pad_frame(body, pads.hidden).side,
        Screen::Files(files) => pad_frame(body, files.pads.hidden).side,
        Screen::Chop(chop) => pad_frame(body, chop.pads.hidden).side,
        _ => Rect::new(
            body.right().saturating_sub(body.width.min(SIDE_W)),
            body.y.saturating_add(1),
            body.width.min(SIDE_W),
            body.height.saturating_sub(1),
        ),
    };
    let top = (METER_ROWS + 1).min(side.height);
    Rect {
        y: side.y + top,
        height: side.height - top,
        ..side
    }
}
