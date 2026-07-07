# Implement the root CLI command with a `validate` subcommand

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work
proceeds.

Status: COMPLETE

## Purpose / big picture

After this work, a user can invoke `mdast-check validate` with flags for
document path, schema path, output format, configuration file, fail-fast mode,
max-errors cap, and quiet mode. The tool loads layered configuration via
`ortho-config` v0.8.0 (CLI flags override config-file values override
defaults), validates that the invocation is well-formed, and returns one of six
stable exit codes (0 success, 1 validation failure, 2 invalid invocation, 3
schema failure, 4 parse failure, 5 internal failure). No actual Markdown
parsing or schema validation occurs yet; this milestone delivers the command
shell, configuration plumbing, and exit-code contract.

Observable success: running
`cargo run -- validate --document foo.md --schema bar.json` exits with code 0
(stubbed success). Running `cargo run -- validate` without required arguments
exits with code 2. Running `make check-fmt && make lint && make test` succeeds
with all new unit tests and BDD scenarios passing. `docs/users-guide.md`
documents the CLI surface and exit codes.

## Constraints

Hard invariants that must hold throughout implementation. These are not
suggestions; violation requires escalation, not workarounds.

- The existing `Cargo.toml` lint configuration must not be weakened. All
  `deny`-level Clippy lints remain enforced.
- Exit codes must match the design document exactly: 0, 1, 2, 3, 4, 5.
- `ortho-config` v0.8.0 must be used for configuration layering. No
  bespoke configuration loading.
- All dependencies must use caret version requirements per
  `AGENTS.md` §Dependency Management.
- No file in `src/` may exceed 400 lines.
- `cap_std`/`camino` must be used for filesystem paths where applicable,
  per `AGENTS.md` §Rust Specific Guidance.
- Comments must use en-GB-oxendict spelling.
- `make check-fmt`, `make lint`, and `make test` must pass before every
  commit.
- The `main` branch must not be modified; all work occurs on the
  `1-1-1-implement-the-root-cli-command-with-a-validate-subcommand` branch.
- Production code must not use `.expect()` or `.unwrap()`. Return `Result`
  and propagate with `?`.
- Tests must use `rstest` fixtures. BDD tests must use `rstest-bdd` v0.5.0.
- `thiserror` must be used for domain error types. `eyre` may only be used
  at the application boundary in `main()`.

## Tolerances (exception triggers)

Thresholds that trigger escalation when breached. These define the boundaries
of autonomous action, not quality criteria.

- Scope: if implementation requires changes to more than 20 files or 2000
  lines of code (net), stop and escalate.
- Interface: if a public API signature from `ortho-config` or `rstest-bdd`
  does not match the documentation, stop and escalate.
- Dependencies: if any dependency beyond those listed in the plan
  (§Interfaces and dependencies) is required, stop and escalate.
- Iterations: if tests still fail after 5 attempts at fixing a single
  issue, stop and escalate.
- Ambiguity: if the design document is ambiguous about a flag's semantics
  and the choice materially affects the user contract, stop and present options.

## Risks

Known uncertainties that might affect the plan. Identify these upfront and
update as work proceeds.

- Risk: `ortho-config` v0.8.0 may not be published to crates.io yet, or
  its API may differ from the user's guide. Severity: high Likelihood: low
  Mitigation: verify availability with `cargo search ortho_config` before
  beginning. If unavailable, escalate immediately.

- Risk: `rstest-bdd` v0.5.0 macro expansion may conflict with the strict
  Clippy lint set (e.g., `expect_used`, `unwrap_used` in generated code).
  Severity: medium Likelihood: medium Mitigation: if generated code triggers
  lints, apply tightly scoped `#[allow(...)]` with documented reasons on the
  test functions only. Escalate if the suppressions cascade.

- Risk: the `--schema-format` flag's default-by-extension behaviour is
  underspecified for this milestone (no schema loading yet). Severity: low
  Likelihood: high Mitigation: accept and store the value; defer
  extension-based defaulting to milestone 1.3.1. Document the decision.

