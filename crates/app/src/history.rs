//! The common edit history and the state restored by Redo.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::{App, Level, Screen, chop, pads, pattern_record};

pub(super) const UNDO_STEPS: usize = 100;
pub(super) const UNDO_JOIN: Duration = Duration::from_secs(1);

pub(super) struct Step {
    pub(super) what: String,
    pub(super) pads: Vec<(usize, pads::PadSnap)>,
    pub(super) fx: Option<[pads::FxSlot; engine::FX_SLOTS]>,
    pub(super) swap: Option<(usize, usize)>,
    pub(super) pattern: Option<pattern_record::Undo>,
    pub(super) at: Instant,
}

pub(super) struct Before {
    pub(super) pads: Vec<pads::PadSnap>,
    pub(super) fx: [pads::FxSlot; engine::FX_SLOTS],
}

pub(super) struct ChopContext {
    file: PathBuf,
    epoch: u32,
    depth: usize,
}

impl ChopContext {
    pub(super) fn current(chop: &chop::Chop) -> Self {
        Self {
            file: chop.file.clone(),
            epoch: chop.epoch,
            depth: chop.depth(),
        }
    }

    fn matches(&self, app: &App) -> bool {
        app.screen == Screen::Chop && self.matches_chop(app)
    }

    fn matches_chop(&self, app: &App) -> bool {
        app.chop.as_ref().is_some_and(|chop| {
            chop.file == self.file && chop.epoch == self.epoch && chop.depth() == self.depth
        })
    }
}

pub(super) enum Redo {
    Common {
        step: Step,
        norm: Option<ChopContext>,
    },
    Cuts {
        context: ChopContext,
        edit: chop::RedoCut,
    },
    Fresh {
        context: ChopContext,
        epoch: u32,
        saved: Option<Box<chop::Chop>>,
    },
    Plan {
        context: ChopContext,
        plan: chop::Plan,
        selected: usize,
    },
}

impl App {
    pub(super) fn remember_redo(&mut self, entry: Redo) {
        if self.redo.len() == UNDO_STEPS {
            self.redo.remove(0);
        }
        self.redo.push(entry);
    }

    pub(super) fn update_pattern_history(
        &mut self,
        undo: Option<pattern_record::Undo>,
        redo: Option<pattern_record::Undo>,
    ) {
        if let Some(before) = undo {
            self.push_pattern_step(before);
        }
        if let Some(pattern) = redo {
            self.redo.clear();
            self.remember_redo(Redo::Common {
                step: Step {
                    what: "pattern pass".into(),
                    pads: Vec::new(),
                    fx: None,
                    swap: None,
                    pattern: Some(pattern),
                    at: Instant::now(),
                },
                norm: None,
            });
        }
    }

    pub(super) fn chop_edit_mark(&self) -> Option<Instant> {
        self.redo.last()?;
        self.chop.as_ref().and_then(|chop| chop.last_edit())
    }

    pub(super) fn discard_redo_after_chop_edit(
        &mut self,
        before: Option<Instant>,
        started: Instant,
    ) {
        if self
            .chop_edit_mark()
            .is_some_and(|mark| Some(mark) != before && mark >= started)
        {
            self.redo.clear();
        }
    }

    pub(super) fn exchange_step(&mut self, step: &Step) -> Result<Step, String> {
        let mut inverse = Step {
            what: step.what.clone(),
            pads: Vec::new(),
            fx: step.fx.map(|_| self.pads.fx),
            swap: step.swap,
            pattern: None,
            at: step.at,
        };
        if let Some(pattern) = &step.pattern {
            inverse.pattern = Some(pattern_record::Undo::new(&self.pads)?);
            let command = pattern.restore(&mut self.pads)?;
            self.reset_pattern_takeover();
            if self
                .audio_alive()
                .is_some_and(|audio| audio.pattern_capture.available())
            {
                self.send(command);
            }
        } else if let Some((a, b)) = step.swap {
            inverse.pads = vec![(a, self.pads.snapshot(a)), (b, self.pads.snapshot(b))];
            let command = self.pads.swap(a, b)?;
            self.load_gen[a] += 1;
            self.load_gen[b] += 1;
            self.send(command);
        } else {
            for (pad, saved) in &step.pads {
                self.load_gen[*pad] += 1;
                inverse.pads.push((*pad, self.pads.snapshot(*pad)));
                for command in self.pads.put_back(*pad, saved) {
                    self.send(command);
                }
            }
        }
        if let Some(fx) = step.fx {
            self.pads.fx = fx;
            for slot in 0..engine::FX_SLOTS {
                self.send(self.pads.fx_command(slot));
                self.send(self.pads.fx_return_command(slot));
            }
        }
        self.dirty();
        Ok(inverse)
    }

