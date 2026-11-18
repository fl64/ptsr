//! Configure the measured SMC-PAD preset over its Private USB MIDI port.

use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

use midir::{Ignore, MidiInput, MidiOutput};

use super::{Raw, Shared};

const PORT: &str = "SMC-PAD-Private";
const SLOT: usize = 4;
const STRIDE: usize = 3539;
const ENCODERS: usize = 115;
const RECORD: usize = 6;
const SIZE: usize = 28312;
const CHUNK: usize = 1009;
const WAIT: Duration = Duration::from_secs(3);
const CHECK_INTERVAL: Duration = Duration::from_secs(1);
const PAD_BANK_INTERVAL: Duration = Duration::from_millis(52);

fn packet(command: u8, data: &[u8]) -> Vec<u8> {
    let mut logical = vec![0, 0x59, command];
    logical.extend_from_slice(&(data.len() as u32).to_le_bytes()[..3]);
    logical.extend_from_slice(data);
    logical.push(!data.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte)));
    let mut wire = vec![0xf0];
    let mut bits = 0u32;
    let mut value = 0u32;
    for byte in logical {
        value |= u32::from(byte) << bits;
        bits += 8;
        while bits >= 7 {
            wire.push((value & 0x7f) as u8);
            value >>= 7;
            bits -= 7;
        }
    }
    if bits > 0 {
        wire.push(value as u8);
    }
    wire.push(0xf7);
    wire
}

fn decode(wire: &[u8]) -> Result<(u8, Vec<u8>), String> {
    if wire.first() != Some(&0xf0) || wire.last() != Some(&0xf7) {
        return Err("bad USB SysEx frame".into());
    }
    let mut logical = Vec::new();
    let (mut bits, mut value) = (0u32, 0u32);
    for byte in &wire[1..wire.len() - 1] {
        if *byte >= 0x80 {
            return Err("bad USB SysEx data byte".into());
        }
        value |= u32::from(*byte) << bits;
        bits += 7;
        while bits >= 8 {
            logical.push(value as u8);
            value >>= 8;
            bits -= 8;
        }
    }
    if logical.len() < 7 || logical[..2] != [0, 0x59] {
        return Err("bad SMC-PAD reply prefix".into());
    }
    let length = u32::from_le_bytes([logical[3], logical[4], logical[5], 0]) as usize;
    if logical.len() != length + 7 {
        return Err("bad SMC-PAD reply length".into());
    }
    let data = &logical[6..6 + length];
    let check = !data.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte));
    if logical[6 + length] != check {
        return Err("bad SMC-PAD reply checksum".into());
    }
    Ok((logical[2], data.to_vec()))
}

trait Transport {
    fn exchange(&mut self, command: u8, data: &[u8]) -> Result<(u8, Vec<u8>), String>;
}

struct MidiTransport {
    _input: midir::MidiInputConnection<()>,
    output: midir::MidiOutputConnection,
    replies: Receiver<Vec<u8>>,
    wait: Duration,
    master: Option<String>,
}

impl Transport for MidiTransport {
    fn exchange(&mut self, command: u8, data: &[u8]) -> Result<(u8, Vec<u8>), String> {
        self.output
            .send(&packet(command, data))
            .map_err(|e| format!("USB send: {e}"))?;
        let mut frame = Vec::new();
        let deadline = Instant::now() + self.wait;
        loop {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .ok_or("USB reply timed out")?;
            let bytes = self
                .replies
                .recv_timeout(remaining)
                .map_err(|e| format!("USB reply: {e}"))?;
            if frame.is_empty() {
                if bytes.first() != Some(&0xf0) {
                    continue;
                }
            } else if bytes.first() == Some(&0xf0) {
                frame.clear();
            }
            frame.extend_from_slice(&bytes);
            if frame.last() == Some(&0xf7) {
                return decode(&frame);
            }
        }
    }
}

fn read<T: Transport>(
    transport: &mut T,
    region: u8,
    address: usize,
    length: usize,
) -> Result<Vec<u8>, String> {
    let mut request = vec![region];
    request.extend_from_slice(&(address as u32).to_le_bytes());
    request.extend_from_slice(&((length as u32).to_le_bytes())[..3]);
    let (command, data) = transport.exchange(0x23, &request)?;
    if command != 0x23 || data.len() != length + 8 || data[..8] != request {
        return Err(format!("USB read mismatch at {address}"));
    }
    Ok(data[8..].to_vec())
}

