#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    const MANUAL_RESOLUTION: &[&str] = &[
        "core/src/application/services.rs",
        "core/src/domain/authentication/services.rs",
        "core/src/domain/common/services.rs",
        "core/src/domain/realm/services.rs",
        "core/src/domain/saml/services.rs",
        "core/src/domain/trident/services.rs",
        "libs/ferriskey-mail/src/email_template/services.rs",
        "libs/ferriskey-mail/src/email_verification/services.rs",
        "libs/ferriskey-portal-layouts/src/services.rs",
        "libs/ferriskey-portal-theme/src/services.rs",
        "libs/ferriskey-seawatch/src/services.rs",
        "libs/ferriskey-webhook/src/services.rs",
    ];

    const MANUAL_COMPARISON: &[&str] = &[
        "core/src/application/services.rs",
        "core/src/domain/authentication/services.rs",
        "core/src/domain/saml/services.rs",
        "core/src/domain/trident/services.rs",
    ];

    const SANCTIONED: &[&str] = &[
        "libs/ferriskey-domain/src/realm/scope.rs",
        "core/src/infrastructure/realm/repositories/realm_postgres_repository.rs",
    ];

    fn repository_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("the api crate always sits one level below the repository root")
            .to_path_buf()
    }

    fn rust_sources(root: &Path) -> Vec<PathBuf> {
        let mut roots = vec![root.join("core/src")];

        let libs = root.join("libs");
        let mut crates: Vec<PathBuf> = fs::read_dir(&libs)
            .expect("libs must be readable")
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.is_dir())
            .map(|path| path.join("src"))
            .filter(|path| path.is_dir())
            .collect();
        crates.sort();
        roots.append(&mut crates);

        let mut sources = Vec::new();
        for directory in roots {
            collect_rust_sources(&directory, &mut sources);
        }
        sources.sort();
        sources
    }

    fn collect_rust_sources(directory: &Path, sources: &mut Vec<PathBuf>) {
        let entries = match fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(_) => return,
        };

        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                collect_rust_sources(&path, sources);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                sources.push(path);
            }
        }
    }

    fn relative(root: &Path, path: &Path) -> String {
        path.strip_prefix(root)
            .expect("every scanned source lives under the repository root")
            .to_string_lossy()
            .replace('\\', "/")
    }

    fn resolves_a_realm_by_hand(source: &str) -> bool {
        source.contains("realm_repository") && source.contains(".get_by_name(")
    }

    fn compares_a_realm_by_hand(source: &str) -> bool {
        source.contains("!= realm.id") || source.contains("realm_id != ")
    }

    fn offenders(predicate: fn(&str) -> bool) -> Vec<String> {
        let root = repository_root();

        let mut found: Vec<String> = rust_sources(&root)
            .into_iter()
            .filter_map(|path| {
                let source = fs::read_to_string(&path).ok()?;
                predicate(&source).then(|| relative(&root, &path))
            })
            .filter(|path| !SANCTIONED.contains(&path.as_str()))
            .collect();

        found.sort();
        found
    }

    fn assert_frozen(kind: &str, frozen: &[&str], found: Vec<String>) {
        let expected: Vec<String> = frozen.iter().map(|path| (*path).to_owned()).collect();

        let added: Vec<&String> = found
            .iter()
            .filter(|path| !expected.contains(path))
            .collect();
        let cleaned: Vec<&String> = expected
            .iter()
            .filter(|path| !found.contains(path))
            .collect();

        assert!(
            added.is_empty(),
            "{kind}: these files were added to the list of sites that still carry the realm rule by hand. \
             Use RealmScope and Unscoped::in_realm instead, or add the path to the frozen list with a reason in the pull request: {added:?}"
        );

        assert!(
            cleaned.is_empty(),
            "{kind}: these files no longer carry the realm rule by hand — remove them from the frozen list in this test so the progress is visible in the diff: {cleaned:?}"
        );
    }

    #[test]
    fn no_new_site_resolves_a_realm_by_hand() {
        assert_frozen(
            "manual realm resolution",
            MANUAL_RESOLUTION,
            offenders(resolves_a_realm_by_hand),
        );
    }

    #[test]
    fn no_new_site_compares_a_realm_by_hand() {
        assert_frozen(
            "manual realm comparison",
            MANUAL_COMPARISON,
            offenders(compares_a_realm_by_hand),
        );
    }

    #[test]
    fn the_sanctioned_implementations_are_the_only_exemptions() {
        let root = repository_root();

        for path in SANCTIONED {
            assert!(
                root.join(path).is_file(),
                "the sanctioned exemption {path} no longer exists — the ratchet would silently stop covering it"
            );
        }
    }
}
