# 1.1.1 Implement the root CLI command with a `validate` subcommand

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & discoveries`, `Decision log`,
`Outcomes & retrospective`, `Conformance basis`, and `Verification plan` must
be kept up to date as work proceeds.

Status: DRAFT (second design review applied; awaiting user approval)

Branch: `1-1-1-root-cli-command-and-validate-subcommand`

Roadmap item: `1.1.1` in `docs/roadmap.md`.

## Purpose / big picture

`mdast-check` is a command-line interface (CLI) that will validate the Markdown
Abstract Syntax Tree (MDAST) of a Markdown document against a JSON Schema
document. Today the binary is a stub: `src/main.rs` prints
`Hello from MDAST Check!` and exits with status 0 whatever arguments it is
given. No configuration is read, no text is localizable, and no exit code
carries meaning.

This plan delivers the command shell that every later roadmap item plugs into.
After the change, a user can run:

```plaintext
mdast-check validate --document docs/article.md --schema schema/article.json
```

and observe:

- every flag in the design's flag table (`--document`, `--schema`,
  `--schema-format`, `--config`, `--format`, `--fail-fast`, `--max-errors`,
  `--quiet`) is parsed, type-checked, and listed in `--help`;
- configuration is layered through `ortho_config` 0.11.0 with one spelling
  per setting across sources: built-in defaults, then a TOML file named by
  `--config` (table `[cmds.validate]`), then environment variables
  (`MDAST_CHECK_CMDS_VALIDATE_<KEY>`), then the command line, each later source
  overriding the earlier ones;
- mistakes in a configuration source (unknown keys, wrong types, keys in the
  wrong table, misspelt environment variables) are reported against the file or
  variable that contains them;
- help text, argument-parsing errors, and every other message the command
  prints come from a Fluent message catalogue selected by the user's locale
  (gettext-style: `LANGUAGE`, `LC_ALL`, `LC_MESSAGES`, `LANG`), falling back to
  English; this release ships the English catalogue, and adding a language
  means adding one catalogue file;
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
- *Layer*: one source of configuration values (defaults, each configuration
  file, the environment, the command line). Layers merge in precedence order; a
  later layer wins for every key it defines.
- *Locale*: a language identifier such as `en` or `ja-JP` that selects which
  message catalogue is used. *Fluent*: the Mozilla localization format used by
  `ortho_config`'s localization support; a *catalogue* is a `.ftl` file of
  `message-id = text` entries. *Pseudo-locale*: a fake language whose catalogue
  is generated mechanically from the English one, used in tests to prove every
  message is localizable.
- *Injected environment*: an object implementing `ortho_config::EnvSource`
  and `ortho_config::ScanEnvSource`, through which all environment reads
  happen. Production passes `ortho_config::ProcessEnv`; tests pass
  `ortho_config::MapEnv`, so no test reads or changes the real environment.

Because the validation pipeline does not exist yet, a well-formed `validate`
invocation in the shipped binary reaches a placeholder driven adapter that
reports `error: document validation is not implemented yet` on standard error
and exits `5`. This is deliberate (decision D4): it never claims a document is
valid when nothing was checked. Exit codes `1`, `3`, and `4`, and `0` for a
validated document, are exercised through the same public entry point with an
injected test pipeline, and become reachable in the shipped binary as roadmap
items 1.2.1 to 1.3.2 land. A named end-to-end tripwire test and a new
acceptance bullet on roadmap item 1.3.2 ensure the placeholder is removed.

`--format`, `--fail-fast`, `--max-errors`, and `--quiet` are parsed, merged,
and carried in the resolved settings, but their behaviour belongs to roadmap
items 2.1.2, 2.2.1, and 2.2.2, which own output rendering. The users' guide
states this plainly.

This plan closes the design's open questions of how configuration is layered
for a subcommand (decision D2, proposed ADR 0002) and where the locale is
resolved and how catalogues are owned (decision D15, proposed ADR 0003).

## Constraints

- Implementation begins only after the user explicitly approves this plan.
- Follow `AGENTS.md` throughout: en-GB-oxendict spelling in comments, docs, and
  the English catalogue; every module starts with a `//!` comment; public items
  carry `///` docs with examples; no file over 400 lines; functions within the
  Clippy thresholds in `clippy.toml` (cognitive complexity 9, 4 arguments, 70
  lines); every `assert!`, `assert_eq!`, and `assert_ne!` carries a diagnostic
  message.
- Do not weaken lint configuration. `Cargo.toml` denies, among others,
  `print_stdout`, `print_stderr`, `expect_used`, `unwrap_used`,
  `indexing_slicing`, `missing_const_for_fn`, `must_use_candidate`, `shadow_*`,
  `allow_attributes`, `missing_docs`, and `result_large_err`. `make lint` also
  runs the Whitaker Dylint suite, whose installed lints include
  `no_std_fs_operations`, `no_expect_outside_tests`,
  `test_must_not_have_example`, `module_max_lines`,
  `module_must_have_inner_docs`, `rstest_helper_should_be_fixture`,
  `bumpy_road_function`, `conditional_max_n_branches`, and
  `function_attrs_follow_docs`. Output goes through injected `std::io::Write`
  sinks, never `print!`. Tests that touch files use `cap_std::fs_utf8`, never
  `std::fs`; catalogues and documents are embedded with `include_str!`.
  Suppression is a last resort and uses item-scoped
  `#[expect(lint, reason = "...")]` only.
- Tightening lint configuration is in scope: `clippy.toml`
  `disallowed-methods` gains `std::env::var`, `var_os`, `vars`, `vars_os`,
  `args`, `args_os`, and `current_dir`, and
  `ortho_config::LocalizedParse::try_parse_localized`, with one item-scoped
  `#[expect(clippy::disallowed_methods, reason = ...)]` at the composition root.
- Remove the `FIXME(#5)` stub and its `#[expect(clippy::print_stdout, ...)]`
  in `src/main.rs` in the same commit that replaces the stub.
- All environment reads go through one injected environment object. Only the
  composition root (`src/main.rs`) constructs `ortho_config::ProcessEnv` and
  reads the process arguments. No code path may call `ortho_config` loaders or
  helpers that read process state: the generated `load`, `load_from_iter`,
  `compose_layers`, and `compose_layers_from_iter`; `SubcmdConfigMerge` and
  `SelectedSubcommandMerge` loaders; `ConfigFilePolicy` and
  `ConfigDiscoveryBuilder`; `CsvEnv`; or `LocalizedParse::try_parse_localized`.
  Do not use `clap`'s `env = ...` attribute. Do not opt into `ortho_config`
  profiles. Tests must not mutate the process environment or working directory;
  end-to-end tests may use `assert_cmd` with `Command::env_clear`,
  `Command::env`, and `Command::current_dir`.
- Keep the application core (`src/app/`) free of `clap`, `ortho_config`,
  Fluent, filesystem, and terminal types. `clap`, `ortho_config`, Fluent, and
  the catalogues are used only in `src/cli/`. Core value types may derive
  `serde` traits, because a data-shape derive carries no I/O.
- Use `camino::Utf8PathBuf` for paths. Non-UTF-8 input on the command line or
  in a configuration environment variable is an invocation error (exit `2`).
- Use `ortho_config` with a caret requirement on `0.11.0`, as the roadmap and
  design require. Its minimum supported Rust version is 1.89.0; the repository's
  `nightly-2026-11-20` toolchain satisfies it.
- Do not change `tests/build_standard_contract.rs`,
  `tests/build_standard_support/`, `.cargo/config.toml`, `rust-toolchain.toml`,
  `.gitignore`, or the CI workflows. Changing the Makefile is limited to
  `TARGET ?= mdast-check` (decision D1).
- Do not implement Markdown parsing, schema loading, validation, output
  rendering, quiet suppression, fail-fast behaviour, error capping, implicit
  configuration-file discovery, a `--locale` flag, or a shipped translation.
- Do not introduce the full `thiserror` error taxonomy of roadmap item 1.1.2.
  This plan introduces only the invocation error type it needs and a
  provisional pipeline-failure enum; 1.1.2 replaces the latter.
- Dependency changes are limited to those listed in
  `Interfaces and dependencies`. Any other dependency requires escalation.
- No source-API compatibility machinery. The crate is pre-1.0 and has no
  release tag, so interfaces change together with their callers.
- Do not edit `typos.toml` by hand; use `typos.local.toml` for exceptions.
  If `make spelling` regenerates `typos.toml` for reasons unrelated to this
  work, commit it on a separate base branch and stack this branch on it.
- `docs/ortho-config-users-guide.md` is vendored from `leynos/ortho-config`
  tag `v0.11.0`; do not edit its text except to re-vendor a later tag.

If satisfying the objective requires violating a constraint, do not proceed.
Document the conflict in `Decision log` and escalate.

## Tolerances (exception triggers)

The comparison base is `c009f8c473e991034bfe5496ad5b92789e696875`, the branch
head after the prerequisite commits (vendored `ortho-config` v0.11.0 guide;
design and roadmap retargeted to 0.11.0). The roadmap wording change that names
the environment layer (made during planning) is counted. Counts exclude this
ExecPlan, any other file under `docs/execplans/`, the vendored
`docs/ortho-config-users-guide.md`, generated `insta` snapshot files
(`**/snapshots/*.snap`), and committed `proptest-regressions/` files; their
number still counts towards the file limit. All line limits measure net lines
(additions minus deletions):

```sh
git diff --numstat c009f8c -- . ':(exclude)docs/execplans' \
  ':(exclude)docs/ortho-config-users-guide.md' \
  ':(exclude)**/*.snap' ':(exclude)**/proptest-regressions/**'
```

Classification is by whole file: `src/**`, `tests/**/*.rs`,
`tests/features/*.feature`, and `locales/**/*.ftl` (runtime text) are code;
`Cargo.toml`, `Makefile`, `clippy.toml`, and `typos.local.toml` are
configuration; `docs/**` (except the exclusions above) and `README.md` are
documentation.

- Files: if implementation changes more than 50 non-ExecPlan files (snapshot
  and regression files included), stop and escalate.
- Code: if code changes exceed 3,100 net lines, stop and escalate.
- Configuration: if configuration changes exceed 80 net lines, stop and
  escalate.
- Documentation: if documentation changes exceed 1,000 net lines, stop and
  escalate.
- Interface: if a public item beyond those in `Interfaces and dependencies`
  is needed, or a listed signature must change materially, stop and escalate.
- Dependencies: if any crate beyond those listed in
  `Interfaces and dependencies` is needed, stop and escalate.
- Architecture: if `ortho_config`'s public API (`MergeComposer`,
  `merge_from_layers`, `load_config_file_as_chain`, `sanitize_value`,
  `ScanEnvSource`, `FluentLocalizer` with `disable_defaults` and
  `with_consumer_bundle`, `try_parse_localized_with_matches`) cannot express
  decisions D2 and D15, stop and escalate rather than writing a bespoke merge
  or localizer.
- Iterations: if a focused test still fails after 4 fix attempts, stop and
  escalate.
- Time: if a milestone takes more than 6 hours of active work, stop and
  escalate.
- Ambiguity: if multiple valid interpretations exist and the choice materially
  affects the user-facing contract (flag names, configuration keys,
  environment-variable names, accepted value spellings, message identifiers,
  exit codes, message wording fixed by snapshots), stop and present options
  with trade-offs.

Each category is enforced independently. Spare capacity in one category never
offsets another.

Budget rationale. Production source is estimated at 1,230 lines: library root
40, run outcome 80, application core 200, CLI adapter 855 (arguments 120, run
and reporting 170, configuration merge 110, file layers 100, environment layer
90, value checks shared by all sources 60, locale resolution 90, catalogues,
localizer construction, help template, and message rendering 115), placeholder
pipeline 25, main 30. The English catalogue is about 90 lines (help, crate
messages, `clap` error texts, help headings). Tests are estimated at 1,650
lines: unit tests within modules 560 (outcome table, argument parsing, layer
presence enumerations, the 64-row covering array and precedence property, file
and environment rows, value checks, run ordering and write failures,
environment-access recording, locale table), argument-robustness property with
the locale metamorphic check 130, localization tests 220 (required-identifier
set and snapshot, pseudo-locale generation, completeness, forbidden
identifiers, leaked-English check, localized output), end-to-end 230, BDD steps
and feature 290, tested-example loader and runner 220. The total of 2,970 plus
about 5% gives 3,100. Configuration is about 55 lines (`Cargo.toml`, one
Makefile line, the `clippy.toml` additions, spelling exceptions); 80 absorbs
feature tables. Documentation is about 800 lines (users' guide 310, ADR 0002
120, ADR 0003 110, design document 90, developers' guide 150, README 25,
roadmap 10); 1,000 leaves a fifth in reserve. History: the first draft budgeted
2,300 code lines for `ortho_config` 0.8.0; the first design review lowered it
to 2,200; the 0.11.0 revision raised it to 2,900 for approved scope growth
(localization, environment layer); the second design review raised it to 3,100
for localization completeness (owning `clap` error texts and help headings) and
a handmade environment layer, which are corrections found in review, not missed
scope during execution. No completed ExecPlan exists on `main` to compare with.

## Risks

- Risk: `#[derive(OrthoConfig)]` emits code that trips this repository's
  strict lints (a private `clap::Parser`, public loaders, items marked
  `#[allow(dead_code, reason = ...)]` while `allow_attributes` is denied).
  Severity: high. Likelihood: medium. Mitigation: `EP-M0` compiles the derive
  under `make lint` (Clippy and Whitaker). Clippy normally skips code expanded
  from external macros; if a lint still fires, isolate the derived type in one
  module with one item-scoped `#[expect(..., reason = ...)]` per lint, recorded
  in `Decision log`; if impossible, escalate.
