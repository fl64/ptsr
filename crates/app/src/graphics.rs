//! UI-thread image placement around text frames. No device or sound access.

use std::io::{self, Write};

use input::Screen;
use ratatui::layout::Rect;
use tui::{Shown, WaveImage};

#[derive(Default)]
pub(super) struct State {
    shown: Shown,
    geometry: Option<(bool, Screen, Rect, (u16, u16))>,
    // Retain pre-frame deletes for best-effort exit cleanup if writing the frame fails.
    pending: Vec<String>,
}

impl State {
    pub(super) fn before(
        &mut self,
        enabled: bool,
        screen: Screen,
        area: Rect,
        cell_px: (u16, u16),
        blockers: &[Rect],
    ) -> Vec<String> {
        let geometry = (enabled, screen, area, cell_px);
        if self.geometry != Some(geometry) {
            self.pending.extend(self.shown.drop_all());
            self.geometry = Some(geometry);
        }
        if enabled {
            self.pending.extend(self.shown.suspend(blockers));
        }
        self.pending.clone()
    }

    /// Call only after the pre-frame commands and terminal text both succeed.
    pub(super) fn after(&mut self, want: &mut Vec<WaveImage>, blockers: &[Rect]) -> Vec<String> {
        self.pending.clear();
        let Some((true, _, _, cell_px)) = self.geometry else {
            return Vec::new();
        };
        want.retain(|image| {
            !blockers.iter().any(|blocker| {
                blocker.width > 0 && blocker.height > 0 && image.area.intersects(*blocker)
            })
        });
        self.shown.update(want, cell_px)
    }

    pub(super) fn drop_all(&mut self) -> Vec<String> {
        let mut commands = std::mem::take(&mut self.pending);
        commands.extend(self.shown.drop_all());
        commands
    }
}

pub(super) fn send(out: &mut impl Write, commands: &[String]) -> io::Result<()> {
    if commands.is_empty() {
        return Ok(());
    }
    out.write_all(commands.concat().as_bytes())?;
    out.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{
        Terminal, TerminalOptions, Viewport, backend::CrosstermBackend, style::Color,
        widgets::Paragraph,
    };

    fn wave(peak: f32) -> Vec<WaveImage> {
        vec![WaveImage {
            id: 1,
            area: Rect::new(2, 3, 8, 3),
            peaks: vec![peak; 16],
            colors: vec![Color::Rgb(10, 20, 30); 8],
        }]
    }

    #[test]
    fn image_frames_hide_before_text_and_restore_current_wave_data() {
        let mut state = State::default();
        let area = Rect::new(0, 0, 80, 26);
        let cell = (4, 8);
        assert!(state.before(true, Screen::Chop, area, cell, &[]).is_empty());
        let first = state.after(&mut wave(1.0), &[]);
        assert!(first.iter().any(|command| command.contains("a=t,f=32")));
        assert!(state.before(true, Screen::Chop, area, cell, &[]).is_empty());
        assert!(state.after(&mut wave(1.0), &[]).is_empty());
        let blockers = [Rect::new(8, 2, 4, 3)];
        let mut transcript = state.before(true, Screen::Chop, area, cell, &blockers);
        assert_eq!(transcript.len(), 1);
        assert!(transcript[0].contains("a=d,d=i,i=2"));
        transcript.push("TEXT FRAME".into());
        transcript.extend(state.after(&mut wave(0.5), &blockers));
        assert_eq!(
            transcript.len(),
            2,
            "no image can cover the text after it draws"
        );
        assert!(
            state
                .before(true, Screen::Chop, area, cell, &blockers)
                .is_empty()
        );
        assert!(state.after(&mut wave(0.7), &blockers).is_empty());
        assert!(state.before(true, Screen::Chop, area, cell, &[]).is_empty());
        let restored = state.after(&mut wave(0.7), &[]);
        let mut fresh = State::default();
        fresh.before(true, Screen::Chop, area, cell, &[]);
        assert_eq!(restored, fresh.after(&mut wave(0.7), &[]));
        assert_ne!(
            restored, first,
            "restore current peaks, not the pre-overlay bitmap"
        );
    }

    #[test]
    fn resize_screen_pixel_changes_and_disabled_graphics_remove_old_images() {
        let mut state = State::default();
        let mut area = Rect::new(0, 0, 80, 26);
        let mut screen = Screen::Chop;
        let mut cell = (4, 8);
        state.before(true, screen, area, cell, &[]);
        state.after(&mut wave(1.0), &[]);
        for change in 0..3 {
            match change {
                0 => area = Rect::new(0, 0, 120, 40),
                1 => screen = Screen::Pads,
                _ => cell = (8, 16),
            }
            let before = state.before(true, screen, area, cell, &[]);
            assert_eq!(before.len(), 1);
            assert!(before[0].contains("a=d,d=i"));
            assert!(
                state
                    .after(&mut wave(0.5), &[])
                    .iter()
                    .any(|command| command.contains("a=t,f=32"))
            );
        }
        assert_eq!(state.before(false, screen, area, cell, &[]).len(), 1);
        assert!(state.after(&mut wave(1.0), &[]).is_empty());
        assert!(state.before(false, screen, area, cell, &[]).is_empty());
        assert!(state.after(&mut wave(1.0), &[]).is_empty());
        assert!(state.drop_all().is_empty());
        let mut output = Vec::new();
        send(&mut output, &[]).unwrap();
        assert!(output.is_empty());
    }

    #[derive(Clone, Default)]
    struct RecordedOutput(std::rc::Rc<std::cell::RefCell<Vec<u8>>>);

    impl Write for RecordedOutput {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0.borrow_mut().write(bytes)
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    struct BrokenOutput {
        flush: bool,
    }
    impl Write for BrokenOutput {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if self.flush {
                Ok(bytes.len())
            } else {
                Err(io::Error::other("image write refused"))
            }
        }
        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::other("image flush refused"))
        }
    }

    #[test]
    fn pre_frame_image_failures_stop_text_and_retain_cleanup() {
        for flush in [false, true] {
            let mut state = State::default();
            let area = Rect::new(0, 0, 80, 26);
            state.before(true, Screen::Chop, area, (4, 8), &[]);
            state.after(&mut wave(1.0), &[]);
            // The fixed viewport prevents terminal queries. The production backend writes only this vector.
            let output = RecordedOutput::default();
            let mut term = Terminal::with_options(
                CrosstermBackend::new(output.clone()),
                TerminalOptions {
                    viewport: Viewport::Fixed(area),
                },
            )
            .unwrap();
            term.draw(|frame| frame.render_widget(Paragraph::new("previous text"), frame.area()))
                .unwrap();
            let original = output.0.borrow().clone();
            let mut text_called = false;
            let error = term
                .try_draw(|frame| -> io::Result<()> {
                    let commands = state.before(true, Screen::Chop, frame.area(), (4, 8), &[area]);
                    send(&mut BrokenOutput { flush }, &commands)?;
                    text_called = true;
                    frame.render_widget(Paragraph::new("must not draw"), frame.area());
                    Ok(())
                })
                .err()
                .unwrap();
            assert_eq!(
                error.to_string(),
                if flush {
                    "image flush refused"
                } else {
                    "image write refused"
                }
            );
            assert!(!text_called);
            assert_eq!(&*output.0.borrow(), &original);
            assert!(
                state
                    .drop_all()
                    .iter()
                    .any(|command| command.contains("a=d,d=i,i=2"))
            );
        }
    }
}
