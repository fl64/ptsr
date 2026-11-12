//! Audio output: JACK (via PipeWire-JACK), fallback cpal (ALSA).

use std::cell::UnsafeCell;
use std::sync::Arc;
use std::sync::atomic::{
    AtomicBool, AtomicU8, AtomicUsize,
    Ordering::{Acquire, Relaxed, Release},
};

use engine::rt::{ProjectCommand, ProjectState, Realtime, Stats};
use engine::{Engine, Retired};
use jack::PortFlags;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    /// JACK, or cpal when JACK is unavailable.
    #[default]
    Auto,
    Jack,
    Cpal,
}

pub struct Audio {
    pub backend: &'static str,
    /// Commands from the UI thread.
    pub commands: rtrb::Producer<ProjectCommand>,
    pub project: Arc<ProjectState>,
    /// Commands from the MIDI thread; taken by it once.
    pub midi_commands: Option<rtrb::Producer<ProjectCommand>>,
    /// Sample and pattern references dropped outside the audio thread.
    garbage: rtrb::Consumer<Retired>,
    pub stats: Arc<Stats>,
    pub pattern_capture: crate::pattern_record::Capture,
    /// Audio stopped on its own: the JACK server went away or the device disappeared.
    pub stopped: Arc<AtomicBool>,
    /// JACK ports appeared or disappeared: connections need to be recomputed.
    pub ports_changed: Arc<AtomicBool>,
    /// Where the preview plays, in frames; `usize::MAX` when it is silent.
    pub preview_pos: Arc<AtomicUsize>,
    /// The recording ring; the app starts and stops it, the audio thread writes it.
    pub recorder: Arc<Recorder>,
    /// The audio stream lives as long as this object.
    stream: Stream,
}

enum Stream {
    Jack {
        client: jack::AsyncClient<JackNotify, JackProcess>,
        outs: [String; 2],
        ins: [String; 2],
        recs: [String; 2],
    },
    Cpal(#[allow(dead_code)] cpal::Stream),
    #[cfg(test)]
    Offline,
}

/// Devices the audio is actually connected to.
#[derive(Default, Clone, PartialEq, Debug)]
pub struct Routed {
    pub output: Option<String>,
    pub input: Option<String>,
    /// The device playback ports the sound actually goes to; the recording tap derives
    /// its monitor ports from them.
    pub playback: Vec<String>,
}

pub struct Devices {
    pub outputs: Vec<String>,
    pub inputs: Vec<String>,
}

impl Audio {
    /// Frees samples the audio thread no longer uses. Call regularly from the UI thread.
    pub fn free_garbage(&mut self) {
        while self.garbage.pop().is_ok() {}
    }

    /// Connects the output tap to the monitor twins of the `playback` ports the sound
    /// actually goes to — including what other programs play. `false` when a monitor
    /// port is missing for any of them.
    pub fn tap_output(&self, playback: &[String]) -> bool {
        let Stream::Jack { client, recs, .. } = &self.stream else {
            return false;
        };
        let c = client.as_client();
        // monitor ports are sources and not physical in PipeWire: look through all audio outputs
        let all_outputs = c.ports(None, Some("audio"), PortFlags::IS_OUTPUT);
        let Some(monitors) = monitors_of(playback, &all_outputs) else {
            log::warn!("no monitor ports for {playback:?}");
            return false;
        };
        for (rec, mon) in recs.iter().zip(&monitors) {
            // the monitor is the source, our tap is the destination
            let _ = c.disconnect_ports_by_name(mon, rec);
            if c.connect_ports_by_name(mon, rec).is_err() {
                log::warn!("cannot connect {rec} to {mon}");
                return false;
            }
        }
        true
    }

    /// Unlinks the output tap from whatever it listens to, so it shows only while recording.
    pub fn untap_output(&self) {
        let Stream::Jack { client, recs, .. } = &self.stream else {
            return;
        };
        let c = client.as_client();
        for rec in recs {
            if let Some(port) = c.port_by_name(rec) {
                let _ = c.disconnect(&port);
            }
        }
    }

    /// Available output and input devices; no choice in cpal mode.
    pub fn devices(&self) -> Option<Devices> {
        let Stream::Jack { client, .. } = &self.stream else {
            return None;
        };
        let c = client.as_client();
        Some(Devices {
            outputs: device_names(&physical(c, PortFlags::IS_INPUT)),
            inputs: device_names(&physical(c, PortFlags::IS_OUTPUT)),
        })
    }

