use std::fs;

use dirpack::budget::BudgetTarget;
use dirpack::config::Config;
use dirpack::packer::{pack, TruncationInfo};
use tempfile::TempDir;

#[test]
fn indicator_reports_tree_and_signature_truncation_separately() {
    let info = TruncationInfo {
        files_scanned: 10,
        files_in_tree: 8,
        files_with_signatures: 2,
        signature_files_candidate: 6,
        signature_files_processed: 4,
        signature_files_partial: 1,
        dirs_truncated: 0,
    };

    assert!(info.has_truncation());
    assert!(info.signatures_truncated());
    assert_eq!(
        info.format_indicator().as_deref(),
        Some("[+2 more files truncated; signatures truncated: 2 files omitted, 1 partial]")
    );
}

#[test]
fn pack_marks_silent_signature_budget_cuts() {
    let root = TempDir::new().expect("tempdir");
    for file_name in ["alpha.ts", "beta.ts"] {
        let mut source = String::new();
        for index in 0..20 {
            source.push_str(&format!(
                "export function {file_name_without_ext}_{index}(value: string): string {{ return value; }}\n",
                file_name_without_ext = file_name.trim_end_matches(".ts"),
            ));
        }
        fs::write(root.path().join(file_name), source).expect("write fixture");
    }

    let mut config = Config::default();
    config.content.enabled = false;
    let result = pack(
        root.path(),
        &config,
        BudgetTarget::Tokens(200),
        false,
        true,
        Some("."),
    );

    assert_eq!(result.truncation.signature_files_candidate, 2);
    assert!(result.truncation.signatures_truncated());
    assert!(
        result.truncation.signature_files_processed < 2
            || result.truncation.signature_files_partial > 0
    );
    assert!(result.output.contains("signatures truncated:"));
}

#[test]
fn nonsignable_top_dirs_do_not_reduce_signature_share() {
    let root = TempDir::new().expect("tempdir");
    for index in 0..8 {
        let dir = root.path().join(format!("data-{index}"));
        fs::create_dir(&dir).expect("create data dir");
        fs::write(dir.join("payload.json"), "{\"value\":true}\n").expect("write data");
    }
    let source_dir = root.path().join("src");
    fs::create_dir(&source_dir).expect("create src");
    fs::write(
        source_dir.join("main.ts"),
        (0..8)
            .map(|index| format!("export function api{index}(value: string): string {{ return value; }}\n"))
            .collect::<String>(),
    )
    .expect("write source");

    let mut config = Config::default();
    config.content.enabled = false;
    let result = pack(
        root.path(),
        &config,
        BudgetTarget::Tokens(600),
        false,
        true,
        Some("."),
    );

    assert_eq!(result.truncation.signature_files_candidate, 1);
    assert_eq!(result.truncation.signature_files_processed, 1);
    assert_eq!(result.truncation.signature_files_partial, 0);
    assert!(!result.truncation.signatures_truncated());
    assert!(result.output.contains("function api7"));
}
