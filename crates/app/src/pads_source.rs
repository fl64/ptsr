use std::sync::atomic::Ordering::Acquire;
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};

use engine::Sample;
use engine::rt::{PadVoice, PadVoices, Stats};
use input::{Action, Keymap, Screen};
use tui::{SourceRange, SourceSample, SourceStrip, SourceVoice};

use super::{App, PAD_KEYS, PADS_PER_BANK, loader, pads};

/// The terminal scales these peaks to the width of the view.
pub(super) const BIG_COLS: usize = 1200;
const FOCUS_HOLD: Duration = Duration::from_millis(150);

#[derive(Default)]
pub(super) struct State {
    rhythm: bool,
    pin: Option<Weak<Sample>>,
    focus: Option<(Weak<Sample>, Instant)>,
    audio: Option<(Weak<Stats>, u64)>,
    voices: Option<PadVoices>,
    peaks: Vec<(Weak<Sample>, Arc<[f32]>)>,
}

impl State {
    pub(super) fn rhythm(&self) -> bool {
        self.rhythm
    }

    pub(super) fn view_name(&self) -> &'static str {
        if self.rhythm { "Rhythm" } else { "Sound" }
    }

    pub(super) fn pinned(&self, pads: &pads::Pads) -> bool {
        self.pin
            .as_ref()
            .and_then(|pin| loaded_sample(pads, pin))
            .is_some()
    }

    fn update_audio(&mut self, audio: Option<&super::Audio>) {
        let Some((audio, epoch)) = audio.and_then(|audio| {
            let epoch = audio.project.epoch.load(Acquire);
            (audio.project.ready.load(Acquire) == epoch).then_some((audio, epoch))
        }) else {
            self.audio = None;
            self.voices = None;
            return;
        };
        if !self.audio.as_ref().is_some_and(|(stats, last_epoch)| {
            *last_epoch == epoch && stats.as_ptr() == Arc::as_ptr(&audio.stats)
        }) {
            self.audio = Some((Arc::downgrade(&audio.stats), epoch));
            self.voices = None;
        }
        let voices = audio.stats.pad_voices(epoch);
        if audio.project.epoch.load(Acquire) != epoch || audio.project.ready.load(Acquire) != epoch
        {
            self.voices = None;
        } else if let Some(voices) = voices {
            self.voices = Some(voices);
        }
    }

    fn peaks(&mut self, sound: &Arc<Sample>) -> Arc<[f32]> {
        if let Some((_, peaks)) = self
            .peaks
            .iter()
            .find(|(cached, _)| cached.as_ptr() == Arc::as_ptr(sound))
        {
            return Arc::clone(peaks);
        }
        let peaks: Arc<[f32]> = loader::envelope(sound, BIG_COLS).into();
        self.peaks.push((Arc::downgrade(sound), Arc::clone(&peaks)));
        peaks
    }

    fn project(&mut self, pads: &pads::Pads, now: Instant) -> Option<SourceStrip> {
        let mut voices: Vec<_> = self
            .voices
            .iter()
            .flat_map(PadVoices::iter)
            .filter(|voice| loaded(pads, voice.pad).is_some_and(|sound| voice.plays(sound)))
            .collect();
        voices.sort_unstable_by_key(|voice| std::cmp::Reverse(voice.serial));
        let mut groups: Vec<(usize, Vec<PadVoice>)> = Vec::new();
        for voice in voices {
            let sound = pads.sound[voice.pad].as_ref().unwrap();
            if let Some((_, group)) = groups.iter_mut().find(|(pad, _)| {
                pads.sound[*pad]
                    .as_ref()
                    .is_some_and(|other| Arc::ptr_eq(other, sound))
            }) {
                group.push(voice);
            } else {
                groups.push((voice.pad, vec![voice]));
            }
        }

        self.peaks.retain(|(sound, _)| sound.strong_count() > 0);
        let find = |source: &Weak<Sample>| loaded_sample(pads, source);
        let pinned = self.pin.as_ref().and_then(find);
        if pinned.is_none() {
            self.pin = None;
        }
        let previous = self
            .focus
            .as_ref()
            .and_then(|(focus, at)| find(focus).map(|sound| (sound, *at)));
        let newest = groups.first().and_then(|(pad, _)| loaded(pads, *pad));
        let focus = pinned.or_else(|| {
            if let Some((sound, at)) = previous
                && (newest.is_none() || now.saturating_duration_since(at) < FOCUS_HOLD)
            {
                return Some(sound);
            }
            newest
                .or_else(|| loaded(pads, pads.selected_abs()))
                .or_else(|| (0..pads.sound.len()).find_map(|pad| loaded(pads, pad)))
        })?;
        if previous.is_none_or(|(last, _)| !Arc::ptr_eq(last, focus)) {
            self.focus = Some((Arc::downgrade(focus), now));
        }

        let mut others = Vec::new();
        let mut focus_voices = Vec::new();
        for (pad, voices) in groups {
            let sound = pads.sound[pad].as_ref().unwrap();
            if Arc::ptr_eq(sound, focus) {
                focus_voices = voices;
            } else {
                others.push(self.sample(pads, sound, voices, false, others.len() < 2));
            }
        }
        Some(SourceStrip {
            focus: self.sample(pads, focus, focus_voices, true, true),
            others,
            pinned: pinned.is_some(),
        })
    }

    fn sample(
        &mut self,
        pads: &pads::Pads,
        sound: &Arc<Sample>,
        voices: Vec<PadVoice>,
        focus: bool,
        peaks: bool,
    ) -> SourceSample {
        let position = |frame: usize| frame.min(sound.len()) as f32 / sound.len() as f32;
        let matching = || {
            (0..pads.sound.len())
                .filter(|&pad| loaded(pads, pad).is_some_and(|other| Arc::ptr_eq(sound, other)))
        };
        let name = matching()
            .find_map(|pad| pads.file[pad].as_deref())
            .map(file_name)
            .unwrap_or_default();
        let mut ranges = if focus {
            matching()
                .map(|pad| {
                    let region = pads.region[pad];
                    SourceRange {
                        start: position(region.start),
                        end: position(region.end),
                        label: pad_label(pads.bank, pad),
                        sounding: voices.iter().any(|voice| {
                            voice.pad == pad
                                && voice.start == region.start
                                && voice.end == region.end.min(sound.len())
                        }),
                    }
                })
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        ranges.sort_by(|a, b| a.start.total_cmp(&b.start));
        SourceSample {
            name,
            peaks: if peaks {
                self.peaks(sound)
            } else {
                Arc::default()
            },
            ranges,
            voices: voices
                .into_iter()
                .map(|voice| SourceVoice {
                    start: position(voice.start),
                    end: position(voice.end),
                    position: position(voice.frame),
                    level: voice.level,
                    label: pad_label(pads.bank, voice.pad),
                })
                .collect(),
        }
    }
}