    /// Connects audio to the chosen devices, or to the first available ones if they are missing.
    /// Touches only connections that differ from the wanted ones, so matching links never drop audio.
    /// The input device is picked and named always, but linked only with `link_input` (26): a
    /// linked USB microphone takes over the clock of the whole graph, Bluetooth output included.
    pub fn route(&self, output: Option<&str>, input: Option<&str>, link_input: bool) -> Routed {
        let Stream::Jack {
            client, outs, ins, ..
        } = &self.stream
        else {
            return Routed::default();
        };
        let c = client.as_client();
        let playback = physical(c, PortFlags::IS_INPUT);
        let capture = physical(c, PortFlags::IS_OUTPUT);
        let mut routed = Routed {
            output: pick(&device_names(&playback), output),
            input: pick(&device_names(&capture), input),
            playback: Vec::new(),
        };

        if let Some(d) = &routed.output {
            routed.playback = device_ports(&playback, d).into_iter().cloned().collect();
        }
        let input_ports = match (&routed.input, link_input) {
            (Some(d), true) => device_ports(&capture, d),
            _ => Vec::new(),
        };
        let want = wanted(
            outs,
            ins,
            &routed.playback.iter().collect::<Vec<_>>(),
            &input_ports,
        );

        let mut have = Vec::new();
        for o in outs {
            let peers = c
                .port_by_name(o)
                .map(|p| p.get_connections())
                .unwrap_or_default();
            have.extend(peers.into_iter().map(|p| (o.clone(), p)));
        }
        for i in ins {
            let peers = c
                .port_by_name(i)
                .map(|p| p.get_connections())
                .unwrap_or_default();
            have.extend(peers.into_iter().map(|p| (p, i.clone())));
        }

        let (remove, add) = diff(&have, &want);
        for (src, dst) in remove {
            if let Err(e) = c.disconnect_ports_by_name(src, dst) {
                log::warn!("JACK: cannot disconnect {src} → {dst}: {e}");
            }
        }
        for (src, dst) in add {
            if let Err(e) = c.connect_ports_by_name(src, dst) {
                log::warn!("JACK: cannot connect {src} → {dst}: {e}");
            }
        }
        routed
    }

    pub fn set_buffer_size(&self, size: u32) -> Result<(), String> {
        match &self.stream {
            Stream::Jack { client, .. } => client
                .as_client()
                .set_buffer_size(size)
                .map_err(|e| format!("buffer size {size} rejected: {e}")),
            Stream::Cpal(_) => Err("buffer size change needs a restart in cpal mode".into()),
            #[cfg(test)]
            Stream::Offline => Err("offline audio has no buffer size".into()),
        }
    }
}

/// Commands per queue: a session restore or slices laid out on all 128 pads send several
/// commands per pad at once, and a full queue drops them.
const QUEUE: usize = 1024;
/// Every hit returns its sample reference when the voice ends; the UI drains this every frame.
const GARBAGE: usize = 1024;

/// An engine whose released samples come back through the returned queue.
fn engine(
    sample_rate: u32,
) -> (
    Engine,
    rtrb::Consumer<Retired>,
    Arc<AtomicUsize>,
    crate::pattern_record::Capture,
) {
    let (tx, rx) = rtrb::RingBuffer::new(GARBAGE);
    let mut engine = Engine::new(sample_rate);
    engine.set_garbage(tx);
    let preview_pos = Arc::new(AtomicUsize::new(usize::MAX));
    engine.set_preview_pos(preview_pos.clone());
    let capture = crate::pattern_record::Capture::new(&mut engine);
    (engine, rx, preview_pos, capture)
}

#[cfg(test)]
pub fn offline(sample_rate: u32) -> (Audio, Realtime) {
    let (engine, garbage, preview_pos, pattern_capture) = engine(sample_rate);
    let (commands, midi_commands, consumers) = queues();
    let stats = Arc::new(Stats::default());
    let project = Arc::new(ProjectState::default());
    let rt = Realtime::with_project(
        engine,
        consumers,
        stats.clone(),
        sample_rate,
        project.clone(),
    );
    let audio = Audio {
        backend: "offline",
        pattern_capture,
        project,
        commands,
        midi_commands: Some(midi_commands),
        garbage,
        stats,
        stopped: Arc::new(AtomicBool::new(false)),
        ports_changed: Arc::new(AtomicBool::new(false)),
        preview_pos,
        recorder: Arc::new(Recorder::new(sample_rate as usize)),
        stream: Stream::Offline,
    };
    (audio, rt)
}

/// Queues into the audio thread: (UI, MIDI) producers and the consumers for `Realtime`.
fn queues() -> (
    rtrb::Producer<ProjectCommand>,
    rtrb::Producer<ProjectCommand>,
    Vec<rtrb::Consumer<ProjectCommand>>,
) {
    let (ui, ui_rx) = rtrb::RingBuffer::new(QUEUE);
    let (midi, midi_rx) = rtrb::RingBuffer::new(QUEUE);
    (ui, midi, vec![ui_rx, midi_rx])
}

/// Starts audio. `Auto` tries cpal if JACK fails; the second value says what failed.
pub fn start(backend: Backend, buffer_size: u32) -> Result<(Audio, Option<String>), String> {
    match backend {
        Backend::Jack => start_jack(buffer_size).map(|a| (a, None)),
        Backend::Cpal => start_cpal(buffer_size).map(|a| (a, None)),
        Backend::Auto => match start_jack(buffer_size) {
            Ok(a) => Ok((a, None)),
            Err(jack_err) => {
                log::warn!("{jack_err}; trying cpal");
                start_cpal(buffer_size)
                    .map(|a| (a, Some(format!("{jack_err}; using fallback output cpal"))))
                    .map_err(|cpal_err| format!("{jack_err}; {cpal_err}"))
            }
        },
    }
}

/// Recording sources.
pub const SOURCE_MIC: u8 = 0;
pub const SOURCE_OUTPUT: u8 = 1;
/// The recording stops by itself after this many seconds.
pub const REC_CAP_SECS: usize = 30;

/// The recording ring: 30 s of interleaved stereo, allocated once at start, written only
/// from the audio thread. The app starts and stops it through atomics and reads the buffer
/// only after `recording` turned false (release/acquire), so no locks touch the callback.
pub struct Recorder {
    buf: UnsafeCell<Box<[f32]>>,
    cap_frames: usize,
    pub recording: AtomicBool,
    pub source: AtomicU8,
    /// Interleaved stereo frames written so far.
    pub frames: AtomicUsize,
    /// The cap stopped the recording by itself.
    pub capped: AtomicBool,
}
// Single writer (the audio thread), readers start after the writer is done.
unsafe impl Sync for Recorder {}

impl Recorder {
    pub fn new(rate: usize) -> Self {
        let cap_frames = rate.max(1) * REC_CAP_SECS;
        Self {
            buf: UnsafeCell::new(vec![0.0; cap_frames * 2].into_boxed_slice()),
            cap_frames,
            recording: AtomicBool::new(false),
            source: AtomicU8::new(SOURCE_MIC),
            frames: AtomicUsize::new(0),
            capped: AtomicBool::new(false),
        }
    }