fn read_all<T: Transport>(transport: &mut T) -> Result<Vec<u8>, String> {
    let mut all = Vec::with_capacity(SIZE);
    while all.len() < SIZE {
        let length = CHUNK.min(SIZE - all.len());
        all.extend_from_slice(&read(transport, 5, all.len(), length)?);
    }
    Ok(all)
}

fn write<T: Transport>(transport: &mut T, address: usize, bytes: &[u8]) -> Result<(), String> {
    let mut request = vec![5];
    request.extend_from_slice(&(address as u32).to_le_bytes());
    request.extend_from_slice(&((bytes.len() as u32).to_le_bytes())[..3]);
    request.extend_from_slice(bytes);
    let (command, data) = transport.exchange(0x22, &request)?;
    if command != 0 || data != [0] {
        return Err(format!("USB write rejected at {address}"));
    }
    Ok(())
}

struct Status {
    mode: u8,
    slot: u16,
    pads: u8,
}

impl Status {
    fn checked_pads(self) -> Result<u8, String> {
        if self.slot != SLOT as u16 + 1 {
            return Err(format!(
                "Preset {} is active. Select preset {} to use the knobs.",
                self.slot,
                SLOT + 1
            ));
        }
        if self.mode != 2 {
            return Err(format!("Preset status has mode {}", self.mode));
        }
        Ok(self.pads)
    }
}

fn read_status<T: Transport>(transport: &mut T) -> Result<Status, String> {
    let data = read(transport, 4, 0, 12)?;
    Ok(Status {
        mode: data[6],
        slot: u16::from(data[10]) + 1,
        pads: data[11],
    })
}

fn active<T: Transport>(transport: &mut T) -> Result<(), String> {
    read_status(transport)?.checked_pads().map(|_| ())
}

/// The PAD BANK hold (131): status 1 reads pressed for as long as the owner holds it.
#[derive(Default)]
struct PadBankHold {
    armed: bool,
    held: bool,
}

impl PadBankHold {
    /// Returns the new hold state on a change. A press counts only after a released status.
    fn observe(&mut self, status: u8) -> Option<bool> {
        let pressed = status == 1 && (self.armed || self.held);
        self.armed = status == 0;
        (pressed != self.held).then(|| {
            self.held = pressed;
            pressed
        })
    }

    /// Forgets the source. Returns true if a hold must end first.
    fn reset(&mut self) -> bool {
        std::mem::take(self).held
    }
}

fn end_hold(tx: &Sender<Raw>, hold: &mut PadBankHold, source: &mut Option<(String, u64)>) {
    if hold.reset()
        && let Some((port, setting)) = source.take()
    {
        let _ = tx.send(Raw::SmcPadBank {
            port,
            requested_at: Instant::now(),
            setting,
            held: false,
        });
    }
    *source = None;
}

fn expected(index: usize) -> [u8; RECORD] {
    [3, 2, 0, 30 + index as u8, 63, 65]
}

fn restore<T: Transport>(transport: &mut T, before: &[u8]) -> Result<(), String> {
    let mut first_error = None;
    for index in 0..16 {
        let address = SLOT * STRIDE + ENCODERS + index * RECORD;
        if let Err(error) = write(transport, address, &before[address..address + RECORD]) {
            first_error.get_or_insert(error);
        }
    }
    let actual =
        read_all(transport).map_err(|error| format!("SMC-PAD restore read-back: {error}"))?;
    if actual != before {
        let write_error = first_error.map_or(String::new(), |error| format!("; {error}"));
        return Err(format!("SMC-PAD restore read-back differs{write_error}"));
    }
    Ok(())
}

