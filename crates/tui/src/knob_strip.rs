use super::{BG, BRIGHT, CYAN, DIM, Feedback, HELP_ROWS, Help, MAGENTA, MARK, PANEL, Screen, TEXT};
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
    /// The dim line without a mode: how PAD BANK shows the commands (138).
    pub idle: &'static str,
}

const STRIP_WIDTH: u16 = 40;

impl Feedback {
    pub fn content_area(&self, mut area: Rect) -> Rect {
        if self.knob_strip.is_some() && area.width >= 72 && area.height >= 26 {
            area.height -= 1;
        }
        area
    }

    pub fn screen_content_area(&self, area: Rect, screen: &Screen<'_>, help: &Help<'_>) -> Rect {
        if help.full.is_none() && super::pattern::compact_a1(area, screen) {
            area
        } else {
            self.content_area(area)
        }
    }
}

pub(super) fn help_width(area: Rect, feedback: &Feedback, compact: bool) -> u16 {
    if compact || feedback.content_area(area) == area {
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
    compact: bool,
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
        if compact {
            let line = if let Some(mode) = &strip.mode {
                Line::from(fit_mode(mode, usize::from(area.width)))
                    .fg(MAGENTA)
                    .bold()
            } else {
                let mut prefix = format!("◀ ▶ {} · {}  ", strip.list, strip.page);
                let (role, ready) = strip.roles[0][0];
                if strip.page == "Pattern" && strip.list == "Bar" && ready {
                    prefix.push_str(&format!("E1 {role}  "));
                }
                let status = strip
                    .value
                    .as_deref()
                    .or(strip.operation)
                    .unwrap_or(strip.idle);
                let width =
                    usize::from(area.width).saturating_sub(Span::raw(prefix.as_str()).width());
                Line::from(vec![prefix.fg(CYAN), fit_mode(status, width).fg(TEXT)])
            };
            frame.render_widget(
                Paragraph::new(std::iter::once(line).chain(rows).collect::<Vec<_>>()),
                Rect {
                    height: frame.area().bottom().saturating_sub(area.y),
                    ..area
                },
            );
        } else {
            draw_strip(frame, area, strip, rows);
        }
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
        None => help.push(Line::from(strip.idle).fg(DIM)),
    }
    let remaining = usize::from(HELP_ROWS + 1).saturating_sub(help.len());
    help.extend(rows.into_iter().take(remaining));
    frame.render_widget(Paragraph::new(help), right);
}

/// The mode line in `width` cells (135, 138): the middle steps give way first. The last part
/// is the way out and stays whole; a line with apply and cancel keeps both.
fn fit_mode(mode: &str, width: usize) -> String {
    let fits = |text: &str| Span::raw(text).width() <= width;
    let parts: Vec<&str> = mode.split(" · ").collect();
    if fits(mode) || parts.len() < 2 {
        return clip_label(mode, width);
    }
    let (head, last) = (parts[0], parts[parts.len() - 1]);
    let name = head.split(' ').next().unwrap_or(head);
    let mut candidates = Vec::new();
    if parts.len() > 2 {
        let tail = &parts[parts.len() - 2..];
        // "PAD BANK + 4 apply · PAD BANK + 1 cancel" says the held button once
        let held: Option<Vec<&str>> = tail
            .iter()
            .map(|part| part.strip_prefix("PAD BANK + "))
            .collect();
        candidates.push(format!("{head} · {}", tail.join(" · ")));
        if let Some(held) = held {
            let short = format!("PAD BANK: {}", held.join(" · "));
            candidates.push(format!("{head} · {short}"));
            candidates.push(format!("{name} · {short}"));
        }
        candidates.push(format!("{name} · {}", tail.join(" · ")));
    }
    candidates.push(format!("{head} · {last}"));
    candidates.push(format!("{name} · {last}"));
    candidates
        .into_iter()
        .find(|candidate| fits(candidate))
        .unwrap_or_else(|| {
            // even the mode name gives way before the way out
            let room = width.saturating_sub(Span::raw(last).width() + 3);
            if room == 0 {
                clip_label(last, width)
            } else {
                format!("{} · {last}", clip_label(name, room))
            }
        })
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

    /// The way out stays whole in 40 cells, also without apply (138, F12).
    #[test]
    fn the_mode_line_keeps_its_way_out() {
        let edit = "EDIT pad 13 · FILTER · LEFT RIGHT page · PAD BANK + 1 close";
        assert_eq!(fit_mode(edit, 40), "EDIT pad 13 · PAD BANK + 1 close");
        let long = "ASK-A-VERY-LONG-MODE-NAME-HERE · PAD BANK + 1 close";
        assert!(fit_mode(long, 30).ends_with(" · PAD BANK + 1 close"));
        assert_eq!(fit_mode("COUNT 4", 40), "COUNT 4");
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
            idle: "PAD BANK hold · PLAY play · STOP stop",
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