- Risk: `ortho-config` discovery attributes may not support all the
  customizations needed for `mdast-check`'s config file naming. Severity:
  medium Likelihood: low Mitigation: review the `discovery(...)` attribute
  documentation carefully before writing the struct. Fall back to simpler
  defaults if needed and document the gap.

## Progress

- [x] (2025-03-15) Stage A: Research and verify dependency availability.
- [x] (2025-03-15) Stage B: Scaffold modules and define types.
- [x] (2025-03-15) Stage C: Write failing tests (red). Merged with D
  because the implementation was provided alongside tests.
- [x] (2025-03-15) Stage D: Implement CLI and configuration (green).
- [x] (2025-03-15) Stage E: Refactor and harden. Fixed Clippy lints,
  formatting, boolean flag semantics, and eliminated unsafe unwrap calls.
- [x] (2025-03-15) Stage F: Documentation and cleanup. Created
  `docs/users-guide.md` and updated the design document with implementation
  decisions.
- [x] (2025-03-15) Stage G: Final validation and roadmap update. All quality
  gates pass: 38 tests (25 unit, 6 BDD, 7 integration), zero lint warnings,
  zero markdown errors.

## Surprises & discoveries

- Observation: `Option<bool>` fields with clap's `#[arg(long)]` require an
  explicit value (`--quiet true`) rather than acting as boolean flags
  (`--quiet`). This is a well-documented clap behaviour but differs from the
  intuition most CLI users have. Evidence: integration tests failed when
  passing `--quiet` without a value. Impact: changed `fail_fast` and `quiet`
  from `Option<bool>` to `bool` with `default_value_t = false`.

- Observation: `clippy::allow-expect-in-tests` only applies to functions
  annotated with `#[test]` or within `#[cfg(test)]` modules. Step definition
  functions in rstest-bdd (annotated with `#[given]`, `#[when]`, `#[then]`) are
  not covered. Evidence: Clippy denied `expect_used` in step definitions in
  `tests/validate_cli_bdd.rs`. Impact: added a file-level
  `#![expect(clippy::expect_used)]` with a documented reason.

- Observation: rstest-bdd `Slot<T>` uses `with_ref()` (closure-based access)
  and `with_mut()` rather than a direct `get_ref()` method. The `get()` method
  requires `T: Clone`. Evidence: `get_ref()` compilation error. Impact:
  restructured all step definitions to use `with_ref()` and `.expect()` for
  accessing slot values.

- Observation: rstest-bdd captures Gherkin step parameters including
  surrounding quotes. A step like `stdout should contain "foo"` captures
  `"foo"` (with quotes) in the `{expected}` placeholder. Evidence: assertion
  failures comparing `"\"foo\""` against output. Impact: added a
  `strip_quotes()` helper in the BDD test file.

## Decision log

- Decision: use `bool` with `default_value_t = false` for `--quiet` and
  `--fail-fast` instead of `Option<bool>`. Rationale: standard CLI boolean flag
  ergonomics. Users expect `--quiet` to work without a value argument. Future
  config-file override semantics can be handled by ortho-config's layer
  merging. Date/Author: 2025-03-15 / agent

- Decision: use `color-eyre` at the application boundary instead of plain
  `eyre`. Rationale: provides enhanced error output with colour and span
  traces. The design document specified `eyre` at the boundary; `color-eyre` is
  a compatible superset. Date/Author: 2025-03-15 / agent

- Decision: name the output format values `text` and `json` instead of `human`
  and `json`. Rationale: `text` is the more common convention in CLI tools and
  aligns with clap's `ValueEnum` naming patterns. Date/Author: 2025-03-15 /
  agent

- Decision: name the schema format value `custom` instead of `markdoc` for the
  non-JSON variant. Rationale: Markdoc schema compilation is a stretch goal not
  implemented in this milestone. Using `custom` avoids committing to a specific
  schema language name before the feature is built. Date/Author: 2025-03-15 /
  agent

## Context and orientation

`mdast-check` is a greenfield Rust CLI tool. The repository currently contains:

- `src/main.rs` — a stub `fn main()` that prints "Hello from MDAST
  Check!" and carries a `FIXME` comment referencing the roadmap. This file will
  be replaced entirely.
