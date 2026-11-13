//! Help, messages, controller guide, and confirmation overlays.

use super::*;

pub(super) fn question_area(area: Rect, question: &Question) -> Rect {
    let width = question
        .body
        .iter()
        .chain(&question.answers)
        .map(|line| Line::from(line.as_str()).width())
        .max()
        .unwrap_or(0)
        .saturating_add(4)
        .min(usize::from(area.width)) as u16;
    let height = question
        .body
        .len()
        .saturating_add(question.answers.len())
        .saturating_add(2)
        .min(usize::from(area.height)) as u16;
    Rect::new(
        area.x.saturating_add((area.width - width) / 2),
        area.y.saturating_add((area.height - height) / 2),
        width,
        height,
    )
}

fn question_rows(question: &Question, rows: usize, width: usize) -> Vec<Line<'static>> {
    let answer_rows = rows.saturating_sub(question.body.len().min(4));
    let mut answers = question.answers[..question.answers.len().min(answer_rows)].to_vec();
    if question.answers.len() > answer_rows && !answers.is_empty() {
        let last = answers.len() - 1;
        answers[last] = format!("{} more input answer rows", question.answers.len() - last);
    }
    let body_rows = rows.saturating_sub(answers.len());
    let mut body = question.body[..question.body.len().min(body_rows)].to_vec();
    if question.body.len() > body_rows && body_rows >= 4 {
        let head = body_rows - 3;
        body.truncate(head);
        body.push(format!(
            "{} more target rows",
            question.body.len() - head - 2
        ));
        body.extend_from_slice(&question.body[question.body.len() - 2..]);
    }
    body.iter()
        .enumerate()
        .map(|(i, text)| {
            let line = Line::from(format!(" {}", clip_label(text, width)));
            if i == 0 {
                line.fg(MAGENTA).bold()
            } else {
                line
            }
        })
        .chain(
            answers
                .iter()
                .map(|text| Line::from(format!(" {}", clip_label(text, width))).fg(CYAN)),
        )
        .collect()
}

pub(super) fn question(frame: &mut Frame, area: Rect, question: &Question) {
    let area = question_area(area, question);
    let text = question_rows(
        question,
        usize::from(area.height.saturating_sub(2)),
        usize::from(area.width.saturating_sub(3)),
    );
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(text).block(
            Block::bordered()
                .border_style(Style::new().fg(MAGENTA))
                .bg(PANEL),
        ),
        area,
    );
}

/// How many rows the help line takes at `width` columns.
pub fn help_rows(groups: &[HelpGroup], width: u16) -> usize {
    help_lines(groups, width).len()
}

/// How many groups drop off at `width` columns, from the end, for want of room.
pub fn help_dropped(groups: &[HelpGroup], width: u16) -> usize {
    let shown = groups.iter().filter(|g| !g.items.is_empty()).count();
    let lines = help_lines(groups, width);
    shown
        - lines.first().map_or(0, |l| {
            l.spans.iter().filter(|s| s.content == " │ ").count() + 1
        })
}

/// The help in `width` columns (43): every group is a column of up to `HELP_ROWS` rows of
/// `key text`, its name beside the first row; a longer group takes more columns under the
/// same name. Groups that do not fit drop off whole from the end (the screens, then REC).
pub(super) fn help_lines(groups: &[HelpGroup], width: u16) -> Vec<Line<'static>> {
    help_lines_at(groups, width, HELP_ROWS as usize)
}

pub(super) fn help_lines_at(groups: &[HelpGroup], width: u16, rows: usize) -> Vec<Line<'static>> {
    let mut blocks: Vec<Vec<Vec<Span<'static>>>> = groups
        .iter()
        .filter(|g| !g.items.is_empty())
        .map(|g| help_block(g, rows))
        .collect();
    let block_width = |b: &Vec<Vec<Span>>| b[0].iter().map(|s| s.width()).sum::<usize>();
    // the leading space, then " │ " between the groups
    let total = |bs: &[Vec<Vec<Span>>]| {
        1 + bs.iter().map(block_width).sum::<usize>() + 3 * bs.len().saturating_sub(1)
    };
    while blocks.len() > 1 && total(&blocks) > width as usize {
        blocks.pop();
    }
    let used = blocks
        .iter()
        .map(|b| {
            b.iter()
                .filter(|r| r.iter().any(|s| !s.content.trim().is_empty()))
                .count()
        })
        .max()
        .unwrap_or(1)
        .max(1);
    (0..used)
        .map(|r| {
            let mut line: Vec<Span> = vec![" ".into()];
            for (i, b) in blocks.iter().enumerate() {
                if i > 0 {
                    line.push(" │ ".fg(BORDER));
                }
                line.extend(b[r].iter().cloned());
            }
            Line::from(line)
        })
        .collect()
}

