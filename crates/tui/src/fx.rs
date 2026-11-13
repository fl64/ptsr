//! Shared effects screen.

use super::*;

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
        body.height.min(if framed { 10 } else { 6 }),
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
            let name = format!("FX {} · {}", i + 1, view.rows[0].1);
            let rule = "─".repeat(inner_width.saturating_sub(name.chars().count() + 3));
            lines.push(
                Line::from(format!("─ {name} {rule}")).style(if is_selected {
                    Style::new().fg(MAGENTA).bold()
                } else {
                    Style::new().fg(DIM)
                }),
            );
        }
        for (param, (label, value)) in view.rows.iter().enumerate() {
            let style = if !is_selected {
                Style::new().fg(DIM)
            } else if f.lit == Some(param) {
                Style::new().fg(BG).bg(CYAN).bold()
            } else if f.param.min(2) == param {
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
                format!("{} ", param + 1),
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
            lines.push(Line::from(spans));
        }
    }
    frame.render_widget(Paragraph::new(lines), text_area);
}
