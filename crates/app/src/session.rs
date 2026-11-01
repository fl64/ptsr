//! The session: which file sits on which pad with what mode and volume, and the last folder.
//! `$XDG_DATA_HOME/ptsr/session.toml`; samples themselves are not copied.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Bumped when the format changes; an older file is migrated, a newer one is refused.
/// 2: a pad can hold a slice of its file (`start`, `end`); version 1 files read as whole files.
/// 3: the cuts of each chopped file (`chops`) and pads linked to them (`link`, `epoch`);
/// version 2 slices are turned into chops when read.
const VERSION: u32 = 3;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Session {
    pub version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub browser_dir: Option<PathBuf>,
    /// Pad index (0..128) as text, TOML keys are strings; pads at defaults are left out.
    #[serde(default)]
    pub pads: BTreeMap<String, PadState>,
    /// Master volume; absent in older sessions, which reads as 0 dB.
    #[serde(default)]
    pub master_db: i8,
    /// The cuts of each chopped file, by file path.
    #[serde(default)]
    pub chops: BTreeMap<String, ChopState>,
    /// The open bank, 0..8 (35); absent in older sessions, which reads as bank 1.
    #[serde(default)]
    pub bank: u8,
    /// The older chops of a file that pads still play slices of (45), by file path;
    /// absent in older sessions, which reads as none.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub kept: BTreeMap<String, Vec<ChopState>>,
    /// The two shared effect slots (56); absent in older sessions, which read as a reverb
    /// in FX 1 and a delay in FX 2.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fx: Vec<FxState>,
}

/// A shared effect slot in the project file (56): its kind and the parameters of each kind.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct FxState {
    /// `reverb` or `delay`.
    pub kind: String,
    pub size: u8,
    pub tone: u8,
    pub time_ms: u16,
    pub feedback: u8,
}

/// The cuts of a file, in seconds.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct ChopState {
    pub start: f64,
    pub end: f64,
    pub cuts: Vec<f64>,
    /// The id of each cut; pads link to these ids, the range start is 0.
    pub ids: Vec<u32>,
    pub epoch: u32,
    pub next_id: u32,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            version: VERSION,
            browser_dir: None,
            pads: BTreeMap::new(),
            master_db: 0,
            chops: BTreeMap::new(),
            bank: 0,
            kept: BTreeMap::new(),
            fx: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct PadState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<PathBuf>,
    pub gate: bool,
    pub volume_db: i8,
    /// Pad pitch in semitones; absent in older sessions, which reads as 0.
    pub pitch: i8,
    /// Pad envelope times in ms; absent in older sessions, which reads as zeros:
    /// the pad sounds as it did before the envelope existed.
    pub attack_ms: u16,
    pub decay_ms: u16,
    pub release_ms: u16,
    /// The pad filter (44): `low-pass`, `high-pass`, `band`; absent or empty is off, so
    /// older projects open unchanged. The cutoff is absent at its default of 1 kHz.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub filter: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cutoff_hz: Option<f32>,
    #[serde(skip_serializing_if = "is_zero")]
    pub reso: u8,
    /// The pad drive and sends in percent (56); absent in older sessions, which reads as
    /// 0: the pad sounds as before the effects.
    #[serde(skip_serializing_if = "is_zero")]
    pub drive: u8,
    #[serde(skip_serializing_if = "no_sends")]
    pub sends: [u8; 2],
    /// The mute group (70), 1–8; absent in older sessions, which reads as 0: off, the pad
    /// rings on as before.
    #[serde(skip_serializing_if = "is_zero")]
    pub group: u8,
    /// The slice the pad plays, in seconds of the file; absent means the whole file.
    /// Seconds rather than frames, so a slice survives a change of the audio rate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<f64>,
    /// Slice number shown after the file name; added without a version change, absent is none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slice: Option<u16>,
    /// The pad follows the cuts of its file: the cut id at its slice start and the chop epoch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epoch: Option<u32>,
}