/// One group as `rows` rows of equal width: the name, then columns of `key text` cells.
/// A group shorter than the band puts its name on the free first row, which saves the
/// name's width.
fn help_block(g: &HelpGroup, rows: usize) -> Vec<Vec<Span<'static>>> {
    if !g.name.is_empty() && g.items.len() < rows {
        let mut inner = help_block(
            &HelpGroup {
                name: "",
                items: g.items.clone(),
            },
            rows - 1,
        );
        let w = inner[0]
            .iter()
            .map(|s| s.width())
            .sum::<usize>()
            .max(g.name.len());
        for row in inner.iter_mut() {
            let have: usize = row.iter().map(|s| s.width()).sum();
            row.push(" ".repeat(w - have).into());
        }
        inner.insert(0, vec![format!("{:<w$}", g.name).fg(DIM)]);
        return inner;
    }
    let name = if g.name.is_empty() {
        String::new()
    } else {
        format!("{} ", g.name)
    };
    let mut out: Vec<Vec<Span>> = (0..rows)
        .map(|r| {
            let text = if r == 0 {
                name.clone()
            } else {
                " ".repeat(name.chars().count())
            };
            vec![text.fg(DIM)]
        })
        .collect();
    for (c, column) in g.items.chunks(rows).enumerate() {
        let caps = column
            .iter()
            .map(|(k, _)| k.chars().count() + 2)
            .max()
            .unwrap_or(0);
        let texts = column
            .iter()
            .map(|(_, t)| t.chars().count())
            .max()
            .unwrap_or(0);
        for (r, row) in out.iter_mut().enumerate() {
            if c > 0 {
                row.push("  ".into());
            }
            match column.get(r) {
                Some((k, t)) => {
                    let cap = key(k);
                    let pad = caps - cap.width();
                    row.push(cap);
                    // the cap's own trailing space parts it from the text
                    row.push(format!("{}{t:<texts$}", " ".repeat(pad)).into());
                }
                None => row.push(" ".repeat(caps + texts).into()),
            }
        }
    }
    out
}

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Help,
    Messages,
    Controller,
}

fn full_help_block(tab: Tab, message_count: Option<usize>, navigation: &str) -> Block<'static> {
    let selected = Style::new().fg(CYAN).bold();
    let idle = Style::new().fg(DIM);
    // Equal-width labels keep every tab name at the same cell on each page.
    let label = |name: &str, this: Tab| {
        if tab == this {
            Span::styled(format!("[{name}]"), selected)
        } else {
            Span::styled(format!(" {name} "), idle)
        }
    };
    let mut title = vec![
        " ".into(),
        label("HELP", Tab::Help),
        " ".into(),
        label("MESSAGES", Tab::Messages),
        " ".into(),
        label("CONTROLLER", Tab::Controller),
    ];
    let mut footer = vec![Span::raw(" Tab/Shift+Tab page · F1/Esc close ")];
    if !navigation.is_empty() {
        footer = vec![Span::raw(format!(
            " Tab/Shift+Tab · F1/Esc · {navigation} "
        ))];
    }
    if let Some(count) = message_count {
        title.push(format!(" · {count} / 50 ").into());
        footer.push("· ↑↓ scroll · PgUp/PgDn page ".into());
    }
    Block::bordered()
        .title(Line::from(title))
        .title_bottom(Line::from(footer))
        .border_style(Style::new().fg(CYAN))
        .bg(PANEL)
}

fn wrap_help_text(text: &str, width: usize) -> Vec<String> {
    let mut rows = Vec::new();
    if width == 0 {
        return rows;
    }
    let cells = |s: &str| Span::raw(s).width();
    for part in text.split('\n') {
        let mut line = String::new();
        // whole words first (98i); a word wider than the column breaks by graphemes
        for word in part.split(' ') {
            let gap = usize::from(!line.is_empty());
            if !line.is_empty() && cells(&line) + gap + cells(word) > width {
                rows.push(std::mem::take(&mut line));
            } else if gap == 1 {
                line.push(' ');
            }
            if cells(word) <= width.saturating_sub(cells(&line)) {
                line.push_str(word);
                continue;
            }
            for grapheme in Span::raw(word).styled_graphemes(Style::default()) {
                let size = cells(grapheme.symbol);
                if size > width {
                    continue;
                }
                if cells(&line) + size > width {
                    rows.push(std::mem::take(&mut line));
                }
                line.push_str(grapheme.symbol);
            }
        }
        rows.push(line);
    }
    rows
}