    /// Starts a fresh recording from `source`.
    pub fn start(&self, source: u8) {
        self.frames.store(0, Relaxed);
        self.capped.store(false, Relaxed);
        self.source.store(source, Relaxed);
        self.recording.store(true, Release);
    }

    /// Stops and reports the frames written.
    pub fn stop(&self) -> usize {
        self.recording.store(false, Release);
        self.frames.load(Acquire)
    }

    /// The cap ended the recording while the app still counts it as running.
    pub fn stopped_by_cap(&self) -> bool {
        self.capped.load(Acquire) && !self.recording.load(Acquire)
    }

    /// Audio thread only: appends one block of stereo frames, stopping at the cap.
    fn write(&self, l: &[f32], r: &[f32]) {
        if !self.recording.load(Relaxed) {
            return;
        }
        // safe: only the audio thread writes, and only while `recording` is set
        let buf = unsafe { &mut *self.buf.get() };
        let mut f = self.frames.load(Relaxed);
        for i in 0..l.len() {
            if f >= self.cap_frames {
                self.capped.store(true, Release);
                self.recording.store(false, Release);
                return;
            }
            buf[f * 2] = l[i];
            buf[f * 2 + 1] = r[i];
            f += 1;
        }
        self.frames.store(f, Release);
    }

    /// The app, after `stop`: a copy of the recorded interleaved stereo frames.
    pub fn take(&self, frames: usize) -> Vec<f32> {
        let n = frames.min(self.cap_frames) * 2;
        // safe: the writer is done (recording is false), see the pairing above
        let buf = unsafe { &*self.buf.get() };
        buf[..n].to_vec()
    }
}

pub struct JackProcess {
    out_l: jack::Port<jack::AudioOut>,
    out_r: jack::Port<jack::AudioOut>,
    in_l: jack::Port<jack::AudioIn>,
    in_r: jack::Port<jack::AudioIn>,
    /// The output tap: connected to the monitor ports of the output device when recording.
    rec_l: jack::Port<jack::AudioIn>,
    rec_r: jack::Port<jack::AudioIn>,
    rt: Realtime,
    stats: Arc<Stats>,
    recorder: Arc<Recorder>,
}

impl jack::ProcessHandler for JackProcess {
    fn process(&mut self, _: &jack::Client, ps: &jack::ProcessScope) -> jack::Control {
        let peak = |buf: &[f32]| buf.iter().fold(0.0f32, |m, s| m.max(s.abs()));
        let input = peak(self.in_l.as_slice(ps)).max(peak(self.in_r.as_slice(ps)));
        self.stats.record_input_peak(input);
        if self.recorder.source.load(Relaxed) == SOURCE_MIC {
            self.recorder
                .write(self.in_l.as_slice(ps), self.in_r.as_slice(ps));
        }
        self.rt
            .process(self.out_l.as_mut_slice(ps), self.out_r.as_mut_slice(ps));
        if self.recorder.source.load(Relaxed) == SOURCE_OUTPUT {
            self.recorder
                .write(self.rec_l.as_slice(ps), self.rec_r.as_slice(ps));
        }
        // the strip shows what goes into the file (25); zero while nothing records
        let rec = match (
            self.recorder.recording.load(Relaxed),
            self.recorder.source.load(Relaxed),
        ) {
            (false, _) => 0.0,
            (true, SOURCE_MIC) => input,
            (true, _) => peak(self.rec_l.as_slice(ps)).max(peak(self.rec_r.as_slice(ps))),
        };
        self.stats.record_rec_peak(rec);
        jack::Control::Continue
    }