fn is_zero(n: &u8) -> bool {
    *n == 0
}

fn no_sends(s: &[u8; 2]) -> bool {
    *s == [0, 0]
}

impl PadState {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

pub fn path() -> PathBuf {
    crate::config::xdg_dir("XDG_DATA_HOME", ".local/share").join("session.toml")
}

/// No file gives an empty session; a broken or newer file is an error and is not overwritten.
pub fn load(path: &Path) -> Result<Session, String> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Session::default()),
        Err(e) => return Err(format!("{}: {e}", path.display())),
    };
    let s: Session =
        toml::from_str(&text).map_err(|e| format!("{}: {}", path.display(), e.message()))?;
    if s.version > VERSION {
        return Err(format!(
            "{}: made by a newer version ({})",
            path.display(),
            s.version
        ));
    }
    let mut s = s;
    if s.version < 3 {
        chops_from_slices(&mut s);
    }
    Ok(s)
}

/// Version 2 kept only each pad's slice edges: the slices of one file become a chop with these
/// edges as cuts, and the pads are linked to it.
fn chops_from_slices(s: &mut Session) {
    let mut files: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for p in s.pads.values() {
        if let (Some(file), Some(_), Some(start), Some(end)) = (&p.file, p.slice, p.start, p.end) {
            let edges = files
                .entry(file.to_string_lossy().into_owned())
                .or_default();
            edges.extend([start, end]);
        }
    }
    for (file, mut edges) in files {
        edges.sort_by(f64::total_cmp);
        edges.dedup();
        let (Some(&start), Some(&end)) = (edges.first(), edges.last()) else {
            continue;
        };
        let cuts = edges[1..edges.len() - 1].to_vec();
        let ids: Vec<u32> = (1..=cuts.len() as u32).collect();
        for p in s.pads.values_mut() {
            let same = p.file.as_ref().is_some_and(|f| f.to_string_lossy() == file);
            if let (true, Some(_), Some(at)) = (same, p.slice, p.start) {
                p.link = Some(cuts.iter().position(|&c| c == at).map_or(0, |i| ids[i]));
                p.epoch = Some(1);
            }
        }
        let next_id = ids.len() as u32 + 1;
        s.chops.insert(
            file,
            ChopState {
                start,
                end,
                cuts,
                ids,
                epoch: 1,
                next_id,
            },
        );
    }
}

pub fn save(path: &Path, session: &Session) -> Result<(), String> {
    let text = toml::to_string(session).map_err(|e| e.to_string())?;
    let err = |e: std::io::Error| format!("{}: {e}", path.display());
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(err)?;
    }
    // via a temp file: if writing fails, the old session stays intact
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, text).map_err(err)?;
    std::fs::rename(&tmp, path).map_err(err)
}

/// (11) The path as stored in a project's `project.toml`: inside the project folder it
/// is relative to it, outside it stays absolute. Lexical only: `..` and symlinks are
/// not resolved.
pub fn to_stored_path(project: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(project)
        .map_or_else(|_| path.to_path_buf(), Path::to_path_buf)
}

