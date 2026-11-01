//! Config: defaults from `config/audio.toml` (working directory),
//! overlaid by the user file `$XDG_CONFIG_HOME/ptsr/config.toml`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::audio::Backend;

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub audio: AudioConfig,
    pub midi: Learned,
    pub browser: BrowserConfig,
    pub ui: UiConfig,
    /// How absolute knobs catch a value (58), saved under `[knobs]`.
    pub knobs: KnobsConfig,
    /// The devices whose profile roles are copied into `[midi]` (59), under `[devices]`.
    pub devices: DevicesConfig,
    /// The two roots the program works from (11).
    pub paths: PathsConfig,
    /// The last opened project's name (11), saved under `[project]`.
    pub project: ProjectConfig,
}

/// Where the projects live and where the files screen starts (11): both rows of the
/// settings Paths tab, saved under `[paths]`.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct PathsConfig {
    pub projects: PathBuf,
    pub samples: PathBuf,
}

/// The last opened project (11): the start reopens it; `last` empty — the start is
/// scratch.
#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct ProjectConfig {
    pub last: Option<String>,
}

impl Default for PathsConfig {
    fn default() -> Self {
        Self {
            projects: expand_home("~/ptsr/projects".into()),
            samples: expand_home("~/ptsr/samples".into()),
        }
    }
}

/// `~/…` at the start becomes the home folder; without `HOME` the path stays as written.
fn expand_home(path: PathBuf) -> PathBuf {
    let Some(rest) = path.to_str().and_then(|p| p.strip_prefix("~/")) else {
        return path;
    };
    match std::env::var_os("HOME") {
        Some(home) if !home.is_empty() => PathBuf::from(home).join(rest),
        _ => path,
    }
}

/// How the screens draw (04b): the waves as kitty pictures instead of Braille, from the
/// next start; off by default. `hide_pads` (32): the pad grid is hidden, the wave takes its rows.
#[derive(Deserialize, Serialize, Default, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct UiConfig {
    pub graphics: bool,
    pub hide_pads: bool,
}

/// `seeded`: the profile names whose roles were copied into `[midi]` once (59); from then
/// on only `[midi]` gives them actions.
#[derive(Deserialize, Serialize, Default, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct DevicesConfig {
    pub seeded: Vec<String>,
}

/// `response`: `jump`, `pickup` or `scale` (58); absent or unknown reads as `jump`.
#[derive(Deserialize, Serialize, Default, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct KnobsConfig {
    pub response: String,
}

#[derive(Deserialize, Serialize, Default, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct BrowserConfig {
    pub favorites: Vec<PathBuf>,
}

/// MIDI learn results: profile name -> screen -> control ("cc 24") -> action ("pad.volume").
pub type Learned = BTreeMap<String, BTreeMap<String, BTreeMap<String, String>>>;

#[derive(Deserialize, Serialize, Clone, PartialEq, Debug)]
#[serde(default)]
pub struct AudioConfig {
    pub backend: Backend,
    pub buffer_size: u32,
    /// Output device name (JACK port prefix); unset means the first available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            backend: Backend::Auto,
            // 0 = auto: take the sound server's size, never reconfigure the shared graph
            buffer_size: 0,
            output: None,
            input: None,
        }
    }
}

const DEFAULTS: &str = "config/audio.toml";

pub struct Loaded {
    pub config: Config,
    /// User file where choices are saved; `None` means saving is off (the file is broken).
    pub user_path: Option<PathBuf>,
    pub error: Option<String>,
}

pub fn user_path() -> PathBuf {
    xdg_dir("XDG_CONFIG_HOME", ".config").join("config.toml")
}

pub fn load() -> Loaded {
    load_from(&user_path())
}