    fn buffer_size(&mut self, c: &jack::Client, size: jack::Frames) -> jack::Control {
        self.stats.set_format(size, c.sample_rate());
        jack::Control::Continue
    }
}

pub struct JackNotify {
    stats: Arc<Stats>,
    stopped: Arc<AtomicBool>,
    ports_changed: Arc<AtomicBool>,
}

// JACK must not be called from notification callbacks: only raise flags.
impl jack::NotificationHandler for JackNotify {
    unsafe fn shutdown(&mut self, _: jack::ClientStatus, _: &str) {
        self.stopped.store(true, Relaxed);
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        self.stats.add_xrun();
        jack::Control::Continue
    }

    fn port_registration(&mut self, _: &jack::Client, _: jack::PortId, _: bool) {
        self.ports_changed.store(true, Relaxed);
    }
}

fn start_jack(buffer_size: u32) -> Result<Audio, String> {
    let err = |e: jack::Error| format!("JACK unavailable: {e}");
    let (client, _) =
        jack::Client::new("ptsr", jack::ClientOptions::NO_START_SERVER).map_err(err)?;
    // 0 = auto: take the server's size as is, never reconfigure the shared graph
    if buffer_size > 0
        && let Err(e) = client.set_buffer_size(buffer_size)
    {
        log::warn!("JACK refused buffer size {buffer_size}: {e}");
    }
    let sample_rate = client.sample_rate();
    let out = |name| client.register_port(name, jack::AudioOut::default());
    let (out_l, out_r) = (out("out_l").map_err(err)?, out("out_r").map_err(err)?);
    let inp = |name| client.register_port(name, jack::AudioIn::default());
    let (in_l, in_r) = (inp("in_l").map_err(err)?, inp("in_r").map_err(err)?);
    let (rec_l, rec_r) = (inp("rec_l").map_err(err)?, inp("rec_r").map_err(err)?);
    let outs = [out_l.name().map_err(err)?, out_r.name().map_err(err)?];
    let ins = [in_l.name().map_err(err)?, in_r.name().map_err(err)?];
    let recs = [rec_l.name().map_err(err)?, rec_r.name().map_err(err)?];

    let stats = Arc::new(Stats::default());
    stats.set_format(client.buffer_size(), sample_rate);
    let stopped = Arc::new(AtomicBool::new(false));
    let ports_changed = Arc::new(AtomicBool::new(false));
    let (commands, midi_commands, rx) = queues();
    let (engine, garbage, preview_pos, pattern_capture) = engine(sample_rate);
    let recorder = Arc::new(Recorder::new(sample_rate as usize));
    let project = Arc::new(ProjectState::default());
    let process = JackProcess {
        out_l,
        out_r,
        in_l,
        in_r,
        rec_l,
        rec_r,
        rt: Realtime::with_project(engine, rx, stats.clone(), sample_rate, project.clone()),
        stats: stats.clone(),
        recorder: recorder.clone(),
    };
    let notify = JackNotify {
        stats: stats.clone(),
        stopped: stopped.clone(),
        ports_changed: ports_changed.clone(),
    };
    let client = client.activate_async(notify, process).map_err(err)?;
    // buffer size may be 0 right after activation; the real one arrives via the buffer_size callback
    log::info!("JACK: active @ {sample_rate} Hz");

    Ok(Audio {
        backend: "jack",
        pattern_capture,
        project,
        commands,
        midi_commands: Some(midi_commands),
        garbage,
        stats,
        stopped,
        ports_changed,
        preview_pos,
        recorder,
        stream: Stream::Jack {
            client,
            outs,
            ins,
            recs,
        },
    })
}

/// Max frames the cpal callback processes in one pass; more is processed in chunks.
const CPAL_CHUNK: usize = 4096;

fn start_cpal(buffer_size: u32) -> Result<Audio, String> {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

    let err = |e: cpal::Error| format!("cpal: {e}");
    let device = cpal::default_host()
        .default_output_device()
        .ok_or_else(|| "cpal: no output device".to_string())?;
    let default = device.default_output_config().map_err(err)?;
    if default.sample_format() != cpal::SampleFormat::F32 {
        return Err(format!(
            "cpal: sample format {:?} is not supported, need f32",
            default.sample_format()
        ));
    }
    let sample_rate = default.sample_rate();
    let channels = default.channels() as usize;

    let stats = Arc::new(Stats::default());
    stats.set_format(buffer_size, sample_rate);
    let stopped = Arc::new(AtomicBool::new(false));

    let build = |buffer: cpal::BufferSize| {
        let (commands, midi_commands, rx) = queues();
        let (engine, garbage, preview_pos, pattern_capture) = engine(sample_rate);
        let project = Arc::new(ProjectState::default());
        let mut rt =
            Realtime::with_project(engine, rx, stats.clone(), sample_rate, project.clone());
        let recorder = Arc::new(Recorder::new(sample_rate as usize));
        let rec_cb = recorder.clone();
        let (mut l, mut r) = (vec![0.0; CPAL_CHUNK], vec![0.0; CPAL_CHUNK]);
        let (cb_stats, err_stats, err_stopped) = (stats.clone(), stats.clone(), stopped.clone());
        device
            .build_output_stream(
                cpal::StreamConfig {
                    channels: channels as u16,
                    sample_rate,
                    buffer_size: buffer,
                },
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    cb_stats.set_format((data.len() / channels) as u32, sample_rate);
                    for chunk in data.chunks_mut(channels * CPAL_CHUNK) {
                        let n = chunk.len() / channels;
                        rt.process(&mut l[..n], &mut r[..n]);
                        // no input stream in cpal: only the output mix can be recorded
                        let recording = rec_cb.recording.load(Relaxed)
                            && rec_cb.source.load(Relaxed) == SOURCE_OUTPUT;
                        if rec_cb.source.load(Relaxed) == SOURCE_OUTPUT {
                            rec_cb.write(&l[..n], &r[..n]);
                        }
                        let peak = l[..n]
                            .iter()
                            .chain(&r[..n])
                            .fold(0.0f32, |m, s| m.max(s.abs()));
                        cb_stats.record_rec_peak(if recording { peak } else { 0.0 });
                        for (i, frame) in chunk.chunks_mut(channels).enumerate() {
                            frame.fill(0.0);
                            frame[0] = l[i];
                            if let Some(s) = frame.get_mut(1) {
                                *s = r[i];
                            }
                        }
                    }
                },
                // called from the audio thread: atomics only
                move |e: cpal::Error| match e.kind() {
                    cpal::ErrorKind::Xrun => err_stats.add_xrun(),
                    _ => err_stopped.store(true, Relaxed),
                },
                None,
            )
            .map(|s| {
                (
                    s,
                    commands,
                    midi_commands,
                    garbage,
                    preview_pos,
                    recorder,
                    project,
                    pattern_capture,
                )
            })
    };
    // 0 = auto: the system picks; an explicit size may be rejected, then the system picks too
    let first = if buffer_size > 0 {
        cpal::BufferSize::Fixed(buffer_size)
    } else {
        cpal::BufferSize::Default
    };
    let (stream, commands, midi_commands, garbage, preview_pos, recorder, project, pattern_capture) =
        build(first)
            .or_else(|e| {
                log::warn!("cpal: buffer size {buffer_size} rejected ({e}), using system default");
                build(cpal::BufferSize::Default)
            })
            .map_err(err)?;
    stream.play().map_err(err)?;
    log::info!("cpal: {sample_rate} Hz, {channels} ch, buffer {buffer_size}");
    Ok(Audio {
        backend: "cpal",
        pattern_capture,
        project,
        commands,
        midi_commands: Some(midi_commands),
        garbage,
        stats,
        stopped,
        ports_changed: Arc::new(AtomicBool::new(false)),
        preview_pos,
        recorder,
        stream: Stream::Cpal(stream),
    })
}

