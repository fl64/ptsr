//! Actions that remove or replace the owner's work ask first and name the undo (138).

use crate::*;

/// What waits for yes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Act {
    /// A cut action that replaces hand cuts: it runs again once they may go.
    Recut(Action),
    /// The threshold knob step that would recut by hits.
    Threshold(i32),
    /// An absolute threshold knob position that would recut by hits.
    ThresholdAt(f32),
    ClearCuts,
    TrimSilence,
    Forget,
}

impl App {
    /// The undo control in the words of the last input device (138).
    pub(super) fn undo_words(&self) -> &'static str {
        if self.input_sources.last_is_smc() {
            "P2 undo"
        } else {
            "Ctrl+Z"
        }
    }

    /// Asks before `act`; `it` names what undo brings back ("it" or "them").
    pub(super) fn ask_destroy(&mut self, act: Act, first: String, it: &str) {
        let undo = if act == Act::Forget {
            "This cannot be undone.".to_string()
        } else {
            format!("{} brings {it} back.", self.undo_words())
        };
        self.ask = Some(Ask::Destroy {
            act,
            lines: vec![format!("{first} {undo}"), "Enter: yes    Esc: keep".into()],
        });
    }

    /// `trim silence` (138): the region goes to the sound, one undo step, a notice.
    pub(super) fn trim_silence(&mut self) {
        let Some(frames) = self.chop.as_mut().and_then(chop::Chop::trim_silence) else {
            return;
        };
        self.sync_links();
        let ms = frames as u64 * 1000 / u64::from(self.rate.max(1));
        let undo = self.undo_words();
        self.post_notice(
            Level::Done,
            (
                format!("silence trimmed: {ms} ms · {undo} brings it back"),
                Instant::now(),
            ),
        );
    }

    /// Hand cuts would go: the cut actions ask before they replace them.
    pub(super) fn hand_cuts(&self) -> Option<usize> {
        self.chop
            .as_ref()
            .filter(|c| c.hand && !c.markers.is_empty())
            .map(|c| c.markers.len())
    }

    pub(super) fn destroy_confirmed(&mut self, act: Act) {
        let undo = self.undo_words();
        match act {
            Act::Recut(action) => {
                let count = self.hand_cuts().unwrap_or(0);
                // the linked pads notice names the undo itself
                let linked = self.linked_pads();
                if let Some(c) = self.chop.as_mut() {
                    c.hand = false;
                }
                self.run(action);
                if matches!(action, Action::ChopParts | Action::ChopEqualParts) && linked == 0 {
                    self.post_notice(
                        Level::Done,
                        (
                            format!("{count} cuts replaced · {undo} brings them back"),
                            Instant::now(),
                        ),
                    );
                }
            }
            Act::Threshold(step) => {
                if let Some(c) = self.chop.as_mut() {
                    c.hand = false;
                }
                self.step_threshold(step);
            }
            Act::ThresholdAt(v) => {
                if let Some(c) = self.chop.as_mut() {
                    c.hand = false;
                    c.set_threshold(v);
                }
                self.sync_links();
            }
            Act::ClearCuts => {
                if let Some(c) = self.chop.as_mut() {
                    c.hand = false;
                }
                self.run(Action::ChopCutsClear);
            }
            Act::TrimSilence => self.trim_silence(),
            Act::Forget => {
                let control = self.forgettable().unwrap_or_default();
                self.forget_midi_row();
                self.post_notice(
                    Level::Done,
                    (format!("{control} forgotten · no undo"), Instant::now()),
                );
            }
        }
    }
}

impl App {
    /// The learned control of the selected MIDI row, named for the forget question (138).
    pub(super) fn forgettable(&self) -> Option<String> {
        match self.midi_list().get(self.midi_row)? {
            MidiListRow::Set { .. } => Some("the controls of this run".into()),
            MidiListRow::Action { info } => {
                let who: Vec<String> = self
                    .live_profiles()
                    .iter()
                    .filter_map(|p| p.assignment(info.group.scope(), info.name))
                    .map(|a| a.split(" (").next().unwrap_or_default().to_string())
                    .collect();
                (!who.is_empty()).then(|| who.join(", "))
            }
            MidiListRow::Disabled { key, .. } => Some(key.clone()),
            _ => None,
        }
    }

    /// Settings MIDI: the learned control of the selected row goes, as `unbind` did.
    pub(super) fn forget_midi_row(&mut self) {
        match self.midi_list().get(self.midi_row) {
            // the run rows clear all together (28)
            Some(MidiListRow::Set { infos: run, .. }) => {
                let infos: &'static [input::ActionInfo] = run;
                let mut profiles: Vec<String> = Vec::new();
                for info in infos {
                    for p in learn::clear(&mut self.learned, info.group, info.name) {
                        if !profiles.contains(&p) {
                            profiles.push(p);
                        }
                    }
                }
                for p in profiles {
                    self.sync_learned(&p, infos[0].group);
                }
                self.save_midi();
            }
            Some(MidiListRow::Action { info }) => {
                for profile in learn::clear(&mut self.learned, info.group, info.name) {
                    self.sync_learned(&profile, info.group);
                }
                self.save_midi();
            }
            Some(MidiListRow::Disabled {
                profile,
                group,
                key,
            }) => {
                let (profile, group, key) = (profile.clone(), group.clone(), key.clone());
                if let Some(bindings) = self
                    .learned
                    .get_mut(&profile)
                    .and_then(|m| m.get_mut(&group))
                {
                    bindings.remove(&key);
                }
                if let Some(group) = input::Group::ALL
                    .into_iter()
                    .find(|candidate| candidate.key() == group)
                {
                    self.sync_learned(&profile, group);
                }
                self.save_midi();
            }
            _ => {}
        }
    }
}

/// What an A1 undo brings back (138), in the words of the step it undoes.
pub(super) fn a1_undo_text(what: &str) -> String {
    match what {
        "pattern pass" => "undo: A1 before the pass".into(),
        "clear" => "undo: A1 hits back".into(),
        _ => match what.strip_prefix("erase ") {
            Some(pad) => format!("undo: {pad} hits back on A1"),
            None => format!("undo: {what} on A1"),
        },
    }
}

/// What a Chop undo brought back (138): `before` and `after` are the cuts and the trim
/// around the undo.
pub(super) fn chop_undo_text(before: &chop::Saved, after: &chop::Saved) -> String {
    let (was, now) = (before.cuts.len() + 1, after.cuts.len() + 1);
    let old: Vec<usize> = before.cuts.iter().map(|(_, at)| *at).collect();
    let back: Vec<usize> = after
        .cuts
        .iter()
        .map(|(_, at)| *at)
        .filter(|at| !old.contains(at))
        .collect();
    if now == was + 1 && back.len() == 1 {
        let k = after
            .cuts
            .iter()
            .position(|(_, at)| *at == back[0])
            .unwrap_or(0)
            + 1;
        format!("undo: slices {k} and {} split again", k + 1)
    } else if now != was {
        format!("undo: {now} slices back (were {was})")
    } else if before.range != after.range {
        "undo: the trim is back".into()
    } else {
        "undo: the cuts are back".into()
    }
}

/// Runs `action` and answers yes to its destroy question, if one comes (tests).
#[cfg(test)]
pub(super) fn yes(app: &mut App, action: Action) {
    app.apply(action);
    if matches!(app.ask, Some(Ask::Destroy { .. })) {
        app.apply(Action::Confirm);
    }
}