fn configure<T: Transport>(transport: &mut T) -> Result<(), String> {
    let (command, _) = transport.exchange(0x11, &[])?;
    if command != 0x11 {
        return Err("SMC-PAD identity reply differs".into());
    }
    active(transport)?;
    let before = read_all(transport)?;
    let mut after = before.clone();
    for index in 0..16 {
        let address = SLOT * STRIDE + ENCODERS + index * RECORD;
        let original = &before[address..address + RECORD];
        if original[1..4] != [2, 0, 30 + index as u8] || !matches!(original[0], 0 | 3) {
            return Err(format!(
                "SMC-PAD encoder {} differs from measured preset",
                index + 1
            ));
        }
        after[address..address + RECORD].copy_from_slice(&expected(index));
    }
    if before == after {
        return Ok(());
    }
    let mut attempted = false;
    let result = (|| {
        for index in 0..16 {
            active(transport)?;
            let address = SLOT * STRIDE + ENCODERS + index * RECORD;
            if before[address..address + RECORD] == after[address..address + RECORD] {
                continue;
            }
            attempted = true;
            write(transport, address, &after[address..address + RECORD])?;
            if read(transport, 5, address, RECORD)? != after[address..address + RECORD] {
                return Err(format!("SMC-PAD encoder {} read-back differs", index + 1));
            }
        }
        active(transport)?;
        if read_all(transport)? != after {
            return Err("SMC-PAD preset changed outside encoder records".into());
        }
        Ok(())
    })();
    if let Err(error) = result {
        if attempted {
            return match restore(transport, &before) {
                Ok(()) => Err(format!("{error}; original records restored")),
                Err(restore_error) => Err(format!("{error}; restore failed: {restore_error}")),
            };
        }
        return Err(error);
    }
    Ok(())
}

fn checked<T: Transport>(transport: &mut T) -> Result<(), String> {
    let (command, _) = transport.exchange(0x11, &[])?;
    if command != 0x11 {
        return Err("SMC-PAD identity reply differs".into());
    }
    active(transport)?;
    let address = SLOT * STRIDE + ENCODERS;
    let records = read(transport, 5, address, 16 * RECORD)?;
    for (index, record) in records.as_chunks::<RECORD>().0.iter().enumerate() {
        if *record != expected(index) {
            return Err(format!(
                "SMC-PAD encoder {} is no longer relative",
                index + 1
            ));
        }
    }
    Ok(())
}

fn connect() -> Result<Option<MidiTransport>, String> {
    let mut input = MidiInput::new("ptsr SMC-PAD setup input").map_err(|e| e.to_string())?;
    input.ignore(Ignore::None);
    let output = MidiOutput::new("ptsr SMC-PAD setup output").map_err(|e| e.to_string())?;
    let Some(input_port) = input
        .ports()
        .into_iter()
        .find(|port| input.port_name(port).is_ok_and(|name| name.contains(PORT)))
    else {
        return Ok(None);
    };
    let private_id = input_port.id();
    let private_client = private_id.split_once(':').map(|(client, _)| client);
    let mut masters = input.ports().into_iter().filter_map(|port| {
        let id = port.id();
        let (client, _) = id.split_once(':')?;
        let name = input.port_name(&port).ok()?;
        (Some(client) == private_client && name.contains("SMC-PAD-Master")).then_some(name)
    });
    let master = masters.next().filter(|_| masters.next().is_none());
    let Some(output_port) = output.ports().into_iter().find(|port| {
        port.id() == private_id && output.port_name(port).is_ok_and(|name| name.contains(PORT))
    }) else {
        return Ok(None);
    };
    let (tx, replies) = mpsc::channel();
    let connection = input
        .connect(
            &input_port,
            "ptsr-smc-setup-in",
            move |_, bytes, _| {
                let _ = tx.send(bytes.to_vec());
            },
            (),
        )
        .map_err(|e| e.to_string())?;
    let output = output
        .connect(&output_port, "ptsr-smc-setup-out")
        .map_err(|e| e.to_string())?;
    // Keep the input connection alive for the complete request sequence.
    Ok(Some(MidiTransport {
        _input: connection,
        output,
        replies,
        wait: WAIT,
        master,
    }))
}

fn available() -> Result<bool, String> {
    let input = MidiInput::new("ptsr SMC-PAD setup scan").map_err(|e| e.to_string())?;
    let output = MidiOutput::new("ptsr SMC-PAD setup scan").map_err(|e| e.to_string())?;
    Ok(input
        .ports()
        .iter()
        .any(|port| input.port_name(port).is_ok_and(|name| name.contains(PORT)))
        && output
            .ports()
            .iter()
            .any(|port| output.port_name(port).is_ok_and(|name| name.contains(PORT))))
}

