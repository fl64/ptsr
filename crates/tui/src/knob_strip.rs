use super::{BG, BRIGHT, CYAN, DIM, Feedback, HELP_ROWS, Help, MAGENTA, MARK, PANEL, TEXT};
use super::{Block, Frame, Line, Paragraph, Rect, Span, Style, Stylize, clip_label};

/// Physical SMC-PAD knob roles, in bank and knob order.
pub struct KnobStrip {
    pub page: &'static str,
    pub list: &'static str,
    pub operation: Option<&'static str>,
    pub roles: [[(&'static str, bool); 8]; 2],
    pub last_bank: Option<usize>,
    /// The last turned knob of `last_bank`, as index 0–7.
    pub last_knob: Option<usize>,
    /// The knob, value name, target and amount while its value pop-up shows.
    pub value: Option<String>,
    /// The active mode, its next step and its cancel control (134).
    pub mode: Option<String>,
}

const STRIP_WIDTH: u16 = 40;

impl Feedback {
    pub fn content_area(&self, mut area: Rect) -> Rect {
        if self.knob_strip.is_some() && area.width >= 72 && area.height >= 26 {
            area.height -= 1;
        }
        area
    }
}

pub(super) fn help_width(area: Rect, feedback: &Feedback) -> u16 {
    if feedback.content_area(area) == area {
        area.width
    } else {
        area.width - STRIP_WIDTH
    }
}

pub(super) fn draw_help(
    frame: &mut Frame,
    area: Rect,
    help: &Help,
    feedback: &Feedback,
    rows: Vec<Line<'_>>,
) {
    if let Some(learn) = help.learn {
        frame.render_widget(Block::new().bg(PANEL), area);
        let lines = [
            Line::from(clip_label(&learn.context, area.width as usize))
                .fg(MARK)
                .bold(),
            Line::from(clip_label(&learn.detail, area.width as usize)).fg(if learn.warning {
                MAGENTA
            } else {
                DIM
            }),
            Line::from(clip_label(&learn.controls, area.width as usize)).fg(CYAN),
        ];
        frame.render_widget(Paragraph::new(lines.to_vec()), area);
    } else if let Some(strip) = &feedback.knob_strip
        && feedback.content_area(frame.area()) != frame.area()
    {
        draw_strip(frame, area, strip, rows);
    } else {
        frame.render_widget(Paragraph::new(rows), area);
    }
}

fn draw_strip(frame: &mut Frame, area: Rect, strip: &KnobStrip, rows: Vec<Line<'_>>) {
    let left = Rect::new(area.x, area.y, STRIP_WIDTH, HELP_ROWS + 1);
    let right = Rect::new(left.right(), area.y, area.width - left.width, HELP_ROWS + 1);
    // Bank 1 in physical rows; bank 2 has only master, so it takes one cell (127).
    let mark = |bank: usize| strip.last_bank == Some(bank);
    let cell = |bank: usize, knob: usize| {
        let (label, available) = strip.roles[bank][knob - 1];
        let color = if !available {
            DIM
        } else if mark(bank) {
            BRIGHT
        } else {
            TEXT
        };
        let span = Span::raw(format!("{knob} {:<6} ", clip_label(label, 6))).fg(color);
        if mark(bank) && strip.last_knob == Some(knob - 1) {
            span.fg(BG).bg(CYAN)
        } else {
            span
        }
    };
    let bank_name = |bank: usize, text: &'static str| {
        Span::styled(
            text,
            if mark(bank) {
                Style::new().fg(BG).bg(CYAN).bold()
            } else {
                Style::new().fg(DIM)
            },
        )
    };
    let mut knobs = Vec::new();
    for (row, first) in [7, 5, 3, 1].into_iter().enumerate() {
        let mut line = vec![
            if row == 0 {
                bank_name(0, "B1 ")
            } else {
                Span::raw("   ")
            },
            cell(0, first),
            cell(0, first + 1),
        ];
        if row == 0 {
            line.push(bank_name(1, " B2 "));
            line.push(cell(1, 8));
        }
        knobs.push(Line::from(line));
    }
    frame.render_widget(Paragraph::new(knobs).style(Style::new().bg(PANEL)), left);
    let mut help = vec![Line::from(format!("◀ ▶ {} · {}", strip.list, strip.page)).fg(CYAN)];
    if let Some(value) = &strip.value {
        help.push(
            Line::from(clip_label(value, usize::from(right.width)))
                .fg(BRIGHT)
                .bold(),
        );
    }
    if let Some(operation) = strip.operation {
        help.push(Line::from(operation).fg(TEXT));
    }
    match &strip.mode {
        Some(mode) => help.push(
            Line::from(fit_mode(mode, usize::from(right.width)))
                .fg(MAGENTA)
                .bold(),
        ),
        None => help.push(Line::from("PAD BANK hold · PLAY play · STOP stop").fg(DIM)),
    }
    let remaining = usize::from(HELP_ROWS + 1).saturating_sub(help.len());
    help.extend(rows.into_iter().take(remaining));
    frame.render_widget(Paragraph::new(help), right);
}

/// The mode line in `width` cells (135): the middle steps give way first, so the mode name
/// and its last two controls (apply and cancel) stay readable at 80×26.
fn fit_mode(mode: &str, width: usize) -> String {
    let fits = |text: &str| Span::raw(text).width() <= width;
    let parts: Vec<&str> = mode.split(" · ").collect();
    if fits(mode) || parts.len() < 3 {
        return clip_label(mode, width);
    }
    let (head, tail) = (parts[0], &parts[parts.len() - 2..]);
    let name = head.split(' ').next().unwrap_or(head);
    // "PAD BANK + 4 apply · PAD BANK + 1 cancel" says the held button once
    let held: Option<Vec<&str>> = tail
        .iter()
        .map(|part| part.strip_prefix("PAD BANK + "))
        .collect();
    let short = held.map(|held| format!("PAD BANK: {}", held.join(" · ")));
    let mut candidates = vec![format!("{head} · {}", tail.join(" · "))];
    if let Some(short) = &short {
        candidates.push(format!("{head} · {short}"));
        candidates.push(format!("{name} · {short}"));
    }
    candidates.push(format!("{name} · {}", tail.join(" · ")));
    candidates
        .into_iter()
        .find(|candidate| fits(candidate))
        .unwrap_or_else(|| clip_label(mode, width))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Assign mode line keeps its apply and cancel controls in 40 cells (135).
    #[test]
    fn the_mode_line_keeps_apply_and_cancel() {
        let smc =
            "ASSIGN 1/4 · slice 1 next · hit a pad · PAD BANK + 4 apply · PAD BANK + 1 cancel";
        assert_eq!(fit_mode(smc, 40), "ASSIGN · PAD BANK: 4 apply · 1 cancel");
        let keys = "ASSIGN 1/4 · slice 1 next · hit a pad · Enter apply · Esc cancel";
        assert_eq!(fit_mode(keys, 40), "ASSIGN 1/4 · Enter apply · Esc cancel");
        assert_eq!(fit_mode(keys, 80), keys, "a wide line stays whole");
    }
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn strip_marks_the_turned_knob_and_shows_its_value() {
        let mut roles = [[("-", false); 8]; 2];
        roles[0][6] = ("volume", true);
        roles[0][7] = ("master", true);
        let strip = KnobStrip {
            page: "TONE",
            list: "Page",
            operation: None,
            roles,
            last_bank: Some(0),
            last_knob: Some(6),
            value: Some("B1 7 volume · pad 1 · +1 dB".into()),
            mode: None,
        };
        for width in [80, 120] {
            let mut term = Terminal::new(TestBackend::new(width, HELP_ROWS + 1)).unwrap();
            term.draw(|frame| draw_strip(frame, frame.area(), &strip, Vec::new()))
                .unwrap();
            let buffer = term.backend().buffer();
            let row = |y: u16| -> String { (0..width).map(|x| buffer[(x, y)].symbol()).collect() };
            assert!(row(1).contains("B1 7 volume · pad 1 · +1 dB"), "{}", row(1));
            let x = row(0).find("7 volume").unwrap() as u16;
            assert_eq!(buffer[(x, 0)].bg, CYAN, "{width}");
            let master = row(0).find("8 master").unwrap() as u16;
            assert_ne!(buffer[(master, 0)].bg, CYAN, "{width}");
        }
    }
}
