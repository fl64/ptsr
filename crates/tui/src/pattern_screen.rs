//! The Pattern screen (F5): one header line, the grid, the velocity lane, the
//! detail line and the setup line.

use super::*;

/// The visible time page of the pattern at the current view division.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternOverview {
    /// A message that replaces the grid when the pattern cannot show.
    pub header: String,
    /// Transport state and position, for example `▶ 1.3` or `Stopped`.
    pub status: String,
    /// Loop length and tempo, for example `2 bars  120 BPM`.
    pub length: String,
    /// The active mode badge, for example `STEP EDIT` or `● REC 1.2`.
    pub mode: Option<String>,
    pub state_label: Option<String>,
    pub next: Option<String>,
    pub rows: Vec<Vec<PatternCell>>,
    pub names: Vec<String>,
    pub active_pads: [u16; BANKS],
    pub hide_empty: bool,
    pub beat_cells: usize,
    pub playhead: Option<usize>,
    pub first_cell: usize,
    pub total_cells: usize,
    pub editing: bool,
    pub cursor: Option<usize>,
    pub cursor_hint: Option<String>,
    pub event_choice: Option<(usize, usize)>,
    pub event_velocity: Option<u8>,
    /// What the selected step holds, for example `step 1.1 · kick · loud (vel 127)`.
    pub detail: Option<String>,
    /// The always visible setup values.
    pub setup: String,
    /// Two lines for a pattern with no visible hits.
    pub empty: Option<[String; 2]>,
}

/// The step cells of one page and the pad label width.
///
/// A page holds one bar when every step gets three columns. Otherwise it holds
/// half a bar, then one beat. A page never splits a beat.
pub(super) fn overview_page_layout(
    width: usize,
    beat_cells: usize,
    total: usize,
) -> (usize, usize) {
    let beat_cells = beat_cells.max(1);
    let bar_cells = 4 * beat_cells;
    let label = (width * 15 / 100).clamp(10, 20);
    let room = width.saturating_sub(label + 1);
    let count = [bar_cells, 2 * beat_cells, beat_cells]
        .into_iter()
        .find(|&count| 3 * count <= room)
        .unwrap_or(beat_cells);
    (count.min(total.max(beat_cells)), label)
}

/// The step cells of one overview page at this terminal width.
pub fn pattern_overview_page_cells(width: usize, beat_cells: usize, total: usize) -> usize {
    overview_page_layout(width, beat_cells, total).0
}

/// The velocity level of a hit: soft, medium or loud.
pub fn velocity_level(velocity: u8) -> &'static str {
    match velocity {
        0 => "no hit",
        1..=42 => "soft",
        43..=84 => "medium",
        _ => "loud",
    }
}

fn hit_glyph(cell: &PatternCell) -> char {
    match cell.count {
        0 => '·',
        1 if cell.off_grid => '~',
        1 => match cell.max_velocity {
            0..=42 => '▪',
            43..=84 => '▓',
            _ => '█',
        },
        count @ 2..=9 => char::from(b'0' + count as u8),
        _ => '+',
    }
}

fn lane_glyph(cell: &PatternCell) -> char {
    const LEVELS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    if cell.count == 0 {
        '·'
    } else {
        LEVELS[usize::from(cell.max_velocity.min(127)) * 8 / 128]
    }
}

fn hit_color(cell: &PatternCell) -> Color {
    match (cell.count, cell.max_velocity) {
        (0, _) => STEP_EMPTY,
        (_, 0..=42) => DIM,
        (_, 43..=84) => TEXT,
        _ => WHITE,
    }
}

/// The rows to show: pads with visible hits and the selected pad, in bank order.
/// With empty rows shown, the selected bank also shows all 16 rows.
fn shown_rows(view: &PatternOverview, selected: usize) -> Vec<usize> {
    (0..BANKS * 16)
        .filter(|&abs| {
            let (bank, row) = (abs / 16, abs % 16);
            view.active_pads[bank] & (1 << row) != 0
                || abs == selected
                || (!view.hide_empty && bank == selected / 16)
        })
        .collect()
}

