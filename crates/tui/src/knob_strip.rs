use super::{BG, BRIGHT, CYAN, DIM, Feedback, HELP_ROWS, Help, MAGENTA, MARK, PANEL, TEXT};
use super::{Block, Frame, Line, Paragraph, Rect, Span, Style, Stylize, clip_label};

/// Physical SMC-PAD knob roles, in bank and knob order.
pub struct KnobStrip {
    pub page: &'static str,
    pub list: &'static str,
    pub roles: [[(&'static str, bool); 8]; 2],
    pub last_bank: Option<usize>,
}

const STRIP_WIDTH: u16 = 44;

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
    let mut knobs = Vec::new();
    for (row, first) in [7, 5, 3, 1].into_iter().enumerate() {
        let mut line = Vec::new();
        for bank in 0..2 {
            let recent = strip.last_bank == Some(bank);
            let bank_name = if row == 0 {
                if bank == 0 { "B1 " } else { "B2 " }
            } else {
                "   "
            };
            let bank_style = if recent {
                Style::new().fg(BG).bg(CYAN).bold()
            } else {
                Style::new().fg(DIM)
            };
            line.push(Span::styled(bank_name, bank_style));
            for knob in [first, first + 1] {
                let (label, available) = strip.roles[bank][knob - 1];
                let color = if !available {
                    DIM
                } else if recent {
                    BRIGHT
                } else {
                    TEXT
                };
                line.push(Span::raw(format!("{knob} {:<6} ", clip_label(label, 6))).fg(color));
            }
            line.push(Span::raw(" "));
        }
        knobs.push(Line::from(line));
    }
    frame.render_widget(Paragraph::new(knobs).style(Style::new().bg(PANEL)), left);
    let last = strip
        .last_bank
        .map_or(String::new(), |bank| format!(" · last {}", bank + 1));
    let mut help = vec![Line::from(format!("< > {} · {}{last}", strip.list, strip.page)).fg(CYAN)];
    help.extend(rows.into_iter().take(HELP_ROWS as usize));
    frame.render_widget(Paragraph::new(help), right);
}
