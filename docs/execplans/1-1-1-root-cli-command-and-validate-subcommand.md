# 1.1.1 Implement the root CLI command with a `validate` subcommand

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & discoveries`, `Decision log`,
`Outcomes & retrospective`, `Conformance basis`, and `Verification plan` must
be kept up to date as work proceeds.

Status: DRAFT (awaiting design review and user approval)

Branch: `1-1-1-root-cli-command-and-validate-subcommand`

Roadmap item: `1.1.1` in `docs/roadmap.md`.

## Purpose / big picture

`mdast-check` is a command-line interface (CLI) that will validate the Markdown
Abstract Syntax Tree (MDAST) of a Markdown document against a JSON Schema
document. Today the binary is a stub: `src/main.rs` prints
`Hello from MDAST Check!` and exits with status 0 whatever arguments it is
given. No configuration is read and no exit code carries meaning.

This plan delivers the command shell that every later roadmap item plugs into.
After the change, a user can run:

```plaintext
mdast-check validate --document docs/article.md --schema schema/article.json
```

and observe:

- every flag in the design's flag table (`--document`, `--schema`,
  `--schema-format`, `--config`, `--format`, `--fail-fast`, `--max-errors`,
  `--quiet`) is parsed, type-checked, and shown in `--help`;
- a configuration file named by `--config` supplies defaults for the
  configurable settings, loaded through `ortho-config` v0.8.0, and any flag on
  the command line overrides the file;
- the process exits with one of six stable codes: `0` success, `1` validation
  failure, `2` invalid invocation or configuration, `3` schema failure, `4`
  Markdown parse failure, and `5` internal failure.

Terms used throughout:

- *Invocation error*: anything wrong with the command line or configuration,
  such as an unknown flag, a missing required value, an unreadable `--config`
  file, or a value of the wrong type. These always exit `2`.
- *Validation pipeline*: the later stages that load the schema, parse the
  Markdown, and validate it (roadmap items 1.2.1 to 1.3.2). This plan defines
  the boundary the pipeline will sit behind but does not implement it.
- *Port*: a Rust trait that the application core defines and an outer adapter
  implements, as in hexagonal architecture. *Driving adapter*: code that calls
  into the core (here the CLI). *Driven adapter*: code the core calls out to
  (here the validation pipeline).
- *Layer*: one source of configuration values (built-in defaults, the
  configuration file, the command line). Layers are merged in precedence order;
  a later layer wins for every key it defines.

Because the validation pipeline does not exist yet, a well-formed `validate`
invocation in the shipped binary reaches a placeholder driven adapter that
reports `error: document validation is not implemented yet` on standard error
and exits `5`. This is deliberate (see decision D4): it never claims a document
is valid when nothing was checked. Exit codes `0`, `1`, `3`, and `4` for a
`validate` run are exercised through the same public entry point with an
injected test pipeline, and become reachable in the shipped binary as roadmap
items 1.2.1 to 1.3.2 land.

`--format`, `--fail-fast`, `--max-errors`, and `--quiet` are parsed, merged,
and carried in the resolved settings, but their behaviour belongs to roadmap
items 2.1.2, 2.2.1, and 2.2.2, which own output rendering. The users' guide
states this plainly.

This plan closes the design's open question of how configuration is layered for
a subcommand (see decision D2 and the proposed ADR 0002).

## Constraints

- Implementation begins only after the user explicitly approves this plan.
- Follow `AGENTS.md` throughout: en-GB-oxendict spelling in comments and docs;
  every module starts with a `//!` comment; public items carry `///` docs with
  examples; no file over 400 lines; functions within the Clippy thresholds in
  `clippy.toml` (cognitive complexity 9, 4 arguments, 70 lines); every
  `assert!`, `assert_eq!`, and `assert_ne!` carries a diagnostic message.
- Do not weaken lint configuration. `Cargo.toml` denies, among others,
  `print_stdout`, `print_stderr`, `expect_used`, `unwrap_used`,
  `indexing_slicing`, `missing_const_for_fn`, `must_use_candidate`, `shadow_*`,
  `allow_attributes`, `missing_docs`, and `result_large_err`. Output must go
  through injected `std::io::Write` sinks, never `print!`. Suppression is a
  last resort and uses item-scoped `#[expect(lint, reason = "...")]` only.
- Remove the `FIXME(#5)` stub and its `#[expect(clippy::print_stdout, ...)]`
  in `src/main.rs` in the same commit that replaces the stub; leaving the
  expectation without the `println!` makes it unfulfilled and fails
  `-D warnings`.
- Production code must not read the process environment except at the
  composition root (`src/main.rs`), which passes `std::env::args_os()` into the
  library. Tests must not mutate the process environment or working directory
  (`clippy.toml` `disallowed-methods`); end-to-end tests may use `assert_cmd`
  with `Command::env_clear`, `Command::env`, and `Command::current_dir`, which
  affect only the child process.
- Keep the domain and application core free of `clap`, `ortho_config`,
  filesystem, and terminal types. `clap` and `ortho_config` appear only in
  `src/cli/`. Domain value types live in `src/validate/`.
- Use `camino::Utf8PathBuf` for paths, per `AGENTS.md`. A non-UTF-8 path on
  the command line is an invocation error (exit `2`).
- Use `ortho-config` with a caret requirement on `0.8.0`, as the roadmap and
  design mandate, even though `0.11.0` exists. Upgrading is out of scope.
- Do not change `tests/build_standard_contract.rs`,
  `tests/build_standard_support/`, `.cargo/config.toml`, `rust-toolchain.toml`,
  or the CI workflows. Changing the Makefile is limited to
  `TARGET ?= mdast-check` (decision D1).
- Do not implement Markdown parsing, schema loading, validation, output
  rendering, quiet suppression, fail-fast behaviour, or error capping. Those
  are roadmap items 1.2.1 to 2.2.2.
- Do not introduce the full `thiserror` error taxonomy of roadmap item 1.1.2.
  This plan introduces only the invocation error type it needs and the
  pipeline-failure categories that the exit-code contract requires; 1.1.2
  refines them.
- Dependency changes are limited to those listed in
  `Interfaces and dependencies`. Any other dependency requires escalation.
- No source-API compatibility machinery. The crate is pre-1.0 and has no
  release tag, so interfaces change together with their callers.
- Do not edit `typos.toml` by hand; use `typos.local.toml` for exceptions.

If satisfying the objective requires violating a constraint, do not proceed.
Document the conflict in `Decision log` and escalate.

## Tolerances (exception triggers)

The comparison base is `788b9d444c418b117b61e9eaa05a98f277228852` (`main` at
planning time). Counts exclude this ExecPlan file and any other file under
`docs/execplans/`. Counts also exclude generated `insta` snapshot files
(`tests/**/snapshots/*.snap`, `src/**/snapshots/*.snap`), which are reviewed as
output rather than authored code; their number is still counted under the file
limit. All line limits measure net lines (additions minus deletions):

```sh
git diff --numstat 788b9d4 -- . ':(exclude)docs/execplans' ':(exclude)**/*.snap'
```

Classification is by whole file: `src/**`, `tests/**/*.rs`, and
`tests/features/*.feature` are code; `Cargo.toml`, `Makefile`, and
`typos.local.toml` are configuration; `docs/**` (except `docs/execplans/`) and
`README.md` are documentation.

- Files: if implementation changes more than 40 non-ExecPlan files (snapshot
  files included), stop and escalate.
- Code: if code changes exceed 2,300 net lines, stop and escalate.
- Configuration: if configuration changes exceed 70 net lines, stop and
  escalate.
- Documentation: if documentation changes exceed 750 net lines, stop and
  escalate.
- Interface: if a public item beyond those in `Interfaces and dependencies`
  is needed, or a listed signature must change materially, stop and escalate.
- Dependencies: if any crate beyond those listed in
  `Interfaces and dependencies` is needed, stop and escalate.
- Architecture: if `ortho-config`'s declarative layer API (`MergeComposer`,
  `merge_from_layers`, `load_config_file_as_chain`) cannot express the layering
  in decision D2, stop and escalate rather than writing a bespoke merge.
- Iterations: if a focused test still fails after 4 fix attempts, stop and
  escalate.
- Time: if a milestone takes more than 6 hours of active work, stop and
  escalate.
- Ambiguity: if multiple valid interpretations exist and the choice materially
  affects the user-facing contract (flag names, configuration keys, exit
  codes), stop and present options with trade-offs.

Each category is enforced independently. Spare capacity in one category never
offsets another.

