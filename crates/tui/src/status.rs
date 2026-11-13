use super::*;

/// Height of a peak on the meters' decibel scale: 0 silent, 1 full scale.
pub(super) fn db_height(p: f32) -> f32 {
    if p > 0.0 {
        ((20.0 * p.log10()).max(METER_FLOOR) - METER_FLOOR) / -METER_FLOOR
    } else {
        0.0
    }
}

/// L/R meters of the master output: a dB bar per channel with a peak-hold mark,
/// and the latching overload light.
pub(super) fn meters(frame: &mut Frame, area: Rect, s: &Status) {
    let (level, peak, clip) = s.meters;
    let rows = Layout::vertical([Constraint::Length(1); METER_ROWS as usize]).split(area);
    // " L " + bar + " -60 dB": the number keeps its width across the range
    let bar_w = (area.width.saturating_sub(11)) as usize;
    let x_of = |db: f32| {
        (((db - METER_FLOOR) / -METER_FLOOR) * bar_w as f32)
            .round()
            .clamp(0.0, bar_w as f32) as usize
    };
    for (ch, name) in [0usize, 1].into_iter().zip(["L", "R"]) {
        let (lvl, peak) = (x_of(level[ch]), x_of(peak[ch]));
        let mut bar = "█".repeat(lvl);
        if peak > lvl {
            bar.push_str(&" ".repeat(peak - lvl - 1));
            bar.push('▎');
        }
        let bar = format!("{bar:<bar_w$}");
        let line = Line::from(vec![
            Span::styled(format!(" {name} "), Style::new().fg(DIM)),
            Span::styled(bar, Style::new().fg(METER)),
            Span::styled(format!(" {:>4.0} dB", level[ch]), Style::new().fg(BRIGHT)),
        ]);
        frame.render_widget(line, rows[ch]);
    }
    let light = if clip {
        Span::styled(" CLIP ", Style::new().fg(BG).bg(MAGENTA))
    } else {
        Span::styled(" clip ", Style::new().fg(DIM))
    };
    frame.render_widget(Line::from(light), rows[2]);
}

/// Input level: -60..0 dB over 30 cells.
pub(super) fn meter(level: f32) -> Span<'static> {
    const WIDTH: usize = 30;
    let db = 20.0 * level.max(1e-6).log10();
    let filled = (((db + 60.0) / 60.0).clamp(0.0, 1.0) * WIDTH as f32).round() as usize;
    let text = format!(
        "{}{} {db:>5.0} dB",
        "█".repeat(filled),
        "░".repeat(WIDTH - filled)
    );
    if level >= 1.0 {
        text.fg(MAGENTA)
    } else {
        text.fg(CYAN)
    }
}

/// The status marker of a terminal without key releases, in the owner's words (68).
pub(super) const NO_RELEASES: &str = "no key releases: gate pads play one-shot";

/// Clip at grapheme boundaries within a terminal-cell budget. Labels cannot inject rows or controls.
pub(super) fn clip_label(text: &str, width: usize) -> String {
    let text: String = text
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let span = Span::raw(text.as_str());
    if span.width() <= width {
        return text;
    }
    let mut clipped = String::new();
    let mut used = 0;
    for grapheme in span.styled_graphemes(Style::default()) {
        let cells = Span::raw(grapheme.symbol).width();
        if used + cells > width.saturating_sub(1) {
            break;
        }
        clipped.push_str(grapheme.symbol);
        used += cells;
    }
    if width > 0 {
        clipped.push('…');
    }
    clipped
}

pub(super) fn status_inputs(inputs: &[InputSource], stage: u8) -> Vec<Span<'static>> {
    if stage >= 6 {
        return Vec::new();
    }
    if inputs.is_empty() {
        return vec![" KB".fg(DIM)];
    }
    let mut spans = Vec::new();
    for input in inputs {
        if !spans.is_empty() {
            spans.push(Span::raw(" "));
        }
        // Keep both marker cells and the application's reserved controller width.
        let label = clip_label(&input.compact, usize::MAX);
        let span = Span::raw(format!("{}{label}", if input.active { "▸" } else { " " }));
        spans.push(if input.active {
            span.fg(CYAN).bold()
        } else {
            span.fg(DIM)
        });
    }
    spans
}

pub(super) fn status_candidate(s: &Status, feedback: &Feedback, stage: u8) -> Line<'static> {
    let full_audio = stage < 2;
    let brief = stage >= 7;
    let mut segments = Vec::new();
    segments.push(s.save.spans());
    if stage != 0 || s.project.is_none() {
        segments.push(vec![s.save.marker()]);
    }
    if s.recording {
        let text = match feedback.recording_seconds {
            Some(seconds) if !brief => format!("rec {seconds}s"),
            _ => "rec".into(),
        };
        segments.push(vec![text.fg(MAGENTA).bold()]);
    }
    segments.push(status_inputs(&feedback.inputs, stage));
    match s.backend {
        None => segments.push(vec!["no audio".fg(MAGENTA).bold()]),
        Some(backend) if full_audio => {
            let ms = s.buffer_size as f32 / s.sample_rate.max(1) as f32 * 1000.0;
            let size = if s.buffer_size == 0 {
                "auto".into()
            } else {
                s.buffer_size.to_string()
            };
            segments.push(vec![Span::raw(format!(
                "{backend} {size} / {} kHz · {ms:.1} ms · master {:+} dB",
                s.sample_rate as f64 / 1000.0,
                s.master_db
            ))]);
        }
        _ => {}
    }
    if s.xruns > 0 {
        let text = if brief {
            "xruns >0".into()
        } else {
            format!("xruns {}", s.xruns)
        };
        segments.push(vec![text.fg(MAGENTA).bold()]);
    }
    if s.no_releases {
        segments.push(vec![NO_RELEASES.fg(DIM)]);
    }
    if stage == 0
        && let Some(project) = s.project
    {
        segments.push(vec![
            Span::raw(clip_label(project, usize::MAX)),
            Span::raw(" "),
            s.save.marker(),
        ]);
    }
    let separator = if stage < 3 { " · " } else { " " };
    let mut spans = Vec::new();
    for segment in segments.into_iter().filter(|segment| !segment.is_empty()) {
        if !spans.is_empty() {
            spans.push(separator.fg(DIM));
        }
        spans.extend(segment);
    }
    Line::from(spans)
}

pub(super) fn status_line_at(s: &Status, feedback: &Feedback, width: u16) -> Line<'static> {
    for stage in 0..=7 {
        let line = status_candidate(s, feedback, stage);
        if line.width() <= width as usize {
            return line;
        }
    }
    // Only unsupported tiny terminals can truncate the minimum safety line.
    status_candidate(s, feedback, 7)
}

#[cfg(test)]
pub(super) fn status_line(s: &Status) -> Line<'static> {
    status_line_at(s, &Feedback::default(), u16::MAX)
}
