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
    help.push(Line::from("PAD BANK panel · PLAY play · STOP stop").fg(DIM));
    let remaining = usize::from(HELP_ROWS + 1).saturating_sub(help.len());
    help.extend(rows.into_iter().take(remaining));
    frame.render_widget(Paragraph::new(help), right);
}

#[cfg(test)]
mod tests {
    use super::*;
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
