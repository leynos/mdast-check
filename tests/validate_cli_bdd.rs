//! BDD scenarios for the validate command CLI surface.
//!
//! Uses `rstest-bdd` step definitions backed by `assert_cmd` to exercise the
//! compiled binary against the feature file at
//! `tests/features/validate_cli.feature`.
//!
//! Step definitions are test infrastructure, but `rstest-bdd` macros do not
//! annotate them with `#[test]`, so `allow-expect-in-tests` does not apply.
#![expect(
    clippy::expect_used,
    reason = "step definitions are test infrastructure; panicking on setup failure is appropriate"
)]

use assert_cmd::Command;
use rstest::fixture;
use rstest_bdd::Slot;
use rstest_bdd_macros::{ScenarioState, given, scenarios, then, when};
use std::process::Output;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Strips surrounding double quotes from a Gherkin step capture.
///
/// Gherkin steps like `stdout should contain "foo"` capture the quotes as
/// part of the `{expected}` placeholder. This helper removes them so that
/// assertions compare only the inner text.
fn strip_quotes(s: &str) -> &str {
    s.strip_prefix('"')
        .and_then(|inner| inner.strip_suffix('"'))
        .unwrap_or(s)
}

/// Asserts that `stream` (raw bytes) contains `needle` as UTF-8 text.
fn assert_stream_contains(stream: &[u8], stream_name: &str, needle: &str) {
    let text = String::from_utf8_lossy(stream);
    assert!(
        text.contains(needle),
        "expected {stream_name} to contain {needle:?}, got: {text}"
    );
}

// ---------------------------------------------------------------------------
// Scenario state
// ---------------------------------------------------------------------------

/// Shared state carried across steps within a single scenario.
#[derive(Default, ScenarioState)]
struct CliState {
    /// Temporary directory for test files.
    work_dir: Slot<TempDir>,
    /// Captured process output from the most recent command invocation.
    output: Slot<Output>,
}

// ---------------------------------------------------------------------------
// Fixture
// ---------------------------------------------------------------------------

/// Provides a fresh [`CliState`] for each scenario.
#[fixture]
fn cli_state() -> CliState {
    CliState::default()
}

// ---------------------------------------------------------------------------
// Given steps
// ---------------------------------------------------------------------------

#[given("a temporary working directory")]
fn a_temporary_working_directory(cli_state: &CliState) {
    let dir = TempDir::new().expect("should create temp dir");
    cli_state.work_dir.set(dir);
}

#[given("a file named {name} with content:")]
fn a_file_named_with_content(cli_state: &CliState, name: String, docstring: String) {
    cli_state
        .work_dir
        .with_ref(|dir| {
            std::fs::write(dir.path().join(&name), &docstring).expect("should write test file");
        })
        .expect("work_dir should be set before creating files");
}

// ---------------------------------------------------------------------------
// When steps
// ---------------------------------------------------------------------------

/// Runs the `mdast-check` binary with the given argument string.
///
/// The step text uses the pattern `I run mdast-check {args}` where `{args}`
/// captures the remainder of the line (the subcommand and flags).
#[when("I run mdast-check {args}")]
fn i_run_mdast_check(cli_state: &CliState, args: String) {
    let mut cmd = Command::cargo_bin("mdast_check").expect("should find binary");

    // The work_dir is always set by the Background step.
    cli_state
        .work_dir
        .with_ref(|dir| {
            cmd.current_dir(dir.path());
        })
        .expect("work_dir should be set before running a command");

    // Split the argument string on whitespace to produce individual args.
    let parts: Vec<&str> = args.split_whitespace().collect();
    cmd.args(&parts);

    let output = cmd.output().expect("should execute command");
    cli_state.output.set(output);
}

// ---------------------------------------------------------------------------
// Then steps
// ---------------------------------------------------------------------------

#[then("the exit code should be {code:i32}")]
fn the_exit_code_should_be(cli_state: &CliState, code: i32) {
    cli_state
        .output
        .with_ref(|output| {
            assert_eq!(
                output.status.code(),
                Some(code),
                "expected exit code {code}, got {:?}",
                output.status.code()
            );
        })
        .expect("output should be captured before asserting exit code");
}

#[then("stderr should contain {expected}")]
fn stderr_should_contain(cli_state: &CliState, expected: String) {
    let needle = strip_quotes(&expected);
    cli_state
        .output
        .with_ref(|output| assert_stream_contains(&output.stderr, "stderr", needle))
        .expect("output should be captured before asserting stderr");
}

#[then("stdout should contain {expected}")]
fn stdout_should_contain(cli_state: &CliState, expected: String) {
    let needle = strip_quotes(&expected);
    cli_state
        .output
        .with_ref(|output| assert_stream_contains(&output.stdout, "stdout", needle))
        .expect("output should be captured before asserting stdout");
}

#[then("stdout should be empty")]
fn stdout_should_be_empty(cli_state: &CliState) {
    cli_state
        .output
        .with_ref(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.is_empty(), "expected empty stdout, got: {stdout}");
        })
        .expect("output should be captured before asserting stdout");
}

// ---------------------------------------------------------------------------
// Scenario discovery
// ---------------------------------------------------------------------------

scenarios!(
    "tests/features",
    fixtures = [cli_state: CliState]
);