Budget rationale. The estimate below is from the module plan in
`Interfaces and dependencies`, compared with the stale prior attempt (see
`Context and orientation`), whose `src/` was about 600 lines and tests about
470 lines without configuration layering, tested examples, or property tests.
Production source is estimated at 950 lines (library root 40, exit status 90,
validate core 260, CLI adapter 520, main 20, placeholder pipeline 20). Tests
are estimated at 1,350 lines (unit tests within modules 450, property tests
170, end-to-end 170, BDD steps and feature 330, users' guide example loader and
runner 230). The code budget of 2,300 adds about 5% contingency, which is
deliberately thin: a breach signals scope growth, not estimation noise.
Configuration is about 35 lines (`[[bin]]` table, 6 runtime and 8 development
dependencies, one Makefile line, a few spelling exceptions); 70 doubles it to
absorb feature tables. Documentation is about 560 lines (users' guide 260, ADR
0002 110, design document 70, developers' guide 80, README 20, roadmap 6,
contents unaffected); 750 leaves a third in reserve because the users' guide
style rules are demanding. No comparable completed ExecPlan exists on `main`
(`docs/execplans/` is absent), so there is no history of budget revisions to
learn from.

## Risks

- Risk: `#[derive(OrthoConfig)]` emits code that trips this repository's
  strict lints (for example `missing_docs`, `must_use_candidate`,
  `missing_const_for_fn`) and cannot be fixed at the call site. Severity: high.
  Likelihood: medium. Mitigation: milestone `EP-M0` compiles the derive under
  `make lint` before any design is committed to. If lints fire inside generated
  code, isolate the derived type in a single module and use one item-scoped
  `#[expect(..., reason = ...)]` per lint, recorded in `Decision log`; if that
  is impossible, escalate under the Architecture tolerance.
- Risk: `merge_from_layers` mishandles `Option<NonZeroUsize>`,
  `Option<Utf8PathBuf>`, or the output-format enum (for example by failing on
  an absent optional key or by deserializing enum spellings differently from
  `clap`). Severity: medium. Likelihood: medium. Mitigation: `EP-M0` probe
  cases for each field type, including absent keys and invalid values.
- Risk: `load_config_file_as_chain` does not expose per-file paths in the shape
  this plan assumes, blocking per-file relative-path rebasing (decision D6).
  Severity: medium. Likelihood: low (the 0.8.0 source returns `FileLayerChain`
  with a UTF-8 path per layer). Mitigation: `EP-M0` probe; if it fails, fall
  back to `load_config_file` and rebase against the top-level file only,
  recording the deviation.
- Risk: `rstest` drops `#[expect]` attributes on generated tests
  (`docs/rust-testing-with-rstest-fixtures.md` lines 555 to 557), while
  `AGENTS.md` forbids `#[allow]`. Severity: low. Likelihood: medium.
  Mitigation: design tests so no suppression is needed (fallible tests return
  `Result`; `expect` is permitted in tests by `clippy.toml`).
- Risk: `rstest-bdd` step functions are not recognized as tests, so
  `clippy::expect_used` fires inside them (observed in the prior attempt).
  Severity: low. Likelihood: high. Mitigation: steps return
  `Result<(), StepError>` and use `?`; no suppression.
- Risk: the vendored `docs/rstest-bdd-users-guide.md` documents `0.5.0`, but
  `0.6.0` is current. Severity: low. Likelihood: high. Mitigation: use `0.6.0`,
  whose migration guide was read during planning; its breaking changes (one
  leading underscore normalized in implicit fixture names; MSRV 1.88; `Result`
  aliases now propagate `Err`) do not affect the planned usage.
- Risk: `Cargo.lock` is ignored by `.gitignore`, so CI resolves the newest
  semver-compatible versions and an upstream patch release could change `clap`
  help text and break `insta` help snapshots. Severity: medium. Likelihood:
  low. Mitigation: snapshot only text this crate authors plus clap's stable
  layout; record any drift as a surprise. Committing `Cargo.lock` is a
  repository policy question outside this plan and is raised in
  `Outcomes & retrospective`, not changed here.
- Risk: the library crate `mdast_check` and binary `mdast-check` produce an
  output-filename collision warning under `cargo doc`. Severity: low.
  Likelihood: medium. Mitigation: set `doc = false` on the `[[bin]]` target;
  verified in `EP-M0`.
- Risk: `StyledStr` rendering from `clap` emits ANSI escapes into the injected
  writers, making snapshots host-dependent. Severity: low. Likelihood: low.
  Mitigation: render with `to_string()` (plain text) and pin
  `ColorChoice::Never` for the in-process runner; verified in `EP-M0`.

## Progress

- [x] (2026-11-20 12:20Z) Reconnaissance completed: repository survey, ortho
  config 0.8.0 source review, prior-attempt review, dependency survey.
- [x] (2026-11-20 13:30Z) Initial draft written.
- [ ] Design review by the expert panel applied.
- [ ] Plan approved by the user.
- [ ] `EP-M0` feasibility probe (prototype; deleted before `EP-M1`).
- [ ] `EP-M1` exit-status contract and library/binary split.
- [ ] `EP-M2` `validate` command surface without a configuration file.
- [ ] `EP-M3` configuration-file layering through `ortho-config`.
- [ ] `EP-M4` documentation, tested examples, and roadmap closure.

## Surprises & discoveries

- Observation: `ortho-config` 0.8.0's subcommand helpers
  (`load_and_merge_subcommand`, `SubcmdConfigMerge::load_and_merge`,
  `load_globals_and_merge_selected_subcommand`) never consult an explicit
  configuration path. They search fixed candidate locations (current directory,
  home directory, XDG directories) and read `PREFIX_CMDS_<NAME>_*` variables
  straight from the process environment. Evidence:
  `ortho_config-0.8.0/src/subcommand/mod.rs`, `load_file_and_env_defaults`, and
  `src/subcommand/paths.rs`, `candidate_paths`, which call
  `std::env::var_os("HOME")` directly. Impact: the subcommand helpers cannot
  honour `--config`, and tests using them would depend on the developer's home
  directory. This drives decision D2.
- Observation: the generated top-level `load_from_iter` adds the explicit
  `--config-path` as a *required* discovery path, but subcommand loading does
  not reuse that machinery. Evidence: `ortho_config_macros-0.8.0`,
  `src/derive/load_impl.rs`, `build_cli_chain_tokens`. Impact: as above.
- Observation: `ortho_config::load_config_file` returns `Ok(None)` for a
  missing file rather than an error. Evidence:
  `ortho_config-0.8.0/src/file/loader.rs`, `with_cycle_detection`. Impact: a
  missing explicit `--config` must be turned into an invocation error by this
  crate (invariant `INV-explicit-config`).
- Observation: `ortho_config::is_display_request` matches only
  `DisplayHelp` and `DisplayVersion`, not
  `DisplayHelpOnMissingArgumentOrSubcommand`. Evidence:
  `ortho_config-0.8.0/src/error/helpers.rs`. Impact: a bare `mdast-check` with
  no subcommand is an invocation error (exit `2`, help on standard error),
  which is the behaviour this plan wants.
- Observation: the earlier attempt on branch
  `1-1-1-implement-the-root-cli-command-with-a-validate-subcommand` (based on
  `e725436`, far behind `main`) declared `ortho_config` but never called it,
  copied boolean flags unconditionally over lower layers, and returned exit `0`
  from a stub. Evidence: Wyvern review of an exported copy of that branch.
  Impact: this plan forbids a bespoke overlay (Architecture tolerance), tests
  that an absent boolean flag leaves the file value intact
  (`INV-absent-cli-transparent`), and refuses to report success without
  validation (decision D4).

## Decision log

- D1. Decision: build a library crate `mdast_check` (`src/lib.rs`) and a
  binary named `mdast-check`
  (`[[bin]] name = "mdast-check", path = "src/main.rs", doc = false`), and set
  `TARGET ?= mdast-check` in the Makefile. Rationale: the design's command is
  `mdast-check`; Cargo would otherwise name the binary `mdast_check`. A library
  target makes the run boundary testable in-process and lets doctests compile,
  which a binary-only crate cannot do (`docs/rust-doctest-dry-guide.md`).
  `doc = false` avoids the lib/bin documentation collision. The build-standard
  contract does not read `Cargo.toml` or the binary name, so it is unaffected.
  Date/Author: 2026-11-20, planning agent.
- D2. Decision: compose configuration layers explicitly with `ortho-config`'s
  declarative API rather than its subcommand helpers. The CLI adapter parses
  arguments with `clap`, then builds a `MergeComposer` with, in order, a
  defaults layer, one file layer per file in the `--config` file's `extends`
  chain (from `ortho_config::load_config_file_as_chain`, focused on the
  `cmds.validate` table), and a command-line layer containing only flags the
  user supplied. `ValidateConfig::merge_from_layers` (generated by
  `#[derive(OrthoConfig)]`) performs the merge. Rationale: the helpers ignore
  `--config` and read ambient state (see `Surprises & discoveries`); the
  declarative API is part of `ortho-config` 0.8.0, keeps the merge semantics in
  the library, and is pure apart from the explicit file read. Rejected
  alternative: top-level `#[derive(OrthoConfig)]` with `load_from_iter` and
  `discovery(...)`, which honours `--config` but flattens the subcommand
  structure and still reads the process environment and home directory during
  tests. Recorded as ADR 0002. Date/Author: 2026-11-20, planning agent.