/// Physical audio ports: `IS_INPUT` are device outputs, `IS_OUTPUT` are device inputs.
fn physical(c: &jack::Client, direction: PortFlags) -> Vec<String> {
    c.ports(None, Some("audio"), direction | PortFlags::IS_PHYSICAL)
}

/// The monitor twin of a playback port: `X:playback_FL` plays where `X:monitor_FL` listens.
fn monitor_of(playback_port: &str) -> Option<String> {
    let mon = playback_port.replace("playback", "monitor");
    (mon != playback_port).then_some(mon)
}

/// The monitor ports of all `playback` ports, found among the `outputs` (monitors are
/// sources); `None` when there is no playback or any twin is missing.
fn monitors_of(playback: &[String], outputs: &[String]) -> Option<Vec<String>> {
    if playback.is_empty() {
        return None;
    }
    playback
        .iter()
        .map(|pb| monitor_of(pb).filter(|m| outputs.contains(m)))
        .collect()
}

/// A device is the port name up to the last `:`.
fn device_of(port: &str) -> &str {
    port.rsplit_once(':').map_or(port, |(d, _)| d)
}

/// Device names without duplicates, in JACK order.
fn device_names(ports: &[String]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for p in ports {
        let d = device_of(p);
        if !names.iter().any(|n| n == d) {
            names.push(d.to_owned());
        }
    }
    names
}

fn device_ports<'a>(ports: &'a [String], device: &str) -> Vec<&'a String> {
    ports.iter().filter(|p| device_of(p) == device).collect()
}

/// The chosen device if present, otherwise the first one.
fn pick(devices: &[String], wanted: Option<&str>) -> Option<String> {
    wanted
        .and_then(|w| devices.iter().find(|d| *d == w))
        .or(devices.first())
        .cloned()
}

