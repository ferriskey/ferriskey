//! Proves `AUTHZ_MATRIX` describes the source tree it claims to describe.
//!
//! The matrix is a specification of the authorization surface. A specification
//! nobody checks drifts, and a drifted authorization spec is worse than none —
//! it is a false sense of coverage. So this test re-derives the call sites from
//! the source and fails when they disagree.
//!
//! It compares `(file, service_fn, policy_fn)` rather than line numbers: line
//! numbers are carried in the rows as diagnostics, but asserting them would
//! break the test on any edit above a call site.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use ferriskey_authz::matrix::AUTHZ_MATRIX;

/// Directories holding authorization-guarded services.
const SCAN_DIRS: [&str; 3] = ["core/src", "libs", "api/src"];

fn repo_root() -> PathBuf {
    // CARGO_MANIFEST_DIR is <root>/libs/ferriskey-authz.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("manifest dir should sit two levels below the repo root")
        .to_path_buf()
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().map(|e| e.path()).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            // `tests` is skipped because integration tests are not part of the
            // authorization surface — and because this very file mentions
            // `ensure_policy(` while scanning for it.
            if name == "target" || name == "node_modules" || name == "tests" {
                continue;
            }
            rust_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

/// Line ranges covered by a `#[cfg(test)]` module.
///
/// Test modules define mock policies whose `can_*` methods are not part of the
/// authorization surface.
fn test_ranges(lines: &[&str]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if !line.contains("cfg(test)") {
            continue;
        }
        let (mut depth, mut started, mut start) = (0i32, false, 0usize);
        for (j, l) in lines.iter().enumerate().skip(i) {
            for ch in l.chars() {
                match ch {
                    '{' => {
                        depth += 1;
                        if !started {
                            started = true;
                            start = j;
                        }
                    }
                    '}' => depth -= 1,
                    _ => {}
                }
            }
            if started && depth <= 0 {
                ranges.push((start, j));
                break;
            }
        }
    }
    ranges
}

fn in_tests(ranges: &[(usize, usize)], idx: usize) -> bool {
    ranges.iter().any(|(a, b)| *a <= idx && idx <= *b)
}

/// The name in `fn <name>`, if the line declares one.
///
/// Handles the visibility and `async` prefixes the codebase uses:
/// `fn`, `pub fn`, `pub(crate) fn`, `async fn`, `pub async fn`.
fn declared_fn(line: &str) -> Option<&str> {
    let mut rest = line.trim_start();
    loop {
        if let Some(after) = rest.strip_prefix("pub") {
            let after = if after.starts_with('(') {
                &after[after.find(')')? + 1..]
            } else if after.starts_with(char::is_whitespace) {
                after
            } else {
                // An identifier merely starting with "pub".
                break;
            };
            rest = after.trim_start();
            continue;
        }
        if let Some(after) = rest.strip_prefix("async ") {
            rest = after.trim_start();
            continue;
        }
        break;
    }

    let rest = rest.strip_prefix("fn ")?;
    let name = rest
        .trim_start()
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .next()?;
    if name.is_empty() { None } else { Some(name) }
}