- D3. Decision: in this item the configuration file is read only when named
  explicitly with `--config`. Implicit discovery (current directory, home, XDG)
  and environment-variable layers are deferred. Rationale: the roadmap requires
  only "CLI overrides taking precedence over config file values"; the design
  lists environment variables as "if exposed". Both deferred layers read
  ambient process state and would need an injected environment
  (`mockable::Env`) and a directory-discovery port; adding them later is an
  additive change (one more `push_environment` or `push_file` call) that does
  not alter the precedence of existing layers. The design document records the
  deferral. Date/Author: 2026-11-20, planning agent.
- D4. Decision: until the validation pipeline exists, the shipped binary's
  driven adapter (`validate::PendingPipeline`) fails every well-formed
  `validate` run with `error: document validation is not implemented yet` and
  exit `5`. Rationale: exit `0` would tell CI a document is valid when nothing
  was checked; `2` would blame the user for a correct invocation; `5`
  ("internal failure") is the only honest category. The placeholder is the real
  current state, not a compatibility shim, and roadmap item 1.3.2 replaces it.
  Alternative for the approver: exit `0` with a warning on standard error,
  which is friendlier for early adopters but unsafe in CI. Date/Author:
  2026-11-20, planning agent.
- D5. Decision: `--document` is command-line only and required by `clap`;
  every other setting (`schema`, `schema_format`, `format`, `fail_fast`,
  `max_errors`, `quiet`) may come from the configuration file. `--schema` is
  required after merging: if neither the command line nor the file supplies it,
  the run fails with exit `2`. Rationale: the design's configuration list names
  a default schema path but never a default document; a configured document
  would make `validate` silently check the wrong file. Date/Author: 2026-11-20,
  planning agent.
- D6. Decision: a relative `schema` path read from a configuration file is
  resolved against the directory of the file that set it (each file in an
  `extends` chain uses its own directory). A relative `--schema` on the command
  line stays relative to the current directory. Rationale: path semantics are
  hard to change after release; resolving against the file's directory makes a
  checked-in configuration work from any working directory, matching common
  tool practice. Date/Author: 2026-11-20, planning agent.
- D7. Decision: unknown keys in the `[cmds.validate]` table are an invocation
  error naming the key, while other tables are ignored. Rationale:
  `ortho-config` silently ignores unknown keys, so a typo such as
  `fail-fast = true` would be lost; rejecting it later would be a breaking
  change. Other tables stay open for future subcommands. Date/Author:
  2026-11-20, planning agent.
- D8. Decision: `--max-errors` takes a positive integer modelled as
  `NonZeroUsize`; `0` is an invocation error from either source. `--fail-fast`
  and `--quiet` are presence flags; the command line can turn them on but not
  off, so a file value of `true` can be overridden only by editing the file.
  `--schema-format` is optional; inference from the file extension is deferred
  to roadmap item 1.3.1, which owns schema loading. Rationale: a zero cap has
  no meaning; negation flags (`--no-quiet`) are not in the design's flag table
  and can be added compatibly later. Date/Author: 2026-11-20, planning agent.
- D9. Decision: help and version requests write plain text to standard output
  and exit `0`; every other `clap` error, including a missing subcommand,
  writes plain text to standard error and exits `2`. Invocation errors always
  render as human-readable text on standard error, even with `--format json`,
  because the JSON envelope belongs to roadmap item 2.2.1. Rationale: keeps the
  exit-code mapping in one place instead of calling `clap::Error::exit`, which
  would bypass injected writers. Date/Author: 2026-11-20, planning agent.
- D10. Decision: no Verus or Kani artefacts in this item (see
  `Formal verification decision`). Date/Author: 2026-11-20, planning agent.

## Outcomes & retrospective

Not yet started. To be completed at each milestone and at the end.

Open item for the approver: `Cargo.lock` is ignored by `.gitignore`. For a
binary crate, committing the lockfile is the Cargo default and would make
snapshot tests reproducible. This plan does not change it.

## Context and orientation

Inspected base revision: `788b9d444c418b117b61e9eaa05a98f277228852` on `main`.
The working branch is `1-1-1-root-cli-command-and-validate-subcommand`.

Repository layout today:

- `Cargo.toml`: package `mdast_check`, edition 2024, no runtime dependencies,
  dev-dependency `rstest = "0.28.0"`, strict `[lints.clippy]`,
  `[lints.rust] missing_docs = "deny"`, and
  `[lints.rustdoc] missing_crate_level_docs = "deny"`.
- `src/main.rs`: the only source file; a stub with `FIXME(#5)` and an
  `#[expect(clippy::print_stdout, ...)]`.
- `tests/build_standard_contract.rs` and `tests/build_standard_support/`:
  a contract that reads `.cargo/config.toml`, `rust-toolchain.toml`, the
  commands `make -n` prints for `test`, `lint`, `build`, and `release`, and the
  `setup-rust` steps of `.github/workflows/ci.yml` and `coverage-main.yml`. It
  does not read `Cargo.toml`, the binary name, or new test files.
- `Makefile`: `TARGET ?= mdast_check`; targets `build`, `release`, `test`,
  `lint` (rustdoc, Clippy, Whitaker), `fmt`, `check-fmt`, `markdownlint`,
  `spelling`, `nixie`, `workflow-contracts`.
- `.github/workflows/ci.yml`: runs `make check-fmt`, `make spelling`,
  `make workflow-contracts`, markdownlint, `make lint`, and coverage (which
  runs the tests). Verus and Kani are not installed in CI.
- `rust-toolchain.toml`: `nightly-2026-11-20` with `rustfmt` and `clippy`.
- `docs/`: the design (`mdast-schema-validation-cli-design.md`), the roadmap,
  ADR 0001 (build standard), the developers' guide, the documentation style
  guide, and vendored guides for `ortho-config`, `rstest-bdd`, `rstest`
  fixtures, dependency injection, doctests, and complexity. There is no
  `docs/users-guide.md` and no `docs/execplans/` directory on `main`.

Tools installed on the host but not configured by the repository: `kani` and
`cargo-kani` in `~/.cargo/bin`, and `prover-tools` (which can install Verus) in
`~/.local/bin`. Neither is wired into the Makefile or CI.

Upstream libraries, versions checked on crates.io on 2026-11-20: `ortho_config`
0.8.0 (current 0.11.0; 0.8.0 depends on `clap ^4` with `derive`,
`figment ^0.10`, `cap-std ^3.4.1`, and enables `serde_json` and `toml` by
default), `clap` 4.6.9, `serde` 1.0.229, `camino` 1.2.6, `thiserror` 2.0.21,
`rstest` 0.28.0, `rstest-bdd` and `rstest-bdd-macros` 0.6.0, `googletest`
0.14.4, `pretty_assertions` 1.4.1, `insta` 1.49.0, `assert_cmd` 2.2.2,
`proptest` 1.12.0, `tempfile` 3.27.0.

`ortho-config` 0.8.0 facts this plan relies on, from its published source:

- `OrthoResult<T>` is `Result<T, Arc<OrthoError>>`. `OrthoError` variants
  include `CliParsing`, `File { path, source }`, `CyclicExtends`, `Gathering`,
  `Merge`, `Validation { key, message }`, and `Aggregate`; the enum is
  `#[non_exhaustive]`.
- `#[derive(OrthoConfig)]` generates
  `merge_from_layers<'a, I: IntoIterator<Item = MergeLayer<'a>>>(layers: I)
  -> OrthoResult<Self>`;
  `MergeComposer` has `push_defaults`, `push_file(value, Option<Utf8PathBuf>)`,
  `push_environment`, `push_cli`, and `layers()`.
- `load_config_file_as_chain(&Path) -> OrthoResult<Option<FileLayerChain>>`
  loads a file and its `extends` ancestors as separate layers, detects cycles,
  and returns `Ok(None)` when the file does not exist.
- `is_display_request(&clap::Error) -> bool` is true only for help and
  version display.

Prior art: a stale earlier attempt exists on the remote branch
`1-1-1-implement-the-root-cli-command-with-a-validate-subcommand`. Its lessons
are recorded under `Surprises & discoveries`; none of its code is reused.

Documentation and skills to consult while implementing:

- `docs/mdast-schema-validation-cli-design.md` §User-facing contract, §Schema
  configuration, §Validation pipeline, §Diagnostics design.
- `docs/ortho-config-users-guide.md` (layering, `MergeComposer`,
  `cmds.<name>` convention, error variants). Where it differs from the 0.8.0
  source, the source wins.
- `docs/rstest-bdd-users-guide.md` plus the upstream `v0-6-0-migration-guide`.
- `docs/rust-testing-with-rstest-fixtures.md`,
  `docs/reliable-testing-in-rust-via-dependency-injection.md`,
  `docs/rust-doctest-dry-guide.md`,
  `docs/complexity-antipatterns-and-refactoring-strategies.md`,
  `docs/documentation-style-guide.md` (users' guide tested-example markers, ADR
  format), and `docs/developers-guide.md`.