/// Our two ports to the device ports. A mono device gets both channels.
fn pairs(ours: &[String; 2], device: &[&String]) -> Vec<(String, String)> {
    if device.is_empty() {
        return Vec::new();
    }
    let device = device.iter().cycle();
    ours.iter()
        .zip(device)
        .map(|(o, d)| (o.clone(), d.to_string()))
        .collect()
}

/// The links to have: our outputs to the device playback ports, the device capture ports
/// to our inputs; no capture ports, no input links.
fn wanted(
    outs: &[String; 2],
    ins: &[String; 2],
    playback: &[&String],
    capture: &[&String],
) -> Vec<(String, String)> {
    let mut want = pairs(outs, playback);
    // input: the link goes from the device port to our port
    want.extend(
        pairs(ins, capture)
            .into_iter()
            .map(|(ours, dev)| (dev, ours)),
    );
    want
}

/// A JACK link: from, to.
type Link<'a> = (&'a str, &'a str);

/// Links to remove and to add to turn `have` into `want`.
fn diff<'a>(
    have: &'a [(String, String)],
    want: &'a [(String, String)],
) -> (Vec<Link<'a>>, Vec<Link<'a>>) {
    let pick = |from: &'a [(String, String)], other: &'a [(String, String)]| {
        from.iter()
            .filter(|l| !other.contains(l))
            .map(|(a, b)| (a.as_str(), b.as_str()))
            .collect()
    };
    (pick(have, want), pick(want, have))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ports(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn half_a_minute_stops_by_itself_and_keeps_the_head() {
        let rec = Recorder::new(100); // cap: 3000 frames
        rec.start(SOURCE_MIC);
        assert!(rec.recording.load(Relaxed));
        // 100 frames of a block, 40 blocks = 4000 frames: the cap stops it at 3000
        let (l, r): (Vec<f32>, Vec<f32>) = (
            (0..100).map(|i| i as f32).collect(),
            (0..100).map(|i| -(i as f32)).collect(),
        );
        for _ in 0..40 {
            rec.write(&l, &r);
        }
        assert!(rec.stopped_by_cap(), "the cap stopped it");
        let take = rec.take(rec.stop());
        assert_eq!(take.len(), 3000 * 2, "the first 30 seconds are kept");
        assert_eq!(take[0], 0.0);
        assert_eq!(take[1], -0.0);
    }

    #[test]
    fn recorder_stop_returns_the_length_and_drops_nothing() {
        let rec = Recorder::new(48);
        rec.start(SOURCE_OUTPUT);
        let l = vec![0.5f32; 10];
        let r = vec![-0.5f32; 10];
        rec.write(&l, &r);
        assert_eq!(rec.stop(), 10);
        assert!(!rec.recording.load(Relaxed));
        let take = rec.take(10);
        assert_eq!(take[9 * 2], 0.5);
        assert_eq!(take[9 * 2 + 1], -0.5);
        // a fresh start forgets the old recording
        rec.start(SOURCE_MIC);
        assert_eq!(rec.frames.load(Relaxed), 0);
        assert!(!rec.capped.load(Relaxed));
    }

    /// A recording bound to a pad plays back what was heard, without clicks:
    /// renders/10-record.wav. A chord with a kick on every beat (100 BPM) plays, the
    /// recording starts in the middle of it and lasts one bar, then a pad plays it twice.
    #[test]
    fn render_a_bound_recording() {
        use engine::offline::{Event, Scenario, render, write_wav};
        use engine::{Command, Engine, checks};
        use std::f32::consts::TAU;

        const RATE: usize = 48_000;
        let beat = RATE * 60 / 100;
        let played = |n: usize| {
            let t = n as f32 / RATE as f32;
            let tb = (n % beat) as f32 / RATE as f32;
            let kick = 0.3 * (TAU * 60.0 * tb).sin() * (-tb * 8.0).exp() * (tb * 200.0).min(1.0);
            let chord = 0.15 * ((TAU * 220.0 * t).sin() + (TAU * 277.2 * t).sin())
                + 0.1 * (TAU * 329.6 * t).sin();
            (chord + kick, chord * 0.8 + kick)
        };
        // the key goes down in the middle of a loud kick: without the fade this is a click
        let start = (RATE / 4..).find(|&n| played(n).0.abs() > 0.4).unwrap();
        let bar = beat * 4;
        let rec = Recorder::new(RATE);
        rec.start(SOURCE_MIC);
        for block in (start..start + bar).step_by(128) {
            let (l, r): (Vec<f32>, Vec<f32>) = (block..block + 128).map(played).unzip();
            rec.write(&l, &r);
        }
        let ring = rec.take(rec.stop());
        let sample = crate::sample_from_interleaved(&ring, RATE as u32).expect("not silent");
        let file = std::env::temp_dir().join(format!("ptsr-render-10-{}.wav", std::process::id()));
        crate::wav::write(&file, &sample, RATE as u32).unwrap();
        let sound = Arc::new(crate::loader::decode(&file, RATE as u32, |_| {}).unwrap());
        std::fs::remove_file(&file).ok();
        assert_eq!(sound.len(), bar);

        let mut engine = Engine::new(RATE as u32);
        engine.handle(
            0,
            Command::SetPadSound {
                pad: 0,
                sound: Some(sound),
            },
        );
        let hits = [RATE / 2, RATE / 2 + bar];
        let press = Command::PadPress {
            pad: 0,
            velocity: 1.0,
        };
        let sc = Scenario {
            sample_rate: RATE as u32,
            frames: hits[1] + bar + RATE / 2,
            events: hits
                .iter()
                .map(|&frame| Event {
                    frame,
                    cmd: press.clone(),
                })
                .collect(),
        };
        let (l, r) = render(&mut engine, &sc, 128);

        assert_eq!(
            checks::first_non_finite(&l).or(checks::first_non_finite(&r)),
            None
        );
        assert!(checks::peak(&l).max(checks::peak(&r)) <= 0.95, "clipping");
        assert_eq!(checks::peak(&l[..hits[0]]), 0.0, "silence before the hit");
        assert_eq!(
            checks::peak(&l[hits[1] + bar..]),
            0.0,
            "silence after the end"
        );
        let onset = checks::first_loud(&l, hits[0] - 100..hits[0] + 100, 1e-6).unwrap();
        assert!(
            onset.abs_diff(hits[0]) <= 1,
            "the pad starts on the hit: {onset}"
        );
        // the chord and the kick move by at most ~0.05 per frame; a click jumps far more
        let step = checks::max_step(&l).max(checks::max_step(&r));
        assert!(step < 0.1, "a click: a step of {step}");
        // past the fades the pad plays what was heard, to 16-bit accuracy
        let fade = RATE / 200;
        for i in fade..bar - fade {
            let (want_l, want_r) = played(start + i);
            let (got_l, got_r) = (l[hits[0] + i], r[hits[0] + i]);
            assert!(
                (got_l - want_l).abs() < 1e-3 && (got_r - want_r).abs() < 1e-3,
                "frame {i}: played {want_l} {want_r}, the pad gives {got_l} {got_r}"
            );
        }
        let out =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../renders/10-record.wav");
        write_wav(&out, RATE as u32, &l, &r).unwrap();
    }

    #[test]
    fn capture_what_plays_finds_the_monitor_among_outputs() {
        // port names as PipeWire-JACK shows them: playback ports are inputs, monitors outputs
        let bt = "bluez_output.0C_17_73_F3_B8_05.1";
        let playback = ports(&[&format!("{bt}:playback_FL"), &format!("{bt}:playback_FR")]);
        let outputs = ports(&[
            &format!("{bt}:monitor_FL"),
            &format!("{bt}:monitor_FR"),
            "ptsr:out_l",
        ]);
        assert_eq!(
            monitors_of(&playback, &outputs),
            Some(ports(&[
                &format!("{bt}:monitor_FL"),
                &format!("{bt}:monitor_FR")
            ]))
        );
        // looking among the inputs (the playback ports themselves) finds nothing
        assert_eq!(monitors_of(&playback, &playback), None);
    }

    #[test]
    fn no_monitor_on_the_device() {
        let playback = ports(&["dev:playback_FL", "dev:playback_FR"]);
        assert_eq!(monitors_of(&playback, &ports(&["dev:monitor_FL"])), None);
        assert_eq!(monitors_of(&[], &ports(&["dev:monitor_FL"])), None);
        assert_eq!(monitors_of(&ports(&["dev:out_1"]), &[]), None);
    }

    #[test]
    fn recorder_ignores_frames_while_stopped() {
        let rec = Recorder::new(48);
        rec.write(&[0.5; 4], &[0.5; 4]);
        assert_eq!(rec.frames.load(Relaxed), 0, "not recording: nothing lands");
    }

    const OURS: [&str; 2] = ["ptsr:out_l", "ptsr:out_r"];

    fn ours() -> [String; 2] {
        OURS.map(String::from)
    }

    fn link(a: &str, b: &str) -> (String, String) {
        (a.into(), b.into())
    }

    #[test]
    fn mono_device_gets_both_channels() {
        let p = ports(&[
            "FreeBuds:playback_MONO",
            "fifine:playback_FL",
            "fifine:playback_FR",
        ]);
        let d = pick(&device_names(&p), None).unwrap();
        assert_eq!(d, "FreeBuds");
        let links = pairs(&ours(), &device_ports(&p, &d));
        assert_eq!(
            links,
            [
                link(OURS[0], "FreeBuds:playback_MONO"),
                link(OURS[1], "FreeBuds:playback_MONO")
            ]
        );
    }

    #[test]
    fn chosen_device_wins_missing_falls_back_to_first() {
        let p = ports(&[
            "Buds:playback_FL",
            "Speaker:playback_FL",
            "Speaker:playback_FR",
        ]);
        let names = device_names(&p);
        assert_eq!(names, ["Buds", "Speaker"]);
        assert_eq!(pick(&names, Some("Speaker")).as_deref(), Some("Speaker"));
        assert_eq!(pick(&names, Some("HDMI")).as_deref(), Some("Buds"));
        assert_eq!(pick(&[], Some("HDMI")), None);
    }

    #[test]
    fn device_name_with_colon() {
        assert_eq!(device_of("USB: Audio:playback_FL"), "USB: Audio");
    }

    /// 26 recording: "Start leaves the microphone alone" — with no capture ports wanted,
    /// only the output links stay; the diff then removes any old microphone link.
    #[test]
    fn start_leaves_the_microphone_alone() {
        let outs = ["ptsr:out_l".to_string(), "ptsr:out_r".to_string()];
        let ins = ["ptsr:in_l".to_string(), "ptsr:in_r".to_string()];
        let playback = ports(&["buds:playback_FL", "buds:playback_FR"]);
        let capture = ports(&["mic:capture_FL", "mic:capture_FR"]);
        let pb: Vec<&String> = playback.iter().collect();
        let cap: Vec<&String> = capture.iter().collect();
        let quiet = wanted(&outs, &ins, &pb, &[]);
        assert!(
            quiet
                .iter()
                .all(|(a, b)| a.starts_with("ptsr:out") && b.starts_with("buds:"))
        );
        assert_eq!(quiet.len(), 2);
        let recording = wanted(&outs, &ins, &pb, &cap);
        assert!(recording.contains(&("mic:capture_FL".into(), "ptsr:in_l".into())));
        // linked before, not wanted now: the microphone links go, the output links stay
        let (remove, add) = diff(&recording, &quiet);
        assert_eq!(remove.len(), 2);
        assert!(remove.iter().all(|(a, _)| a.starts_with("mic:")));
        assert!(add.is_empty());
    }

    /// Scenario "Another program connected to the sound" (audio-settings): a port change reroutes, the matching links stay.
    #[test]
    fn diff_keeps_matching_links() {
        let have = [
            link(OURS[0], "A:FL"),
            link(OURS[1], "A:FR"),
            link(OURS[1], "manual:x"),
        ];
        let want = [link(OURS[0], "A:FL"), link(OURS[1], "A:FR")];
        let (remove, add) = diff(&have, &want);
        assert_eq!(remove, [(OURS[1], "manual:x")]);
        assert!(add.is_empty());
    }

    /// Scenario "Switch to the speakers" (audio-settings): the headphone links go, the speaker links come.
    #[test]
    fn diff_switches_device() {
        let have = [link(OURS[0], "A:FL"), link(OURS[1], "A:FR")];
        let want = [link(OURS[0], "B:FL"), link(OURS[1], "B:FR")];
        let (remove, add) = diff(&have, &want);
        assert_eq!(remove.len(), 2);
        assert_eq!(add, [(OURS[0], "B:FL"), (OURS[1], "B:FR")]);
    }

    /// The links from our outputs to the device `pick` takes among `ports` for `saved`.
    fn routed_to(ports: &[String], saved: Option<&str>) -> (String, Vec<(String, String)>) {
        let ins = ["ptsr:in_l".to_string(), "ptsr:in_r".to_string()];
        let device = pick(&device_names(ports), saved).unwrap();
        let links = wanted(&ours(), &ins, &device_ports(ports, &device), &[]);
        (device, links)
    }

    /// Scenario "Headphones turned off and on" (audio-settings): the saved headphones go
    /// away, the links move to the first device left; they come back and the links return.
    /// The saved name is only read here, the config keeps it.
    #[test]
    fn headphones_turned_off_and_on() {
        let both = ports(&[
            "Speaker:playback_FL",
            "Speaker:playback_FR",
            "Buds:playback_FL",
            "Buds:playback_FR",
        ]);
        let speaker = ports(&["Speaker:playback_FL", "Speaker:playback_FR"]);
        let (d, on) = routed_to(&both, Some("Buds"));
        assert_eq!(d, "Buds");
        let (d, off) = routed_to(&speaker, Some("Buds"));
        assert_eq!(d, "Speaker", "the first device left");
        let (remove, add) = diff(&on, &off);
        assert!(remove.iter().all(|(_, b)| b.starts_with("Buds:")) && remove.len() == 2);
        assert!(add.iter().all(|(_, b)| b.starts_with("Speaker:")) && add.len() == 2);
        let (d, back) = routed_to(&both, Some("Buds"));
        assert_eq!(d, "Buds", "back to the headphones");
        assert_eq!(back, on);
    }

    /// Scenario "Start without the selected device" (audio-settings): the saved device is
    /// not there, the sound goes to the first one; the message is `missing_device` in main.
    #[test]
    fn start_without_the_selected_device() {
        let speaker = ports(&["Speaker:playback_FL", "Speaker:playback_FR"]);
        let (d, links) = routed_to(&speaker, Some("Buds"));
        assert_eq!(d, "Speaker");
        assert_eq!(links.len(), 2);
    }
}
