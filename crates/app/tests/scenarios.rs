//! Every scenario of the main specs is named by a test, or listed with its reason in
//! `openspec/untested.md` (AGENTS.md, "Tests").

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn files(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    for e in fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            files(&p, ext, out);
        } else if p.extension().is_some_and(|x| x == ext) {
            out.push(p);
        }
    }
}

fn snake(s: &str) -> String {
    let s: String = s
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    s.split('_')
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

/// (spec, scenario) of every main spec.
fn scenarios() -> Vec<(String, String)> {
    let mut specs = Vec::new();
    files(&root().join("openspec/specs"), "md", &mut specs);
    let mut out = Vec::new();
    for f in specs {
        let spec = f
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        for l in fs::read_to_string(&f).unwrap().lines() {
            if let Some(name) = l.strip_prefix("#### Scenario: ") {
                out.push((spec.clone(), name.trim().to_string()));
            }
        }
    }
    out
}

/// The names in `openspec/untested.md`: the text in the first backticks of each list line.
fn untested() -> BTreeSet<String> {
    fs::read_to_string(root().join("openspec/untested.md"))
        .unwrap()
        .lines()
        .filter_map(|l| l.strip_prefix("- `"))
        .filter_map(|l| l.split_once('`'))
        .map(|(name, _)| name.to_string())
        .collect()
}

#[test]
fn every_scenario_names_its_test() {
    let mut sources = Vec::new();
    files(&root().join("crates"), "rs", &mut sources);
    let code: String = sources
        .iter()
        .map(|f| fs::read_to_string(f).unwrap().to_lowercase())
        .collect();
    let named = |s: &str| code.contains(&s.to_lowercase()) || code.contains(&snake(s));
    let skip = untested();
    let all = scenarios();
    assert!(all.len() > 400, "the specs were read: {}", all.len());

    let missing: Vec<String> = all
        .iter()
        .filter(|(_, s)| !named(s) && !skip.contains(s))
        .map(|(spec, s)| format!("{s} ({spec})"))
        .collect();
    assert!(
        missing.is_empty(),
        "scenarios with no test: name a test after each, or list it with a reason in \
         openspec/untested.md:\n  {}",
        missing.join("\n  ")
    );

    let names: BTreeSet<&str> = all.iter().map(|(_, s)| s.as_str()).collect();
    let stale: Vec<&String> = skip
        .iter()
        .filter(|s| !names.contains(s.as_str()) || named(s))
        .collect();
    assert!(
        stale.is_empty(),
        "openspec/untested.md lists scenarios that are gone or have a test now: {stale:?}"
    );
}