- Skills: `rust-router` (then `domain-cli-and-daemons`, `rust-errors`,
  `rust-types-and-apis`), `hexagonal-architecture`, `rust-unit-testing`,
  `proptest`, `rust-verification`, `execplans`, `commit-message`,
  `en-gb-oxendict-style`, and `codegraph-mcp` for navigation.

## Conformance basis

Upstream artefacts, all at revision `788b9d4`:

- Terms of Reference: none exists for this repository.
- Technical design: `docs/mdast-schema-validation-cli-design.md` (status
  Proposed, dated 2026-03-07).
- Roadmap: `docs/roadmap.md` item 1.1.1.
- ADRs: ADR 0001 (build standard) constrains build flags; this plan adds
  ADR 0002 (configuration layer composition).
- Standards: `AGENTS.md`, `docs/documentation-style-guide.md`.

Requirements derived for this item:

- `REQ-flags`: parse `--document`, `--schema`, `--schema-format`, `--config`,
  `--format`, `--fail-fast`, `--max-errors`, and `--quiet` on
  `mdast-check validate` (roadmap 1.1.1; design §Proposed flags, Table 1).
- `REQ-explicit-verb`: the validation command is the explicit `validate`
  subcommand of the root command (design §Command surface).
- `REQ-stdin`: `--document -` selects standard input (design Table 1).
- `REQ-config-precedence`: configuration is loaded through `ortho-config`
  v0.8.0 and command-line values override configuration-file values, which
  override built-in defaults (roadmap 1.1.1; design §Schema configuration).
- `REQ-exit-codes`: exit codes 0 to 5 as listed in design §Exit codes.
- `REQ-early-stop`: invalid invocation or configuration stops before any
  pipeline stage runs (design §Validation stages, stage 1).
- `REQ-docs`: users' guide, design, developers' guide, and an ADR describe the
  new contract (`AGENTS.md` documentation maintenance).

Gaps between baseline and target:

- `GAP-stub`: `src/main.rs` ignores arguments and always exits `0`.
- `GAP-no-lib`: there is no library target, so nothing is testable in-process.
- `GAP-no-config`: no configuration loading exists.
- `GAP-no-users-guide`: `docs/users-guide.md` does not exist.

Trace links:

```plaintext
REQ-exit-codes -> TDD §Exit codes -> exit::ExitStatus -> EP-M1 -> exit::tests::code_table_matches_design
REQ-flags, REQ-explicit-verb, REQ-stdin -> TDD Table 1 -> cli::args
  -> EP-M2 (seven flags), EP-M3 (--config)
  -> tests/features/validate_command.feature, tests/validate_cli.rs
REQ-early-stop -> TDD §Validation stages -> cli::run -> EP-M2, EP-M3
  -> cli::run::tests::pipeline_not_called_on_invocation_error
REQ-config-precedence -> TDD §Schema configuration -> ADR 0002 -> cli::config -> EP-M3
  -> cli::config::tests (property: layering_matches_oracle)
GAP-stub, GAP-no-lib -> EP-M1, EP-M2
GAP-no-users-guide, REQ-docs -> EP-M4 -> tests/users_guide_examples.rs
```

Design deviations requiring acceptance with this plan: decisions D3 (defer
environment and discovery layers) and D4 (exit `5` placeholder). Both are
recorded in the design document in `EP-M4`.

## Verification plan

### Abstract model

Let `K = {schema, schema_format, format, fail_fast, max_errors, quiet}` be the
configurable keys. A layer is a partial function from `K` to values. There are
three layers in precedence order: `D` (defaults, defined for `format`,
`fail_fast`, and `quiet`, undefined elsewhere), `F` (the merged file chain,
where later files in an `extends` chain override earlier ones), and `C` (the
command line, defined exactly on the keys whose flags were supplied). The
resolved setting is:

```plaintext
resolve(k) = C(k) if C defines k, else F(k) if F defines k, else D(k)
```

If `resolve(schema)` is undefined the run is an invocation error.

A run maps an argument vector and the file system (read-only) to exactly one
`ExitStatus`. Stages are ordered: parse arguments, load and merge
configuration, build the validation request, call the pipeline port. Any
failure in the first three stages yields `InvalidInvocation` and the pipeline
port is never called. Otherwise the pipeline returns `Valid`, `Invalid`, or a
failure with kind `Schema`, `MarkdownParse`, or `Internal`, mapped to `Success`,
`ValidationFailed`, `SchemaFailure`, `MarkdownParseFailure`, and
`InternalFailure`. Help and version displays yield `Success` without calling
the pipeline.

The exit-code function is the fixed table `Success -> 0`,
`ValidationFailed -> 1`, `InvalidInvocation -> 2`, `SchemaFailure -> 3`,
`MarkdownParseFailure -> 4`, `InternalFailure -> 5`.

### Invariants and lemmas

- `INV-exit-table`: `ExitStatus::code` equals the design table for all six
  variants; the codes are pairwise distinct; `code(s) = 0` if and only if
  `s = Success`. Establishes `REQ-exit-codes`.
- `INV-single-outcome`: every run returns exactly one `ExitStatus` and never
  panics; the binary's process exit code equals `ExitStatus::code` of that
  value. Establishes `REQ-exit-codes`.
- `INV-stage-order`: on any invocation or configuration error, the pipeline
  port is called zero times and standard output is empty. Establishes
  `REQ-early-stop`.
- `INV-precedence`: for every key in `K`, the resolved value equals
  `resolve(k)` above. Establishes `REQ-config-precedence`.
- `INV-absent-cli-transparent`: a flag the user did not supply contributes
  nothing to `C`; in particular an absent `--fail-fast` or `--quiet` never
  overrides a file value of `true`, and an absent `--format` never overrides a
  file value of `json`.
- `INV-explicit-config`: if `--config` names a file that is missing,
  unreadable, malformed, part of an `extends` cycle, contains an unknown key in
  `[cmds.validate]`, or contains a value of the wrong type, the run exits `2`
  with a message naming the file, and the pipeline is not called.
- `INV-path-rebase`: a relative `schema` from a configuration file becomes
  `dir(file) / schema`; an absolute path is unchanged; a command-line
  `--schema` is never rebased.
- `INV-document-source`: `--document -` yields `DocumentSource::Stdin`; any
  other value yields `DocumentSource::Path` with that exact path.
- `INV-max-errors-positive`: `max_errors`, when resolved, is at least 1.
- `INV-streams`: help and version go to standard output; all diagnostics go to
  standard error.

- `L-cli-layer`: the command-line layer built from parsed arguments has key
  `k` if and only if the flag for `k` was supplied, and its value is the parsed
  value. Together with `AXIOM-ortho-merge` this yields `INV-precedence` and
  `INV-absent-cli-transparent`.
- `L-file-layer`: each file layer equals the file's `cmds.validate` table with
  `schema` rebased per `INV-path-rebase`, and is rejected per
  `INV-explicit- config` when it contains an unknown key.
- `L-request-total`: building a `ValidateRequest` from merged configuration
  either succeeds with every field typed (document source, schema path,
  optional schema format, output format, flags, optional positive cap) or fails
  only because the schema is missing.

### Axioms (trusted, not verified here)

- `AXIOM-ortho-merge`: `merge_from_layers` applies layers in the order pushed
  and, for scalar keys, a later layer's value replaces an earlier one; a key
  absent from a later layer leaves the earlier value. Source:
  `docs/ortho-config-users-guide.md` (precedence and `MergeComposer`) and
  `ortho_config` 0.8.0 `declarative::merge_value`. Depended on by
  `cli::config`. Exercised at the boundary by the `INV-precedence` property
  test, which uses the real library.
- `AXIOM-ortho-file`: `load_config_file_as_chain` parses TOML, follows
  `extends`, reports cycles as `CyclicExtends`, reports parse failures as an
  error, and returns `Ok(None)` for a missing file. Depended on by
  `cli::config`. Exercised by end-to-end and unit tests with real files in
  temporary directories.
- `AXIOM-clap`: `clap` 4 reports help and version requests as error kinds
  `DisplayHelp` and `DisplayVersion`, and all other parse failures as other
  kinds; `Utf8PathBuf` and `NonZeroUsize` value parsers reject invalid input.
  Depended on by `cli::run`.
- `AXIOM-process`: `std::process::ExitCode::from(u8)` becomes the process exit
  status on supported platforms. Depended on by `src/main.rs`. Exercised by
  `assert_cmd` tests.
- `AXIOM-serde-nonzero`: `serde` rejects `0` when deserializing
  `NonZeroUsize`. Depended on by `cli::config`.

### Methods per obligation