- Risk: the new `disallowed-methods` entries fire inside macro expansions of
  test dependencies (`insta`, `proptest`, `rstest-bdd`). Severity: medium.
  Likelihood: low. Mitigation: `EP-M0` probe; if they do, keep only the entries
  that do not, record why, and rely on the environment-access recording test
  for the rest.
- Risk: the generated `merge_from_layers` deserializes only the merged value,
  so a type error in one source cannot be attributed to it. Severity: high.
  Likelihood: certain. Mitigation: decision D7 checks each layer before it is
  pushed, with the same value checks for every source.
- Risk: `clap` English leaks into localized output: the help template's
  headings, the generated `help` subcommand, `[possible values: ...]`
  annotations, and the tail of errors ("For more information, try '--help'."),
  none of which a localizer is asked for. Severity: medium. Likelihood: certain
  without countermeasures. Mitigation: decision D17 (help template and headings
  from the catalogue, help subcommand disabled, possible values written into
  argument help, error tail replaced); a leaked-English test with the
  pseudo-locale; any residual that `EP-M0` shows cannot be removed is recorded
  in ADR 0003 and the users' guide.
- Risk: Fluent wraps interpolated values in Unicode bidirectional isolation
  marks (U+2068, U+2069). Severity: medium. Likelihood: high. Mitigation: build
  consumer bundles with
  `fluent_bundle::concurrent::FluentBundle:: new_concurrent` and
  `set_use_isolating(false)`, passed through `with_consumer_bundle`;
  `disable_defaults()` keeps `ortho_config`'s isolating default bundles out
  (decision D15).
- Risk: a missing catalogue entry silently falls back to `clap`'s stock
  English text. Severity: medium. Likelihood: medium. Mitigation: the
  required-identifier set is checked against every catalogue, and the
  pseudo-locale test fails on any English left in the output.
- Risk: an `ortho_config` upgrade renames identifiers, so catalogue entries
  stop being requested and English returns. Severity: medium. Likelihood:
  medium over time. Mitigation: dead-entry check and an `insta` snapshot of the
  required-identifier set make the change visible; the caret requirement stays
  on 0.11.
- Risk: strict environment checking breaks a CI job that sets an unknown or
  differently spelt `MDAST_CHECK_CMDS_VALIDATE_*` variable, or an older binary
  meets a variable for a key added later. Severity: medium. Likelihood: medium.
  Mitigation: the accepted value spellings are documented and tested in the
  users' guide; the error names the variable and lists the accepted names;
  strictness and version skew are stated in the users' guide and ADR 0002.
- Risk: Whitaker's `no_std_fs_operations` and `no_expect_outside_tests` fire
  in tests, `rstest-bdd` steps, and shared helpers. Severity: medium.
  Likelihood: high. Mitigation: `cap-std` in tests, `Result`-returning steps
  and helpers, `rstest` fixtures; probed in `EP-M0`.
- Risk: `rstest` drops `#[expect]` attributes on generated tests, while
  `AGENTS.md` forbids `#[allow]`. Severity: low. Likelihood: medium.
  Mitigation: design tests so no suppression is needed.
- Risk: the vendored `docs/rstest-bdd-users-guide.md` documents `0.5.0`, but
  `0.6.0` is current. Severity: low. Likelihood: high. Mitigation: use `0.6.0`;
  its migration guide's breaking changes do not affect the plan.
- Risk: `Cargo.lock` is ignored, so CI resolves the newest compatible
  versions and a `clap` release could change help layout. Severity: medium.
  Likelihood: medium. Mitigation: help is checked by message content and flag
  presence, not whole-screen snapshots; decision D11.
- Risk: configuration-file paths are canonicalized and non-UTF-8 directories
  converted lossily by `ortho_config`. Severity: low. Likelihood: medium.
  Mitigation: decision D6 documents canonical-directory rebasing; tested with a
  symbolic link.
- Risk: lib/bin documentation collision and ANSI escapes in rendered help.
  Severity: low. Likelihood: medium. Mitigation: `doc = false` on `[[bin]]`;
  `ColorChoice::Never`; verified in `EP-M0`.

## Progress

- [x] (2026-11-20 12:20Z) Reconnaissance for `ortho_config` 0.8.0.
- [x] (2026-11-20 13:30Z) Initial draft written and committed.
- [x] (2026-11-20 14:10Z) First expert-panel review applied (proceed with
  conditions).
- [x] (2026-11-20 14:40Z) Direction received: use `ortho_config` 0.11.0 for
  layered configuration with localized help; vendor the v0.11.0 guide first.
- [x] (2026-11-20 14:45Z) Prerequisite: `docs/ortho-config-users-guide.md`
  replaced with the `v0.11.0` tag's guide (commit `5b61faf`); design and
  roadmap retargeted to 0.11.0 with a localization sub-task (commit `c009f8c`).
- [x] (2026-11-20 15:20Z) Reconnaissance of `ortho_config`,
  `ortho_config_macros`, and `ortho_l10n` 0.11.0 and the v0.9.0 to v0.11.0
  migration guides; plan revised (commit `5e58b8b`).
- [x] (2026-11-20 16:10Z) Second expert-panel review (localization and
  contracts; verification, failure modes, and scope) applied: both "proceed
  with conditions"; roadmap 1.1.1 now names the environment layer.
- [ ] Plan approved by the user.
- [ ] `EP-M0` feasibility probe (prototype; deleted before `EP-M1`).
- [ ] `EP-M1` run-outcome contract and library/binary split.
- [ ] `EP-M2` `validate` command surface and the English catalogue.
- [ ] `EP-M3` layered configuration: file and environment.
- [ ] `EP-M4` locale selection and localization completeness.
- [ ] `EP-M5` documentation, tested examples, and roadmap closure.

## Surprises & discoveries

- Observation: in both 0.8.0 and 0.11.0, the subcommand loaders have no input
  for an explicit file path; they discover files from a base directory, the
  home directory, and XDG directories. The root derived loader honours a renamed
  `--config` flag but loads top-level keys only. Evidence:
  `ortho_config-0.11.0/src/subcommand/sources.rs` (`SubcommandFileContext`),
  `src/subcommand/paths.rs`,
  `ortho_config_macros-0.11.0/src/derive/build/ config_flag.rs`. Impact:
  decision D2 stands.
- Observation: `ConfigFilePolicy` reads ambient state even with only a
  command-line selector: with no explicit path it falls through to system,
  user, and project discovery, and `ConfigDiscoveryBuilder::build` uses
  `ProcessEnv` and the process working directory unless told otherwise; for an
  explicit path it simply calls `load_config_file_as_chain`. Evidence:
  `src/discovery/policy.rs`, `src/discovery/builder.rs`. Impact: D2 uses
  `load_config_file_as_chain` directly and only when `--config` is given.
- Observation: 0.11.0 adds process-free injection: `EnvSource`,
  `ScanEnvSource`, `ProcessEnv`, `MapEnv`. `CsvEnv::with_source` is
  process-free but matches prefixes case-insensitively, trims and splits values
  on commas, types scalars through Figment (so a path `2024` becomes a number),
  converts non-UTF-8 values lossily, and loses the original variable name.
  Evidence: `src/env_source.rs`, `src/csv_env/injected.rs`,
  `src/csv_env/mod.rs`. Impact: D3 builds the environment layer directly from
  `ScanEnvSource::scan()`.
- Observation: `LocalizedParse::try_parse_localized_from` discards the
  `ArgMatches` and already localizes `clap` errors;
  `try_parse_localized_with_matches` returns both. `CliValueExtractor` is
  generated only for a struct deriving both `clap` and `OrthoConfig` with at
  least one `cli_default_as_absent` field. Evidence:
  `src/localizer/clap_command/parse.rs`,
  `ortho_config_macros-0.11.0/src/derive/generate/ortho_impl.rs`. Impact: D2a
  builds the command-line layer with `sanitize_value` over `Option` fields; D9
  does not localize errors twice.
- Observation: `FluentLocalizer::try_build` fails with `UnsupportedLocale` for
  any language other than `en` or `ja` unless `disable_defaults()` is called;
  bundled defaults are matched by language subtag, so `en-GB` users would get
  `en-US` `clap` error texts; the bundled bundles isolate placeables.
  `with_consumer_bundle` takes
  `fluent_bundle::concurrent::FluentBundle< Arc<FluentResource>>`, which
  `ortho_config` does not re-export. Evidence:
  `ortho_l10n-0.11.0/src/fluent.rs`, `src/lib.rs`. Impact: D15 disables
  defaults, owns every `clap-error-*` text, and adds `fluent-bundle`.
- Observation: the localized parse requests `about`, `long_about`, `usage`,
  `version`, `long_version`, `after_help`, and `after_long_help` for each
  command and `help`, `long_help`, and `value_name` for each argument; defining
  `usage` or `version` would replace `clap`'s generated usage line or the
  package version with fixed text. Evidence:
  `src/localizer/clap_command/mod.rs`. Impact: a required-identifier set and
  forbidden identifiers (D15).
- Observation: `localize_clap_error_with_command` translates only the first
  line of an error, keeps `clap`'s tail verbatim, and passes only the argument
  and value to the catalogue for `ValueValidation` errors, dropping a custom
  parser's reason. Evidence: `src/localizer/clap_error.rs`. Impact: D8 moves
  value checks out of `clap` parsers; D17 replaces the error tail.
- Observation: `#[ortho_config(localization_base = "mdast-check")]` would
  generate `mdast-check-args-*` constants, while the runtime path for the
  subcommand is `mdast-check-validate-args-*`. Evidence:
  `ortho_config_macros-0.11.0/src/derive/generate/localization/mod.rs`. Impact:
  the derive's identifier constants are not used; the recorded identifiers are
  authoritative.
- Observation: much of upstream `docs/cli-localization-design.md` (an
  environment locale resolver, embedded-catalogue localizer, coverage
  reporting) is marked proposed and absent from the 0.11.0 code. Impact: this
  crate owns locale resolution and coverage tests.
- Observation (0.8.0 era, still relevant): the earlier attempt on branch
  `1-1-1-implement-the-root-cli-command-with-a-validate-subcommand` never called
  `ortho_config`, let absent booleans override file values, and returned exit
  `0` from a stub. Impact: no bespoke overlay; absent-flag and `null` controls;
  no false success.

## Decision log

- D1. Decision: build a library crate `mdast_check` (`src/lib.rs`) and a
  binary named `mdast-check`
  (`[[bin]] name = "mdast-check", path = "src/main.rs", doc = false`), set
  `TARGET ?= mdast-check` in the Makefile, and fix
  `#[command(name = "mdast-check", bin_name = "mdast-check")]`. Rationale: the
  design's command is `mdast-check`; a library target makes the run boundary
  testable in-process and lets doctests compile; `doc = false` avoids the
  lib/bin documentation collision; the fixed name makes usage text and message
  identifiers independent of `argv[0]`. Date/Author: 2026-11-20, planning agent.
- D2. Decision: compose configuration layers explicitly with `ortho_config`'s
  declarative API. The CLI adapter parses arguments with
  `try_parse_localized_with_matches`, then builds a `MergeComposer` with, in
  order: a defaults layer; when `--config` is given, one file layer per element
  of `load_config_file_as_chain(path)` (a relative `--config` is resolved
  against the injected environment's `current_dir`; `Ok(None)` becomes a
  missing-file invocation error), each focused on `cmds.validate` and checked
  (D7); an environment layer (D3); and a command-line layer.
  `ValidateConfig::merge_from_layers` performs the merge. Rationale:
  `ortho_config` 0.11.0 still offers no subcommand loader that honours an
  explicit file; `ConfigFilePolicy` reads ambient state; the declarative API
  keeps merge semantics in the library and is process-free. Rejected
  alternatives: the subcommand `_with_env` loaders (no explicit file); the root
  derived loader with `config_cli_long = "config"` (flattens the subcommand and
  its file shape); `ConfigFilePolicy` (ambient discovery). Recorded as ADR 0002
  with a revisit trigger: re-evaluate when `ortho_config` gains explicit-file
  support for subcommands. Date/Author: 2026-11-20, planning agent; revised for
  0.11.0 and second review.
- D2a. Decision: every configurable field of `ValidateArgs` is an `Option<T>`
  with no `clap` default, and the command-line layer is
  `ortho_config::sanitize_value(&args)` (which drops `None` values), with
  `#[serde(skip)]` on `document` and `config`. Rationale: `CliValueExtractor`
  is unavailable for a plain `clap` struct, and `Option` fields make "an absent
  flag contributes nothing" a property of the type and the library's sanitizer
  rather than of handwritten code. Date/Author: 2026-11-20, second review.
- D3. Decision: configuration sources are built-in defaults, a file named
  explicitly by `--config`, environment variables, and the command line, in
  that precedence order. The environment layer is built directly from the
  injected `ScanEnvSource::scan()`: only variables whose names start with the
  exact, case-sensitive prefix `MDAST_CHECK_CMDS_VALIDATE_` are considered; the
  remainder must be one of `SCHEMA`, `SCHEMA_FORMAT`, `FORMAT`, `FAIL_FAST`,
  `MAX_ERRORS`, or `QUIET` (upper case); values must be UTF-8 and are parsed
  with the same value checks as the command line (D8). Any other variable with
  the prefix, including `MDAST_CHECK_CMDS_VALIDATE_DOCUMENT`, is an invocation
  error naming the variable and listing the accepted names. Implicit file
  discovery and a configuration-path environment variable are deferred; when
  discovery is added, an explicit `--config` disables it, and the name
  `.mdast-check.toml` is reserved now in ADR 0002. Rationale: matches the
  design's precedence and `ortho_config`'s orthographic naming
  (`[cmds.validate]` keys, `MDAST_CHECK_CMDS_VALIDATE_<KEY>` variables,
  `--<key>` flags); the direct scan avoids `CsvEnv`'s case-insensitive
  matching, comma splitting, Figment scalar typing, lossy conversion, and lost
  variable names. Date/Author: 2026-11-20, planning agent; revised for 0.11.0
  and second review.