fn loaded(pads: &pads::Pads, pad: usize) -> Option<&Arc<Sample>> {
    let sound = pads.sound.get(pad)?.as_ref()?;
    (pads.file[pad].is_some()
        && !pads.missing[pad]
        && pads.loading[pad].is_none()
        && !sound.is_empty())
    .then_some(sound)
}

fn loaded_sample<'a>(pads: &'a pads::Pads, sample: &Weak<Sample>) -> Option<&'a Arc<Sample>> {
    (0..pads.sound.len())
        .find_map(|pad| loaded(pads, pad).filter(|sound| sample.as_ptr() == Arc::as_ptr(sound)))
}

fn pad_label(bank: usize, pad: usize) -> String {
    let label = tui::pads::key_tag(PAD_KEYS[pad % PADS_PER_BANK]);
    let source_bank = pad / PADS_PER_BANK;
    if source_bank == bank {
        label
    } else {
        format!("B{}:{label}", source_bank + 1)
    }
}

impl App {
    pub(super) fn source_pin_available(&self) -> bool {
        self.source_view.pinned(&self.pads)
            || loaded(&self.pads, self.pads.selected_abs()).is_some()
    }

    pub(super) fn apply_source_action(&mut self, action: Action) {
        if self.screen != Screen::Pads
            || self.pattern_view.overview
            || self.help_open
            || self.transport_input_blocked()
            || self.recording.is_some()
            || self.pattern_panel.is_some()
        {
            return;
        }
        match action {
            Action::PadsStripToggle => self.source_view.rhythm = !self.source_view.rhythm,
            Action::PadsSourcePin => {
                if self.source_view.pinned(&self.pads) {
                    self.source_view.pin = None;
                    self.source_view.focus = None;
                } else {
                    self.source_view.pin =
                        loaded(&self.pads, self.pads.selected_abs()).map(Arc::downgrade);
                }
            }
            _ => {}
        }
    }