- Obligation: `INV-exit-table`.
  Method: exhaustive parameterized test. Rationale: the domain is six values,
  so enumerating them is a complete proof once the enumeration is shown to be
  complete. Domain: all six variants. Artefact: `src/exit.rs`, test
  `code_table_matches_design` (an `rstest` table whose expected codes are
  literals copied from the design, not computed from the implementation) and
  `codes_are_distinct_and_zero_only_for_success`, which iterates over
  `ExitStatus::ALL`. Completeness of `ALL` is enforced by a test helper that
  matches on `ExitStatus` with no wildcard arm, so adding a variant without
  extending `ALL` fails to compile. Evidence: `cargo test --lib exit::` fails
  before `ExitStatus` exists (missing symbol) and passes after. Non-vacuity:
  every variant appears as a case; negative control: temporarily swap the codes
  of `SchemaFailure` and `MarkdownParseFailure`; both table cases must fail;
  revert.
- Obligation: `INV-single-outcome`.
  Method: lints plus end-to-end tests. Rationale: panics are excluded
  statically by the denied `unwrap_used`, `expect_used`, `indexing_slicing`,
  `panic_in_result_fn`, and `unreachable` lints; the process boundary needs a
  real process. Artefact: `tests/validate_cli.rs` asserts the exact exit code
  for each reachable shipped status (`0` help and version, `2` invocation
  errors, `5` placeholder pipeline). Non-vacuity: each test also asserts the
  expected stream content, so a crash with the same code would fail.
- Obligation: `INV-stage-order` and `INV-streams`.
  Method: parameterized in-process tests with a recording fake pipeline.
  Rationale: the property is about ordering of effects inside one run; an
  in-process fake makes the call count observable. Domain: one case per
  invocation-error class (unknown flag, missing document, missing schema,
  invalid format, zero cap, missing config file, malformed config, unknown
  config key). Artefact: `src/cli/run.rs` tests using `RecordingPipeline` (a
  test-only struct with a `Cell<usize>` call counter). Evidence: each case
  asserts status `InvalidInvocation`, zero calls, empty stdout, non-empty
  stderr. Non-vacuity: a control case with a valid invocation asserts exactly
  one call; seeded fault: call the pipeline before checking the schema is
  present, which must fail the missing-schema case.
- Obligation: `INV-precedence` with `L-cli-layer` and
  `INV-absent-cli-transparent`. Method: exhaustive presence table plus property
  test against an independent oracle. Rationale: presence is finite (each key
  is absent or present in each of `F` and `C`), so a table covers every
  presence class; values vary widely (paths, enum values, booleans, positive
  integers), so a property test samples them. Domain: `rstest` table over each
  key with the four presence combinations (24 cases); `proptest` over arbitrary
  `Option<value>` for every key in both `F` and `C` at once. Artefact:
  `src/cli/config/tests.rs`, tests `presence_table` and
  `layering_matches_oracle`. The oracle is a ten-line function in the test
  module that applies `resolve(k)` per key to plain `Option` values; it does
  not call `MergeComposer`, `ortho_config`, or any production helper. The
  production path under test is the real `cli::config::resolve` with `F`
  written as TOML into a temporary directory and `C` as an argument vector, so
  the property crosses the real `clap` parse, the real file load, and the real
  `ortho-config` merge. Evidence: before `cli::config` exists the tests fail to
  compile; afterwards `cargo test --lib cli::config` passes with 256 property
  cases (the proptest default). Non-vacuity: the strategy draws `Some` and
  `None` with equal weight per key per layer, so every presence class occurs;
  the test counts cases where `C` and `F` both define a key with different
  values and asserts the count is non-zero at the end of the run. Negative
  control: change the command-line layer builder to always emit
  `fail_fast: false` when the flag is absent; `layering_matches_oracle` and the
  `fail_fast` presence rows must fail; revert.
- Obligation: `INV-explicit-config` and `L-file-layer`.
  Method: parameterized tests with real files. Domain: missing file, directory
  instead of file, malformed TOML, `extends` cycle, unknown key in
  `[cmds.validate]`, unknown top-level table (accepted), wrong-type values
  (`max_errors = "ten"`, `max_errors = 0`, `format = "xml"`). Artefact:
  `src/cli/config/tests.rs`, and end-to-end rows in `tests/validate_cli.rs`.
  Evidence: each error row asserts exit `2` and that the message names the file
  path. Non-vacuity: the accepted unknown-table row proves the unknown-key
  check is scoped and not a blanket rejection.
- Obligation: `INV-path-rebase`.
  Method: parameterized tests plus a property test. Domain: relative, absolute,
  and `..`-containing schema paths; base file and extending file in different
  directories. Artefact: `src/cli/config/tests.rs`, test
  `rebase_relative_schema_paths` and property `rebase_is_join_for_relative`.
  The oracle is `Utf8Path::join` applied by the test, compared with the
  resolved request. Non-vacuity: includes an absolute path that must not change
  and a command-line relative path that must not be rebased.
- Obligation: `INV-document-source`, `INV-max-errors-positive`.
  Method: parameterized tests on argument parsing. Domain: `-`, `./-`, a normal
  path, an empty string (rejected); `--max-errors 0`, `1`,
  `18446744073709551615`, `-1`, `ten`. Artefact: `src/cli/args.rs` tests.
- Obligation: `L-request-total`.
  Method: parameterized tests on `ValidateRequest` construction. Artefact:
  `src/cli/config/tests.rs`, test `request_requires_schema`.

### Formal verification decision

No Verus proof and no Kani harness are planned for this item (decision D10).

Verus is warranted when a small, pure, repository-owned kernel must satisfy a
property for unboundedly many inputs. This item has no such kernel. The only
non-trivial business rule, layer precedence, is implemented by `ortho-config`'s
merge; this crate owns only the construction of the layers. A Verus proof would
have to be about a model of the merge, not the code that runs, so it would
prove the oracle rather than the implementation. The property test above
instead checks the real composition, including the third-party merge, against
an independent oracle. The exit-code table is a six-element finite function,
which exhaustive enumeration already proves. Path rebasing is a single call to
`Utf8Path::join`. Kani would add bounded exhaustiveness to code that has no
arithmetic, no `unsafe`, and no state machine, so it would be ceremonial.

Candidates for formal verification in later items, recorded so they are not
forgotten: the fallback span selection of roadmap 2.1.1 (narrowest available
span is chosen and always contains the reported instance), the deterministic
error ordering and capping of roadmap 2.2.1 and 2.2.2 (the reported list is a
prefix of a total order and its length is `min(n, cap)`), and the instance-path
registration of roadmap 1.2.2 (every registered path addresses a node in the
projected JSON). Each of these is a pure, owned kernel over unbounded input.

### Suggested BDD features

Feature file: `tests/features/validate_command.feature`, bound by
`tests/validate_command_bdd.rs` with `rstest-bdd` 0.6.0. Steps drive the public
`mdast_check::cli::run` entry point in-process with a scripted fake pipeline,
so every exit code is reachable; a few scenarios are mirrored by `assert_cmd`
tests against the real binary. Linked milestones: `EP-M2` (invocation and exit
codes) and `EP-M3` (configuration).

```gherkin
Feature: Validate command invocation
  The validate command parses its flags, merges configuration, and reports a
  stable exit code for every outcome.

  Scenario: A valid invocation reaches the validation pipeline
    Given a validation pipeline that reports the document valid
    When the user runs "validate --document article.md --schema article.json"
    Then the exit code is 0
    And the pipeline received document "article.md" and schema "article.json"

  Scenario Outline: Pipeline outcomes map to stable exit codes
    Given a validation pipeline that reports <outcome>
    When the user runs "validate --document article.md --schema article.json"
    Then the exit code is <code>

    Examples:
      | outcome                  | code |
      | the document valid       | 0    |
      | the document invalid     | 1    |
      | a schema failure         | 3    |
      | a Markdown parse failure | 4    |
      | an internal failure      | 5    |

  Scenario: Standard input is selected with a dash
    Given a validation pipeline that reports the document valid
    When the user runs "validate --document - --schema article.json"
    Then the pipeline received the document from standard input

  Scenario: A missing schema is an invalid invocation
    Given a validation pipeline that reports the document valid
    When the user runs "validate --document article.md"
    Then the exit code is 2
    And standard error mentions "--schema"
    And the pipeline was not called

  Scenario: An unknown flag is an invalid invocation
    Given a validation pipeline that reports the document valid
    When the user runs "validate --document article.md --colour"
    Then the exit code is 2
    And the pipeline was not called

  Scenario: Help is printed on standard output
    Given a validation pipeline that reports the document valid
    When the user runs "validate --help"
    Then the exit code is 0
    And standard output mentions "--max-errors"
    And standard error is empty

  Scenario: The configuration file supplies the schema
    Given a configuration file setting schema "schemas/article.json"
    And a validation pipeline that reports the document valid
    When the user runs "validate --document article.md" with that configuration
    Then the exit code is 0
    And the pipeline received a schema path ending in "schemas/article.json"

  Scenario: The command line overrides the configuration file
    Given a configuration file setting format "json"
    And a validation pipeline that reports the document valid
    When the user runs "validate --document a.md --schema s.json --format human" with that configuration
    Then the pipeline received output format "human"

  Scenario: An absent flag keeps the configured value
    Given a configuration file setting fail_fast to true
    And a validation pipeline that reports the document valid
    When the user runs "validate --document a.md --schema s.json" with that configuration
    Then the pipeline received fail-fast enabled

  Scenario: A missing configuration file is an invalid invocation
    Given a validation pipeline that reports the document valid
    When the user runs "validate --document a.md --schema s.json --config missing.toml"
    Then the exit code is 2
    And standard error mentions "missing.toml"
    And the pipeline was not called

  Scenario: A misspelt configuration key is rejected
    Given a configuration file containing the key "fail-fast"
    And a validation pipeline that reports the document valid
    When the user runs "validate --document a.md --schema s.json" with that configuration
    Then the exit code is 2
    And standard error mentions "fail-fast"
```