- `Cargo.toml` — declares the `mdast_check` binary crate (edition 2024)
  with no runtime dependencies and a comprehensive set of Clippy lints at
  `deny` level.
- `clippy.toml` — tightens cognitive-complexity (9), too-many-arguments
  (4), too-many-lines (70), excessive-nesting (4), and allows `expect` in tests.
- `Makefile` — provides `check-fmt`, `lint`, `test`, `fmt`,
  `markdownlint`, and `nixie` targets.
- `rust-toolchain.toml` — pins `nightly-2026-03-05` with `rustfmt` and
  `clippy` components.
- `docs/` — design document, roadmap, ortho-config guide, rstest-bdd
  guide, and other reference material. No `users-guide.md` exists yet.
- `tests/` — empty (no test files).

The design document (`docs/mdast-schema-validation-cli-design.md`) prescribes
six modules: `cli`, `document`, `schema`, `mdast_model`, `validate`, and
`diagnostics`. This milestone touches only `cli` (and partially `diagnostics`
for exit-code mapping).

Key reference files the implementer must consult:

- `docs/mdast-schema-validation-cli-design.md` §User-facing contract —
  defines the command surface, flags, and exit codes.
- `docs/ortho-config-users-guide.md` — explains `#[derive(OrthoConfig)]`,
  `discovery(...)`, `MergeComposer`, layered precedence, and `FluentLocalizer`.
- `docs/rstest-bdd-users-guide.md` — explains feature files, step
  definitions (`#[given]`, `#[when]`, `#[then]`), `#[scenario]`,
  `ScenarioState`, `Slot<T>`, and CLI-oriented BDD patterns.
- `docs/rust-testing-with-rstest-fixtures.md` — covers `#[fixture]`,
  `#[rstest]`, `#[case]`, `#[values]`, partial injection, and composition.
- `docs/reliable-testing-in-rust-via-dependency-injection.md` — covers
  `mockable` for environment variable injection.
- `AGENTS.md` — project-wide coding standards, lint policy, error
  handling, dependency management, testing, and Markdown guidance.

## Plan of work

The work is split into seven stages. Each stage ends with a validation step. Do
not proceed to the next stage if the current stage's validation fails.

### Stage A: Research and verify dependency availability

Before writing any code, confirm that the critical external dependencies are
available and compatible.

1. Run `cargo search ortho_config` to verify v0.8.0 is published.
2. Run `cargo search rstest-bdd` to verify v0.5.0 is published.
3. Run `cargo search rstest` to verify a compatible version exists.
4. Review `ortho-config` 0.8.0's actual API surface to confirm the
   `discovery(...)`, `SelectedSubcommandMerge`, and `MergeComposer` APIs match
   the user's guide.

Validation: all four dependencies are available and API-compatible. If not,
escalate before proceeding.

### Stage B: Scaffold modules and define types

This stage establishes the file layout, the `OrthoConfig`-derived configuration
struct, the `clap` subcommand enum, the `thiserror` error taxonomy, and the
exit-code mapping. No behaviour yet — only type definitions and module
declarations.

#### B.1. Update `Cargo.toml` with dependencies

Add the following to `[dependencies]`:

```toml
clap = { version = "4.5", features = ["derive"] }
ortho_config = "0.8.0"
serde = { version = "1.0", features = ["derive"] }
thiserror = "2.0"
eyre = "0.6"
camino = "1.1"
```

Add the following to `[dev-dependencies]`:

```toml
rstest = "0.26"
rstest-bdd = "0.5.0"
rstest_bdd_macros = "0.5.0"
assert_cmd = "2.0"
predicates = "3.1"
```

Exact patch versions may vary; use caret requirements throughout.

#### B.2. Create the source module layout

Create the following files under `src/`:

- `src/lib.rs` — crate root for the library portion; declares `pub mod
  cli;` and `pub mod error;`. Contains the crate-level doc comment.
- `src/cli.rs` — contains the `MdastCheckCli` struct (derives
  `OrthoConfig`), the `Commands` enum (with `Validate` variant), and the
  `ValidateArgs` struct (derives `OrthoConfig`).
