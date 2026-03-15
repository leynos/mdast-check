//! Integration tests for the validate command CLI.
//!
//! Uses `assert_cmd` for end-to-end testing of the CLI surface.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Tests that missing --document returns exit code 2 (invalid invocation).
#[test]
fn missing_document_returns_exit_code_2() {
    let mut cmd = Command::cargo_bin("mdast_check").expect("failed to find binary");
    cmd.args(["validate", "--schema", "schema.json"]);
    cmd.assert().code(2).stderr(predicate::str::contains(
        "missing required argument: --document",
    ));
}

/// Tests that missing --schema returns exit code 2 (invalid invocation).
#[test]
fn missing_schema_returns_exit_code_2() {
    let mut cmd = Command::cargo_bin("mdast_check").expect("failed to find binary");
    cmd.args(["validate", "--document", "doc.md"]);
    cmd.assert().code(2).stderr(predicate::str::contains(
        "missing required argument: --schema",
    ));
}

/// Tests that a valid invocation with both required arguments succeeds.
#[test]
fn valid_invocation_succeeds() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");

    // Create test files.
    std::fs::write(temp_dir.path().join("doc.md"), "# Test Document\n")
        .expect("failed to write doc");
    std::fs::write(temp_dir.path().join("schema.json"), "{}").expect("failed to write schema");

    let mut cmd = Command::cargo_bin("mdast_check").expect("failed to find binary");
    cmd.current_dir(&temp_dir);
    cmd.args([
        "validate",
        "--document",
        "doc.md",
        "--schema",
        "schema.json",
    ]);
    cmd.assert()
        .code(0)
        .stdout(predicate::str::contains("Validating"));
}

/// Tests that quiet mode suppresses stdout output.
#[test]
fn quiet_mode_suppresses_output() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");

    // Create test files.
    std::fs::write(temp_dir.path().join("doc.md"), "# Test Document\n")
        .expect("failed to write doc");
    std::fs::write(temp_dir.path().join("schema.json"), "{}").expect("failed to write schema");

    let mut cmd = Command::cargo_bin("mdast_check").expect("failed to find binary");
    cmd.current_dir(&temp_dir);
    cmd.args([
        "validate",
        "--document",
        "doc.md",
        "--schema",
        "schema.json",
        "--quiet",
    ]);
    cmd.assert().code(0).stdout(predicate::str::is_empty());
}

/// Tests that --help displays usage information.
#[test]
fn help_flag_displays_usage() {
    let mut cmd = Command::cargo_bin("mdast_check").expect("failed to find binary");
    cmd.args(["validate", "--help"]);
    cmd.assert()
        .code(0)
        .stdout(predicate::str::contains("--document"))
        .stdout(predicate::str::contains("--schema"));
}

/// Tests that --version displays version information.
#[test]
fn version_flag_displays_version() {
    let mut cmd = Command::cargo_bin("mdast_check").expect("failed to find binary");
    cmd.args(["--version"]);
    cmd.assert().code(0);
}

/// Tests that optional arguments are accepted.
#[test]
fn optional_arguments_accepted() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");

    // Create test files.
    std::fs::write(temp_dir.path().join("doc.md"), "# Test Document\n")
        .expect("failed to write doc");
    std::fs::write(temp_dir.path().join("schema.json"), "{}").expect("failed to write schema");

    let mut cmd = Command::cargo_bin("mdast_check").expect("failed to find binary");
    cmd.current_dir(&temp_dir);
    cmd.args([
        "validate",
        "--document",
        "doc.md",
        "--schema",
        "schema.json",
        "--schema-format",
        "json",
        "--format",
        "text",
        "--fail-fast",
        "--max-errors",
        "10",
    ]);
    cmd.assert().code(0);
}