Runner: `cargo test --test validate_command_bdd` (also run by `make test`).
Scenarios in the first block (through "Help is printed") are written red at the
start of `EP-M2`; the configuration scenarios are written red at the start of
`EP-M3`.

## Plan of work

### `EP-M0` feasibility probe (prototype, deleted)

Question: can the planned `ortho-config` 0.8.0 usage compile and behave as
assumed under this repository's lints? Work in the branch worktree after
approval. Add the runtime dependencies from `Interfaces and dependencies` to
`Cargo.toml` and a throwaway integration test `tests/probe_ortho_config.rs`
that:

1. derives `OrthoConfig`, `Deserialize`, `Serialize`, and `Default` on a
   probe `ValidateConfig` with the field types planned for the real one;
2. merges defaults, a TOML file layer from `load_config_file_as_chain`
   focused on `cmds.validate`, and a sparse command-line JSON layer through
   `MergeComposer` and `merge_from_layers`;
3. checks absent optional keys, invalid enum spellings, `max_errors = 0`, an
   unknown key (expected: silently ignored by `ortho-config`), a missing file
   (expected `Ok(None)`), and an `extends` chain's per-layer paths;
4. renders `clap` help and an error through `to_string()` and checks for ANSI
   escapes;
5. confirms `#[rstest]` with `#[gtest]` from `googletest` compiles in this
   crate, and that a `rstest-bdd` 0.6.0 scenario compiles beside `rstest`
   0.28.0.

Also run `make lint` with `[[bin]] doc = false` and an empty `src/lib.rs` to
check for the documentation collision. Go/no-go: all five checks behave as
expected and `make lint` reports no warnings from generated code; otherwise
record findings in `Surprises & discoveries`, adjust the affected decision,
and, if decision D2 cannot hold, stop under the Architecture tolerance. Delete
`tests/probe_ortho_config.rs` before committing; record the transcript in
`Artefacts and notes`. The dependency additions are kept for `EP-M1` only if go.

### `EP-M1` exit-status contract and library/binary split

Assigned: `REQ-exit-codes`, `GAP-no-lib`, decision D1.

Red: create `src/lib.rs` with only the crate-level `//!` documentation and
`pub mod exit;`, and `src/exit.rs` with the test module first:
`code_table_matches_design` (an `rstest` table of the six variants and the
literal codes from the design) and
`codes_are_distinct_and_zero_only_for_ success`. Run `cargo test --lib exit::`;
it fails to compile because `ExitStatus` does not exist.

Green: define `ExitStatus` with the six variants, `ExitStatus::ALL`,
`const fn code(self) -> u8`, and
`impl From<ExitStatus> for std::process::ExitCode`. Add the `[[bin]]` table and
set `TARGET ?= mdast-check` in the Makefile. `src/main.rs` keeps its stub in
this milestone so the plateau is coherent; the stub is replaced in `EP-M2`.

Refactor: add a doctest example on `ExitStatus::code`. Run the negative control
from `Verification plan` and revert it.

### `EP-M2` `validate` command surface without a configuration file

Assigned: `REQ-flags`, `REQ-explicit-verb`, `REQ-stdin`, `REQ-early-stop`,
`GAP-stub`, decisions D4, D5, D8, D9.

Red: write the first block of `tests/features/validate_command.feature` and its
step file `tests/validate_command_bdd.rs`; write `tests/validate_cli.rs` with
`assert_cmd` tests for `--help`, `--version`, `validate --help`, no subcommand
(exit `2`), an unknown flag (exit `2`), a missing `--schema` (exit `2`), and a
well-formed invocation (exit `5`, standard error
`error: document validation is not implemented yet`); write unit tests for
argument parsing and `run`. Run `cargo test --test validate_command_bdd` and
`cargo test --test validate_cli`; they fail to compile or fail on the stub's
behaviour.

Green, in this order:

1. `src/validate/settings.rs`: domain value types `DocumentSource`,
   `SchemaFormat`, `OutputFormat`, and `ValidateRequest` (see
   `Interfaces and dependencies`). No `clap` or `serde` here.
2. `src/validate/pipeline.rs`: the driven port `ValidationPipeline`, the
   result types `Verdict` and `PipelineFailure` with `PipelineFailureKind`, and
   `ExitStatus::from_pipeline_result`. `src/validate/pending.rs`:
   `PendingPipeline`, which returns an `Internal` failure with the placeholder
   message.
3. `src/cli/args.rs`: `clap` derive types `Cli`, `Command`, and `ValidateArgs`,
   carrying every flag except `--config`, which arrives with its behaviour in
   `EP-M3` so that no plateau accepts a file it ignores. Use explicit long
   names, no short flags except `-h` and `-V` (avoiding the derived-short
   collisions between `--schema` and `--schema-format`, and between `--format`
   and `--fail-fast`), `value_enum` for the two format enums, and value parsers
   for `DocumentSource`, `Utf8PathBuf`, and `NonZeroUsize`.
4. `src/cli/config.rs` (first cut): `ValidateConfig` with
   `#[derive(OrthoConfig, Deserialize, Serialize, Default)]`, the command-line
   layer builder (`cli_layer`, emitting only supplied flags), and `resolve`,
   which merges defaults and the command-line layer through `MergeComposer` and
   builds a `ValidateRequest`, failing with `InvocationError::MissingSchema`
   when no schema is resolved.
5. `src/cli/error.rs`: `InvocationError` (`thiserror`), boxed where payloads
   are large, with variants `Usage(Box<clap::Error>)`, `MissingSchema`, and the
   configuration variants added in `EP-M3`.
6. `src/cli/run.rs`: the `run` function (signature in
   `Interfaces and dependencies`), which parses with `Cli::try_parse_from`,
   handles display requests and errors per decision D9, resolves the request,
   calls the pipeline once, and reports pipeline failures as `error: <message>`
   on standard error. Write failures on standard output while printing help
   yield `InternalFailure`; a write failure on standard error leaves the
   already-decided status unchanged, because no channel remains to report it.
7. `src/main.rs`: the composition root. Lock `std::io::stdout()` and
   `std::io::stderr()`, build `Streams`, call `mdast_check::cli::run` with
   `std::env::args_os()` and `PendingPipeline`, and return the
   `std::process::ExitCode`. Delete the stub and its `#[expect]` in the same
   commit.

Refactor: extract helpers to keep `run` under the complexity threshold, add
`insta` snapshots of root help, `validate --help`, and the missing-schema error
text (`src/cli/snapshots/`), and run the seeded fault for `INV-stage-order`.

### `EP-M3` configuration-file layering through `ortho-config`

Assigned: `REQ-config-precedence`, `GAP-no-config`, decisions D2, D3, D6, D7.

Red: add the configuration scenarios to the feature file; add unit tests
`presence_table`, `layering_matches_oracle`, `rebase_relative_schema_paths`,
`rebase_is_join_for_relative`, and the `INV-explicit-config` table to
`src/cli/config/tests.rs`; add end-to-end rows for a missing `--config` and a
configuration-supplied schema reaching the placeholder pipeline (exit `5`,
proving the file was read). Run the focused commands; they fail because
`--config` is not yet acted on.

Green: add `--config <PATH>` to `ValidateArgs` (a `Utf8PathBuf`, never part of
the command-line layer because it selects the file layer rather than setting a
key). Split `src/cli/config.rs` into `src/cli/config/mod.rs` (merge and request
building), `src/cli/config/file.rs` (loading the `extends` chain, focusing
`cmds.validate`, rejecting unknown keys, and rebasing `schema`), and
`src/cli/config/tests.rs`. Push one file layer per chain element between the
defaults and command-line layers. Map `OrthoError` values and the missing-file
case to `InvocationError::Config { path, reason }`, where `reason` is a short,
crate-authored description and the source error is kept as `#[source]`.