/// The first `.can_*(` in the slice, which is the policy the call site checks.
fn called_policy(window: &str) -> Option<String> {
    let idx = window.find(".can_")?;
    let rest = &window[idx + 1..];
    let end = rest.find('(')?;
    let name = rest[..end].trim();
    if name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        Some(name.to_string())
    } else {
        None
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct Site {
    file: String,
    service_fn: String,
    policy_fn: String,
}

fn scan_sources() -> Vec<Site> {
    let root = repo_root();
    let mut files = Vec::new();
    for dir in SCAN_DIRS {
        rust_files(&root.join(dir), &mut files);
    }

    let mut sites = Vec::new();
    for path in files {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let lines: Vec<&str> = text.lines().collect();
        let ranges = test_ranges(&lines);
        let rel = path
            .strip_prefix(&root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");

        for (i, line) in lines.iter().enumerate() {
            if !line.contains("ensure_policy(") || in_tests(&ranges, i) {
                continue;
            }
            let t = line.trim_start();
            if t.starts_with("//") || t.starts_with('*') {
                continue;
            }
            // The definition itself and the re-export shims are not call sites.
            if line.contains("pub fn ensure_policy") || line.contains("use ") {
                continue;
            }

            let end = (i + 8).min(lines.len());
            let window = lines[i..end].join("\n");
            let policy_fn = called_policy(&window)
                .unwrap_or_else(|| panic!("{rel}:{}: ensure_policy without a can_* call", i + 1));
            let service_fn = lines[..=i]
                .iter()
                .rev()
                .find_map(|l| declared_fn(l))
                .unwrap_or_else(|| panic!("{rel}:{}: no enclosing fn", i + 1))
                .to_string();

            sites.push(Site {
                file: rel.clone(),
                service_fn,
                policy_fn,
            });
        }
    }
    sites
}

fn tally(sites: impl IntoIterator<Item = Site>) -> HashMap<Site, usize> {
    let mut m = HashMap::new();
    for s in sites {
        *m.entry(s).or_insert(0) += 1;
    }
    m
}

#[test]
fn matrix_covers_every_call_site() {
    let found = tally(scan_sources());
    let declared = tally(AUTHZ_MATRIX.iter().map(|r| Site {
        file: r.file.to_string(),
        service_fn: r.service_fn.to_string(),
        policy_fn: r.policy_fn.to_string(),
    }));

    let mut missing: Vec<String> = Vec::new();
    let mut stale: Vec<String> = Vec::new();

    for (site, n) in &found {
        let d = declared.get(site).copied().unwrap_or(0);
        if d < *n {
            missing.push(format!(
                "{}  {} -> {}  (source {n}x, matrix {d}x)",
                site.file, site.service_fn, site.policy_fn
            ));
        }
    }
    for (site, n) in &declared {
        let f = found.get(site).copied().unwrap_or(0);
        if f < *n {
            stale.push(format!(
                "{}  {} -> {}  (matrix {n}x, source {f}x)",
                site.file, site.service_fn, site.policy_fn
            ));
        }
    }
    missing.sort();
    stale.sort();

    assert!(
        missing.is_empty() && stale.is_empty(),
        "AUTHZ_MATRIX is out of sync with the source tree.\n\n\
         Call sites missing from the matrix ({}):\n{}\n\n\
         Matrix rows with no call site ({}):\n{}\n\n\
         Every ensure_policy must have a row: the matrix is the authorization \
         specification, and an undocumented check is an unreviewed one.",
        missing.len(),
        missing.join("\n"),
        stale.len(),
        stale.join("\n"),
    );
}

#[test]
fn every_row_declares_permissions() {
    let empty: Vec<_> = AUTHZ_MATRIX
        .iter()
        .filter(|r| r.permissions.is_empty())
        .map(|r| format!("{}:{} {}", r.file, r.line, r.service_fn))
        .collect();

    assert!(
        empty.is_empty(),
        "rows with no permission set — the policy grants unconditionally, \
         or the extraction missed its slice:\n{}",
        empty.join("\n"),
    );
}

#[test]
fn actions_are_well_formed() {
    for row in AUTHZ_MATRIX {
        let name = row.action.as_str();
        let (resource, verb) = name
            .split_once(':')
            .unwrap_or_else(|| panic!("action {name:?} is not `resource:verb`"));
        assert!(
            !resource.is_empty() && !verb.is_empty(),
            "action {name:?} has an empty half",
        );
        assert!(
            resource
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
            "action {name:?} resource half must be snake_case",
        );
        assert!(
            verb.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
            "action {name:?} verb half must be snake_case",
        );
    }
}

#[test]
fn catalogue_matches_matrix() {
    for row in AUTHZ_MATRIX {
        assert!(
            ferriskey_authz::actions::by_name(row.action.as_str()).is_some(),
            "action {:?} is used by the matrix but missing from the catalogue",
            row.action.as_str(),
        );
    }
    for action in ferriskey_authz::actions::ALL {
        assert!(
            AUTHZ_MATRIX.iter().any(|r| r.action == *action),
            "action {:?} is in the catalogue but no call site uses it",
            action.as_str(),
        );
    }
}