    pub(super) fn live_source_strip(&mut self, now: Instant) -> Option<SourceStrip> {
        if self.screen != Screen::Pads || self.pattern_view.overview {
            return None;
        }
        let audio = self
            .audio
            .as_ref()
            .filter(|audio| !audio.stopped.load(Acquire));
        self.source_view.update_audio(audio);
        if !self.source_view.rhythm
            && !self.edit
            && self.recording.is_none()
            && self.pattern_panel.is_none()
            && !self.erase_pads
            && !self.splash
        {
            return self.source_view.project(&self.pads, now);
        }
        None
    }

    pub(super) fn pads_strip(
        &self,
        view: &mut tui::Pads,
        keymap: &Keymap,
        width: u16,
        source: Option<SourceStrip>,
    ) {
        if self.screen != Screen::Pads {
            return;
        }
        if self.pattern_view.overview {
            view.overview = Some(self.pattern_overview_at(keymap, width));
            return;
        }
        view.source = source;
        if view.source.is_none() {
            let mut rhythm = self.pattern_strip(keymap);
            if self.source_view.rhythm {
                rhythm.header = format!("Rhythm · {}", rhythm.header);
            }
            view.pattern = Some(rhythm);
        }
    }
}

pub(super) fn file_name(path: &std::path::Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[cfg(test)]
pub(super) fn fixture(app: &mut App) -> engine::rt::Realtime {
    use engine::{Command, Region};

    let (audio, mut rt) = super::audio::offline(48_000);
    app.audio = Some(audio);
    app.splash = false;
    let mut sounds = (0..4).map(|index| {
        let sound = (0..24_000)
            .map(|frame| {
                let t = frame as f32 / 48_000.0;
                let pitch = 72.0 + 62.0 * index as f32;
                let phase = t * (9 + index) as f32 * std::f32::consts::TAU;
                let beat = 0.2 + 0.8 * (0.5 + 0.5 * phase.cos()).powi(4);
                (t * pitch * std::f32::consts::TAU).sin() * beat * 0.16
            })
            .collect();
        Arc::new(Sample::mono(sound))
    });
    let break_sound = sounds.next().unwrap();
    let sources = [
        (
            "break",
            break_sound.clone(),
            Region {
                start: 0,
                end: 6_000,
            },
        ),
        (
            "break",
            break_sound.clone(),
            Region {
                start: 12_000,
                end: 18_000,
            },
        ),
        ("bass", sounds.next().unwrap(), Region::WHOLE),
        ("hat", sounds.next().unwrap(), Region::WHOLE),
        ("noise", sounds.next().unwrap(), Region::WHOLE),
        (
            "break",
            break_sound.clone(),
            Region {
                start: 6_000,
                end: 12_000,
            },
        ),
        (
            "break",
            break_sound,
            Region {
                start: 18_000,
                end: 24_000,
            },
        ),
    ];
    for (pad, (name, sound, region)) in sources.into_iter().enumerate() {
        app.pads.file[pad] = Some(format!("{name}.wav").into());
        app.pads.custom_name[pad] = None;
        app.pads.sound[pad] = Some(sound.clone());
        app.pads.region[pad] = region;
        app.send(Command::SetPadSound {
            pad,
            sound: Some(sound),
        });
        app.send(Command::SetPadRegion { pad, region });
    }
    rt.process(&mut [0.0; 128], &mut [0.0; 128]);
    rt
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::Command;

    fn hit(app: &mut App, pad: usize) {
        app.send(Command::PadPress { pad, velocity: 1.0 });
    }

    #[test]
    fn sound_strip_groups_multiple_slices_of_one_sample() {
        let mut app = super::super::tests::test_app();
        let mut rt = fixture(&mut app);
        for pad in [2, 0, 1] {
            hit(&mut app, pad);
        }
        rt.process(&mut [0.0; 128], &mut [0.0; 128]);
        let strip = app.live_source_strip(Instant::now()).unwrap();
        assert_eq!(strip.focus.name, "break.wav");
        assert_eq!(strip.others.len(), 1);
        assert_eq!(strip.others[0].name, "bass.wav");
        assert_eq!(strip.focus.ranges.len(), 4);
        assert_eq!(
            strip
                .focus
                .ranges
                .iter()
                .map(|range| range.sounding)
                .collect::<Vec<_>>(),
            [true, false, true, false]
        );
        let positions: Vec<_> = strip
            .focus
            .voices
            .iter()
            .map(|voice| voice.position)
            .collect();
        assert_eq!(positions, [12_128.0 / 24_000.0, 128.0 / 24_000.0]);
    }

    #[test]
    fn sound_strip_rejects_replaced_samples_and_old_projects() {
        let mut app = super::super::tests::test_app();
        let mut rt = fixture(&mut app);
        hit(&mut app, 0);
        hit(&mut app, 2);
        rt.process(&mut [0.0; 128], &mut [0.0; 128]);
        let now = Instant::now();
        assert_eq!(app.live_source_strip(now).unwrap().focus.name, "bass.wav");
        app.pads.sound[2] = Some(app.pads.sound[3].as_ref().unwrap().clone());
        let replaced = app.live_source_strip(now).unwrap();
        assert_eq!(replaced.focus.name, "break.wav");
        assert!(replaced.others.is_empty());
        app.audio
            .as_ref()
            .unwrap()
            .project
            .epoch
            .fetch_add(1, std::sync::atomic::Ordering::Release);
        let switched = app.live_source_strip(now).unwrap();
        assert!(switched.focus.voices.is_empty());
        assert!(switched.others.is_empty());
    }

    #[test]
    fn sound_strip_keeps_the_last_wave_in_silence() {
        let mut app = super::super::tests::test_app();
        let mut rt = fixture(&mut app);
        hit(&mut app, 3);
        rt.process(&mut [0.0; 128], &mut [0.0; 128]);
        let now = Instant::now();
        let active = app.live_source_strip(now).unwrap();
        assert_eq!(active.focus.name, "hat.wav");
        rt.process(&mut vec![0.0; 24_000], &mut vec![0.0; 24_000]);
        let silent = app.live_source_strip(now + Duration::from_secs(1)).unwrap();
        assert_eq!(silent.focus.name, "hat.wav");
        assert!(Arc::ptr_eq(&active.focus.peaks, &silent.focus.peaks));
        assert!(silent.focus.voices.is_empty());
    }

    #[test]
    fn sound_strip_holds_focus_and_pins_the_selected_sample() {
        let mut app = super::super::tests::test_app();
        let mut rt = fixture(&mut app);
        let at = Instant::now();
        hit(&mut app, 0);
        rt.process(&mut [0.0; 128], &mut [0.0; 128]);
        assert_eq!(app.live_source_strip(at).unwrap().focus.name, "break.wav");
        hit(&mut app, 2);
        rt.process(&mut [0.0; 128], &mut [0.0; 128]);
        let later = at + FOCUS_HOLD;
        assert_eq!(
            app.live_source_strip(later - Duration::from_millis(1))
                .unwrap()
                .focus
                .name,
            "break.wav"
        );
        assert_eq!(app.live_source_strip(later).unwrap().focus.name, "bass.wav");
        app.pads.selected = 3;
        app.apply(Action::PadsSourcePin);
        let pinned = app.live_source_strip(later + FOCUS_HOLD).unwrap();
        assert!(pinned.pinned);
        assert_eq!(pinned.focus.name, "hat.wav");
        app.apply(Action::PadsSourcePin);
        let automatic = app.live_source_strip(later + FOCUS_HOLD).unwrap();
        assert!(!automatic.pinned);
        assert_eq!(automatic.focus.name, "bass.wav");
        app.apply(Action::PadsSourcePin);
        app.apply(Action::PadEdit);
        app.pads.sound[3] = None;
        assert!(!app.source_view.pinned(&app.pads));
        app.pads.selected = 0;
        app.apply(Action::PadsSourcePin);
        assert!(app.source_view.pinned(&app.pads));
        app.apply(Action::Back);
        assert_eq!(
            app.live_source_strip(later + FOCUS_HOLD)
                .unwrap()
                .focus
                .name,
            "break.wav"
        );
    }

    #[test]
    fn sound_strip_follows_a_restarted_voice_and_its_fade() {
        let mut app = super::super::tests::test_app();
        let mut rt = fixture(&mut app);
        hit(&mut app, 1);
        rt.process(&mut [0.0; 512], &mut [0.0; 512]);
        let earlier = app
            .live_source_strip(Instant::now())
            .unwrap()
            .focus
            .voices
            .remove(0);
        hit(&mut app, 1);
        rt.process(&mut [0.0; 16], &mut [0.0; 16]);
        let strip = app.live_source_strip(Instant::now()).unwrap();
        let next = &strip.focus.voices[0];
        let fade = &strip.focus.voices[1];
        assert_eq!(next.position, 12_016.0 / 24_000.0);
        assert_eq!(fade.position, 12_528.0 / 24_000.0);
        assert!(fade.level < earlier.level);
    }

    #[test]
    fn sound_strip_gives_priority_to_edit_and_recording() {
        let mut app = super::super::tests::test_app();
        let mut rt = fixture(&mut app);
        hit(&mut app, 0);
        rt.process(&mut [0.0; 128], &mut [0.0; 128]);
        let now = Instant::now();
        assert!(app.live_source_strip(now).is_some());
        app.apply(Action::PadEdit);
        assert!(app.live_source_strip(now).is_none());
        app.apply(Action::Back);
        assert!(app.live_source_strip(now).is_some());
        app.recording = Some((super::super::audio::SOURCE_MIC, now));
        assert!(app.live_source_strip(now).is_none());
        app.recording = None;
        assert!(app.live_source_strip(now).is_some());
    }

    #[test]
    fn sound_strip_render_keeps_pad_sound_clean() {
        use engine::checks::{first_loud, first_non_finite, max_step, peak};
        use engine::offline::write_wav;
        use std::path::Path;

        let mut app = super::super::tests::test_app();
        let mut rt = fixture(&mut app);
        let mut left = vec![0.0; 28_672];
        let mut right = left.clone();
        for block in (0..left.len()).step_by(128) {
            match block {
                128 => hit(&mut app, 0),
                2_048 => hit(&mut app, 1),
                4_096 => hit(&mut app, 2),
                _ => {}
            }
            rt.process(
                &mut left[block..block + 128],
                &mut right[block..block + 128],
            );
        }
        for channel in [&left, &right] {
            assert_eq!(first_non_finite(channel), None);
            assert!((0.05..0.90).contains(&peak(channel)));
            assert_eq!(first_loud(channel, 0..256, 1e-4), Some(129));
            assert!(max_step(channel) < 0.05);
            assert!(channel[28_096..].iter().all(|sample| *sample == 0.0));
        }
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/139-source-view.wav");
        write_wav(&path, 48_000, &left, &right).unwrap();
    }
}