Refactor: confirm each file is under 400 lines; run the negative control for
`INV-precedence` and revert it.

### `EP-M4` documentation, tested examples, and roadmap closure

Assigned: `REQ-docs`, `GAP-no-users-guide`.

1. Create `docs/users-guide.md` for release `0.1.0` (unreleased, early
   adopter). Sections, with imperative headings: install and build; run the
   first validation; choose the document and schema (`-` for standard input);
   configure defaults with a file (`[cmds.validate]` keys, relative-path rule,
   unknown-key rule, `extends`); understand precedence; interpret exit codes
   (table); output streams and accessibility (plain text, no colour, help on
   standard output, diagnostics on standard error); current limitations
   (validation not yet implemented, exit `5`; `--format`, `--fail-fast`,
   `--max-errors`, `--quiet` accepted but not yet acting; no environment
   variables or implicit discovery). Precede every fenced example with
   `<!-- tested-example: <id> -->`.
2. Add `tests/support/users_guide.rs`, a shared loader that extracts marked
   fences from `docs/users-guide.md` and fails on an unmarked fence, an
   unterminated fence, a missing identifier, or a duplicate identifier; and
   `tests/users_guide_examples.rs`, which runs each `sh` example (one
   `mdast-check` command per fence, split on whitespace, no shell) against the
   binary with `assert_cmd` in a temporary directory seeded with the guide's
   `toml` examples, asserting the exit code recorded for that identifier in the
   test's table. The BDD step file reuses the same loader for the configuration
   example. The loader's own rules get unit tests with inline Markdown.
3. Write `docs/adrs/adr-0002-explicit-configuration-layer-composition.md`
   (decision D2, with D3 as a consequence), following the ADR 0001 format.
4. Update `docs/mdast-schema-validation-cli-design.md`: in §Schema
   configuration, the 1.1.1 precedence (command line over file over defaults),
   the `[cmds.validate]` shape, the relative-path and unknown-key rules, and
   the deferral of environment variables and discovery, linking ADR 0002; in
   §Exit codes, the placeholder exit `5` until roadmap 1.3.2; in §Module
   boundaries, the realized `cli`, `validate`, and `exit` modules and the
   `ValidationPipeline` port.