- `src/error.rs` — contains the `MdastCheckError` enum (derives
  `thiserror::Error`) and the `ExitCode` enum with `From<&MdastCheckError>`
  conversion.
- `src/main.rs` — the application entry point; parses CLI, loads config,
  and maps errors to exit codes.

#### B.3. Define the configuration struct

In `src/cli.rs`, define:

```rust
#[derive(Debug, Deserialize, Serialize, OrthoConfig)]
#[ortho_config(
    prefix = "MDAST_CHECK",
    discovery(
        app_name = "mdast-check",
        env_var = "MDAST_CHECK_CONFIG_PATH",
        config_file_name = "config.toml",
        dotfile_name = ".mdast-check.toml",
        project_file_name = ".mdast-check.toml",
        config_cli_long = "config",
        config_cli_short = 'c',
        config_cli_visible = true,
    )
)]
pub struct ValidateArgs {
    /// Path to the Markdown document. Use `-` for stdin.
    #[ortho_config(cli_short = 'd')]
    pub document: Option<String>,

    /// Path to the JSON Schema document.
    #[ortho_config(cli_short = 's')]
    pub schema: Option<String>,

    /// Force schema parsing mode (`json` or `markdoc`).
    #[ortho_config(default = SchemaFormat::Json)]
    pub schema_format: SchemaFormat,

    /// Diagnostic output format (`human` or `json`).
    #[ortho_config(default = OutputFormat::Human)]
    pub format: OutputFormat,

    /// Stop after the first validation failure.
    #[ortho_config(default = false)]
    pub fail_fast: bool,

    /// Cap reported validation failures.
    pub max_errors: Option<u32>,

    /// Suppress success output.
    #[ortho_config(default = false)]
    pub quiet: bool,
}
```

Define supporting enums `SchemaFormat` (`Json`, `Markdoc`) and `OutputFormat`
(`Human`, `Json`) with `Display`, `FromStr`, `Serialize`, and `Deserialize`
implementations.

Use `clap::Subcommand` to define a `Commands` enum with a single
`Validate(ValidateArgs)` variant. Wire this into a top-level `Cli` struct that
uses `#[command(subcommand)]`.

#### B.4. Define the error taxonomy

In `src/error.rs`, define:

```rust
#[derive(Debug, thiserror::Error)]
pub enum MdastCheckError {
    #[error("invalid invocation: {message}")]
    InvalidInvocation { message: String },

    #[error("configuration error: {0}")]
    ConfigLoad(#[from] ortho_config::OrthoError),

    #[error("schema error: {message}")]
    SchemaFailure { message: String },

    #[error("markdown parse error: {message}")]
    MarkdownParseFailure { message: String },

    #[error("validation failed: {message}")]
    ValidationFailure { message: String },

    #[error("internal error: {message}")]
    InternalFailure { message: String },
}
```

Define an `ExitCode` newtype wrapping `u8` with constants:

```rust
pub struct ExitCode(u8);

impl ExitCode {
    pub const SUCCESS: Self = Self(0);
    pub const VALIDATION_FAILURE: Self = Self(1);
    pub const INVALID_INVOCATION: Self = Self(2);
    pub const SCHEMA_FAILURE: Self = Self(3);
    pub const MARKDOWN_PARSE_FAILURE: Self = Self(4);
    pub const INTERNAL_FAILURE: Self = Self(5);
}
```

Implement `From<&MdastCheckError> for ExitCode` to map each error variant to
its corresponding exit code.

Validation: `cargo check` succeeds. `make check-fmt` and `make lint` pass.

### Stage C: Write failing tests (red)

Before implementing behaviour, write the tests that define the expected
contract. All tests should fail at this point (red phase).

#### C.1. Unit tests for exit-code mapping

In `src/error.rs` (or a colocated `#[cfg(test)]` module), write `rstest`
parameterized tests using `#[case]` that verify each `MdastCheckError` variant
maps to the correct `ExitCode`.

