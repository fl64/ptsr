//! Send a recorded SMC-PAD script through a virtual ALSA MIDI port.

use std::time::Duration;

use midir::MidiOutput;
use midir::os::unix::VirtualOutput;

enum Command {
    Wait(Duration),
    Send(Vec<u8>),
}

fn number(value: Option<&str>, label: &str, min: u8, max: u8) -> Result<u8, String> {
    let value = value.ok_or_else(|| format!("missing {label}"))?;
    let parsed = value
        .parse::<u8>()
        .map_err(|_| format!("bad {label}: {value}"))?;
    if (min..=max).contains(&parsed) {
        Ok(parsed)
    } else {
        Err(format!("{label} must be {min}..{max}"))
    }
}

fn parse(line: &str) -> Result<Option<Command>, String> {
    let body = line.split('#').next().unwrap_or("").trim();
    if body.is_empty() {
        return Ok(None);
    }
    let mut parts = body.split_whitespace();
    let kind = parts.next().unwrap();
    let command = match kind {
        "wait" => {
            let value = parts.next().ok_or("missing wait duration")?;
            let ms = value
                .strip_suffix("ms")
                .ok_or("wait duration must end in ms")?
                .parse::<u64>()
                .map_err(|_| format!("bad wait duration: {value}"))?;
            Command::Wait(Duration::from_millis(ms))
        }
        "cc" | "note" | "noteoff" | "pressure" => {
            let channel = number(parts.next(), "channel", 1, 16)? - 1;
            let data = if kind == "pressure" {
                vec![0xd0 | channel, number(parts.next(), "value", 0, 127)?]
            } else {
                let key = number(parts.next(), "number", 0, 127)?;
                match kind {
                    "cc" => vec![0xb0 | channel, key, number(parts.next(), "value", 0, 127)?],
                    "note" => vec![
                        0x90 | channel,
                        key,
                        number(parts.next(), "velocity", 1, 127)?,
                    ],
                    _ => vec![0x80 | channel, key, 64],
                }
            };
            Command::Send(data)
        }
        _ => return Err(format!("unknown command: {kind}")),
    };
    if parts.next().is_some() {
        return Err("extra words on line".into());
    }
    Ok(Some(command))
}

fn run() -> Result<(), String> {
    if std::env::var("PTSR_DEV_VIRTUAL").as_deref() != Ok("1") {
        return Err("set PTSR_DEV_VIRTUAL=1 for both ptsr and midi_replay".into());
    }
    let mut args = std::env::args().skip(1);
    let path = args.next().ok_or("usage: midi_replay <script.txt>")?;
    if args.next().is_some() {
        return Err("usage: midi_replay <script.txt>".into());
    }
    let source = std::fs::read_to_string(&path).map_err(|e| format!("{path}: {e}"))?;
    let commands: Vec<_> = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            parse(line)
                .transpose()
                .map(|result| result.map_err(|e| format!("{path}:{}: {e}", index + 1)))
        })
        .collect::<Result<_, _>>()?;
    let output = MidiOutput::new("ptsr MIDI replay").map_err(|e| e.to_string())?;
    let mut connection = output
        .create_virtual("SMC-PAD-Master (virtual)")
        .map_err(|e| e.to_string())?;
    println!("SMC-PAD-Master (virtual) ready; waiting for ptsr scan");
    std::thread::sleep(Duration::from_millis(1500));
    let mut sent = 0;
    for command in commands {
        match command {
            Command::Wait(duration) => std::thread::sleep(duration),
            Command::Send(bytes) => {
                connection.send(&bytes).map_err(|e| e.to_string())?;
                sent += 1;
            }
        }
    }
    println!("sent {sent} MIDI messages");
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_recorded_pad_and_button_bytes() {
        assert!(
            matches!(parse("wait 200ms").unwrap(), Some(Command::Wait(d)) if d == Duration::from_millis(200))
        );
        assert!(
            matches!(parse("note 10 48 100").unwrap(), Some(Command::Send(b)) if b == [0x99, 48, 100])
        );
        assert!(
            matches!(parse("noteoff 10 48").unwrap(), Some(Command::Send(b)) if b == [0x89, 48, 64])
        );
        assert!(
            matches!(parse("cc 1 27 127").unwrap(), Some(Command::Send(b)) if b == [0xb0, 27, 127])
        );
        assert!(
            matches!(parse("pressure 10 80").unwrap(), Some(Command::Send(b)) if b == [0xd9, 80])
        );
        assert!(parse("note 17 48 100").is_err());
    }

    #[test]
    fn shipped_smc_script_parses() {
        for (line, source) in include_str!("../../../../tests/midi/smc-profile.txt")
            .lines()
            .enumerate()
        {
            assert!(parse(source).is_ok(), "line {}: {source}", line + 1);
        }
    }
}