/// The same load with the user file given: tests read it back without touching the
/// process environment.
pub fn load_from(user: &Path) -> Loaded {
    let mut error = None;
    let mut table = read_table(Path::new(DEFAULTS)).unwrap_or_else(|e| {
        error = Some(e);
        toml::Table::new()
    });
    let user_ok = match read_table(user) {
        Ok(t) => {
            merge(&mut table, t);
            true
        }
        Err(e) => {
            error = Some(format!("{e}; settings will not be saved"));
            false
        }
    };
    let config = toml::Value::Table(table).try_into().unwrap_or_else(|e| {
        error = Some(format!("bad config: {e}"));
        Config::default()
    });
    if let Some(e) = &error {
        log::error!("{e}");
    }
    Loaded {
        config,
        user_path: user_ok.then(|| user.to_path_buf()),
        error,
    }
}

/// Writes one section (`audio`, `midi`) to the user file and leaves the rest untouched.
pub fn save_section(path: &Path, name: &str, section: &impl Serialize) -> Result<(), String> {
    let mut table = read_table(path)?;
    let value = toml::Value::try_from(section).map_err(|e| e.to_string())?;
    table.insert(name.into(), value);
    let text = toml::to_string(&table).map_err(|e| e.to_string())?;
    let err = |e: std::io::Error| format!("{}: {e}", path.display());
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(err)?;
    }
    // via a temp file: if writing fails, the old config stays intact
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, text).map_err(err)?;
    std::fs::rename(&tmp, path).map_err(err)
}