5. Update `docs/developers-guide.md` with: the module layout and the
   hexagonal boundary (only `src/cli/` may use `clap` and `ortho_config`); the
   `ValidationPipeline` port and how later items replace `PendingPipeline`; the
   `Streams` injection rule; how to add a configuration key (field on
   `ValidateConfig`, flag on `ValidateArgs`, row in the presence table and
   generator in the property test, users' guide entry); and the tested-example
   loader and its reuse policy (all user-facing docs use it; no copies). Also
   update the Cranelift paragraph to record that the repository now has
   behaviour tests and the measurement is due (the measurement itself is out of
   scope).
6. Update `README.md`: a short usage section linking the users' guide, and
   remove the stale `make typecheck` mention.
7. Mark roadmap item 1.1.1 and its three sub-bullets `[x]` in
   `docs/roadmap.md` only after all gates pass.

## Milestones and plateaus

- Identifier and outcome: `EP-M0`, a recorded go or no-go for decision D2
  with no committed code.
  - Requirements and gaps: none discharged; de-risks `REQ-config-precedence`.
  - Acceptance evidence: probe transcript in `Artefacts and notes`.
  - Conformance check: design unchanged unless the probe falsifies D2, in
    which case the plan becomes `BLOCKED`.
  - Recovery: delete the probe file; `git checkout Cargo.toml` if no-go.
  - Remaining gaps: everything.
  - Compatibility decision: none.
- Identifier and outcome: `EP-M1`, a library crate exposing the exit-status
  contract and a binary named `mdast-check` (still the stub).
  - Requirements and gaps: `REQ-exit-codes` (contract only), `GAP-no-lib`.
  - Acceptance evidence: `exit::tests` pass; `make build` produces
    `target/debug/mdast-check`.
  - Conformance check, one line per item:
    - exit table matches design §Exit codes;
    - module layout matches design §Module boundaries;
    - no assumption falsified;
    - public interface limited to `exit::ExitStatus`;
    - no trust boundary or persisted format;
    - trace link `REQ-exit-codes -> EP-M1` current.
  - Recovery: revert the milestone commit.
  - Remaining gaps: the stub, the CLI, configuration, docs.
  - Compatibility decision: none.
- Identifier and outcome: `EP-M2`, the shipped binary parses the full
  `validate` flag set, rejects invalid invocations with exit `2`, prints help
  and version with exit `0`, and reports the placeholder with exit `5`; the
  in-process runner maps every pipeline outcome to its code.
  - Requirements and gaps: `REQ-flags`, `REQ-explicit-verb`, `REQ-stdin`,
    `REQ-early-stop`, `GAP-stub`.
  - Acceptance evidence: first-block BDD scenarios, `tests/validate_cli.rs`,
    unit tests, and help snapshots pass.
  - Conformance check, one line per item:
    - flags match design Table 1;
    - `clap` confined to `src/cli/`;
    - D4 placeholder behaviour as approved;
    - public interface as listed;
    - no persisted format;
    - trace links current.
  - Recovery: revert the milestone commits; `EP-M1` remains a valid plateau.
  - Remaining gaps: `--config` does not exist until `EP-M3`, so the plateau
    never accepts a configuration file and silently ignores it.
  - Compatibility decision: none.
- Identifier and outcome: `EP-M3`, `--config` files supply defaults with the
  command line taking precedence, through `ortho-config`.
  - Requirements and gaps: `REQ-config-precedence`, `GAP-no-config`.
  - Acceptance evidence: configuration BDD scenarios, presence table,
    property tests, rebasing tests, and end-to-end rows pass; negative control
    rejected.
  - Conformance check, one line per item:
    - precedence matches design §Schema configuration as amended by D3;
    - `ortho_config` confined to `src/cli/config/`;
    - `AXIOM-ortho-merge` held under the property test;
    - no unapproved interface or dependency;
    - the configuration-file shape is a new user-facing format, approved by
      this plan;
    - trace links current.
  - Recovery: revert the milestone commits; `EP-M2` remains valid.
  - Remaining gaps: documentation.
  - Compatibility decision: none (first release of the format).
- Identifier and outcome: `EP-M4`, documentation matches behaviour and every
  users' guide example is executed by the suite; roadmap 1.1.1 is done.
  - Requirements and gaps: `REQ-docs`, `GAP-no-users-guide`.
  - Acceptance evidence: `tests/users_guide_examples.rs` passes; markdown
    gates pass; roadmap ticked.
  - Conformance check: design document and ADR 0002 updated for D2, D3, D4,
    D6, and D7; no remaining unrecorded deviation.
  - Recovery: documentation commits revert independently.
  - Remaining gaps: roadmap 1.1.2 onwards.
  - Compatibility decision: none.

## Concrete steps

Working directory for every command: the repository root of the branch
worktree. Run gates sequentially, never in parallel, and enable `pipefail` so a
failing gate cannot appear to pass through `tee`:

```sh
set -o pipefail
BRANCH=$(git branch --show-current)
make check-fmt 2>&1 | tee /tmp/check-fmt-mdast-check-$BRANCH.out
make lint 2>&1 | tee /tmp/lint-mdast-check-$BRANCH.out
make test 2>&1 | tee /tmp/test-mdast-check-$BRANCH.out
```

For milestones that change Markdown (`EP-M4`, and this plan's own updates),
also run:

```sh
make fmt 2>&1 | tee /tmp/fmt-mdast-check-$BRANCH.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mdast-check-$BRANCH.out
make nixie 2>&1 | tee /tmp/nixie-mdast-check-$BRANCH.out
```

`make markdownlint` runs `make spelling` first. If `make spelling` rewrites
`typos.toml`, commit the regenerated file in its own commit.

Focused commands per milestone:

```plaintext
EP-M0  cargo test --test probe_ortho_config -> all probe cases print their observations; then delete the file
EP-M1  Red:   cargo test --lib exit::            -> error[E0412]: cannot find type `ExitStatus`
       Green: cargo test --lib exit::            -> 2 tests (8 cases) pass
EP-M2  Red:   cargo test --test validate_cli     -> stub exits 0 for every case; assertions on codes 2 and 5 fail
       Red:   cargo test --test validate_command_bdd -> fails to compile: cannot find `mdast_check::cli::run`
       Green: both commands pass; cargo test --lib cli:: passes
EP-M3  Red:   cargo test --lib cli::config       -> presence rows for file-supplied keys fail (--config ignored)
       Green: cargo test --lib cli::config       -> pass, including 256 property cases per property
       Green: cargo test --test validate_command_bdd --test validate_cli -> pass
EP-M4  Red:   cargo test --test users_guide_examples -> fails: docs/users-guide.md not found
       Green: cargo test --test users_guide_examples -> pass
```

Commit after each green step and each refactor that passes the three gates. Use
the `commit-message` skill; subjects in the imperative mood, at most 50
characters, bodies wrapped at 72 columns.

## Validation and acceptance

Behaviour a human can verify after `EP-M4`, from the repository root after
`make build`:

```plaintext
$ target/debug/mdast-check --help; echo "exit=$?"
(usage listing the validate command)
exit=0

$ target/debug/mdast-check validate --document README.md; echo "exit=$?"
error: no schema given; pass --schema or set `schema` under [cmds.validate] in a configuration file
exit=2

$ target/debug/mdast-check validate --document README.md --schema s.json --config missing.toml; echo "exit=$?"
error: cannot read configuration file missing.toml: file not found
exit=2

$ target/debug/mdast-check validate --document README.md --schema s.json; echo "exit=$?"
error: document validation is not implemented yet
exit=5
```

The exact wording of these messages is fixed by `insta` snapshots in `EP-M2` and
`EP-M3`; the lines above are the intended wording and may be refined during
implementation, in which case this section is updated.

Red-Green-Refactor evidence is recorded per milestone in `Artefacts and notes`,
using the focused commands above.

Quality criteria:

- Tests: `make test` passes, including the new unit, property, BDD, snapshot,
  end-to-end, and users' guide example tests, plus the existing build-standard
  contract.
- Verification: every obligation in `Verification plan` has its artefact
  passing and its negative control recorded as rejected.
- Lint and format: `make check-fmt` and `make lint` pass with no new
  `#[expect]` other than any recorded in `Decision log`.
- Documentation: `make markdownlint` and `make nixie` pass.
- Performance: none required for this item.
- Security: the only untrusted inputs are command-line arguments and the
  configuration file named by the user; both are parsed by typed parsers and
  never executed. No new network or privileged effect.

## Idempotence and recovery

All steps are re-runnable. Tests write only to `tempfile` temporary
directories, which are removed on drop. If a milestone fails part-way, reset to
the previous milestone commit (`git reset --hard <commit>` on the branch, after
confirming with `git status` that nothing unrelated is lost) and repeat.
Negative controls are applied as uncommitted edits and reverted with
`git checkout -- <file>`; confirm with `git diff --stat` that the tree is clean
before committing. Delete the `EP-M0` probe file before the first `EP-M1`
commit. Remove stray `*.snap.new` files with `cargo insta reject` or by
deleting them.

## Artefacts and notes

Planning-time observations (reconnaissance, not implementation evidence):

```plaintext
$ curl crates.io .../ortho_config/versions
0.11.0 2026-11-15, 0.10.0, 0.9.0, 0.8.0 2026-03-05 (rust_version 1.88)
$ grep -n "pub fn is_display_request" -A5 ortho_config-0.8.0/src/error/helpers.rs
matches!(err.kind(), ErrorKind::DisplayHelp | ErrorKind::DisplayVersion)
$ sed -n '/fn load_file_and_env_defaults/,/^}/p' ortho_config-0.8.0/src/subcommand/mod.rs
let paths = candidate_paths(prefix);            # fixed locations only
let env_provider = Env::prefixed(&env_prefix)   # process environment
```

Milestone evidence is appended here during implementation.

## Interfaces and dependencies

Runtime dependencies (caret requirements, `[dependencies]`):

- `ortho_config = "0.8.0"` (default features `serde_json` and `toml`): the
  declarative merge, file loading, and display-request helper. Re-exported
  `ortho_config::serde_json` is used for layer values, so no direct
  `serde_json` dependency is added.
- `clap = { version = "4.6.9", features = ["derive"] }`: argument parsing.
- `serde = { version = "1.0.229", features = ["derive"] }`: configuration
  (de)serialization.
- `camino = { version = "1.2.6", features = ["serde1"] }`: UTF-8 paths.
- `thiserror = "2.0.21"`: `InvocationError`.

Development dependencies (`[dev-dependencies]`, in addition to
`rstest = "0.28.0"`):

- `rstest-bdd = "0.6.0"`, `rstest-bdd-macros = "0.6.0"`: behavioural tests.
- `googletest = "0.14.4"`: matchers (`assert_that!`, `expect_that!`).
- `pretty_assertions = "1.4.1"`: readable equality diffs.
- `insta = "1.49.0"`: help and error-text snapshots.
- `assert_cmd = "2.2.2"`: end-to-end process tests.
- `proptest = "1.12.0"`: precedence and rebasing properties.
- `tempfile = "3.27.0"`: temporary configuration files and directories.

No `mockable` dependency is added, because no environment-dependent behaviour
is introduced (decision D3). No `eyre` dependency is added: `main` returns an
`ExitCode` and all error text is crate-authored.

Module layout and public items at the end of `EP-M3`:

```plaintext
src/main.rs                 composition root (binary `mdast-check`)
src/lib.rs                  crate docs; pub mod cli, exit, validate
src/exit.rs                 ExitStatus
src/validate/mod.rs         application core re-exports
src/validate/settings.rs    DocumentSource, SchemaFormat, OutputFormat, ValidateRequest
src/validate/pipeline.rs    ValidationPipeline, Verdict, PipelineFailure, PipelineFailureKind
src/validate/pending.rs     PendingPipeline
src/cli/mod.rs              driving adapter re-exports: run, Streams
src/cli/args.rs             Cli, Command, ValidateArgs (clap)
src/cli/run.rs              run, Streams
src/cli/error.rs            InvocationError
src/cli/config/mod.rs       ValidateConfig (OrthoConfig), resolve
src/cli/config/file.rs      file-chain loading, focusing, rebasing, key checking
src/cli/config/tests.rs     unit and property tests
```

Key signatures:

```rust
// src/exit.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitStatus {
    Success,
    ValidationFailed,
    InvalidInvocation,
    SchemaFailure,
    MarkdownParseFailure,
    InternalFailure,
}

impl ExitStatus {
    pub const ALL: [Self; 6] = [/* every variant, in code order */];
    #[must_use]
    pub const fn code(self) -> u8 { /* exhaustive match */ }
}

impl From<ExitStatus> for std::process::ExitCode { /* ExitCode::from(code) */ }

// src/validate/settings.rs
pub enum DocumentSource { Stdin, Path(camino::Utf8PathBuf) }
pub enum SchemaFormat { Json, Markdoc }
pub enum OutputFormat { Human, Json }

pub struct ValidateRequest {
    pub document: DocumentSource,
    pub schema: camino::Utf8PathBuf,
    pub schema_format: Option<SchemaFormat>,
    pub format: OutputFormat,
    pub fail_fast: bool,
    pub max_errors: Option<std::num::NonZeroUsize>,
    pub quiet: bool,
}

// src/validate/pipeline.rs
pub enum Verdict { Valid, Invalid }
pub enum PipelineFailureKind { Schema, MarkdownParse, Internal }
pub struct PipelineFailure { pub kind: PipelineFailureKind, pub message: String }

pub trait ValidationPipeline {
    /// Runs schema loading, parsing, and validation for one request.
    fn validate(&self, request: &ValidateRequest) -> Result<Verdict, PipelineFailure>;
}

impl ExitStatus {
    #[must_use]
    pub const fn from_pipeline_result(result: &Result<Verdict, PipelineFailure>) -> Self;
}

// src/cli/run.rs
pub struct Streams<'a> {
    pub stdout: &'a mut dyn std::io::Write,
    pub stderr: &'a mut dyn std::io::Write,
}

pub fn run<I, T>(args: I, streams: &mut Streams<'_>, pipeline: &dyn ValidationPipeline) -> ExitStatus
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone;
```

`SchemaFormat` and `OutputFormat` gain `clap::ValueEnum` and `serde`
implementations in the CLI adapter through thin adapter enums or
`#[cfg_attr]`-free wrapper conversions, so the core module stays free of `clap`
and `serde`; if the probe shows the wrappers add more than about 40 lines, the
alternative of deriving `serde` (not `clap`) on the core enums is taken and
recorded in `Decision log`.

Reuse policy:

- `ExitStatus` is the only place exit codes are defined. Roadmap 1.1.2 maps
  its error taxonomy onto `ExitStatus` and must not introduce a second table.
- `ValidationPipeline` is the only boundary between the CLI adapter and the
  validation stages. Roadmap items 1.2.1 to 1.3.2 implement it in new
  `validate` submodules and replace `PendingPipeline` in `src/main.rs`; they
  must not call `clap` or `ortho_config`.
- `Streams` is the only route to standard output and standard error. Later
  renderers (2.1.2, 2.2.1) write through it.
- `cli::config` owns all configuration loading. New configuration keys are
  added there, never parsed ad hoc elsewhere.
- `tests/support/users_guide.rs` is the shared loader for tested examples in
  all user-facing documents.

## Revision note

- 2026-11-20: Initial draft from reconnaissance of the repository, the
  `ortho-config` 0.8.0 source, the stale prior attempt, and the vendored
  guides. Awaiting design review and user approval.