    pub(super) fn redo_refusal(&self) -> Option<&'static str> {
        let Some(entry) = self.redo.last() else {
            return Some("nothing to redo");
        };
        if self.pattern_owns_pads() {
            return Some("finish A1 before redo");
        }
        match entry {
            Redo::Common { step, norm } => {
                let common = matches!(self.screen, Screen::Pads | Screen::Files | Screen::Fx);
                let in_chop = norm.as_ref().is_some_and(|context| context.matches(self));
                (!(step.pattern.is_some() || common || in_chop) || self.plan.is_some())
                    .then_some("return to the original edit to redo")
            }
            Redo::Cuts { context, .. } | Redo::Fresh { context, .. } => (!context.matches(self)
                || self.plan.is_some())
            .then_some("return to the chop to redo"),
            Redo::Plan { context, .. } => (!context.matches(self) || self.plan.is_none())
                .then_some("return to Assign to redo"),
        }
    }

    pub(super) fn redo_edit(&mut self) {
        self.flush_audio();
        if let Some(reason) = self.redo_refusal() {
            self.post_notice(Level::Info, (reason.into(), Instant::now()));
            return;
        }
        let Some(mut entry) = self.redo.pop() else {
            return;
        };
        match self.restore_edit(&mut entry) {
            Ok(text) => self.post_notice(Level::Done, (format!("redo: {text}"), Instant::now())),
            Err(error) => {
                self.redo.push(entry);
                self.post_notice(Level::Error, (error, Instant::now()));
            }
        };
    }

    fn restore_edit(&mut self, entry: &mut Redo) -> Result<String, String> {
        match entry {
            Redo::Common { step, norm } => {
                let mut inverse = self.exchange_step(step)?;
                // The next edit can join other new edits, but never the replayed step.
                let now = Instant::now();
                inverse.at = now.checked_sub(UNDO_JOIN).unwrap_or(now);
                if self.history.len() == UNDO_STEPS {
                    self.history.remove(0);
                }
                self.history.push(inverse);
                if norm
                    .as_ref()
                    .is_some_and(|context| context.matches_chop(self))
                    && let Some(chop) = self.chop.as_mut()
                {
                    chop.norm_step = Some((chop.depth(), self.history.len()));
                }
                if step.pattern.is_some() {
                    return Ok(if step.what == "pattern pass" {
                        "A1 pass".into()
                    } else {
                        step.what.clone()
                    });
                }
                let on = match (step.swap, step.pads.as_slice()) {
                    (Some(_), _) => "two pads".into(),
                    (_, []) => "the shared slot".into(),
                    (_, [(pad, _)]) => format!("pad {}", tui::pads::pad_tag(*pad)),
                    (_, pads) => format!("{} pads", pads.len()),
                };
                Ok(format!("{} on {on}", step.what))
            }
            Redo::Cuts { edit, .. } => {
                let chop = self.chop.as_mut().ok_or("The chop is closed.")?;
                chop.restore_cut_edit(edit);
                Ok(format!("{} slices", chop.markers.len() + 1))
            }
            Redo::Fresh { epoch, saved, .. } => self.redo_fresh(*epoch, saved),
            Redo::Plan { plan, selected, .. } => {
                let chop = self.chop.as_mut().ok_or("The chop is closed.")?;
                let old =
                    std::mem::replace(self.plan.as_mut().ok_or("Assign is closed.")?, plan.clone());
                self.plan_undo.push((old, chop.selected));
                chop.select_slice(*selected);
                Ok(format!(
                    "{} · {} placed",
                    self.assign_method(),
                    self.placed_slices()
                ))
            }
        }
    }
}
