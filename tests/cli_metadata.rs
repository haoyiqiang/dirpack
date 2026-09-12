use std::fs;

use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;
use tempfile::TempDir;

#[test]
fn cli_exclude_pattern_filters_files() {
    let root = TempDir::new().expect("tempdir");
    fs::write(root.path().join("keep.ts"), "export function keep() {}\n").unwrap();
    fs::write(root.path().join("drop.ts"), "export function drop() {}\n").unwrap();

    let output = cargo_bin_cmd!("dirpack")
        .args([
            "pack",
            root.path().to_str().unwrap(),
            "--target-tokens",
            "1000",
            "--exclude",
            "drop.ts",
            "--no-git",
            "--no-cache",
            "--quiet",
        ])
        .output()
        .expect("run dirpack");

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("keep.ts"));
    assert!(!stdout.contains("drop.ts"));
}

#[test]
fn cli_tree_only_flags_skip_signatures_and_content() {
    let root = TempDir::new().expect("tempdir");
    fs::write(
        root.path().join("main.ts"),
        "export function main(value: string): string { return value; }\n".repeat(20),
    )
    .unwrap();

    let output = cargo_bin_cmd!("dirpack")
        .args([
            "pack",
            root.path().to_str().unwrap(),
            "--target-tokens",
            "3000",
            "--no-signatures",
            "--no-content",
            "--no-git",
            "--no-cache",
            "--quiet",
        ])
        .output()
        .expect("run dirpack");

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("main.ts"));
    assert!(!stdout.contains("main.ts:function"));
    assert!(!stdout.contains("CONTENT:"));
}

#[test]
fn json_output_exposes_coverage_and_signature_metadata() {
    let root = TempDir::new().expect("tempdir");
    fs::write(
        root.path().join("main.ts"),
        "export function main(value: string): string { return value; }\n",
    )
    .unwrap();

    let output = cargo_bin_cmd!("dirpack")
        .args([
            "pack",
            root.path().to_str().unwrap(),
            "--target-tokens",
            "1000",
            "--format",
            "json",
            "--no-git",
            "--no-cache",
            "--quiet",
        ])
        .output()
        .expect("run dirpack");

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let value: Value = serde_json::from_slice(&output.stdout).expect("valid json");
    let truncation = value.get("truncation").expect("truncation metadata");

    assert_eq!(truncation["files_scanned"], 1);
    assert_eq!(truncation["files_in_tree"], 1);
    assert_eq!(truncation["signature_files_candidate"], 1);
    assert!(truncation.get("signatures_truncated").is_some());
    assert!(value["output"].as_str().unwrap().contains("main.ts"));
}