pub(super) fn pattern_overview(frame: &mut Frame, area: Rect, pads: &Pads, view: &PatternOverview) {
    if area.width < 40 || area.height == 0 {
        return;
    }
    let width = area.width as usize;
    let beat_cells = view.beat_cells.max(1);
    let bar_cells = 4 * beat_cells;
    let count = view.rows.first().map_or(0, Vec::len);
    let (_, label_width) = overview_page_layout(width, beat_cells, view.total_cells);
    let first = view.first_cell;
    let line = |y: u16| Rect::new(area.x, y, area.width, 1);

    // One header line: screen, transport, length, page and the mode badge.
    let pages = view.total_cells.div_ceil(count.max(1)).max(1);
    let page = first / count.max(1) + 1;
    let bars = view.total_cells / bar_cells;
    let mut title = format!(" F5 PATTERN  {}", view.status);
    if !view.length.is_empty() {
        title.push_str(&format!("  {}", view.length));
    }
    if view.total_cells > 0 {
        let unit = if count >= bar_cells { "bar" } else { "page" };
        let total = if count >= bar_cells { bars } else { pages };
        title.push_str(&format!("  {unit} {page}/{total}"));
    }
    if let Some(state) = &view.state_label {
        title.push_str(&format!("  {state}"));
    }
    let badge = view.mode.as_ref().map(|mode| format!("[{mode}] "));
    let badge_width = badge.as_deref().map_or(0, |badge| Span::raw(badge).width());
    let title = clip_label(&title, width.saturating_sub(badge_width));
    let gap = width.saturating_sub(Span::raw(&title).width() + badge_width);
    let mut spans = vec![
        Span::styled(title, Style::new().fg(CYAN).bold()),
        Span::raw(" ".repeat(gap)),
    ];
    if let Some(badge) = badge {
        spans.push(Span::styled(badge, Style::new().fg(MARK).bold()));
    }
    frame.render_widget(Line::from(spans), line(area.y));

    // The detail and setup lines follow the grid. They keep two rows at the bottom.
    let bottom = area.bottom();
    let grid_bottom = if area.height >= 4 { bottom - 2 } else { bottom };
    let footer = |frame: &mut Frame, y: u16| footer(frame, area, view, y.min(grid_bottom));

    if let Some([record, steps]) = &view.empty {
        for (offset, text) in [record, steps].into_iter().enumerate() {
            let y = area.y + 2 + offset as u16;
            if y < grid_bottom {
                frame.render_widget(Line::from(format!("  {text}")).fg(TEXT), line(y));
            }
        }
        footer(frame, area.y + 5);
        return;
    }
    if view.total_cells == 0 || count == 0 {
        if area.height > 2 {
            frame.render_widget(
                Line::from(clip_label(&format!(" {}", view.header), width)).fg(TEXT),
                line(area.y + 2),
            );
        }
        footer(frame, area.y + 4);
        return;
    }

    let cell_width = (width.saturating_sub(label_width + 1) / count).max(3);
    let cell_style = |cell: usize, base: Style| {
        if view.cursor == Some(cell) {
            Style::new()
                .fg(if view.playhead == Some(cell) {
                    PLATE
                } else {
                    BG
                })
                .bg(CYAN)
                .bold()
        } else if view.playhead == Some(cell) {
            Style::new().fg(BG).bg(PLATE).bold()
        } else if (cell / beat_cells) % 2 == 1 {
            base.bg(BEAT_BG)
        } else {
            base
        }
    };
    let centered = |glyph: String, style: Style| {
        let left = (cell_width - 1) / 2;
        [
            Span::styled(" ".repeat(left), style),
            Span::styled(glyph, style),
            Span::styled(" ".repeat(cell_width - left - 1), style),
        ]
    };
    let label = |text: String, style: Style| {
        let text = clip_label(&text, label_width);
        let padding = label_width + 1 - Span::raw(text.as_str()).width();
        Span::styled(format!("{text}{}", " ".repeat(padding)), style)
    };

    // Beat numbers over their first cell.
    if area.height > 1 {
        let mut spans = vec![label(String::new(), Style::new())];
        for cell in first..first + count {
            let glyph = if cell.is_multiple_of(beat_cells) {
                ((cell % bar_cells) / beat_cells + 1).to_string()
            } else {
                "·".into()
            };
            spans.extend(centered(glyph, cell_style(cell, Style::new().fg(DIM))));
        }
        frame.render_widget(Line::from(spans), line(area.y + 1));
    }

    let selected = pads.bank * 16 + pads.selected;
    let rows = shown_rows(view, selected);
    // Room under the beat numbers, above the velocity lane. Start early enough to
    // fill it, but late enough that the selected row and its bank rules fit.
    let room = usize::from(grid_bottom.saturating_sub(area.y + 3));
    let at = rows.iter().position(|&abs| abs == selected).unwrap_or(0);
    let mut start = at;
    let mut used = 1;
    while start > 0 {
        let rule = usize::from(rows[start - 1] / 16 != rows[start] / 16);
        if used + 1 + rule > room {
            break;
        }
        used += 1 + rule;
        start -= 1;
    }
    let mut y = area.y + 2;
    let mut last_bank = None;
    let mut drawn = 0;
    for &abs in rows.iter().skip(start) {
        if y + 1 >= grid_bottom {
            break;
        }
        let bank = abs / 16;
        if last_bank.is_some_and(|last| last != bank) {
            frame.render_widget(
                Line::from(" ".to_string() + &"─".repeat(width - 2)).fg(DIM),
                line(y),
            );
            y += 1;
            if y + 1 >= grid_bottom {
                break;
            }
        }
        last_bank = Some(bank);
        let key = pads
            .pads
            .get(abs % 16)
            .map_or("?".to_string(), |pad| pads::key_tag(pad.key));
        let name = view.names.get(abs).map_or("empty", String::as_str);
        let marker = if abs == selected { '▶' } else { ' ' };
        let style = Style::new().fg(BANK_COLORS[bank]);
        let mut spans = vec![label(
            format!("{marker} {key} {name}"),
            if abs == selected { style.bold() } else { style },
        )];
        let cells = view.rows.get(abs);
        for offset in 0..count {
            let mark = cells
                .and_then(|cells| cells.get(offset))
                .cloned()
                .unwrap_or_default();
            let style = cell_style(first + offset, Style::new().fg(hit_color(&mark)));
            spans.extend(centered(hit_glyph(&mark).to_string(), style));
        }
        frame.render_widget(Line::from(spans), line(y));
        y += 1;
        drawn += 1;
    }
    let hidden = rows.len() - drawn;
    if hidden > 0 && area.height > 1 {
        frame.render_widget(
            Line::from(format!(" +{hidden} rows")).fg(DIM),
            Rect::new(area.x, area.y + 1, label_width as u16, 1),
        );
    }

    // The velocity lane of the selected row.
    if y < grid_bottom {
        let mut spans = vec![label(" velocity".into(), Style::new().fg(DIM))];
        let cells = view.rows.get(selected);
        for offset in 0..count {
            let mark = cells
                .and_then(|cells| cells.get(offset))
                .cloned()
                .unwrap_or_default();
            let style = cell_style(first + offset, Style::new().fg(hit_color(&mark)));
            spans.extend(centered(lane_glyph(&mark).to_string(), style));
        }
        frame.render_widget(Line::from(spans), line(y));
    }
    footer(frame, y + 1);
}

/// The detail line and, under it, the setup line.
fn footer(frame: &mut Frame, area: Rect, view: &PatternOverview, y: u16) {
    let width = area.width as usize;
    if area.height < 4 || y + 1 >= area.bottom() + 1 {
        return;
    }
    let parts = [
        view.detail.as_deref(),
        view.next.as_deref(),
        view.cursor_hint.as_deref(),
    ];
    let mut detail = String::new();
    for part in parts.into_iter().flatten() {
        let next = if detail.is_empty() {
            format!(" {part}")
        } else {
            format!("{detail}  ·  {part}")
        };
        if Span::raw(next.as_str()).width() <= width || detail.is_empty() {
            detail = next;
        }
    }
    let line = |y: u16| Rect::new(area.x, y, area.width, 1);
    frame.render_widget(Line::from(clip_label(&detail, width)).fg(CYAN), line(y));
    frame.render_widget(
        Line::from(clip_label(&format!(" setup  {}", view.setup), width)).fg(DIM),
        line(y + 1),
    );
}