pub(super) fn watch(tx: Sender<Raw>, shared: Arc<Shared>) {
    let mut present = false;
    let mut ready = false;
    let mut previous_source: Option<(String, u64)> = None;
    let mut hold = PadBankHold::default();
    loop {
        let mut deadline = Instant::now() + CHECK_INTERVAL;
        match available() {
            Ok(true) => {
                present = true;
                let result = connect().and_then(|transport| {
                    let mut transport = transport.ok_or("SMC-PAD ports vanished")?;
                    if ready {
                        checked(&mut transport)?;
                    } else {
                        configure(&mut transport)?;
                    }
                    Ok(transport)
                });
                ready = result.is_ok();
                let _ = tx.send(Raw::SmcSetup(
                    result.as_ref().map(|_| ()).map_err(Clone::clone),
                ));
                deadline = Instant::now() + CHECK_INTERVAL;
                if let Ok(mut transport) = result
                    && let Some(port) = transport.master.take()
                    && let Some(setting) = shared.pad_bank_shortcut()
                {
                    let source = (port.clone(), setting);
                    if previous_source.as_ref() != Some(&source) {
                        end_hold(&tx, &mut hold, &mut previous_source);
                        previous_source = Some(source);
                    }
                    transport.wait = PAD_BANK_INTERVAL;
                    while shared.pad_bank_shortcut() == Some(setting) {
                        let requested_at = Instant::now();
                        if requested_at >= deadline {
                            break;
                        }
                        // A failed read or a preset change ends the hold (131).
                        let Ok(status) = read_status(&mut transport) else {
                            end_hold(&tx, &mut hold, &mut previous_source);
                            break;
                        };
                        let value = match status.checked_pads() {
                            Ok(value) => value,
                            Err(error) => {
                                ready = false;
                                end_hold(&tx, &mut hold, &mut previous_source);
                                let _ = tx.send(Raw::SmcSetup(Err(error)));
                                break;
                            }
                        };
                        if let Some(held) = hold.observe(value) {
                            let _ = tx.send(Raw::SmcPadBank {
                                port: port.clone(),
                                requested_at,
                                setting,
                                held,
                            });
                        }
                        std::thread::sleep(
                            (requested_at + PAD_BANK_INTERVAL)
                                .saturating_duration_since(Instant::now()),
                        );
                    }
                } else {
                    end_hold(&tx, &mut hold, &mut previous_source);
                }
            }
            Ok(false) if present => {
                present = false;
                ready = false;
                end_hold(&tx, &mut hold, &mut previous_source);
                let _ = tx.send(Raw::SmcLost);
            }
            Err(error) => {
                ready = false;
                end_hold(&tx, &mut hold, &mut previous_source);
                let _ = tx.send(Raw::SmcSetup(Err(error)));
            }
            Ok(false) => {}
        }
        std::thread::sleep(deadline.saturating_duration_since(Instant::now()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fake {
        state: [u8; 12],
        memory: Vec<u8>,
        writes: usize,
        fail_at: Vec<usize>,
        switch_slot_at: Option<usize>,
    }

    impl Fake {
        fn measured() -> Self {
            let mut state = [0; 12];
            state[6] = 2;
            state[10] = SLOT as u8;
            let mut memory = vec![0; SIZE];
            for index in 0..16 {
                let address = SLOT * STRIDE + ENCODERS + index * RECORD;
                memory[address..address + RECORD].copy_from_slice(&[
                    0,
                    2,
                    0,
                    30 + index as u8,
                    0,
                    127,
                ]);
            }
            Self {
                state,
                memory,
                writes: 0,
                fail_at: Vec::new(),
                switch_slot_at: None,
            }
        }
    }

    impl Transport for Fake {
        fn exchange(&mut self, command: u8, data: &[u8]) -> Result<(u8, Vec<u8>), String> {
            if command == 0x11 {
                return Ok((0x11, vec![]));
            }
            if data.len() < 8 {
                return Err("short request".into());
            }
            let region = data[0];
            let address = u32::from_le_bytes(data[1..5].try_into().unwrap()) as usize;
            let length = u32::from_le_bytes([data[5], data[6], data[7], 0]) as usize;
            if command == 0x23 {
                let bytes = match region {
                    4 => self.state.get(address..address + length),
                    5 => self.memory.get(address..address + length),
                    _ => None,
                }
                .ok_or("read out of range")?;
                let mut reply = data.to_vec();
                reply.extend_from_slice(bytes);
                return Ok((0x23, reply));
            }
            if command == 0x22 {
                self.writes += 1;
                if self.fail_at.contains(&self.writes) {
                    return Err("simulated write error".into());
                }
                if region != 5 || data.len() != 8 + length {
                    return Err("bad write request".into());
                }
                self.memory[address..address + length].copy_from_slice(&data[8..]);
                if self.switch_slot_at == Some(self.writes) {
                    self.state[10] = 5;
                }
                return Ok((0, vec![0]));
            }
            Err("unknown command".into())
        }
    }

    #[test]
    fn pad_bank_status_gives_hold_edges_without_writes() {
        let mut device = Fake::measured();
        let mut hold = PadBankHold::default();
        // A press before the first released status does not count.
        let samples = [1, 0, 1, 1, 0, 2, 1, 0, 1];
        let edges: Vec<_> = samples
            .into_iter()
            .map(|value| {
                device.state[11] = value;
                hold.observe(read_status(&mut device).unwrap().checked_pads().unwrap())
            })
            .collect();
        assert_eq!(
            edges,
            [
                None,
                None,
                Some(true),
                None,
                Some(false),
                None,
                None,
                None,
                Some(true)
            ]
        );
        assert!(hold.reset());
        assert!(!hold.reset());
        assert_eq!(device.writes, 0);
        device.state[10] = 0;
        assert!(read_status(&mut device).unwrap().checked_pads().is_err());
    }

    #[test]
    fn usb_packet_round_trip() {
        let wire = packet(0x23, &[5, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(decode(&wire), Ok((0x23, vec![5, 1, 2, 3, 4, 5, 6, 7])));
        assert_eq!(
            packet(0x11, &[]),
            [0xf0, 0, 0x32, 0x45, 0, 0, 0, 0x40, 0x7f, 0xf7]
        );
    }

    #[test]
    fn verified_relative_encoder_setup() {
        let mut device = Fake::measured();
        let before = device.memory.clone();
        configure(&mut device).unwrap();
        assert_eq!(device.writes, 16);
        let mut expected_memory = before;
        for index in 0..16 {
            let address = SLOT * STRIDE + ENCODERS + index * RECORD;
            expected_memory[address..address + RECORD].copy_from_slice(&expected(index));
        }
        assert_eq!(device.memory, expected_memory);
        configure(&mut device).unwrap();
        assert_eq!(device.writes, 16);
    }

    #[test]
    fn interrupted_setup_restores_original_records() {
        let mut device = Fake::measured();
        let before = device.memory.clone();
        device.fail_at = vec![2];
        let error = configure(&mut device).unwrap_err();
        assert!(error.contains("original records restored"), "{error}");
        assert_eq!(device.memory, before);
    }

    #[test]
    fn preset_change_during_setup_restores_original_records() {
        let mut device = Fake::measured();
        let before = device.memory.clone();
        device.switch_slot_at = Some(1);
        let error = configure(&mut device).unwrap_err();
        assert!(error.contains("original records restored"), "{error}");
        assert_eq!(device.writes, 17);
        assert_eq!(device.memory, before);
    }

    #[test]
    fn failed_restore_still_attempts_all_records() {
        let mut device = Fake::measured();
        let before = device.memory.clone();
        device.fail_at = vec![2, 3];
        let error = configure(&mut device).unwrap_err();
        assert!(error.contains("restore failed"), "{error}");
        assert!(error.contains("restore read-back differs"), "{error}");
        assert_eq!(device.writes, 18);
        assert_ne!(device.memory, before);
    }

    #[test]
    fn changed_preset_refuses_setup() {
        let mut device = Fake::measured();
        let before = device.memory.clone();
        device.state[10] = 5;
        assert!(configure(&mut device).is_err());
        assert_eq!(device.writes, 0);
        assert_eq!(device.memory, before);
    }

    /// Scenario "Preset change after setup disables relative steps" (input-devices).
    #[test]
    fn preset_change_after_setup_disables_relative_steps() {
        let mut device = Fake::measured();
        configure(&mut device).unwrap();
        checked(&mut device).unwrap();
        device.state[10] = 5;
        let message = "Preset 6 is active. Select preset 5 to use the knobs.";
        assert_eq!(checked(&mut device).unwrap_err(), message);
        assert_eq!(
            read_status(&mut device)
                .unwrap()
                .checked_pads()
                .unwrap_err(),
            message
        );
        device.state[10] = SLOT as u8;
        checked(&mut device).unwrap();
        let address = SLOT * STRIDE + ENCODERS;
        device.memory[address] = 0;
        assert!(checked(&mut device).is_err());
    }
}
