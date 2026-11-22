use super::*;

pub struct BigView {
    pub wave: Vec<f32>,
    pub env: Vec<f32>,
    pub attack: f32,
    pub decay: f32,
    pub release: f32,
}

pub struct SourceStrip {
    pub focus: SourceSample,
    pub others: Vec<SourceSample>,
    pub pinned: bool,
}

pub struct SourceSample {
    pub name: String,
    pub peaks: Arc<[f32]>,
    pub ranges: Vec<SourceRange>,
    pub voices: Vec<SourceVoice>,
}

pub struct SourceRange {
    pub start: f32,
    pub end: f32,
    pub label: String,
    pub sounding: bool,
}

pub struct SourceVoice {
    pub start: f32,
    pub end: f32,
    pub position: f32,
    pub level: f32,
    pub label: String,
}

fn column(position: f32, width: usize) -> usize {
    ((position * width as f32) as usize).min(width.saturating_sub(1))
}

fn draw_wave(frame: &mut Frame, area: Rect, sample: &SourceSample) {
    let width = usize::from(area.width);
    if width == 0 || area.height == 0 {
        return;
    }
    let mut markers = vec![(0u8, false); width];
    for range in &sample.ranges {
        markers[column(range.start, width)].1 = true;
        if range.end < 1.0 {
            markers[column(range.end, width)].1 = true;
        }
    }
    for voice in &sample.voices {
        let start = column(voice.start, width);
        let end = ((voice.end * width as f32).ceil() as usize).min(width);
        let shade = if voice.level >= 0.65 {
            4
        } else if voice.level >= 0.25 {
            3
        } else {
            2
        };
        for (mark, _) in &mut markers[start..end.max(start + 1)] {
            *mark = (*mark).max(shade);
        }
    }
    for voice in &sample.voices {
        markers[column(voice.position, width)].0 = 5;
    }
    let rows = braille(&sample.peaks, width, usize::from(area.height));
    for (row, text) in rows.iter().enumerate() {
        let spans: Vec<_> = text
            .chars()
            .zip(&markers)
            .map(|(glyph, &(mark, cut))| {
                let glyph = if cut { '┆' } else { glyph };
                let (glyph, color) = match mark {
                    5 => ('│', WHITE),
                    4 => (glyph, CYAN),
                    3 => (glyph, Color::Rgb(0x25, 0xAA, 0xC0)),
                    2 => (glyph, Color::Rgb(0x30, 0x73, 0x84)),
                    _ => (glyph, DIM),
                };
                Span::styled(glyph.to_string(), Style::new().fg(color))
            })
            .collect();
        frame.render_widget(
            Line::from(spans),
            Rect {
                y: area.y + row as u16,
                height: 1,
                ..area
            },
        );
    }
}

fn draw_ranges(frame: &mut Frame, area: Rect, ranges: &[SourceRange]) {
    let mut next = 0;
    for range in ranges {
        let start = column(range.start, usize::from(area.width)).max(next);
        let text = Span::styled(
            &range.label,
            Style::new().fg(if range.sounding { CYAN } else { TEXT }),
        );
        next = start + text.width() + 1;
        if next > usize::from(area.width) + 1 {
            break;
        }
        frame.render_widget(
            Line::from(text),
            Rect::new(area.x + start as u16, area.y, area.width - start as u16, 1),
        );
    }
}

pub(super) fn draw(frame: &mut Frame, area: Rect, source: &SourceStrip) {
    if area.height < 2 || area.width == 0 {
        return;
    }
    let count = source.focus.voices.len()
        + source
            .others
            .iter()
            .map(|sample| sample.voices.len())
            .sum::<usize>();
    let status = match count {
        0 => "silent".into(),
        1 => "▶ 1 voice".into(),
        _ => format!("▶ {count} voices"),
    };
    let status_width = Span::raw(&status).width().min(usize::from(area.width)) as u16;
    let row_limit = if frame.area().width >= 120 && frame.area().height >= 40 {
        2
    } else {
        1
    };
    let available = usize::from(area.height.saturating_sub(2));
    let visible = source.others.len().min(row_limit.min(available));
    let hidden = &source.others[visible..];
    let overflow_row = !hidden.is_empty() && available > visible;
    let mut wave = Rect::new(
        area.x,
        area.y + 1,
        area.width,
        area.height - 1 - (visible + usize::from(overflow_row)) as u16,
    );
    let title_width = usize::from(area.width.saturating_sub(status_width + 1));
    let prefix = if source.pinned {
        "Sound · pinned · "
    } else {
        "Sound · "
    };
    let names = if wave.height == 1 {
        source
            .focus
            .ranges
            .iter()
            .filter(|range| range.sounding)
            .map(|range| range.label.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        String::new()
    };
    let suffix = if names.is_empty() {
        names
    } else {
        format!(" · {names}")
    };
    let filename_width =
        title_width.saturating_sub(Span::raw(prefix).width() + Span::raw(&suffix).width());
    let header = format!(
        "{prefix}{}{suffix}",
        clip_name(&source.focus.name, filename_width)
    );
    frame.render_widget(
        Line::styled(header, Style::new().fg(TEXT)),
        Rect::new(area.x, area.y, title_width as u16, 1),
    );
    frame.render_widget(
        Line::styled(status, Style::new().fg(if count == 0 { DIM } else { CYAN })).right_aligned(),
        Rect::new(area.right() - status_width, area.y, status_width, 1),
    );
    if wave.height > 1 && !source.focus.ranges.is_empty() {
        draw_ranges(frame, wave, &source.focus.ranges);
        wave.y += 1;
        wave.height -= 1;
    }
    draw_wave(frame, wave, &source.focus);

    let inline_overflow = !hidden.is_empty() && !overflow_row && visible > 0;
    let overflow_width = if inline_overflow {
        area.width / 2
    } else {
        area.width
    };
    for (index, sample) in source.others.iter().take(visible).enumerate() {
        let y = wave.bottom() + index as u16;
        let row_width = if inline_overflow && index + 1 == visible {
            area.width - overflow_width
        } else {
            area.width
        };
        let label_width = (row_width / 3).min(20);
        frame.render_widget(
            Line::styled(
                clip_name(&sample.name, usize::from(label_width.saturating_sub(1))),
                Style::new().fg(TEXT),
            ),
            Rect::new(area.x, y, label_width, 1),
        );
        draw_wave(
            frame,
            Rect::new(area.x + label_width, y, row_width - label_width, 1),
            sample,
        );
    }
    if overflow_row || inline_overflow {
        frame.render_widget(
            Line::styled(
                overflow_text(hidden, usize::from(overflow_width)),
                Style::new().fg(TEXT),
            ),
            Rect::new(
                area.right() - overflow_width,
                area.bottom() - 1,
                overflow_width,
                1,
            ),
        );
    }
}

fn overflow_text(hidden: &[SourceSample], width: usize) -> String {
    let mut pads: Vec<_> = hidden
        .iter()
        .flat_map(|sample| sample.voices.iter().map(|voice| voice.label.as_str()))
        .collect();
    pads.sort_unstable();
    pads.dedup();
    let mut text = format!("+ {} more · ", hidden.len());
    for (index, pad) in pads.iter().enumerate() {
        let separator = if index == 0 { "" } else { ", " };
        let room = usize::from(index + 1 != pads.len());
        if Line::raw(&text).width() + separator.len() + pad.len() + room > width {
            text.push('…');
            break;
        }
        text.push_str(separator);
        text.push_str(pad);
    }
    text
}