/// No file gives an empty table; an unreadable file is an error.
fn read_table(path: &Path) -> Result<toml::Table, String> {
    match std::fs::read_to_string(path) {
        Ok(text) => text
            .parse()
            .map_err(|e: toml::de::Error| format!("{}: {}", path.display(), e.message())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(toml::Table::new()),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

fn merge(base: &mut toml::Table, over: toml::Table) {
    for (k, v) in over {
        match (base.get_mut(&k), v) {
            (Some(toml::Value::Table(b)), toml::Value::Table(o)) => merge(b, o),
            (_, v) => {
                base.insert(k, v);
            }
        }
    }
}

/// `$VAR/ptsr`, otherwise `~/<fallback>/ptsr`.
pub fn xdg_dir(var: &str, fallback: &str) -> PathBuf {
    xdg_base(var, fallback).join("ptsr")
}

/// `$VAR`, otherwise `~/<fallback>`: the folder that holds the program's folder.
pub fn xdg_base(var: &str, fallback: &str) -> PathBuf {
    std::env::var_os(var)
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(fallback)))
        .unwrap_or_else(std::env::temp_dir)
}

/// For each base, copies the folder of the working name `arasaka-404` into `ptsr` when
/// only the old folder exists (12); the old folders stay as they are. Gives the first
/// old folder that was copied.
pub fn move_old_folders(bases: &[PathBuf]) -> Result<Option<PathBuf>, String> {
    let mut moved = None;
    for base in bases {
        let (old, new) = (base.join("arasaka-404"), base.join("ptsr"));
        if new.exists() || !old.is_dir() {
            continue;
        }
        // via a side folder: a copy cut short leaves no half folder that wins the next start
        let part = base.join("ptsr.part");
        let _ = std::fs::remove_dir_all(&part);
        copy_dir(&old, &part)
            .and_then(|()| std::fs::rename(&part, &new))
            .map_err(|e| format!("cannot copy {} to {}: {e}", old.display(), new.display()))?;
        moved.get_or_insert(old);
    }
    Ok(moved)
}

fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let (src, dst) = (entry.path(), to.join(entry.file_name()));
        if src.is_dir() {
            copy_dir(&src, &dst)?;
        } else {
            std::fs::copy(&src, &dst)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_config_follows_the_server_buffer() {
        let c: Config = toml::from_str("").unwrap();
        assert_eq!(c.audio.buffer_size, 0, "auto by default");
        let c: Config = toml::from_str("[audio]\nbuffer_size = 0\n").unwrap();
        assert_eq!(c.audio.buffer_size, 0, "zero round-trips");
    }

    #[test]
    fn user_overrides_defaults_per_key() {
        let mut base: toml::Table = "[audio]\nbackend = \"jack\"\nbuffer_size = 128\n"
            .parse()
            .unwrap();
        merge(
            &mut base,
            "[audio]\noutput = \"Speaker\"\n".parse().unwrap(),
        );
        let c: Config = toml::Value::Table(base).try_into().unwrap();
        assert_eq!(c.audio.backend, Backend::Jack);
        assert_eq!(c.audio.buffer_size, 128);
        assert_eq!(c.audio.output.as_deref(), Some("Speaker"));
    }

    #[test]
    fn save_keeps_other_sections_and_refuses_broken_file() {
        let dir = std::env::temp_dir().join(format!("ptsr-cfg-{}", std::process::id()));
        let path = dir.join("config.toml");
        std::fs::create_dir_all(&dir).unwrap();

        std::fs::write(&path, "[other]\nx = 1\n").unwrap();
        let audio = AudioConfig {
            output: Some("Speaker".into()),
            ..Default::default()
        };
        save_section(&path, "audio", &audio).unwrap();
        let mut learned = Learned::new();
        learned
            .entry("X-Session Pro".into())
            .or_default()
            .entry("pads".into())
            .or_default()
            .insert("cc 31".into(), "pad.volume".into());
        save_section(&path, "midi", &learned).unwrap();
        let t = read_table(&path).unwrap();
        assert_eq!(t["other"]["x"].as_integer(), Some(1));
        assert_eq!(t["audio"]["output"].as_str(), Some("Speaker"));
        let c: Config = toml::Value::Table(t).try_into().unwrap();
        assert_eq!(c.midi["X-Session Pro"]["pads"]["cc 31"], "pad.volume");

        std::fs::write(&path, "[audio\n").unwrap();
        assert!(save_section(&path, "audio", &audio).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "[audio\n");

        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// The last project's name round-trips through `[project]` (11); a file without the
    /// section reads as no project ever opened.
    #[test]
    fn last_project_round_trips() {
        let c: Config = toml::from_str("").unwrap();
        assert_eq!(c.project, ProjectConfig { last: None });
        let saved = ProjectConfig {
            last: Some("MyBeat".into()),
        };
        let text = toml::to_string(&saved).unwrap();
        let back: ProjectConfig = toml::from_str(&text).unwrap();
        assert_eq!(back, saved);
        let c: Config = toml::from_str("[project]\nlast = \"MyBeat\"\n").unwrap();
        assert_eq!(c.project.last.as_deref(), Some("MyBeat"));
    }

    /// The roots default to `~/ptsr/…` (11); a saved `[paths]` section overrides them.
    #[test]
    fn the_roots_default_under_home_and_round_trip() {
        let c: Config = toml::from_str("").unwrap();
        if let Some(home) = std::env::var_os("HOME").filter(|h| !h.is_empty()) {
            let home = PathBuf::from(home);
            assert_eq!(c.paths.projects, home.join("ptsr/projects"));
            assert_eq!(c.paths.samples, home.join("ptsr/samples"));
        }
        let c: Config = toml::from_str("[paths]\nprojects = \"/beats\"\n").unwrap();
        assert_eq!(c.paths.projects, PathBuf::from("/beats"));
    }

    /// Scenario "User file with an error" (audio-settings): a broken user file gives the
    /// default values and an error, the app gets no path to save to, and the file stays.
    #[test]
    fn user_file_with_an_error() {
        let dir = std::env::temp_dir().join(format!("ptsr-badcfg-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");
        std::fs::write(&path, "[audio\nbuffer_size = 256\n").unwrap();
        let loaded = load_from(&path);
        assert_eq!(
            loaded.config.audio.buffer_size,
            AudioConfig::default().buffer_size
        );
        assert!(
            loaded
                .error
                .as_deref()
                .is_some_and(|e| e.contains("settings will not be saved")),
            "{:?}",
            loaded.error
        );
        assert_eq!(loaded.user_path, None, "nothing writes the file");
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "[audio\nbuffer_size = 256\n"
        );
        std::fs::remove_dir_all(&dir).ok();
    }
}