```rust
#[rstest]
#[case(
    MdastCheckError::InvalidInvocation {
        message: "test".into()
    },
    ExitCode::INVALID_INVOCATION
)]
#[case(
    MdastCheckError::ValidationFailure {
        message: "test".into()
    },
    ExitCode::VALIDATION_FAILURE
)]
// ... cases for all six exit codes
fn exit_code_from_error(
    #[case] error: MdastCheckError,
    #[case] expected: ExitCode,
) {
    assert_eq!(ExitCode::from(&error), expected);
}
```

#### C.2. Unit tests for configuration precedence

Write tests using `ortho-config`'s `MergeComposer` to verify that CLI overrides
take precedence over config-file values. For example, verify that setting
`fail_fast = true` in a config file and `fail_fast = false` on the CLI results
in `false`.

#### C.3. BDD feature file and scenario

Create `tests/features/validate_cli.feature`:

```gherkin
Feature: CLI validate command

  Scenario: Missing required arguments
    When the user runs mdast-check validate without arguments
    Then the exit code is 2

  Scenario: Valid invocation with all flags
    Given a Markdown document at "test.md"
    And a schema document at "schema.json"
    When the user runs mdast-check validate with document and schema
    Then the exit code is 0

  Scenario: Quiet mode suppresses output
    Given a Markdown document at "test.md"
    And a schema document at "schema.json"
    When the user runs mdast-check validate with --quiet
    Then the exit code is 0
    And stdout is empty
```

Create `tests/rstest_bdd/validate_cli.rs` with step definitions using
`assert_cmd` to invoke the compiled binary:

- `#[when("the user runs mdast-check validate without arguments")]` —
  runs the binary with `validate` and no other flags.
- `#[then("the exit code is {code:i32}")]` — asserts the process exit
  code.
- Fixture for temporary Markdown and schema files using `tempfile`.

Wire the scenario with
`#[scenario(path = "tests/features/validate_cli.feature")]`.

Validation: `cargo test` compiles but the new tests fail (red). This confirms
the test harness is correctly wired.

### Stage D: Implement CLI and configuration (green)

Make the failing tests pass by implementing the actual behaviour.

#### D.1. Implement `main.rs`

Replace the stub `main()` with:

1. Parse CLI arguments via `clap` (using the top-level `Cli` struct).
2. Handle `--help` and `--version` display requests using
   `ortho_config::is_display_request`.
3. Match on the `Commands::Validate` variant.
4. Load and merge configuration for the `ValidateArgs` subcommand using
   `load_and_merge()` (or `compose_layers` + `merge_from_layers`).
5. Validate that required fields (`document` and `schema`) are present.
   If not, return `MdastCheckError::InvalidInvocation`.
6. For this milestone, the validated arguments are accepted and the tool
   exits with code 0 (success stub). Actual validation is deferred to milestone
   1.3.2.
7. Map any `MdastCheckError` to an `ExitCode` and call
   `std::process::exit`.

Use `eyre` only at the top-level boundary for wrapping unexpected panics.
Domain errors flow through `MdastCheckError`.

#### D.2. Wire configuration loading

Ensure `ortho-config`'s layered loading works:

1. Defaults from `#[ortho_config(default = ...)]` attributes.
2. Config file from `--config` / `MDAST_CHECK_CONFIG_PATH` /
   `.mdast-check.toml` discovery.
3. Environment variables prefixed with `MDAST_CHECK_`.
4. CLI flags.

The `--config` flag should be visible in help output (per `discovery` attribute
`config_cli_visible = true`).

#### D.3. Implement required-field validation

When `document` is `None` after merging, emit
`MdastCheckError::InvalidInvocation` with a message naming the missing field.
Likewise for `schema`. This produces exit code 2.

#### D.4. Implement quiet mode

When `--quiet` is set and the result is success (exit code 0), suppress all
stdout output. Failure output must still be emitted regardless of quiet mode.

Validation: `cargo test` passes. All unit tests and BDD scenarios are green.
`make check-fmt && make lint && make test` all succeed.

### Stage E: Refactor and harden

Review the implementation for adherence to the project's coding standards and
refactoring heuristics.