/// (11) Back from the stored form: a relative path joins the project folder, an
/// absolute one stays as it is.
pub fn from_stored_path(project: &Path, stored: &Path) -> PathBuf {
    if stored.is_absolute() {
        stored.to_path_buf()
    } else {
        project.join(stored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_refuse_newer() {
        let dir = std::env::temp_dir().join(format!("ptsr-session-{}", std::process::id()));
        let path = dir.join("session.toml");
        assert_eq!(load(&path).unwrap(), Session::default());

        let mut s = Session {
            browser_dir: Some("/music/breaks".into()),
            master_db: -6,
            ..Default::default()
        };
        s.pads.insert(
            "21".into(),
            PadState {
                file: Some("/music/amen.wav".into()),
                gate: true,
                volume_db: -6,
                pitch: 0,
                attack_ms: 0,
                decay_ms: 0,
                release_ms: 0,
                filter: "band".into(),
                cutoff_hz: Some(2_000.0),
                reso: 40,
                drive: 30,
                sends: [40, 0],
                group: 2,
                start: Some(0.548),
                end: Some(0.809),
                slice: Some(3),
                link: Some(7),
                epoch: Some(2),
            },
        );
        s.chops.insert(
            "/music/amen.wav".into(),
            ChopState {
                start: 0.0,
                end: 2.61,
                cuts: vec![0.548, 0.809],
                ids: vec![3, 7],
                epoch: 2,
                next_id: 8,
            },
        );
        save(&path, &s).unwrap();
        assert_eq!(load(&path).unwrap(), s);

        // version 2 from 04a: three slices of one break, one whole file
        std::fs::write(
            &path,
            "version = 2\n\n[pads.0]\nfile = \"/b.wav\"\nstart = 0.0\nend = 0.5\nslice = 1\n\n[pads.1]\nfile = \"/b.wav\"\nstart = 0.5\nend = 1.0\nslice = 2\n\n[pads.7]\nfile = \"/b.wav\"\nstart = 1.0\nend = 2.0\nslice = 3\n\n[pads.9]\nfile = \"/k.wav\"\n",
        )
        .unwrap();
        let v2 = load(&path).unwrap();
        let chop = &v2.chops["/b.wav"];
        assert_eq!((chop.start, chop.end), (0.0, 2.0));
        assert_eq!(
            (chop.cuts.clone(), chop.ids.clone()),
            (vec![0.5, 1.0], vec![1, 2])
        );
        assert_eq!(v2.pads["0"].link, Some(0));
        assert_eq!(v2.pads["1"].link, Some(1));
        assert_eq!(v2.pads["7"].link, Some(2));
        assert_eq!(v2.pads["7"].epoch, Some(1));
        assert_eq!(v2.pads["9"].link, None, "a whole file stays free");
        assert!(!v2.chops.contains_key("/k.wav"));

        // a version 1 file, as 03 wrote it: still opens, pads play whole files
        std::fs::write(
            &path,
            "version = 1\nmaster_db = -3\n\n[pads.5]\nfile = \"/music/kick.wav\"\ngate = false\nvolume_db = -2\n",
        )
        .unwrap();
        let old = load(&path).unwrap();
        assert_eq!(old.master_db, -3);
        let pad = &old.pads["5"];
        assert_eq!((pad.start, pad.end, pad.volume_db), (None, None, -2));
        // spec: "Old session" — no envelope in the file, the pad sounds as it did
        assert_eq!((pad.attack_ms, pad.decay_ms, pad.release_ms), (0, 0, 0));

        std::fs::write(&path, "version = 99\n").unwrap();
        assert!(load(&path).unwrap_err().contains("newer"));
        std::fs::write(&path, "version = [\n").unwrap();
        assert!(load(&path).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn relative_paths_inside_the_folder() {
        let project = Path::new("/p/MyBeat");
        let stored = to_stored_path(project, Path::new("/p/MyBeat/samples/a.wav"));
        assert_eq!(stored, Path::new("samples/a.wav"));
        assert_eq!(
            from_stored_path(project, &stored),
            Path::new("/p/MyBeat/samples/a.wav")
        );
    }

    #[test]
    fn outside_stays_absolute_and_untouched() {
        let project = Path::new("/p/MyBeat");
        let stored = to_stored_path(project, Path::new("/music/amen.wav"));
        assert_eq!(stored, Path::new("/music/amen.wav"));
        assert_eq!(
            from_stored_path(project, &stored),
            Path::new("/music/amen.wav")
        );
    }

    #[test]
    fn moving_the_project() {
        let stored = to_stored_path(
            Path::new("/old/MyBeat"),
            Path::new("/old/MyBeat/samples/a.wav"),
        );
        assert_eq!(stored, Path::new("samples/a.wav"));
        assert_eq!(
            from_stored_path(Path::new("/new/MyBeat"), &stored),
            Path::new("/new/MyBeat/samples/a.wav")
        );
    }
}
