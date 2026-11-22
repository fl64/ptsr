//! Shared effects screen.

use super::*;

/// One project-wide effect. Rows contain the type, its two parameters, and the return.
/// Labels and values come from the app; this view does not interpret effect settings.
pub struct FxBlock {
    pub rows: [(&'static str, String); 4],
}

/// Shared effects screen (F4). Block and parameter indices start at zero.
pub struct Fx {
    pub blocks: [FxBlock; 2],
    pub selected: usize,
    pub param: usize,
    /// A takeover hint for the selected block, shown above the bottom help.
    pub pickup: Option<String>,
    /// Learned controls of the selected block: (parameter index, control label).
    pub learn: Vec<(usize, String)>,
    /// The parameter that changed a moment ago, in the selected block.
    pub lit: Option<usize>,
}

pub(super) fn draw(frame: &mut Frame, area: Rect, f: &Fx, s: &Status) {
    let [header, body] = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);
    let mut heading = title(
        vec![Span::styled(" F4 FX ", Style::new().fg(BG).bg(TEXT))],
        &s.badges,
        header.width,
        24,
    );
    heading.spans.push("· shared project effects".fg(DIM));
    frame.render_widget(heading, header);
    let label_width = f
        .blocks
        .iter()
        .flat_map(|block| &block.rows)
        .map(|(label, _)| Line::from(label.trim()).width())
        .max()
        .unwrap_or(0);
    // one edit frame for both slots, as the pad edit: the slots are its pages (98e)
    let selected = f.selected.min(1);
    let row_width = f
        .blocks
        .iter()
        .enumerate()
        .flat_map(|(i, block)| block.rows.iter().enumerate().map(move |r| (i, r)))
        .map(|(i, (param, (label, value)))| {
            let mut line = Line::from(format!(
                " {} {:label_width$} {value}",
                param + 1,
                label.trim()
            ));
            if i == selected
                && let Some((_, control)) = f.learn.iter().find(|(p, _)| *p == param)
            {
                line.spans.push(Span::raw(format!(" ◀ {control}")));
            }
            line.width()
        })
        .max()
        .unwrap_or(0);
    let width = (row_width.max(34) as u16 + 2).min(body.width);
    let framed = body.height >= 10;
    let show_return_row = body.height >= if framed { 12 } else { 8 };
    let row_count = if show_return_row { 4 } else { 3 };
    // a low terminal drops the frame and the block lines, and names the slot before its
    // first row, so both slots fit
    let width = if framed {
        width
    } else {
        body.width.min(width + 7)
    };
    let area = Rect::new(
        body.x,
        body.y,
        width,
        body.height
            .min((row_count as u16) * 2 + if framed { 4 } else { 0 }),
    );
    let text_area = if framed {
        let pages = ["FX 1", "FX 2"];
        let pages_len: usize = pages.iter().map(|n| n.chars().count() + 1).sum::<usize>() + 1;
        let mut title = Vec::new();
        if pages_len + 7 <= usize::from(width.saturating_sub(2)) {
            title.push(Span::styled(" EDIT ·", Style::new().fg(MAGENTA).bold()));
        }
        for (i, name) in pages.into_iter().enumerate() {
            title.push(" ".into());
            title.push(if i == selected {
                Span::styled(name, Style::new().fg(BG).bg(MAGENTA).bold())
            } else {
                name.fg(DIM)
            });
        }
        title.push(" ".into());
        let block = Block::bordered()
            .border_style(Style::new().fg(MAGENTA))
            .title(Line::from(title));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        inner
    } else {
        area
    };
    let inner_width = usize::from(text_area.width);
    let mut lines = Vec::new();
    for (i, view) in f.blocks.iter().enumerate() {
        let is_selected = i == selected;
        let color = if is_selected { MAGENTA } else { DIM };
        if framed {
            let mut name = format!("FX {} · {}", i + 1, view.rows[0].1);
            if !show_return_row {
                name.push_str(&format!(" · {}", view.rows[3].1));
            }
            let rule = "─".repeat(inner_width.saturating_sub(name.chars().count() + 3));
            lines.push(
                Line::from(format!("─ {name} {rule}")).style(if is_selected {
                    Style::new().fg(MAGENTA).bold()
                } else {
                    Style::new().fg(DIM)
                }),
            );
        }
        for (param, (label, value)) in view.rows.iter().take(row_count).enumerate() {
            let style = if !is_selected {
                Style::new().fg(DIM)
            } else if f.lit == Some(param) {
                Style::new().fg(BG).bg(CYAN).bold()
            } else if f.param.min(3) == param {
                Style::new().fg(CYAN).bold()
            } else {
                Style::new().fg(TEXT)
            };
            let mut spans = Vec::new();
            if !framed {
                spans.push(if param == 0 {
                    Span::styled(
                        format!("{} FX {} ", if is_selected { "▶" } else { " " }, i + 1),
                        Style::new().fg(color).bold(),
                    )
                } else {
                    "       ".into()
                });
            }
            spans.push(Span::styled(
                pads::fx_row_knob(param, is_selected)
                    .map_or("  ".into(), |knob| format!("{knob} ")),
                Style::new().fg(color),
            ));
            spans.push(Span::styled(
                format!("{:label_width$} ", label.trim()),
                style,
            ));
            spans.push(Span::styled(value.clone(), style));
            if is_selected && let Some((_, control)) = f.learn.iter().find(|(p, _)| *p == param) {
                spans.extend(learn_spans(&format!(" ◀ {control}")));
            }
            if !framed && !show_return_row && param == 0 {
                let return_style = if is_selected && f.param == 3 {
                    Style::new().fg(CYAN).bold()
                } else {
                    Style::new().fg(color)
                };
                spans.push(Span::styled(format!(" · {}", view.rows[3].1), return_style));
            }
            lines.push(Line::from(spans));
        }
    }
    frame.render_widget(Paragraph::new(lines), text_area);
}

#[cfg(test)]
mod tests {
    use crate::tests::{draw_rows, shared_fx};

    fn text(rows: &[Vec<(String, ratatui::style::Color, ratatui::style::Color)>]) -> Vec<String> {
        rows.iter()
            .map(|row| row.iter().map(|(symbol, _, _)| symbol.as_str()).collect())
            .collect()
    }

    /// Scenario "FX rows show their knobs" (screen-layout): knobs 2–4 turn the selected slot.
    #[test]
    fn fx_rows_show_their_knobs() {
        crate::pads::set_fx_knobs(Some([2, 3, 4, 5]));
        let rows = text(&draw_rows(&crate::Screen::Fx(shared_fx()), 120, 40));
        crate::pads::set_fx_knobs(None);
        let joined = rows.join("\n");
        let slot = |n: usize| {
            let start = rows
                .iter()
                .position(|row| row.contains(&format!("FX {n} ·")))
                .unwrap();
            rows[start + 1..start + 4].to_vec()
        };
        for (row, knob) in slot(1).iter().zip(["2 ", "3 ", "4 "]) {
            assert!(row.trim_start_matches('│').starts_with(knob), "{joined}");
        }
        for row in slot(2) {
            let body = row.trim_start_matches('│');
            assert!(!body.starts_with(char::is_numeric), "{joined}");
        }
    }
}