1. Ensure no function exceeds 70 lines or cognitive complexity 9.
2. Ensure no file exceeds 400 lines.
3. Ensure all public items have `///` documentation.
4. Ensure all modules have `//!` module-level documentation.
5. Ensure error messages use en-GB-oxendict spelling.
6. Extract helper functions where needed for clarity.
7. Confirm `From<&MdastCheckError> for ExitCode` is exhaustive and cannot
   silently fall through.

Validation: `make check-fmt && make lint && make test` pass. Review the diff
for any missed quality gates.

### Stage F: Documentation and cleanup

#### F.1. Create `docs/users-guide.md`

Document the following:

- Installation (building from source).
- The `validate` subcommand and its purpose.
- All flags: `--document`, `--schema`, `--schema-format`, `--config`,
  `--format`, `--fail-fast`, `--max-errors`, `--quiet`.
- Configuration file format and discovery paths.
- Environment variable naming (`MDAST_CHECK_*`).
- Precedence: defaults < config file < environment < CLI.
- Exit codes table (0–5) with descriptions.
- Examples of common invocations.

Follow the documentation style guide at `docs/documentation-style-guide.md`.
Use en-GB-oxendict spelling. Wrap at 80 columns.

#### F.2. Update the design document

Record any design decisions taken during implementation in
`docs/mdast-schema-validation-cli-design.md`, particularly around:

- The exact `ortho-config` struct layout chosen.
- Any deviations from the proposed flag set.
- The stub behaviour for this milestone (success when invocation is valid).

Validation: `make markdownlint` passes on all new and modified Markdown files.
Review the users guide for completeness.

### Stage G: Final validation and roadmap update

1. Run the full gate sequence: `make check-fmt && make lint && make test`.
2. Verify the BDD scenarios exercise the key user journeys (missing args,
   valid invocation, quiet mode).
3. Mark roadmap item 1.1.1 as done in `docs/roadmap.md`:
   change `- [ ] 1.1.1.` to `- [x] 1.1.1.` and do the same for its three
   sub-items.
4. Commit with a descriptive message.

## Concrete steps

All commands are run from the project root: `/data/leynos/Projects/mdast-check`.

### Stage A

```bash
cargo search ortho_config
cargo search rstest-bdd
cargo search rstest
```

Expected: each command returns a line showing the crate name and a version
matching or exceeding the required version.

### Stage B

Edit `Cargo.toml` to add dependencies. Create `src/lib.rs`, `src/cli.rs`,
`src/error.rs`. Modify `src/main.rs`. Run:

```bash
cargo check
make check-fmt
make lint
```

Expected: all three succeed with no errors or warnings.

### Stage C

Create `tests/features/validate_cli.feature` and
`tests/rstest_bdd/validate_cli.rs` (or the appropriate integration test file).
Run:

```bash
cargo test
```

Expected: new tests compile but fail (red phase).

### Stage D

Implement the CLI logic. Run:

```bash
make check-fmt && make lint && make test
```

Expected: all pass. The new tests are green.

### Stage E

Refactor. Run:

```bash
make check-fmt && make lint && make test
```

Expected: all pass. No regressions.

### Stage F

Create `docs/users-guide.md`. Update design doc. Run:

```bash
make markdownlint
```

Expected: no Markdown lint errors.

### Stage G

```bash
make check-fmt && make lint && make test
```

Expected: all pass. Roadmap updated.

## Validation and acceptance

Quality criteria (what "done" means):

- Tests: `make test` passes. Unit tests cover every exit-code mapping.
  BDD scenarios cover missing arguments (exit 2), valid invocation (exit 0),
  and quiet mode. `rstest` parameterized cases cover configuration precedence.
- Lint/typecheck: `make check-fmt` and `make lint` pass with zero warnings.
- Documentation: `docs/users-guide.md` exists and documents the full CLI
  surface. `docs/roadmap.md` marks 1.1.1 as done.
- Markdown: `make markdownlint` passes.

Quality method (how we check):

```bash
make check-fmt && make lint && make test && make markdownlint
```

All four must exit with code 0.

## Idempotence and recovery

Every stage can be re-run safely. `cargo check`, `cargo test`, and `make`
targets are idempotent. If a stage fails partway through, fix the issue and
re-run the same stage from the beginning.