/// Wrapped key and description columns shared by drawing and page bounds.
pub fn help_key_lines(rows: &[(String, String)], width: u16) -> Vec<Line<'static>> {
    let width = usize::from(width);
    if width == 0 {
        return Vec::new();
    }
    let key_width = rows
        .iter()
        .map(|(k, _)| Span::raw(k).width())
        .max()
        .unwrap_or(0)
        .min(width / 3);
    let mut lines = Vec::new();
    for (keys, description) in rows {
        if keys.is_empty() || width < 8 {
            if !keys.is_empty() {
                lines.extend(wrap_help_text(keys, width).into_iter().map(|text| {
                    Line::from(Span::styled(text, Style::new().fg(BRIGHT).bg(BORDER)))
                }));
            }
            lines.extend(
                wrap_help_text(description, width)
                    .into_iter()
                    .map(|text| Line::from(text).fg(DIM).bold()),
            );
            continue;
        }
        let keys = wrap_help_text(keys, key_width);
        let descriptions = wrap_help_text(description, width - key_width - 3);
        for index in 0..keys.len().max(descriptions.len()) {
            let keys = keys.get(index).map_or("", String::as_str);
            let pad = key_width - Span::raw(keys).width();
            lines.push(Line::from(vec![
                " ".into(),
                Span::styled(keys.to_owned(), Style::new().fg(BRIGHT).bg(BORDER)),
                format!("{:pad$}  ", "").into(),
                descriptions.get(index).cloned().unwrap_or_default().into(),
            ]));
        }
    }
    lines
}

/// Visible text rows in either help tab, after an optional recording caption.
pub fn help_page_height(terminal_height: u16, caption: bool) -> usize {
    messages_page_height(terminal_height).saturating_sub(usize::from(caption))
}

fn help_caption(frame: &mut Frame, inner: Rect, caption: Option<&str>) -> Rect {
    let Some(caption) = caption else { return inner };
    let height = inner.height.min(1);
    frame.render_widget(
        Paragraph::new(caption).fg(MAGENTA).bold(),
        Rect { height, ..inner },
    );
    Rect {
        y: inner.y.saturating_add(height),
        height: inner.height - height,
        ..inner
    }
}

pub(super) fn full_help(frame: &mut Frame, area: Rect, rows: &[(String, String)]) {
    full_help_paged(frame, area, rows, 0, None, "");
}

pub(super) fn full_help_paged(
    frame: &mut Frame,
    area: Rect,
    rows: &[(String, String)],
    scroll: usize,
    caption: Option<&str>,
    navigation: &str,
) {
    let block = full_help_block(Tab::Help, None, navigation);
    let inner = block.inner(area);
    let lines = help_key_lines(rows, inner.width);
    let height = usize::from(inner.height).saturating_sub(usize::from(caption.is_some()));
    let start = scroll.min(lines.len().saturating_sub(height));
    let end = (start + height).min(lines.len());
    let block = block.title_top(
        Line::from(format!(
            " {}-{}/{} ",
            if lines.is_empty() { 0 } else { start + 1 },
            end,
            lines.len()
        ))
        .right_aligned(),
    );
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);
    let inner = help_caption(frame, inner, caption);
    frame.render_widget(
        Paragraph::new(
            lines
                .into_iter()
                .skip(start)
                .take(height)
                .collect::<Vec<_>>(),
        ),
        inner,
    );
}

/// Visible message rows after status, notice, compact help and frame borders.
pub fn messages_page_height(terminal_height: u16) -> usize {
    terminal_height.saturating_sub(HELP_ROWS + 4) as usize
}

