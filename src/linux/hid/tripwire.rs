use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
struct TripwireHit {
    path: PathBuf,
    detail: String,
}

const FORBIDDEN_CONTENT_MARKERS: &[&str] = &[
    "write-once",
    "confirm-hid-write",
    "SetFeature",
    "GetFeature",
    "HidD_SetFeature",
    "HidD_GetFeature",
    "open_path",
    "/dev/hidraw",
];

const FORBIDDEN_FILE_NAMES: &[&str] = &["write_once.rs", "device.rs"];
const SELF_TRIPWIRE_PATH: &str = "src/linux/hid/tripwire.rs";

fn scan_ms7e75_hid_tripwire_targets(repo_root: &Path) -> Result<Vec<TripwireHit>, String> {
    let mut hits = Vec::new();

    scan_src_tree(&repo_root.join("src"), &mut hits)?;
    scan_text_file(&repo_root.join("Cargo.toml"), &mut hits)?;

    Ok(hits)
}

fn scan_src_tree(src_root: &Path, hits: &mut Vec<TripwireHit>) -> Result<(), String> {
    let entries = fs::read_dir(src_root)
        .map_err(|err| format!("failed to read {}: {err}", src_root.display()))?;

    for entry in entries {
        let entry = entry.map_err(|err| format!("failed to read src entry: {err}"))?;
        let path = entry.path();
        let metadata = entry
            .metadata()
            .map_err(|err| format!("failed to stat {}: {err}", path.display()))?;

        if metadata.is_dir() {
            scan_src_tree(&path, hits)?;
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }

        if path.ends_with(Path::new(SELF_TRIPWIRE_PATH)) {
            continue;
        }

        if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
            if FORBIDDEN_FILE_NAMES.contains(&file_name) {
                hits.push(TripwireHit {
                    path: path.clone(),
                    detail: format!("forbidden file name `{file_name}`"),
                });
            }
        }

        scan_text_file(&path, hits)?;
    }

    Ok(())
}

fn scan_text_file(path: &Path, hits: &mut Vec<TripwireHit>) -> Result<(), String> {
    let contents = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;

    let mut ranges: Vec<(usize, usize)> = Vec::new();
    let mut markers = FORBIDDEN_CONTENT_MARKERS.to_vec();
    markers.sort_by_key(|marker| std::cmp::Reverse(marker.len()));

    for marker in markers {
        for (offset, _) in contents.match_indices(marker) {
            let end = offset + marker.len();
            if ranges.iter().any(|(existing_start, existing_end)| {
                offset < *existing_end && end > *existing_start
            }) {
                continue;
            }

            ranges.push((offset, end));
            hits.push(TripwireHit {
                path: path.to_path_buf(),
                detail: format!("forbidden marker `{marker}`"),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{FORBIDDEN_CONTENT_MARKERS, scan_ms7e75_hid_tripwire_targets};

    #[test]
    fn ms7e75_hid_source_tree_contains_no_phase4_write_markers() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let hits = scan_ms7e75_hid_tripwire_targets(repo_root).unwrap();

        assert!(
            hits.is_empty(),
            "MS-7E75 HID safety tripwire hit(s):\n{}",
            format_hits(&hits)
        );
    }

    #[test]
    fn tripwire_ignores_docs_and_flags_rust_sources_and_cargo_toml() {
        let fixture_root = create_fixture_root("tripwire_fixture");
        let docs_dir = fixture_root.join("docs");
        let src_dir = fixture_root.join("src").join("linux").join("hid");

        fs::create_dir_all(&docs_dir).unwrap();
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(
            fixture_root.join("Cargo.toml"),
            "[package]\nname = \"fixture\"\n",
        )
        .unwrap();
        fs::write(docs_dir.join("notes.md"), "future flag confirm-hid-write").unwrap();
        fs::write(src_dir.join("report.rs"), "pub fn dry_run() {}\n").unwrap();

        let hits = scan_ms7e75_hid_tripwire_targets(&fixture_root).unwrap();

        assert!(hits.is_empty());
    }

    #[test]
    fn tripwire_flags_forbidden_markers_in_rust_files() {
        let fixture_root = create_fixture_root("tripwire_markers");
        let src_dir = fixture_root.join("src").join("linux").join("hid");

        fs::create_dir_all(&src_dir).unwrap();
        fs::write(
            fixture_root.join("Cargo.toml"),
            "[package]\nname = \"fixture\"\n",
        )
        .unwrap();
        fs::write(
            src_dir.join("report.rs"),
            "const BAD: &str = \"SetFeature\";\n",
        )
        .unwrap();

        let hits = scan_ms7e75_hid_tripwire_targets(&fixture_root).unwrap();

        assert_eq!(hits.len(), 1);
        assert!(hits[0].detail.contains(FORBIDDEN_CONTENT_MARKERS[2]));
    }

    #[test]
    fn tripwire_flags_forbidden_file_names() {
        let fixture_root = create_fixture_root("tripwire_filenames");
        let src_dir = fixture_root.join("src").join("linux").join("hid");

        fs::create_dir_all(&src_dir).unwrap();
        fs::write(
            fixture_root.join("Cargo.toml"),
            "[package]\nname = \"fixture\"\n",
        )
        .unwrap();
        fs::write(src_dir.join("device.rs"), "pub struct Placeholder;\n").unwrap();

        let hits = scan_ms7e75_hid_tripwire_targets(&fixture_root).unwrap();

        assert_eq!(hits.len(), 1);
        assert!(hits[0].detail.contains("forbidden file name"));
    }

    #[test]
    fn tripwire_flags_forbidden_markers_in_cargo_toml() {
        let fixture_root = create_fixture_root("tripwire_cargo");
        let src_dir = fixture_root.join("src");

        fs::create_dir_all(&src_dir).unwrap();
        fs::write(
            fixture_root.join("Cargo.toml"),
            "[package]\nname = \"fixture\"\n# HidD_SetFeature\n",
        )
        .unwrap();
        fs::write(src_dir.join("main.rs"), "fn main() {}\n").unwrap();

        let hits = scan_ms7e75_hid_tripwire_targets(&fixture_root).unwrap();

        assert_eq!(hits.len(), 1);
        assert!(hits[0].detail.contains("HidD_SetFeature"));
    }

    fn format_hits(hits: &[super::TripwireHit]) -> String {
        hits.iter()
            .map(|hit| format!("- {}: {}", hit.path.display(), hit.detail))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn create_fixture_root(prefix: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = env::temp_dir().join(format!("ms7e75_{prefix}_{unique}"));
        fs::create_dir_all(&root).unwrap();
        root
    }
}