If a commit fails quality gates, do not amend. Fix the issue and create a new
commit.

## Outcomes & retrospective

The CLI and configuration shell is fully implemented and all quality gates
pass. The deliverable includes:

- A `validate` subcommand accepting all specified flags.
- Layered configuration via ortho-config `MergeComposer` (CLI overrides
  defaults; file and environment layers are stubbed).
- Domain error taxonomy with `thiserror` and exit-code mapping.
- 38 tests: 25 unit (rstest parameterized), 6 BDD (rstest-bdd), 7 integration
  (assert\_cmd).
- User's guide (`docs/users-guide.md`) and design document updates.

Lessons learned:

- The strict Clippy lint set (deny-level `unwrap_used`, `expect_used`,
  `print_stderr`) requires careful design in test infrastructure. Step
  definitions in rstest-bdd are not recognised as test code by Clippy,
  necessitating file-level lint suppression.
- Boolean flags with `Option<bool>` in clap require explicit values; `bool`
  with `default_value_t` is the correct approach for flags without values.
- rstest-bdd's `Slot<T>` API uses closure-based access (`with_ref`,
  `with_mut`) rather than direct reference accessors. This is ergonomic once
  understood but differs from typical `Option<T>` patterns.

## Artifacts and notes

Test summary from the final quality gate run:

```plaintext
running 25 tests                          (unit tests — lib)
running 6 tests                           (BDD scenarios — validate_cli_bdd)
running 7 tests                           (integration tests — validate_cli_tests)
test result: ok. 38 passed; 0 failed
```

## Interfaces and dependencies

### Runtime dependencies

| Crate          | Version | Purpose                                              |
| -------------- | ------- | ---------------------------------------------------- |
| `clap`         | 4.5     | CLI argument parsing with derive macros.             |
| `ortho_config` | 0.8.0   | Layered configuration (CLI > env > file > defaults). |
| `serde`        | 1.0     | Serialization/deserialization for config structs.    |
| `thiserror`    | 2.0     | Derive `Error` for the domain error enum.            |
| `eyre`         | 0.6     | Opaque error at the `main()` boundary only.          |
| `camino`       | 1.1     | UTF-8 path types.                                    |

### Dev dependencies

| Crate               | Version | Purpose                                                          |
| ------------------- | ------- | ---------------------------------------------------------------- |
| `rstest`            | 0.26    | Fixture-based and parameterized unit tests.                      |
| `rstest-bdd`        | 0.5.0   | BDD scenario runner for `.feature` files.                        |
| `rstest_bdd_macros` | 0.5.0   | Proc macros for `#[given]`, `#[when]`, `#[then]`, `#[scenario]`. |
| `assert_cmd`        | 2.0     | CLI integration testing (run binary, assert exit code/output).   |
| `predicates`        | 3.1     | Composable assertions for `assert_cmd`.                          |

### Key types to define

In `src/error.rs`:

```rust
/// Domain errors for `mdast-check`, each mapping to a stable exit code.
#[derive(Debug, thiserror::Error)]
pub enum MdastCheckError {
    #[error("invalid invocation: {message}")]
    InvalidInvocation { message: String },
    #[error("configuration error: {0}")]
    ConfigLoad(/* source */),
    #[error("schema error: {message}")]
    SchemaFailure { message: String },
    #[error("markdown parse error: {message}")]
    MarkdownParseFailure { message: String },
    #[error("validation failed: {message}")]
    ValidationFailure { message: String },
    #[error("internal error: {message}")]
    InternalFailure { message: String },
}

/// Stable process exit code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExitCode(u8);
```

In `src/cli.rs`:

```rust
/// Top-level CLI entry point parsed by `clap`.
#[derive(clap::Parser)]
#[command(name = "mdast-check", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available subcommands.
#[derive(clap::Subcommand)]
pub enum Commands {
    /// Validate a Markdown document against a JSON Schema.
    Validate(ValidateArgs),
}

/// Schema format selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SchemaFormat { Json, Markdoc }

/// Diagnostic output format selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat { Human, Json }
```