/// The same wrapped rows determine drawing and the application's scroll bounds.
pub fn message_lines(entries: &[Message<'_>], width: u16) -> Vec<Line<'static>> {
    fn append(rows: &mut Vec<Line<'static>>, text: &str, width: usize, style: Style) {
        for part in text.split('\n') {
            let span = Span::raw(part);
            let mut line = String::new();
            let mut used = 0;
            for grapheme in span.styled_graphemes(style) {
                let cells = Span::raw(grapheme.symbol).width();
                if used + cells > width && !line.is_empty() {
                    rows.push(Line::from(std::mem::take(&mut line)).style(style));
                    used = 0;
                }
                if cells <= width {
                    line.push_str(grapheme.symbol);
                    used += cells;
                }
            }
            rows.push(Line::from(line).style(style));
        }
    }

    let mut rows = Vec::new();
    if width == 0 {
        return rows;
    }
    if entries.is_empty() {
        append(
            &mut rows,
            "No messages yet",
            width as usize,
            Style::new().fg(DIM),
        );
    }
    for entry in entries {
        let (glyph, label, style) = match entry.level {
            NoticeLevel::Info => ("·", "Info", Style::new().fg(DIM)),
            NoticeLevel::Done => ("✓", "Done", Style::new().fg(CYAN)),
            NoticeLevel::Warn => ("!", "Warn", Style::new().fg(MAGENTA)),
            NoticeLevel::Error => ("✗", "Error", Style::new().fg(MAGENTA).bold()),
        };
        append(
            &mut rows,
            &format!("{glyph} {label} · {}", entry.origin),
            width as usize,
            style,
        );
        append(&mut rows, entry.text, width as usize, style);
        rows.push(Line::default());
    }
    rows
}

pub(super) fn controller(frame: &mut Frame, area: Rect, c: &Controller) {
    let block = full_help_block(Tab::Controller, None, &c.navigation);
    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);
    let inner = help_caption(frame, inner, c.caption.as_deref());
    let width = usize::from(inner.width);
    let mut devices = Vec::new();
    for (i, name) in c.devices.iter().enumerate() {
        if i > 0 {
            devices.push(Span::raw(" · ").fg(DIM));
        }
        devices.push(if i == c.current {
            Span::styled(format!("[{name}]"), Style::new().fg(CYAN).bold())
        } else {
            Span::styled(name.clone(), Style::new().fg(DIM))
        });
    }
    let mut lines = vec![Line::from(devices), Line::default()];
    match &c.body {
        ControllerBody::Empty { profiles } => {
            lines.push(Line::from("No MIDI device is connected."));
            lines.push(Line::default());
            lines.push(Line::from("Known profiles:").fg(DIM));
            for name in profiles {
                lines.push(Line::from(format!(
                    "  {}",
                    clip_label(name, width.saturating_sub(2))
                )));
            }
        }
        ControllerBody::Drawing { rows, selected } => {
            let w = usize::from(PLACE_W);
            let places = rows.iter().map(Vec::len).max().unwrap_or(0);
            let indent = " ".repeat(width.saturating_sub(places * w) / 2);
            for (r, row) in rows.iter().enumerate() {
                let mut names = vec![Span::raw(indent.clone())];
                let mut labels = vec![Span::raw(indent.clone())];
                for (i, place) in row.iter().enumerate() {
                    let Some(cell) = place else {
                        names.push(Span::raw(" ".repeat(w)));
                        labels.push(Span::raw(" ".repeat(w)));
                        continue;
                    };
                    let picked = *selected == (r, i);
                    let style = match (picked, cell.free) {
                        (true, _) => Style::new().fg(BG).bg(CYAN).bold(),
                        (false, true) => Style::new().fg(DIM),
                        (false, false) => Style::new().fg(TEXT),
                    };
                    names.push(Span::styled(
                        format!("{:^w$}", clip_label(&cell.name, w)),
                        Style::new().fg(DIM),
                    ));
                    labels.push(Span::styled(
                        format!("{:^w$}", clip_label(&cell.label, w)),
                        style,
                    ));
                }
                lines.push(Line::from(names));
                lines.push(Line::from(labels));
            }
        }
        ControllerBody::List {
            rows,
            selected,
            scroll,
        } => {
            let name_w = rows
                .iter()
                .map(|(n, _)| n.len())
                .max()
                .unwrap_or(0)
                .min(width / 2);
            for (i, (name, action)) in rows.iter().enumerate().skip(*scroll) {
                let text = format!("{:name_w$}  {action}", clip_label(name, name_w));
                let line = Line::from(clip_label(&text, width));
                lines.push(if i == *selected {
                    line.style(Style::new().fg(BG).bg(CYAN).bold())
                } else {
                    line
                });
                if lines.len() >= usize::from(inner.height.saturating_sub(2)) {
                    break;
                }
            }
        }
    }
    frame.render_widget(Paragraph::new(lines), inner);
    if inner.height > 0 {
        let detail = Rect {
            y: inner.y + inner.height - 1,
            height: 1,
            ..inner
        };
        frame.render_widget(
            Paragraph::new(clip_label(&c.detail, width)).fg(CYAN),
            detail,
        );
    }
}

pub(super) fn message_history(
    frame: &mut Frame,
    area: Rect,
    entries: &[Message<'_>],
    scroll: usize,
) {
    message_history_caption(frame, area, entries, scroll, None);
}

pub(super) fn message_history_caption(
    frame: &mut Frame,
    area: Rect,
    entries: &[Message<'_>],
    scroll: usize,
    caption: Option<&str>,
) {
    let block = full_help_block(Tab::Messages, Some(entries.len()), "");
    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);
    let inner = help_caption(frame, inner, caption);
    let lines = message_lines(entries, inner.width);
    let start = scroll.min(lines.len().saturating_sub(inner.height as usize));
    frame.render_widget(
        Paragraph::new(lines.into_iter().skip(start).collect::<Vec<_>>()),
        inner,
    );
}