- D4. Decision: until the validation pipeline exists, the shipped binary's
  driven adapter (`pipeline::PendingPipeline`) fails every well-formed
  `validate` run with the catalogue message
  `mdast-check-msg-validation-unavailable` ("document validation is not
  implemented yet") and exit `5`. Rationale: exit `0` would tell CI an
  unchecked document is valid; `2` would blame a correct invocation; `5` is the
  only honest non-zero code, although the design defines it as an unexpected
  failure. The users' guide states that `5` is not transient. A tripwire test,
  `placeholder_pipeline_exits_5_until_1_3_2`, and an acceptance bullet on
  roadmap 1.3.2 carry the removal obligation. Date/Author: 2026-11-20, planning
  agent; confirmed by design review.
- D5. Decision: `--document` is command-line only and required; every other
  setting (`schema`, `schema_format`, `format`, `fail_fast`, `max_errors`,
  `quiet`) may come from any source. `--schema` is required after merging.
  `--config` selects the file layers and is never itself a configuration key.
  Rationale: the design names a default schema path but never a default
  document. Date/Author: 2026-11-20, planning agent.
- D6. Decision: a relative `schema` path read from a configuration file is
  resolved against the canonical directory of the file that set it; a relative
  `schema` from the environment or command line stays relative to the current
  directory; an empty `schema` from any source is an invocation error.
  Date/Author: 2026-11-20, planning agent; amended after review.
- D7. Decision: every configuration source is checked on its own before
  merging. Each file in the `extends` chain is deserialized into
  `#[serde(deny_unknown_fields)]` structs (root: only `cmds`; `cmds`: only
  `validate`; `validate`: only the six keys, each `Option`), then each present
  value passes the shared value checks (D8). The environment layer is checked
  as in D3. Failures are invocation errors naming the file or variable and the
  key. Only checked layers are re-serialized, omitting absent keys, and pushed.
  Rationale: `ortho_config` ignores unknown keys and reports type errors only
  after merging, without a source. Date/Author: 2026-11-20, design review;
  extended to the environment layer.
- D8. Decision: value checks live in one module shared by all sources rather
  than in `clap` value parsers, so every source accepts the same spellings and
  every rejection has a catalogue message: `schema` must be a non-empty UTF-8
  path other than `-` (a file named `-` remains reachable as `./-`);
  `max_errors` must be a positive integer (`0` is rejected); `format` is
  `human` or `json`; `schema_format` is `json` or `markdoc`; booleans are
  `true` or `false` (environment `1` and `0` are rejected with a message
  listing the accepted spellings). On the command line `--fail-fast` and
  `--quiet` follow `ortho_config`'s `--flag[=BOOL]` convention
  (`value_name = "BOOL"`, `num_args = 0..=1`, `require_equals = true`,
  `default_missing_value = "true"`): a bare flag sets `true`, `--quiet=false`
  overrides a configured `true`, and `--quiet false` is rejected. `clap` still
  parses integers, so `--max-errors ten` is a localized `clap` value error.
  Repeating a flag is an invocation error. `--schema-format` inference from
  file extensions is deferred to 1.3.1. Date/Author: 2026-11-20, planning
  agent; revised for second review.
- D9. Decision: help and version requests write localized plain text to
  standard output and exit `0`; every other `clap` error, including a missing
  subcommand, is written to standard error and exits `2` (decided from
  `clap::Error::kind`, which localization preserves). Errors are localized
  once, by the localized parse. Invocation errors render as human-readable text
  on standard error even with `--format json`, because the JSON envelope
  belongs to 2.2.1. A failed write to standard output while printing help or
  version yields exit `5`; a failed write to standard error leaves the decided
  status unchanged. Do not use `arg_required_else_help`. Date/Author:
  2026-11-20, planning agent; revised for localization and second review.
- D10. Decision: no Verus or Kani artefacts in this item (see
  `Formal verification decision`). Date/Author: 2026-11-20, planning agent;
  confirmed by both design reviews.
- D11. Decision for the approver: whether to commit `Cargo.lock` (a separate
  pull request). The plan works either way. Recommendation: commit it, as Cargo
  recommends for binaries. Date/Author: 2026-11-20, design review.
- D12. Decision: `--schema -` (and `schema = "-"` from any source) is an
  invocation error, reserving `-` for the document. Date/Author: 2026-11-20,
  design review.
- D13. Decision: name the application core `src/app/`, the placeholder
  adapter's module `src/pipeline/`, and the outcome type `RunOutcome`.
  Rationale: the design reserves `validate` for the validator wrapper;
  `RunOutcome` avoids confusion with `std::process::ExitStatus`. Date/Author:
  2026-11-20, design review.
- D14. Decision: the pipeline port is provisional, with input split into
  `ValidationInput` and `ReportOptions` and failures in a small
  `PipelineFailure` enum without text, carrying
  `const fn outcome(&self) -> RunOutcome`. The CLI renders every message from
  catalogue identifiers. Roadmap 1.1.2 replaces `PipelineFailure`; 1.3.2 adds
  normalized failures to `Verdict::Invalid`; 2.1.2 and 2.2.1 add renderers
  behind the CLI's single reporting function. Date/Author: 2026-11-20, design
  review.
- D15. Decision: localize all user-facing text of this item through
  `ortho_config` 0.11.0, with catalogues owned by this crate.
  - Locale resolution (gettext-style, from the injected environment): take
    the first non-empty of `LC_ALL`, `LC_MESSAGES`, `LANG` as the effective
    locale; if it is `C` or `POSIX` (after stripping any `.codeset` and
    `@modifier`), use English and ignore `LANGUAGE`; otherwise consider the
    colon-separated entries of a non-empty `LANGUAGE` first, then the
    effective locale; normalize each candidate (`ja_JP.UTF-8@euro` becomes
    `ja-JP`; modifiers are ignored and documented); choose the first whose
    language has a catalogue in the injected `Catalogues`; otherwise English.
    Resolution is total, never fails, and never builds a file path from a
    locale value.
  - Localizer construction: `FluentLocalizer::builder(locale)
    .disable_defaults().with_consumer_bundle(bundle).try_build()`, where the
    bundle is a `fluent_bundle::concurrent::FluentBundle` created with
    `new_concurrent` and `set_use_isolating(false)` from the chosen catalogue;
    if construction fails, use the English catalogue the same way; if that
    fails (prevented by tests), use `NoOpLocalizer`, and crate messages render
    as their identifiers.
  - Catalogues: `locales/<language>/messages.ftl`, embedded with
    `include_str!`; this crate's catalogues define every `clap-error-*` text
    the run can reach, so no bundled `ortho_config` text (en-US spelling,
    isolating placeables) is used.
  - Identifiers: those `ortho_config` derives for the command tree
    (`mdast-check-about`, `mdast-check-validate-about`,
    `mdast-check-validate-args-<arg>-help`, `...-value_name` for arguments
    that take values), `clap-error-<kind>`, and crate messages under
    `mdast-check-msg-*`; the subcommand names `msg` and `error` are reserved
    so derived identifiers cannot collide. Catalogues must not define
    `*-usage`, `*-version`, or `*-long_version`, which would replace `clap`'s
    generated usage line and the package version.
  - Rationale: the library leaves locale resolution to applications;
    resolving it at the boundary from the injected environment keeps tests
    hermetic; disabling defaults gives one source of truth per language and
    lets any language (including test pseudo-locales) be selected. Recorded
    as ADR 0003. Date/Author: 2026-11-20, planning agent; revised for second
    review.
- D16. Decision for the approver: ship only the English catalogue
  (`locales/en/messages.ftl`, en-GB Oxford spelling) in this item; prove that
  every message is localizable with a test-only pseudo-locale generated
  mechanically from the English catalogue (private-use language tag `qaa`;
  every message value wrapped and marked, placeables preserved), selected
  through the same `Catalogues` injection that production uses. The roadmap
  annotation states that 1.1.1 delivers localizable output with English
  shipped, and a follow-up roadmap item for the first translation is proposed
  to the approver, not added. Rationale: a shipped translation needs a fluent
  reviewer and the `localization-docs` set (style guide, glossary, translator's
  guide); a generated pseudo-locale cannot drift from English and exercises
  every identifier. Alternative: ship a Japanese catalogue now, including all
  `clap-error-*` texts (because defaults are disabled), with the documentation
  set; about 150 code and 300 documentation lines more. Date/Author:
  2026-11-20, planning agent; revised for second review.
- D17. Decision: keep `clap`'s English out of localized output.
  `disable_help_subcommand(true)` on the root; explicit `help` and `version`
  arguments (`ArgAction::Help`, `ArgAction::Version`) whose descriptions are
  catalogue messages; a help template whose headings ("Usage:", "Options:",
  "Commands:") are catalogue messages applied after localization and before
  parsing (through `parse_localized_command` with the prepared command if the
  `_with_matches` entry point cannot accept it); `hide_possible_values(true)`
  with accepted values written into each argument's catalogue help; and `clap`
  errors rendered as the localized first line followed by a catalogue
  `mdast-check-msg-try-help` line instead of `clap`'s English tail. If `EP-M0`
  shows any of these cannot be achieved with `clap` 4 and `ortho_config`
  0.11.0, the residual English is recorded in ADR 0003 and the users' guide,
  and the leaked-English test lists it as an explicit allowance. Date/Author:
  2026-11-20, second review.

## Outcomes & retrospective

Not yet started. To be completed at each milestone and at the end.

First design review (2026-11-20, against the 0.8.0 draft): "proceed with
conditions"; adopted typed per-source layers, banned ambient loaders, the split
provisional port, module renames, Whitaker and `cap-std` handling, chain-order
and `null` controls, an argument-robustness property, placeholder removal
obligations, the discovery reservation, `--schema -`, and the `Cargo.lock`
question. Declined: dropping `googletest`, `pretty_assertions`, and `proptest`.

Second design review (2026-11-20, against the 0.11.0 revision): both reviewers
"proceed with conditions". Adopted: `load_config_file_as_chain` instead of
`ConfigFilePolicy`; `try_parse_localized_with_matches` and `sanitize_value`
instead of `try_parse_localized_from` and `CliValueExtractor`; a direct
environment scan instead of `CsvEnv`; `disable_defaults` with crate-owned
`clap-error-*` texts and an unconditional `fluent-bundle` dependency; locale
resolution against the injected catalogues, with `LANGUAGE` support; a
generated pseudo-locale; a required-identifier set with forbidden identifiers
and a snapshot; D17 (no `clap` English leaks); value checks shared by all
sources; `clippy.toml` disallowed methods and an environment-access recording
test; a 64-row pairwise covering array for precedence; a locale metamorphic
check; and naming the environment layer in roadmap 1.1.1. Considered and kept
as one roadmap item: splitting localization or the environment layer into
separate items (both were directed by the user).

## Context and orientation

Inspected base revision: `788b9d444c418b117b61e9eaa05a98f277228852` on `main`.
The working branch is `1-1-1-root-cli-command-and-validate-subcommand`, stacked
on `refresh-typos-config` (pull request #143), which regenerates `typos.toml`
only. Branch commits so far: the plan and its revisions, the vendored
`ortho-config` v0.11.0 user's guide (`5b61faf`), and the design and roadmap
retargeting (`c009f8c`).

Repository layout today:

- `Cargo.toml`: package `mdast_check`, edition 2024, no runtime dependencies,
  dev-dependency `rstest = "0.28.0"`, strict `[lints.clippy]`,
  `[lints.rust] missing_docs = "deny"`, and
  `[lints.rustdoc] missing_crate_level_docs = "deny"`.
- `clippy.toml`: complexity thresholds and `disallowed-methods` for
  `std::env::set_var`, `remove_var`, and `set_current_dir`.
- `src/main.rs`: the only source file; a stub with `FIXME(#5)`.
- `tests/build_standard_contract.rs` and `tests/build_standard_support/`: a
  contract over `.cargo/config.toml`, `rust-toolchain.toml`, `make -n` output,
  and CI `setup-rust` steps. It does not read `Cargo.toml`, the binary name, or
  new test files, and does not call `std::env`.
- `Makefile`: `TARGET ?= mdast_check`; targets `build`, `release`, `test`,
  `lint` (rustdoc, Clippy, Whitaker), `fmt`, `check-fmt`, `markdownlint` (which
  runs `spelling` first), `spelling`, `nixie`, `workflow-contracts`.
- `.github/workflows/ci.yml`: `make check-fmt`, `make spelling`,
  `make workflow-contracts`, markdownlint, `make lint`, and coverage (which
  runs the tests). Verus and Kani are not installed in CI.
- `.gitignore` ignores `Cargo.lock`.
- `rust-toolchain.toml`: `nightly-2026-11-20` with `rustfmt` and `clippy`.
- `docs/`: the design, the roadmap, ADR 0001 (build standard), the
  developers' guide, the documentation style guide (no localization section),
  and vendored guides, including `ortho-config-users-guide.md` at v0.11.0.
  There is no `docs/users-guide.md`.

Tools installed on the host but not configured by the repository: `kani`,
`cargo-kani`, and `prover-tools` (which can install Verus). Whitaker lint
libraries are under `~/.local/share/whitaker/`.

Upstream libraries, versions checked on crates.io on 2026-11-20: `ortho_config`
0.11.0 (MSRV 1.89.0; depends on `ortho_l10n` 0.11.0, `clap ^4` with `derive`,
`env`, and `string`, `figment ^0.10`, `cap-std ^4`, `camino ^1`, `toml ^1`;
default features `serde_json` and `toml`; re-exports `serde_json`, `toml`,
`langid!`, `LanguageIdentifier`, `FluentValue`, and the localization API),
`ortho_l10n` 0.11.0 (`fluent-bundle ^0.16`, `fluent-syntax ^0.12`,
`unic-langid ^0.9`), `fluent-bundle` 0.16, `clap` 4.6.9, `serde` 1.0.229,
`camino` 1.2.6, `thiserror` 2.0.21, `cap-std` 4.0.3, `rstest` 0.28.0,
`rstest-bdd` and `rstest-bdd-macros` 0.6.0, `googletest` 0.14.4,
`pretty_assertions` 1.4.1, `insta` 1.49.0, `assert_cmd` 2.2.2, `proptest`
1.12.0, `tempfile` 3.27.0.

`ortho_config` 0.11.0 facts this plan relies on, from its published source:

- `OrthoResult<T>` is `Result<T, Arc<OrthoError>>`; `OrthoError` is
  `#[non_exhaustive]` with variants including `CliParsing`,
  `File { path, source }`, `CyclicExtends`, `Gathering`, `Merge`, and
  `Aggregate`.
- `#[derive(OrthoConfig)]` generates `merge_from_layers`; `MergeComposer` has
  `push_defaults`, `push_file(value, Option<Utf8PathBuf>)`, `push_environment`,
  `push_cli`, and `layers()`; `merge_value` lets a later scalar, including
  `null`, replace an earlier one.
- `load_config_file_as_chain(&Path) -> OrthoResult<Option<FileLayerChain>>`
  returns `(value, Utf8PathBuf)` pairs from root ancestor to the named file,
  `extends` stripped and paths canonicalized; `Ok(None)` for a missing file;
  errors for a directory, a missing `extends` parent, a cycle, and `.json`
  without the `json5` feature.
- `sanitize_value` serializes a value and removes `None` entries.
- `EnvSource` (`get`, `current_dir`, `home_fallback`, `config_dir_fallback`),
  `ScanEnvSource` (`scan`), `ProcessEnv`, and `MapEnv` (`new`, `with_var`,
  `with_current_dir`; implements both traits). `Arc<dyn Sub>` upcasts to
  `Arc<dyn Super>` on Rust 1.86 or later.
- Localization: `FluentLocalizer::builder(locale)` with `disable_defaults`,
  `with_consumer_bundle(fluent_bundle::concurrent::FluentBundle<
  Arc<FluentResource>>)`,
  and `try_build`;
  `LocalizedParse::try_parse_localized_with_matches(iter, &dyn Localizer)`
  returning the value and `ArgMatches`; `parse_localized_command`; `clap`
  errors localized to `clap-error-<kind>` with the first line translated and
  the tail kept; identifiers from `message_id_for` (hyphen-joined, lower-case
  segments, underscores kept, for example
  `mdast-check-validate-args-schema_format-help`).
- `is_display_request(&clap::Error)` is true only for help and version.

Prior art: the stale branch
`1-1-1-implement-the-root-cli-command-with-a-validate-subcommand`; lessons are
recorded under `Surprises & discoveries`; no code is reused.

Documentation and skills to consult while implementing:

- `docs/mdast-schema-validation-cli-design.md` §User-facing contract, §Schema
  configuration, §Validation pipeline, §Diagnostics design.
- `docs/ortho-config-users-guide.md` (v0.11.0): "Give each subcommand its own
  settings", "Switch a boolean off from the command line", "Resolve discovery
  and merging without process state", "Handle errors at the application
  boundary", "Localize help and parse failures together". Upstream
  `docs/cli-localization-design.md` and
  `docs/localizable-rust-libraries-with-fluent.md` at the `v0.11.0` tag explain
  the identifier scheme; sections marked proposed are not implemented.
- `docs/rstest-bdd-users-guide.md` plus the upstream `v0-6-0-migration-guide`
  in `leynos/rstest-bdd`.
- `docs/rust-testing-with-rstest-fixtures.md`,
  `docs/reliable-testing-in-rust-via-dependency-injection.md`,
  `docs/rust-doctest-dry-guide.md`,
  `docs/complexity-antipatterns-and-refactoring-strategies.md`,
  `docs/documentation-style-guide.md`, and `docs/developers-guide.md`.
- Skills: `rust-router` (then `domain-cli-and-daemons`, `rust-errors`,
  `rust-types-and-apis`), `hexagonal-architecture`, `rust-unit-testing`,
  `proptest`, `rust-verification`, `addressing-whitaker-findings`,
  `localization-docs` (only if decision D16 ships a translation), `execplans`,
  `commit-message`, `en-gb-oxendict-style`, and `codegraph-mcp`.

## Conformance basis

Upstream artefacts at the branch head after planning:

- Terms of Reference: none exists for this repository.
- Technical design: `docs/mdast-schema-validation-cli-design.md` (status
  Proposed), naming `ortho-config` v0.11.0 and localized help.
- Roadmap: `docs/roadmap.md` item 1.1.1, with the environment layer and the
  localization sub-task named.
- ADRs: ADR 0001 (build standard); this plan adds ADR 0002 (configuration
  layer composition) and ADR 0003 (locale resolution and catalogues).
- Standards: `AGENTS.md`, `docs/documentation-style-guide.md`.

Requirements derived for this item:

- `REQ-flags`: parse the eight flags on `mdast-check validate` (roadmap
  1.1.1; design Table 1).
- `REQ-explicit-verb`: the validation command is the explicit `validate`
  subcommand (design §Command surface).
- `REQ-stdin`: `--document -` selects standard input (design Table 1).
- `REQ-config-precedence`: configuration is loaded through `ortho-config`
  v0.11.0 from a `--config` file and `MDAST_CHECK_CMDS_VALIDATE_*` variables;
  the command line overrides the environment, which overrides files, which
  override defaults (roadmap 1.1.1; design §Schema configuration).
- `REQ-l10n`: command help and argument-parsing errors are localized through
  `ortho-config` v0.11.0 (roadmap 1.1.1; design §Design drivers).
- `REQ-exit-codes`: exit codes 0 to 5 (design §Exit codes).
- `REQ-early-stop`: invalid invocation or configuration stops before any
  pipeline stage runs (design §Validation stages).
- `REQ-docs`: users' guide, design, developers' guide, and ADRs describe the
  new contract (`AGENTS.md`).

Gaps between baseline and target:

- `GAP-stub`: `src/main.rs` ignores arguments and always exits `0`.
- `GAP-no-lib`: there is no library target.
- `GAP-no-config`: no configuration loading exists.
- `GAP-no-l10n`: no text is localizable.
- `GAP-no-users-guide`: `docs/users-guide.md` does not exist.

Trace links:

```plaintext
REQ-exit-codes -> TDD §Exit codes -> outcome::RunOutcome -> EP-M1
  -> outcome::tests::exit_codes_match_design
REQ-flags, REQ-explicit-verb, REQ-stdin -> TDD Table 1 -> cli::args
  -> EP-M2 (seven flags), EP-M3 (--config)
  -> tests/features/validate_command.feature, tests/validate_cli.rs
REQ-early-stop -> TDD §Validation stages -> cli::run -> EP-M2, EP-M3
  -> cli::run::tests::pipeline_not_called_on_invocation_error
REQ-config-precedence -> TDD §Schema configuration -> ADR 0002 -> cli::config -> EP-M3
  -> cli::config::tests::{precedence_covering_array, precedence_matches_oracle}
REQ-l10n -> TDD §Localization -> ADR 0003 -> cli::l10n -> EP-M2, EP-M4
  -> cli::l10n::tests::{required_identifiers_resolve, pseudo_locale_leaks_no_english}
GAP-stub, GAP-no-lib -> EP-M1, EP-M2
GAP-no-users-guide, REQ-docs -> EP-M5 -> tests/users_guide_examples.rs
```

Design deviations requiring acceptance with this plan: D3 (discovery deferred),
D4 (exit `5` placeholder), D12 (`--schema -` reserved), D13 (module naming),
D16 (English-only shipped catalogue), and any residual English recorded under
D17. All are recorded in the design document in `EP-M5`; D4 also adds an
acceptance bullet to roadmap 1.3.2.

## Verification plan

### Abstract model

Let `K = {schema, schema_format, format, fail_fast, max_errors, quiet}` be the
configurable keys. A layer is a partial function from `K` to values. Layers in
precedence order: `D` (defaults: `format = human`, `fail_fast = false`,
`quiet = false`), `F_1 .. F_n` (one per file in the `extends` chain, root
ancestor first; `n = 0` without `--config`), `E` (the environment, defined on
keys whose `MDAST_CHECK_CMDS_VALIDATE_<KEY>` variable is set), and `C` (the
command line, defined on keys whose flags were supplied). Write `F` for the
chain folded so that later files win. Then:

```plaintext
resolve(k) = C(k) if defined, else E(k) if defined, else F(k) if defined, else D(k)
```

If `resolve(schema)` is undefined the run is an invocation error. Each `F_i`,
`E`, and `C` is admitted only if every value passes the shared value checks
(D7, D8).

Locale: from the injected values of `LANGUAGE`, `LC_ALL`, `LC_MESSAGES`, and
`LANG`, and the set `A` of languages with an injected catalogue,
`locale = choose(candidates, A)` per D15, which always yields a language in `A`
(English is always in `A`). Every message the run prints is
`lookup(catalogue(locale), id, args)`.

A run maps an argument vector, an injected environment, an injected set of
catalogues, and the read-only file system to exactly one `RunOutcome`. Stages:
resolve the locale and build the localizer; parse arguments; load and check
each file layer; build and check the environment layer; check the command-line
layer; merge; build the validation input; call the pipeline port. Any failure
from parsing onwards before the pipeline yields `InvalidInvocation`, and the
pipeline is never called. The pipeline returns `Valid`, `Invalid`, or a failure
of kind `Schema`, `MarkdownParse`, or `Internal`, mapped to `Success`,
`ValidationFailed`, `SchemaFailure`, `MarkdownParseFailure`, and
`InternalFailure`. Help and version yield `Success` without calling the
pipeline. Locale resolution never fails.

Exit codes: `Success -> 0`, `ValidationFailed -> 1`, `InvalidInvocation -> 2`,
`SchemaFailure -> 3`, `MarkdownParseFailure -> 4`, `InternalFailure -> 5`.

### Invariants and lemmas

- `INV-exit-table`: `RunOutcome::exit_code` equals the design table; codes
  are pairwise distinct; `0` if and only if `Success`. (`REQ-exit-codes`)
- `INV-single-outcome`: for every argument vector, environment, and locale,
  `run` returns exactly one `RunOutcome` without panicking, calls the pipeline
  at most once, and exactly when the outcome came from the pipeline; the
  binary's exit code equals `exit_code` of that outcome. (`REQ-exit-codes`)
- `INV-locale-independent-outcome`: for the same argument vector and
  configuration, the outcome is identical under every locale. (`REQ-l10n`)
- `INV-stage-order`: on any invocation or configuration error the pipeline is
  called zero times and standard output is empty. (`REQ-early-stop`)
- `INV-precedence`: every key resolves to `resolve(k)`, including across
  `extends` chains. (`REQ-config-precedence`)
- `INV-absent-transparent`: an unsupplied flag or unset variable contributes
  nothing to `C` or `E`, not even a JSON `null`.
- `INV-bool-override`: `--quiet=false` and `--fail-fast=false` override a
  `true` from the environment or a file.
- `INV-ambient-isolation`: the run consults, through the injected object
  only, the named variables `LANGUAGE`, `LC_ALL`, `LC_MESSAGES`, and `LANG`,
  the prefix scan for `MDAST_CHECK_CMDS_VALIDATE_`, and `current_dir` only to
  resolve a relative `--config`; it never calls `home_fallback` or
  `config_dir_fallback`, and never reads a configuration file without
  `--config`.
- `INV-explicit-config`: if `--config` names a file that is missing, a
  directory, malformed, part of an `extends` cycle, not TOML, or any of whose
  chain files contains an unknown key at any level or a value that fails the
  value checks, the run exits `2`, the message names the offending file, and
  the pipeline is not called.
- `INV-env-strict`: a variable with the exact prefix but an unknown suffix, a
  non-UTF-8 value, or a value that fails the value checks exits `2`, and the
  message names the variable; variables without the exact, case-sensitive
  prefix are ignored.
- `INV-value-checks-uniform`: the same value is accepted or rejected
  identically from the command line, the environment, and a file, for every key.
- `INV-path-rebase`: a relative `schema` from a file becomes the canonical
  directory of that file joined with the value; absolute values and values from
  the environment or command line are unchanged.
- `INV-document-source`: `--document -` yields `DocumentSource::Stdin`; any
  other non-empty value yields `DocumentSource::Path`; `schema = -` and empty
  paths are invocation errors.
- `INV-max-errors-positive`: a resolved `max_errors` is at least 1.
- `INV-streams`: help and version go to standard output; all diagnostics go to
  standard error; write failures follow D9.
- `INV-locale-resolution`: resolution follows D15 exactly and is total,
  always yielding a language with an injected catalogue. (`REQ-l10n`)
- `INV-required-identifiers`: the required-identifier set (below) resolves in
  every shipped catalogue and in the pseudo-locale; no catalogue defines a
  forbidden identifier or an identifier outside the required set. (`REQ-l10n`)
- `INV-no-leaked-english`: under the pseudo-locale, help, version, `clap`
  errors, invocation errors, and the placeholder contain no English text from
  `clap` or from this crate, apart from recorded D17 allowances, literal flag
  names, values, and paths. (`REQ-l10n`)

The required-identifier set is: `about` for the root and `validate` commands;
`help` for every argument, including the explicit help and version arguments;
`value_name` for every argument that takes a value; the help headings and
`try-help` line of D17; every `mdast-check-msg-*` identifier the crate renders;
and `clap-error-<kind>` for every error kind the run maps (at least unknown
argument, invalid value, value validation, missing required argument, missing
subcommand, invalid subcommand, argument conflict, invalid UTF-8, no equals,
and wrong number of values; extended by every kind the robustness property
observes). Forbidden: any `*-usage`, `*-version`, and `*-long_version`
identifier.

- `L-cli-layer`: the command-line layer has key `k` if and only if the flag
  for `k` was supplied, with the parsed value.
- `L-env-layer`: the environment layer has key `k` if and only if
  `MDAST_CHECK_CMDS_VALIDATE_<K>` is present in the injected scan, with the
  checked value.
- `L-file-layer`: each pushed file layer is that file's checked
  `cmds.validate` table, absent keys omitted, `schema` rebased.
- `L-layer-order`: layers are pushed as defaults, files (chain order),
  environment, command line. With `AXIOM-ortho-merge` and the three layer
  lemmas this yields `INV-precedence`.
- `L-input-total`: building `ValidationInput` and `ReportOptions` from merged
  configuration succeeds with every field typed or fails only on a missing
  schema.

### Axioms (trusted, not verified here)

- `AXIOM-ortho-merge`: `merge_from_layers` applies layers in push order; for
  scalar keys a later value replaces an earlier one; an absent key leaves it; a
  `null` replaces it. Source: `ortho_config` 0.11.0 `declarative::merge_value`.
  Exercised at the boundary by the precedence tests, which use the real library.
- `AXIOM-ortho-file`: `load_config_file_as_chain` parses TOML, follows
  `extends`, orders the chain root first, canonicalizes paths, reports cycles,
  parse failures, directories, and non-TOML extensions as errors, and returns
  `Ok(None)` for a missing file. Exercised with real files.
- `AXIOM-ortho-l10n`: `try_parse_localized_with_matches` applies catalogue
  text for every identifier it requests, leaves `clap` text where a lookup
  returns nothing, and localizes errors to `clap-error-<kind>` preserving
  `clap::Error::kind`; with `disable_defaults`, `FluentLocalizer` consults only
  the consumer bundle. Exercised by the localization tests.
- `AXIOM-clap`: `clap` 4 reports help and version as `DisplayHelp` and
  `DisplayVersion`, rejects repeated flags and non-UTF-8 input for string
  parsers, and honours `require_equals`, `hide_possible_values`,
  `disable_help_subcommand`, and help templates;
  `Cli::command() .debug_assert()` checks the derive.
- `AXIOM-fluent`: `fluent-bundle` with `set_use_isolating(false)` inserts no
  isolation marks.
- `AXIOM-serde`: `serde` names unknown fields under `deny_unknown_fields`.
- `AXIOM-process`: `std::process::ExitCode::from(u8)` becomes the process exit
  status. Exercised by `assert_cmd` tests.

### Methods per obligation

- Obligation: `INV-exit-table`. Method: exhaustive `rstest` table with
  literal expected codes copied from the design, plus a distinctness test over
  `RunOutcome::ALL`. Artefact: `src/outcome.rs` tests `exit_codes_match_design`
  and `codes_are_distinct_and_zero_only_for_success`. Residual gap: nothing
  forces `ALL` to grow with a new variant; review and the design table guard
  it. Negative control: swap two codes; both rows fail.
- Obligation: `INV-single-outcome`, `INV-streams`,
  `INV-locale-independent-outcome`. Method: `proptest` over argument vectors of
  0 to 8 tokens (every flag in long and `=` forms, `validate`, `--help`,
  `--version`, `-`, `--`, plausible and junk values, and on Unix non-UTF-8
  bytes), each run twice, under English and under the pseudo-locale, with an
  otherwise empty `MapEnv`, plus end-to-end tests. Artefact:
  `src/cli/run/tests.rs`, property `run_always_yields_one_outcome` with a
  `RecordingPipeline` (call counter, returns `Valid`); it asserts the two runs'
  outcomes and pipeline call counts are equal, and records every
  `clap::error::ErrorKind` observed into a set that a companion test compares
  with the `clap-error-*` part of the required-identifier set. Unit test
  `Cli::command().debug_assert()`; `FailingWriter` and flush-failure tests;
  `tests/validate_cli.rs` with exact exit codes and stream contents, version
  compared with `env!("CARGO_PKG_VERSION")`, and the tripwire
  `placeholder_pipeline_exits_5_until_1_3_2`. Non-vacuity: the strategy
  produces a fully valid invocation shape with probability above one half;
  counters held in `Cell` (safe under shrinking re-runs) are asserted after the
  run to show both pipeline-called and pipeline-not-called classes occurred. The
  `TestRunner` is configured with a `source_file` so failures persist under
  `proptest-regressions/`, which is committed.
- Obligation: `INV-stage-order`. Method: `rstest` table with
  `RecordingPipeline`, one row per invocation-error class (unknown flag,
  missing document, missing schema, `--schema -`, empty schema, invalid format,
  zero cap, repeated `--quiet`, `--quiet false`, missing configuration file,
  malformed configuration, unknown file key, unknown environment variable,
  wrongly typed environment value, non-UTF-8 environment value). Artefact:
  `src/cli/run/tests.rs`, `pipeline_not_called_on_invocation_error`.
  Non-vacuity: a valid control row asserts one call; seeded fault: call the
  pipeline before the schema check; the missing-schema row fails.
- Obligation: `L-cli-layer` and `L-env-layer`. Method: exhaustive presence
  enumeration per layer (`2^6 = 64` subsets each), which is complete for the
  finite space. Artefact: `src/cli/config/tests.rs`,
  `cli_layer_presence_is_exact` (real `clap` parse of the argument vector for
  each subset; layer key set equals the subset; values equal fixed witnesses)
  and `env_layer_presence_is_exact` (real scan of a `MapEnv`). Negative
  controls: replace `sanitize_value` with plain serialization (emits `null` for
  absent keys); give `--fail-fast` a `clap` default of `false`; each makes the
  respective rows fail.
- Obligation: `INV-precedence`, `INV-absent-transparent`, `L-layer-order`.
  Method: a deterministic pairwise covering array plus a property test, both
  against an independent oracle, plus targeted rows. Rationale: each key's
  class (which of `F`, `E`, `C` define it) has 8 values, so the full space is
  `8^6 = 262,144` combinations; at an estimated 0.3 to 2 ms per case in a debug
  build (to be measured in `EP-M0`) that is 1.5 to 9 minutes, too slow for the
  default suite. Keys are merged independently by `merge_value`, so failures
  are expected to involve one key or an interaction of two; a strength-2
  orthogonal array over `GF(8)` (64 rows, every pair of classes for every pair
  of keys appearing exactly once) covers all pairwise interactions
  deterministically. A `proptest` with 1,024 random class vectors samples
  higher-order interactions. Full enumeration is provided as an `#[ignore]`d
  test, `precedence_exhaustive`, run on demand with
  `cargo test -- --ignored precedence_exhaustive` and once in `EP-M3`, with its
  run time recorded. Witness values depend on the class: each present layer
  holds a value distinct from the others and from the default; booleans
  alternate starting with `true` at the lowest present layer; schema witnesses
  are absolute so in-memory file layers need no rebasing. Artefact:
  `src/cli/config/tests.rs`, `precedence_covering_array`,
  `precedence_matches_oracle`, `precedence_exhaustive`; file layers are pushed
  through the real D7 check from in-memory TOML parsed with
  `ortho_config::toml`; `E` through the real scan of a `MapEnv`; `C` through
  the real `clap` parse; the real `merge_from_layers`. The oracle is a short
  function in the test module applying `resolve(k)` with literal defaults
  copied from the design. Non-vacuity: a test counts the covering array's pairs
  and asserts every pair of classes appears for every pair of keys; chain-order
  rows (parent `format = "json"`, child `"human"` gives `human`; child unset
  gives `json`) and the cross-source row (environment `json` over file `human`
  gives `json`) are explicit. Negative control: push the environment layer
  after the command-line layer; the array, property, and rows fail.
- Obligation: `INV-ambient-isolation`. Method: an environment-access recorder
  plus mechanical and end-to-end checks. Artefacts: a `RecordingEnv` test
  double wrapping `MapEnv` that records every `get` key and every call to
  `current_dir`, `home_fallback`, `config_dir_fallback`, and `scan`; a test in
  `src/cli/run/tests.rs` asserting, for runs with and without a relative
  `--config`, that `get` keys are a subset of the four locale variables,
  `current_dir` is called only with a relative `--config`, and the fallbacks
  are never called; the `clippy.toml` `disallowed-methods` entries; and
  `tests/validate_cli.rs`, `ambient_configuration_is_ignored`, running the
  binary with `env_clear()`, `HOME` and `XDG_CONFIG_HOME` pointing at a
  temporary directory with candidate files, `current_dir` containing
  `.mdast-check.toml`, asserting exit `2` with the missing-schema message.
  Non-vacuity: a companion row passes `--config` to the same file and reaches
  the placeholder (exit `5`).
- Obligation: `INV-explicit-config`, `INV-env-strict`, `L-file-layer`.
  Method: `rstest` tables with real files and `MapEnv`. Domain: missing file,
  directory, malformed TOML, `extends` cycle, `.json` extension, extensionless
  file (accepted), unknown key, `document` or `config` under `[cmds.validate]`,
  root-level `schema`, `[cmd.validate]`, `[cmds.lint]`, wrong types,
  `schema = ""`, errors in a parent file of a chain;
  `MDAST_CHECK_CMDS_VALIDATE_FAILFAST`, `..._DOCUMENT`, `..._MAX_ERRORS=0`,
  `..._QUIET=1`, a non-UTF-8 value (Unix), a lower-case
  `mdast_check_cmds_validate_quiet` (ignored), and a schema path containing a
  comma (accepted unchanged). Artefact: `src/cli/config/tests.rs`,
  `invalid_configuration_is_rejected`, `environment_rows`, plus end-to-end
  rows. Evidence: exit `2` and standard error names the file or variable, or
  the stated acceptance. Non-vacuity: accepted rows prove the checks are not
  blanket rejections; parent-file rows prove the whole chain is checked.
- Obligation: `INV-value-checks-uniform`, `INV-document-source`,
  `INV-max-errors-positive`, `INV-bool-override`, `L-input-total`. Method: one
  `rstest` table of values (valid and invalid for every key) applied through
  all three sources, asserting identical acceptance and the same message
  identifier on rejection. Artefacts: `src/cli/values/tests.rs`,
  `values_are_checked_uniformly`; `src/cli/config/tests.rs`,
  `boolean_can_be_switched_off` and `input_requires_schema`.
- Obligation: `INV-path-rebase`. Method: `rstest` rows on the real file
  system, including a Unix symbolic link to a configuration file elsewhere.
  Oracle independent of `join`: create the schema file at its intended place in
  a canonicalized temporary directory and compare canonical paths. Artefact:
  `relative_schema_paths_resolve_against_their_file`.
- Obligation: `INV-locale-resolution`. Method: `rstest` table plus a
  `proptest` totality property. Domain: rows for each variable winning; empty
  values skipped; `LANGUAGE` honoured (`LANGUAGE=qaa:en` with
  `LANG=de_DE.UTF-8` selects `qaa`), ignored under `C` and `POSIX`, and skipped
  for entries without a catalogue; POSIX forms (`en_GB.UTF-8`,
  `ja_JP.eucJP@euro`, `C.UTF-8`, `POSIX.UTF-8`); non-UTF-8 values; garbage; all
  unset. The property draws arbitrary strings (including non-UTF-8 on Unix) for
  the four variables and asserts resolution returns a language in the injected
  set. Artefact: `src/cli/l10n/tests.rs`, `locale_resolution_table` and
  `locale_resolution_is_total`. Negative control: swap the `LC_ALL` and `LANG`
  order; the precedence rows fail.
- Obligation: `INV-required-identifiers`. Method: a recording localizer and
  catalogue parsing. A `RecordingLocalizer` (as in `ortho_config`'s own tests)
  wraps the real localizer; a test parses root and `validate` help, renders
  every `MessageId`, and triggers each mapped error kind, then computes the
  required set (requested identifiers filtered to the rules above, plus the
  error kinds and messages) and asserts every member resolves in every shipped
  catalogue and the pseudo-locale, no catalogue defines a forbidden identifier,
  and no catalogue defines an identifier outside the set. The sorted set is an
  `insta` snapshot, so an `ortho_config` upgrade that renames identifiers
  appears in review. Artefact: `src/cli/l10n/tests.rs`,
  `required_identifiers_resolve`,
  `catalogues_define_no_forbidden_or_unused_identifiers`, and the snapshot.
  Non-vacuity: the set is asserted to contain each argument's `help`
  identifier; negative controls: delete one English entry (the test names it);
  add `mdast-check-version` (the forbidden check fails).
- Obligation: `INV-no-leaked-english`. Method: pseudo-locale output scan.
  The pseudo-locale catalogue is generated in the test from the English
  catalogue (every message value wrapped in `⟦` and `⟧`, letters replaced by
  accented forms, placeables kept). With `LANG=qaa`, the test renders root and
  `validate` help, `--version`, one error of each mapped kind, every invocation
  error, and the placeholder, and asserts that every line is either produced
  from the catalogue (contains `⟦`) or consists only of flag names, values,
  paths, usage syntax, and recorded D17 allowances; it also asserts that a list
  of `clap` English tokens ("Usage:", "Options:", "Commands:", "For more
  information", "possible values", "Print help", "Print version") never
  appears. Artefact: `src/cli/l10n/tests.rs`, `pseudo_locale_leaks_no_english`.
  Non-vacuity: the same scan under English must find the English headings
  (proving the scan sees them); negative control: re-enable the help
  subcommand; the scan fails.
- Obligation: user-facing wording. Method: `insta` snapshots of every English
  crate message and of one localized `clap` error per mapped kind, with
  temporary paths filtered to `[TMP]`. Help is asserted by content (each
  argument's catalogue text and flag name present), not snapshotted, while
  `Cargo.lock` is ignored (D11).

### Formal verification decision

No Verus proof and no Kani harness are planned for this item (decision D10).

Verus is warranted when a small, pure, repository-owned kernel must satisfy a
property for unboundedly many inputs. This item has two candidate kernels, and
neither justifies a proof. Layer precedence is implemented by `ortho_config`'s
merge; this crate owns the construction of the layers, whose presence behaviour
is a finite space enumerated exhaustively per layer, and whose combination is
checked against the real merge with a pairwise covering array, a property, and
an on-demand exhaustive run; a Verus proof would verify a model of a
third-party library rather than the code that runs. Locale resolution is owned
and pure, but its structure is a short priority rule over at most four values
plus string normalization delegated to `unic-langid` parsing, which Verus
cannot reason about without trusting the parser; the proof would rest on an
axiom equivalent to the property. A precedence table, a totality property, and
negative controls give proportionate assurance. The exit-code table is a
six-element finite function, which enumeration proves. Kani would add bounded
exhaustiveness to code with no arithmetic, no `unsafe`, and no owned state
machine, and leans on `serde_json`, `clap`, and Fluent, which are impractical
to model check.

Candidates for formal verification in later items: the fallback span selection
of roadmap 2.1.1, the deterministic ordering and capping of roadmaps 2.2.1 and
2.2.2 (a prefix of a total order of length `min(n, cap)`), and the
instance-path registration of roadmap 1.2.2.

### Suggested BDD features

Feature file: `tests/features/validate_command.feature`, bound by
`tests/validate_command_bdd.rs` with `rstest-bdd` 0.6.0. Steps drive
`mdast_check::cli::run` in-process with a scripted fake pipeline, a `MapEnv`
whose current directory is a per-scenario temporary directory, and injected
catalogues, so every exit code and locale is reachable without touching the
process. Steps return `Result` and never call `expect`. Milestones: `EP-M2`
(first block), `EP-M3` (configuration block), `EP-M4` (localization block).

```gherkin
Feature: Validate command invocation
  The validate command parses its flags, merges configuration, localizes its
  messages, and reports a stable exit code for every outcome.

  Scenario Outline: Pipeline outcomes map to stable exit codes
    Given a validation pipeline that reports <outcome>
    When the user runs "validate --document article.md --schema article.json"
    Then the exit code is <code>
    And the pipeline was called once

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
    And standard error mentions "--colour"
    And the pipeline was not called

  Scenario: Help is printed on standard output
    Given a validation pipeline that reports the document valid
    When the user runs "validate --help"
    Then the exit code is 0
    And standard output mentions "--max-errors"
    And standard error is empty
    And the pipeline was not called

  Scenario: The configuration file supplies the schema
    Given a configuration file "conf/mdast-check.toml" containing:
      """
      [cmds.validate]
      schema = "schemas/article.json"
      """
    And a validation pipeline that reports the document valid
    When the user runs "validate --document article.md" with that configuration
    Then the exit code is 0
    And the pipeline received schema "conf/schemas/article.json"

  Scenario: The environment overrides the configuration file
    Given a configuration file "mdast-check.toml" containing:
      """
      [cmds.validate]
      format = "human"
      """
    And the environment variable "MDAST_CHECK_CMDS_VALIDATE_FORMAT" is "json"
    And a validation pipeline that reports the document valid
    When the user runs "validate --document a.md --schema s.json" with that configuration
    Then the pipeline received output format "json"

  Scenario: The command line overrides the environment
    Given the environment variable "MDAST_CHECK_CMDS_VALIDATE_FORMAT" is "json"
    And a validation pipeline that reports the document valid
    When the user runs "validate --document a.md --schema s.json --format human"
    Then the pipeline received output format "human"

  Scenario: A configured boolean can be switched off for one run
    Given a configuration file "mdast-check.toml" containing:
      """
      [cmds.validate]
      fail_fast = true
      """
    And a validation pipeline that reports the document valid
    When the user runs "validate --document a.md --schema s.json --fail-fast=false" with that configuration
    Then the pipeline received fail-fast disabled

  Scenario: A missing configuration file is an invalid invocation
    Given a validation pipeline that reports the document valid
    When the user runs "validate --document a.md --schema s.json --config missing.toml"
    Then the exit code is 2
    And standard error mentions "missing.toml"
    And the pipeline was not called

  Scenario: A misspelt configuration key is rejected
    Given a configuration file "mdast-check.toml" containing:
      """
      [cmds.validate]
      fail-fast = true
      """
    And a validation pipeline that reports the document valid
    When the user runs "validate --document a.md --schema s.json" with that configuration
    Then the exit code is 2
    And standard error mentions "fail-fast"
    And standard error mentions "mdast-check.toml"

  Scenario: A misspelt environment variable is rejected
    Given the environment variable "MDAST_CHECK_CMDS_VALIDATE_FAILFAST" is "true"
    And a validation pipeline that reports the document valid
    When the user runs "validate --document a.md --schema s.json"
    Then the exit code is 2
    And standard error mentions "MDAST_CHECK_CMDS_VALIDATE_FAILFAST"

  Scenario: Help follows the user's locale
    Given the pseudo-locale catalogue is installed
    And the environment variable "LANG" is "qaa"
    When the user runs "validate --help"
    Then every line of standard output is localized or literal

  Scenario: Errors follow the user's locale without changing the exit code
    Given the pseudo-locale catalogue is installed
    And the environment variable "LANG" is "qaa"
    When the user runs "validate --document a.md --colour"
    Then the exit code is 2
    And every line of standard error is localized or literal

  Scenario: An unknown locale falls back to English
    Given the environment variable "LANG" is "zz_ZZ.UTF-8"
    When the user runs "validate --help"
    Then standard output contains the English help for "--schema"
```

"Localized or literal" has the meaning defined for `INV-no-leaked-english`.

Runner: `cargo test --test validate_command_bdd` (also run by `make test`).

## Plan of work

### `EP-M0` feasibility probe (prototype, deleted)

Question: can the planned `ortho_config` 0.11.0 usage compile and behave as
assumed under this repository's Clippy and Whitaker lints? Work in the branch
worktree after approval. Add the dependencies from
`Interfaces and dependencies`, an empty `src/lib.rs` with crate documentation,
the `clippy.toml` additions, and a throwaway integration test
`tests/probe_ortho_config.rs` that:

1. derives `OrthoConfig`, `Deserialize`, `Serialize`, and `Default` on the
   planned `ValidateConfig` and checks `make lint` over the generated code;
2. loads a two-file `extends` chain with `load_config_file_as_chain` and
   records chain order, per-file paths, and the missing-file result;
3. scans a `MapEnv` and checks non-UTF-8 handling of `scan()` values on Unix;
4. deserializes per-source layers into the strict structs and prints the
   messages for an unknown key and a wrong type;
5. merges defaults, files, environment, and the `sanitize_value` command-line
   layer through `MergeComposer` and `merge_from_layers`, including
   `--quiet=false` over a file `true`, and times one merge to estimate the
   exhaustive precedence run;
6. builds a `FluentLocalizer` with `disable_defaults` and a non-isolating
   concurrent consumer bundle for `en` and for a generated `qaa` pseudo-locale,
   parses with `try_parse_localized_with_matches` (and
   `parse_localized_command` with a prepared help template), renders `--help`
   with explicit help and version arguments, `hide_possible_values`, and
   `disable_help_subcommand`, renders a localized `clap` error, and records
   every line of `clap` English that remains, ANSI escapes, and U+2068 or
   U+2069 characters;
7. compiles one `#[rstest]` plus `#[gtest]` test, one `rstest-bdd` scenario,
   one `cap_std::fs_utf8` write, one `Result`-returning step, one `insta`
   assertion, and one `proptest!` block under the new `disallowed-methods`.

Go/no-go: checks 1, 2, 4, 5, 6 (apart from a residual list), and 7 behave as
expected, and no lint fires inside generated code that cannot be met with a
recorded item-scoped `#[expect]`. The residual English list from check 6
becomes the D17 allowance list (expected empty or nearly so). Record the
transcript in `Artefacts and notes` and findings in `Surprises & discoveries`.
If D2 or D15 cannot hold, set the status to `BLOCKED`. Delete the probe before
committing.

### `EP-M1` run-outcome contract and library/binary split

Assigned: `REQ-exit-codes`, `GAP-no-lib`, decisions D1 and D13.

Red: create `src/lib.rs` with crate documentation and `pub mod outcome;`, and
`src/outcome.rs` with `exit_codes_match_design` and
`codes_are_distinct_and_zero_only_for_success` first.
`cargo test --lib outcome::` fails to compile.

Green: define `RunOutcome`, `RunOutcome::ALL`, `const fn exit_code`, and
`impl From<RunOutcome> for std::process::ExitCode`. Add the `[[bin]]` table,
`TARGET ?= mdast-check`, and the `clippy.toml` additions. `src/main.rs` keeps
its stub in this milestone (it reads no environment, so needs no new
expectation).

Refactor: add a doctest on `RunOutcome::exit_code`; run the negative control.

### `EP-M2` `validate` command surface and the English catalogue

Assigned: `REQ-flags` (seven flags), `REQ-explicit-verb`, `REQ-stdin`,
`REQ-early-stop`, `REQ-l10n` (English catalogue and localized parse),
`GAP-stub`, decisions D4, D5, D8, D9, D12, D14, D15 (catalogue and localizer,
English only), D17.

Introducing the English catalogue here, rather than writing inline English and
moving it later, means every message has an identifier from the start.

Red: the first block of `tests/features/validate_command.feature` and
`tests/validate_command_bdd.rs`; `tests/validate_cli.rs` with `assert_cmd`
tests for `--help`, `--version`, `validate --help`, no subcommand (`2`), an
unknown flag (`2`), a missing `--schema` (`2`), a non-UTF-8 argument on Unix
(`2`), and `placeholder_pipeline_exits_5_until_1_3_2`; unit tests for argument
parsing, value checks, `run`, and the argument-robustness property (English
only for now). They fail to compile or fail on the stub's behaviour.

Green, in this order:

1. `src/app/settings.rs`: `DocumentSource`, `SchemaFormat`, `OutputFormat`,
   `ValidationInput`, `ReportOptions` (`serde` derives on the enums, no `clap`).
2. `src/app/pipeline.rs`: `ValidationPipeline`, `Verdict`, `PipelineFailure`
   with `outcome()`. `src/pipeline/mod.rs`: `PendingPipeline`.
3. `src/cli/values.rs`: the shared value checks of D8, returning a
   `MessageId` on rejection.
4. `src/cli/args.rs`: `clap` types `Cli`, `Command`, `ValidateArgs` with every
   flag except `--config`; configurable fields as `Option<T>` without `clap`
   defaults (strings for paths and enums, `usize` for `--max-errors`,
   `--flag[=BOOL]` booleans); explicit long names; no short flags; explicit
   `help` and `version` arguments; `disable_help_subcommand`;
   `hide_possible_values`; the catalogue help template; a `debug_assert` unit
   test.
5. `src/cli/l10n/mod.rs`: `MessageId` (crate messages), `Catalogues`
   (`shipped()` with `locales/en/messages.ftl`, and `from_sources`),
   `build_localizer` per D15 (English only in this milestone), and `render`.
   `locales/en/messages.ftl`: every required identifier for the command tree,
   help headings, `try-help`, `clap-error-*` kinds, and crate messages.
6. `src/cli/config/mod.rs` (first cut): `pub(crate) ValidateConfig` with the
   `OrthoConfig` derive; the command-line layer (`sanitize_value`, D2a) and its
   value checks; `resolve`, merging defaults and the command-line layer and
   building `ValidationInput` and `ReportOptions`, failing with the
   missing-schema invocation error.
7. `src/cli/error.rs`: `pub(crate) InvocationError` with variants for usage
   errors (wrapping the localized `clap::Error`), value-check failures (key,
   source, `MessageId`), the missing schema, and the configuration variants
   added in `EP-M3`; rendering always goes through `l10n::render`.
8. `src/cli/run.rs`: `run` and `RunContext` (signatures in
   `Interfaces and dependencies`): build the English localizer, parse with
   `try_parse_localized_with_matches`, handle display requests and errors per
   D9 and D17, resolve, call the pipeline once, and pass every outcome to one
   private reporting function.
9. `src/main.rs`: the composition root: `ProcessEnv`, locked standard streams,
   process arguments, `PendingPipeline`, `Catalogues::shipped()`; delete the
   stub and its `#[expect(clippy::print_stdout)]` in the same commit, and add
   the single `#[expect(clippy::disallowed_methods, reason = ...)]` for the
   process-argument read.

Refactor: keep `run` under the complexity thresholds; `insta` snapshots of the
English crate messages and of each mapped `clap` error kind; seeded fault for
`INV-stage-order`.

### `EP-M3` layered configuration: file and environment

Assigned: `REQ-flags` (`--config`), `REQ-config-precedence`, `GAP-no-config`,
decisions D2, D2a, D3, D6, D7.

Red: the configuration block of the feature file; `cli_layer_presence_is_exact`,
`env_layer_presence_is_exact`, `precedence_covering_array`,
`precedence_matches_oracle`, `precedence_exhaustive`, chain-order and
cross-source rows, `invalid_configuration_is_rejected`, `environment_rows`,
`values_are_checked_uniformly` (environment and file columns),
`relative_schema_paths_resolve_against_their_file`,
`boolean_can_be_switched_off`; the environment-access recorder test and
`ambient_configuration_is_ignored`. They fail because `--config` does not exist
(`clap` rejects it) and the environment is not read.

Green: add `--config <PATH>` (resolved against the injected `current_dir` when
relative). Add `src/cli/config/file.rs` (chain loading, missing-file error,
strict per-file structs, value checks, rebasing, re-serialization, `push_file`
with path) and `src/cli/config/env.rs` (exact-prefix scan, suffix and UTF-8
checks, value checks, `push_environment`). Push layers in the order of
`L-layer-order`. Map `OrthoError` values to
`InvocationError::Config { source: ConfigSource, reason: MessageId }`, where
`ConfigSource` names a file or a variable and the original error is kept as
`#[source]`. Add the catalogue entries for the new messages.

Refactor: keep files under 400 lines; run the negative controls for the layer
lemmas and precedence; run `precedence_exhaustive` once and record its run time
in `Artefacts and notes`.

### `EP-M4` locale selection and localization completeness

Assigned: `REQ-l10n`, `GAP-no-l10n`, decisions D15, D16, D17.

Red: `locale_resolution_table`, `locale_resolution_is_total`,
`required_identifiers_resolve`,
`catalogues_define_no_forbidden_or_unused_identifiers` and its snapshot,
`pseudo_locale_leaks_no_english`, the robustness property's second
(pseudo-locale) run and error-kind comparison, the localization block of the
feature file, and end-to-end rows for `LANG=C`, `LANG=zz_ZZ.UTF-8`, and
`LANGUAGE=zz:en`. They fail because locale resolution does not exist (every run
is English) and the pseudo-locale is never selected.

Green:

1. `src/cli/l10n/locale.rs`:
   `resolve_locale(env: &dyn EnvSource, available: &Catalogues) -> LanguageIdentifier`
   per D15.
2. `tests/support/pseudo_locale.rs`: generate the `qaa` pseudo-locale from the
   English catalogue text (embedded with `include_str!`) and build `Catalogues`
   through the public `Catalogues::from_sources`; unit tests in
   `src/cli/l10n/tests.rs` use a private copy of the same small generator, kept
   in `src/cli/l10n/pseudo.rs` under `#[cfg(test)]`.
3. Switch `run` to resolve the locale from the injected environment against
   `context.catalogues` before building the localizer.
4. Complete any catalogue entries the recorder shows missing, and record any
   D17 residuals in the allowance list.

Refactor: review the diff to confirm no user-facing string literal remains
outside the catalogue; run the negative controls for `INV-locale-resolution`,
`INV-required-identifiers`, and `INV-no-leaked-english`.

### `EP-M5` documentation, tested examples, and roadmap closure

Assigned: `REQ-docs`, `GAP-no-users-guide`.

Red: `tests/support/markdown_examples.rs`, test first, with inline-Markdown
unit tests for marked, unmarked, unterminated, unidentified, duplicate,
indented, and `~~~` fences. `cargo test --test users_guide_examples` fails to
compile.

Green and documentation:

1. Implement the loader over `include_str!`-embedded documents, returning
   `(identifier, language, body)`. Allowed languages: `sh` (one `mdast-check`
   command, whitespace-split; leading `NAME=value` tokens are passed through
   `Command::env`; shell metacharacters rejected), `toml` (a configuration
   file), and `text` (expected standard-error substring for the preceding `sh`
   example). Any other fence language in a user-facing document fails. One
   function, used in full by both consumers.
2. `docs/users-guide.md` for release `0.1.0` (unreleased, early adopter),
   with imperative headings: install and build; check an invocation; choose the
   document and schema; configure defaults with a file (TOML only,
   `[cmds.validate]`, strict keys, relative paths, `extends`); set defaults
   with environment variables (names, accepted value spellings, strictness,
   version skew); override for one run (precedence; `--quiet=false`); choose
   the language of help and messages (`LANGUAGE`, `LC_ALL`, `LC_MESSAGES`,
   `LANG`; fallback to English; English is the only shipped language; Windows
   users set `LANG` or `LANGUAGE` explicitly); interpret exit codes (`5` is not
   transient); output streams and accessibility (plain text, no colour, help on
   standard output, diagnostics on standard error, no meaning carried by
   colour); current limitations. Every fenced example preceded by
   `<!-- tested-example: <id> -->`, including an environment-variable example
   and a `LANG=zz_ZZ.UTF-8` fallback example.
3. `tests/users_guide_examples.rs`: runs every `sh` example of
   `docs/users-guide.md` and `README.md` against the binary with `assert_cmd`
   and `env_clear()` in a temporary directory seeded with the guide's `toml`
   examples (file names mapped from identifiers), asserting the exit code
   recorded per identifier; identifier tables match one to one. The BDD step
   file reuses the loader for the configuration example.
4. `docs/adrs/adr-0002-explicit-configuration-layer-composition.md` (D2, D2a,
   D3, D6, D7, D8; reserved discovery name; strictness and version skew;
   revisit trigger) and
   `docs/adrs/adr-0003-locale-resolution-and-message-catalogues.md` (D15, D16,
   D17; identifier scheme and reserved subcommand names; disabled defaults;
   isolation; residual English, if any), following ADR 0001's format.
5. `docs/mdast-schema-validation-cli-design.md`: §Schema configuration
   (sources, precedence, strictness, environment names, deferred discovery, ADR
   0002); a new §Localization (locale resolution, catalogues, identifier
   scheme, ADR 0003); §Exit codes (placeholder `5` until 1.3.2); §Module
   boundaries (`cli`, `app`, `pipeline`, `outcome`; `validate` reserved);
   §Proposed flags (`--schema -`, `--flag[=BOOL]`).
6. `docs/developers-guide.md`: module layout and hexagonal boundary; the
   injected environment and the `disallowed-methods` rule; the provisional
   pipeline port and its extension by 1.1.2, 1.3.2, 2.1.2, 2.2.1; `Streams` and
   the obligation for 1.2.1 to inject standard input through it; how to add a
   configuration key (both structs, value checks, flag, presence witnesses,
   catalogue entries, users' guide); how to add or change a message (catalogue
   first, identifier conventions, completeness and leaked-English tests,
   snapshot review); what must be in place before shipping a language (a new
   `locales/<language>/messages.ftl` with every `clap-error-*` text, a fluent
   reviewer, and the `localization-docs` documentation set); the tested-example
   loader; and the Cranelift note that behaviour tests now exist.
7. `README.md`: a short marked usage example linking the users' guide; remove
   the stale `make typecheck` mention.
8. `docs/roadmap.md`: add to 1.3.2 the bullet "Replace `PendingPipeline`; a
   valid document exits `0`." Mark 1.1.1 and its sub-bullets `[x]` only after
   all gates pass, annotating the exit-code bullet ("contract and mapping;
   codes 1, 3, and 4 become reachable with 1.2.1 to 1.3.2") and the
   localization bullet ("localizable output; English shipped; first translation
   proposed as a follow-up item").

## Milestones and plateaus

- `EP-M0`: a recorded go or no-go for D2, D15, and D17, the D17 allowance
  list, and the measured merge cost, with no committed code. Recovery: delete
  the probe; restore `Cargo.toml` and `clippy.toml` if no-go. Compatibility
  decision: none.
- `EP-M1`: a library exposing `RunOutcome` and a binary named
  `mdast-check` (still the stub), with the tightened `clippy.toml`.
  Requirements: `REQ-exit-codes` (contract), `GAP-no-lib`. Evidence:
  `outcome::tests` pass; `make build` produces `target/debug/mdast-check`;
  `make lint` passes with the new disallowed methods. Recovery: revert the
  commit. Compatibility decision: none.
- `EP-M2`: the binary parses the `validate` flags except `--config`, rejects
  invalid invocations with `2`, prints English help and version with `0`, and
  reports the placeholder with `5`; every message comes from the English
  catalogue; the in-process runner maps every pipeline outcome. Requirements:
  `REQ-flags` (seven), `REQ-explicit-verb`, `REQ-stdin`, `REQ-early-stop`,
  `REQ-l10n` (catalogue), `GAP-stub`. Conformance: flags match Table 1 except
  `--config`; `clap` and Fluent confined to `src/cli/`. Remaining gaps: no
  configuration sources; locale fixed to English. Recovery: revert to `EP-M1`.
  Compatibility decision: none.
- `EP-M3`: `--config` files and `MDAST_CHECK_CMDS_VALIDATE_*` variables
  supply defaults with the command line taking precedence, all checked
  strictly. Requirements: `REQ-flags` (`--config`), `REQ-config-precedence`,
  `GAP-no-config`. Conformance: precedence matches the design; the file shape
  and variable names are new user-facing formats approved by this plan.
  Remaining gaps: locale fixed to English. Recovery: revert to `EP-M2`.
  Compatibility decision: none (first release of both formats).
- `EP-M4`: the locale is resolved from the injected environment against the
  injected catalogues; the pseudo-locale proves every message is localizable
  with no English leaking. Requirements: `REQ-l10n`, `GAP-no-l10n`.
  Conformance: message identifiers are an internal contract documented in ADR
  0003 and snapshotted. Recovery: revert to `EP-M3`. Compatibility decision:
  none.
- `EP-M5`: documentation matches behaviour; every users' guide and README
  example runs in the suite; roadmap 1.1.1 done; 1.3.2 carries the
  placeholder-removal bullet. Requirements: `REQ-docs`, `GAP-no-users-guide`.
  Conformance: design, ADR 0002, and ADR 0003 updated for every decision; no
  unrecorded deviation. Recovery: documentation commits revert independently.
  Compatibility decision: none.

## Concrete steps

Working directory: the repository root of the branch worktree. Run gates
sequentially with `pipefail`:

```sh
set -o pipefail
BRANCH=$(git branch --show-current)
make check-fmt 2>&1 | tee /tmp/check-fmt-mdast-check-$BRANCH.out
make lint 2>&1 | tee /tmp/lint-mdast-check-$BRANCH.out
make test 2>&1 | tee /tmp/test-mdast-check-$BRANCH.out
```

For milestones that change Markdown (`EP-M5`, and this plan's updates):

```sh
make fmt 2>&1 | tee /tmp/fmt-mdast-check-$BRANCH.out
make markdownlint 2>&1 | tee /tmp/markdownlint-mdast-check-$BRANCH.out
make nixie 2>&1 | tee /tmp/nixie-mdast-check-$BRANCH.out
```

`make markdownlint` runs `make spelling` first; commit any regenerated
`typos.toml` separately (on a base branch if unrelated to this work).

Focused commands per milestone:

```plaintext
EP-M0  cargo test --test probe_ortho_config -> observations printed; make lint findings recorded
EP-M1  Red:   cargo test --lib outcome::             -> cannot find type `RunOutcome`
       Green: cargo test --lib outcome::             -> pass
EP-M2  Red:   cargo test --test validate_cli         -> stub exits 0; assertions on 2 and 5 fail
       Red:   cargo test --test validate_command_bdd -> cannot find `mdast_check::cli::run`
       Green: both pass; cargo test --lib cli:: passes, including the robustness property
EP-M3  Red:   cargo test --lib cli::config           -> file and environment rows fail
       Green: cargo test --lib cli::config           -> pass
       Once:  cargo test --lib -- --ignored precedence_exhaustive -> pass; run time recorded
EP-M4  Red:   cargo test --lib cli::l10n             -> cannot find `resolve_locale`
       Green: cargo test --lib cli::l10n cli::run    -> pass; English snapshots unchanged
EP-M5  Red:   cargo test --test users_guide_examples -> loader unit tests fail to compile
       Green: cargo test --test users_guide_examples -> pass
```

Commit after each green step and each refactor that passes the three gates,
using the `commit-message` skill.

## Validation and acceptance

Behaviour a human can verify after `EP-M5`, from the repository root after
`make build`:

```plaintext
$ target/debug/mdast-check --help; echo "exit=$?"
(usage listing the validate command)
exit=0

$ target/debug/mdast-check validate --document README.md; echo "exit=$?"
error: no schema given; pass --schema, set MDAST_CHECK_CMDS_VALIDATE_SCHEMA,
  or set `schema` under [cmds.validate] in a configuration file
exit=2

$ MDAST_CHECK_CMDS_VALIDATE_FAILFAST=true target/debug/mdast-check validate \
    --document README.md --schema s.json; echo "exit=$?"
error: unknown environment variable MDAST_CHECK_CMDS_VALIDATE_FAILFAST;
  accepted: MDAST_CHECK_CMDS_VALIDATE_SCHEMA, ..._SCHEMA_FORMAT, ..._FORMAT,
  ..._FAIL_FAST, ..._MAX_ERRORS, ..._QUIET
exit=2

$ target/debug/mdast-check validate --document README.md --schema s.json \
    --config missing.toml; echo "exit=$?"
error: cannot read configuration file missing.toml: file not found
exit=2

$ target/debug/mdast-check validate --document README.md --schema s.json; echo "exit=$?"
error: document validation is not implemented yet
exit=5
```

Wording is fixed by the English catalogue and its `insta` snapshots; the lines
above are intended wording and may be refined during implementation, in which
case this section is updated.

Quality criteria:

- Tests: `make test` passes, including the new unit, property, BDD, snapshot,
  localization, end-to-end, and tested-example tests, plus the existing
  build-standard contract; `precedence_exhaustive` passes when run.
- Verification: every obligation has its artefact passing and its negative
  controls recorded as rejected.
- Lint and format: `make check-fmt` and `make lint` (Clippy with the
  tightened `disallowed-methods`, and Whitaker) pass with no `#[expect]` beyond
  those recorded in `Decision log`.
- Documentation: `make markdownlint` and `make nixie` pass.
- Performance: the default `make test` run stays within the existing CI
  coverage job's time budget.
- Security: untrusted inputs are command-line arguments, environment values,
  and user-named configuration files, all parsed by typed parsers and never
  executed. Locale values only select an embedded catalogue; they never form a
  file path.

## Idempotence and recovery

All steps are re-runnable. Tests write only to `tempfile` directories through
`cap_std::fs_utf8`. If a milestone fails part-way, reset to the previous
milestone commit after checking `git status`, and repeat. Negative controls are
uncommitted edits reverted with `git checkout -- <file>`; confirm a clean
`git diff --stat` before committing. Delete the `EP-M0` probe before the first
`EP-M1` commit. Remove stray `*.snap.new` files. Commit new
`proptest-regressions/` files with the fix they guard.

## Artefacts and notes

Planning-time observations (reconnaissance, not implementation evidence):

```plaintext
$ crates.io: ortho_config 0.11.0 (2026-11-15), rust_version 1.89.0
$ ortho_config-0.11.0/src/subcommand/sources.rs:
  SubcommandFileContext { base: &Path, discovery: &dyn EnvSource }  (no explicit file path input)
$ ortho_config-0.11.0/src/discovery/builder.rs: build() uses ProcessEnv unless .env_source(..) is given
$ ortho_config-0.11.0/src/localizer/clap_command/parse.rs:
  try_parse_localized_from = try_parse_localized_with_matches(..).map(|(v, _)| v)
$ ortho_l10n-0.11.0/src/fluent.rs:
  with_consumer_bundle(fluent_bundle::concurrent::FluentBundle<Arc<FluentResource>>)
  try_build fails with UnsupportedLocale for languages other than en and ja unless disable_defaults()
$ ortho_l10n-0.11.0/locales: en-US/messages.ftl, ja/messages.ftl (11 identifiers each)
```

Milestone evidence is appended here during implementation.

## Interfaces and dependencies

Runtime dependencies (caret requirements):

- `ortho_config = "0.11.0"` (default features `serde_json` and `toml`):
  declarative merge, file loading, injected environment, `sanitize_value`,
  localized parsing, and the Fluent localizer. Its re-exports of `serde_json`,
  `toml`, `langid!`, `LanguageIdentifier`, and `FluentValue` are used, so no
  direct `serde_json`, `toml`, or `unic-langid` dependency is added.
- `fluent-bundle = "0.16.0"`: the concurrent `FluentBundle` and
  `FluentResource` types required by `with_consumer_bundle`, with Unicode
  isolation disabled; the requirement matches `ortho_l10n`'s.
- `clap = { version = "4.6.9", features = ["derive", "string"] }`.
- `serde = { version = "1.0.229", features = ["derive"] }`.
- `camino = { version = "1.2.6", features = ["serde1"] }`.
- `thiserror = "2.0.21"`.

Development dependencies (in addition to `rstest = "0.28.0"`):
`rstest-bdd = "0.6.0"`, `rstest-bdd-macros = "0.6.0"`, `googletest = "0.14.4"`
(with `#[gtest]` beneath `#[rstest]`), `pretty_assertions = "1.4.1"`,
`insta = "1.49.0"`, `assert_cmd = "2.2.2"`, `proptest = "1.12.0"`,
`tempfile = "3.27.0"`, and
`cap-std = { version = "4.0.3", features = ["fs_utf8"] }`.

`mockable` is not added: `ortho_config::EnvSource`, `ScanEnvSource`,
`ProcessEnv`, and `MapEnv` already provide the injected environment, and the
abstraction policy in `AGENTS.md` requires reusing an equivalent from an
approved dependency. `ProcessEnv` is constructed only in `src/main.rs`.

Module layout at the end of `EP-M4`:

```plaintext
src/main.rs                 composition root (binary `mdast-check`)
src/lib.rs                  crate docs; pub mod app, cli, outcome, pipeline
src/outcome.rs              RunOutcome
src/app/mod.rs              application core re-exports
src/app/settings.rs         DocumentSource, SchemaFormat, OutputFormat, ValidationInput, ReportOptions
src/app/pipeline.rs         ValidationPipeline, Verdict, PipelineFailure (provisional)
src/pipeline/mod.rs         PendingPipeline (driven adapter; replaced by roadmap 1.3.2)
src/cli/mod.rs              driving adapter: pub use run, RunContext, Streams, RunEnvironment, Catalogues
src/cli/args.rs (+ tests)   pub(crate) Cli, Command, ValidateArgs (clap)
src/cli/values.rs (+ tests) shared value checks for every source
src/cli/run.rs (+ tests)    run, RunContext, Streams, private reporting function
src/cli/error.rs            pub(crate) InvocationError, ConfigSource
src/cli/config/mod.rs       pub(crate) ValidateConfig (OrthoConfig), resolve
src/cli/config/file.rs      file chain, strict structs, rebasing
src/cli/config/env.rs       environment scan and checks
src/cli/config/tests.rs     presence enumerations, precedence tests, rows
src/cli/l10n/mod.rs         MessageId, Catalogues, CatalogueError, build_localizer, render
src/cli/l10n/locale.rs      resolve_locale
src/cli/l10n/pseudo.rs      pseudo-locale generator for unit tests (cfg(test))
src/cli/l10n/tests.rs       locale, identifier, and leaked-English tests
locales/en/messages.ftl     English catalogue (source of truth)
tests/validate_cli.rs, tests/validate_command_bdd.rs, tests/features/validate_command.feature
tests/support/markdown_examples.rs, tests/support/pseudo_locale.rs, tests/users_guide_examples.rs
```

Key signatures:

```rust
// src/outcome.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunOutcome {
    Success,
    ValidationFailed,
    InvalidInvocation,
    SchemaFailure,
    MarkdownParseFailure,
    InternalFailure,
}

impl RunOutcome {
    pub const ALL: [Self; 6] = [/* every variant, in code order */];
    #[must_use]
    pub const fn exit_code(self) -> u8 { /* exhaustive match */ }
}

impl From<RunOutcome> for std::process::ExitCode { /* ExitCode::from(exit_code) */ }

// src/app/settings.rs
pub enum DocumentSource { Stdin, Path(camino::Utf8PathBuf) }
pub enum SchemaFormat { Json, Markdoc }   // serde: "json", "markdoc"
pub enum OutputFormat { Human, Json }     // serde: "human", "json"

pub struct ValidationInput {
    pub document: DocumentSource,
    pub schema: camino::Utf8PathBuf,
    pub schema_format: Option<SchemaFormat>,
    pub fail_fast: bool,
}

pub struct ReportOptions {
    pub format: OutputFormat,
    pub max_errors: Option<std::num::NonZeroUsize>,
    pub quiet: bool,
}

// src/app/pipeline.rs (provisional; roadmap 1.1.2 replaces PipelineFailure)
pub enum Verdict { Valid, Invalid }

pub enum PipelineFailure {
    Schema,
    MarkdownParse,
    Internal(InternalFailureKind),
}

pub enum InternalFailureKind { ValidationUnavailable }

impl PipelineFailure {
    #[must_use]
    pub const fn outcome(&self) -> RunOutcome;
}

pub trait ValidationPipeline {
    /// Runs schema loading, parsing, and validation for one input.
    fn validate(&self, input: &ValidationInput) -> Result<Verdict, PipelineFailure>;
}

// src/cli/l10n/mod.rs
/// Embedded or injected Fluent catalogues, keyed by language. English is
/// always present.
pub struct Catalogues { /* language -> catalogue source text */ }

impl Catalogues {
    /// The catalogues shipped in the binary.
    #[must_use]
    pub fn shipped() -> Self;
    /// Builds catalogues from Fluent sources; fails if English is absent or
    /// a source does not parse.
    pub fn from_sources(
        sources: Vec<(ortho_config::LanguageIdentifier, String)>,
    ) -> Result<Self, CatalogueError>;
}

/// Why injected catalogues were rejected.
pub enum CatalogueError {
    MissingEnglish,
    Parse { language: ortho_config::LanguageIdentifier },
}

// src/cli/run.rs
pub struct Streams<'a> {
    pub stdout: &'a mut dyn std::io::Write,
    pub stderr: &'a mut dyn std::io::Write,
}

/// The injected environment: named lookups (locale variables, current
/// directory) and prefix scans (configuration variables).
pub trait RunEnvironment: ortho_config::EnvSource + ortho_config::ScanEnvSource {}
impl<T: ortho_config::EnvSource + ortho_config::ScanEnvSource> RunEnvironment for T {}

/// Everything a run needs besides its arguments. `main` builds it with
/// `ProcessEnv`, locked standard streams, `PendingPipeline`, and
/// `Catalogues::shipped()`; tests substitute `MapEnv` (or a recording
/// wrapper), in-memory writers, a recording pipeline, and catalogues that
/// include the pseudo-locale.
pub struct RunContext<'a> {
    pub env: &'a dyn RunEnvironment,
    pub streams: Streams<'a>,
    pub pipeline: &'a dyn ValidationPipeline,
    pub catalogues: &'a Catalogues,
}

pub fn run<I, T>(args: I, context: &mut RunContext<'_>) -> RunOutcome
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone;
```

`PipelineFailure` carries no free text, because every message the CLI prints is
rendered from the catalogue; later items add structured payloads that the CLI
renders. `ReportOptions` is resolved and validated by `run` and passed to the
private reporting function, documented as the extension point for 2.1.2, 2.2.1,
and 2.2.2; tests of `resolve` assert its values. `CatalogueError` derives
`thiserror::Error`. `Catalogues::from_sources` is the approved public seam for
injecting test catalogues; no `#[cfg(test)]` seam is needed.

Reuse policy:

- `RunOutcome` is the only place exit codes are defined; 1.1.2 maps its
  taxonomy onto it through `outcome()` methods.
- `ValidationPipeline` is the only boundary to the validation stages; it is
  provisional, and later items implement it outside `src/app/`.
- `Streams` is the only route to standard output and error; 1.2.1 adds
  standard input to it.
- `RunEnvironment` is the only route to environment variables and the working
  directory; `clippy.toml` forbids `std::env` reads outside `src/main.rs`.
- `cli::values` owns every value check; sources never parse values ad hoc.
- `cli::config` owns all configuration loading; new keys are added there, in
  the strict structs, in `cli::values`, and in the catalogue.
- `cli::l10n` and `locales/` own all user-facing text; no user-facing string
  literal outside them.
- `tests/support/markdown_examples.rs` is the shared loader for tested
  examples in all user-facing documents; `tests/support/pseudo_locale.rs` is
  the shared pseudo-locale generator for integration and BDD tests.

## Revision note

- 2026-11-20: Initial draft for `ortho_config` 0.8.0.
- 2026-11-20: First design review applied (typed layers, banned ambient
  loaders, split provisional port, renames, stronger verification).
- 2026-11-20: Revised at the user's direction to use `ortho_config` 0.11.0
  for layered configuration with localized help; prerequisites completed
  (vendored v0.11.0 guide; design and roadmap retargeted).
- 2026-11-20: Second design review applied. Changed: D2 uses
  `load_config_file_as_chain` directly (not `ConfigFilePolicy`, which reads
  ambient state); D2a uses `Option` fields and `sanitize_value`
  (`CliValueExtractor` is unavailable); D3 builds the environment layer from a
  direct, exact-prefix scan (not `CsvEnv`); D8 moves value checks into a module
  shared by all sources; D9 localizes errors once; D15 disables
  `ortho_config`'s bundled defaults, owns all `clap-error-*` texts, honours
  `LANGUAGE`, resolves against injected catalogues, and adds an unconditional
  `fluent-bundle` dependency; D16 uses a generated `qaa` pseudo-locale; new D17
  keeps `clap` English out of localized output. Verification: a
  required-identifier set with forbidden identifiers and a snapshot; a
  leaked-English scan; a locale metamorphic run in the robustness property; an
  environment-access recorder and tightened `clippy.toml`; a 64-row pairwise
  covering array plus a property and an on-demand exhaustive run for
  precedence. Milestones: the English catalogue moves into `EP-M2` (no inline
  English to migrate later); `EP-M4` becomes locale selection and completeness.
  Roadmap 1.1.1 now names the environment layer. Budgets: code 3,100,
  configuration 80, documentation 1,000. Milestone order unchanged apart from
  that move.
