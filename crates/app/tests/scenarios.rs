//! Main scenarios have a named test or a reason in `openspec/untested.md`.
//! Active deltas require a test name or an attached scenario description on a test.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

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

/// The capability and scenario names below one specification directory.
fn scenarios(directory: &Path) -> Vec<(String, String)> {
    let mut specs = Vec::new();
    files(directory, "md", &mut specs);
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

/// Read the project's plain test declarations, not arbitrary mentions in helper bodies.
fn test_names(code: &str) -> Vec<(String, String)> {
    let mut header = Vec::new();
    let mut out = Vec::new();
    for line in code.lines().map(str::trim) {
        if line.is_empty() || line.starts_with("///") || line.starts_with("#[") {
            header.push(line);
            continue;
        }
        if header.contains(&"#[test]")
            && let Some((name, _)) = line
                .strip_prefix("fn ")
                .and_then(|line| line.split_once('('))
        {
            out.push((name.trim().into(), header.join("\n").to_lowercase()));
        }
        header.clear();
    }
    out
}

fn project_tests() -> &'static BTreeSet<(String, String)> {
    static TESTS: OnceLock<BTreeSet<(String, String)>> = OnceLock::new();
    TESTS.get_or_init(|| {
        let mut sources = Vec::new();
        files(&root().join("crates"), "rs", &mut sources);
        sources
            .iter()
            .flat_map(|path| test_names(&fs::read_to_string(path).unwrap()))
            .collect()
    })
}

fn named_test(scenario: &str) -> bool {
    let name = snake(scenario);
    let description = scenario.to_lowercase();
    project_tests()
        .iter()
        .any(|(test, header)| test.contains(&name) || header.contains(&description))
}

#[test]
fn active_delta_scenarios_name_real_tests() {
    let mut missing = Vec::new();
    for change in fs::read_dir(root().join("openspec/changes"))
        .unwrap()
        .map(Result::unwrap)
    {
        if change.file_name() == "archive" {
            continue;
        }
        let directory = change.path().join("specs");
        if !directory.is_dir() {
            continue;
        }
        for (spec, scenario) in scenarios(&directory) {
            if !named_test(&scenario) {
                missing.push(format!(
                    "{scenario} ({}/{spec})",
                    change.file_name().to_string_lossy()
                ));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "active scenarios need a named test or an attached scenario description:\n  {}",
        missing.join("\n  ")
    );
}

#[test]
fn test_name_audit_requires_a_real_test_header() {
    let source = concat!(
        "/// A scenario named by a helper is not coverage.\n",
        "fn helper() {}\n",
        "/// A named scenario\n",
        "#[test]\n",
        "#[allow(unused)]\n",
        "fn real_test() {}\n",
        "/// An orphan description\n",
        "const NOT_A_TEST: &str = \"fn another()\";\n",
        "fn another_helper() {}\n",
    );
    assert_eq!(
        test_names(source),
        vec![(
            "real_test".into(),
            "/// a named scenario\n#[test]\n#[allow(unused)]".into()
        )]
    );
}

#[test]
fn every_scenario_names_its_test() {
    let skip = untested();
    let all = scenarios(&root().join("openspec/specs"));
    assert!(all.len() > 400, "the specs were read: {}", all.len());

    let missing: Vec<String> = all
        .iter()
        .filter(|(_, s)| !named_test(s) && !skip.contains(s))
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
        .filter(|s| !names.contains(s.as_str()) || named_test(s))
        .collect();
    assert!(
        stale.is_empty(),
        "openspec/untested.md lists scenarios that are gone or have a test now: {stale:?}"
    );
}
