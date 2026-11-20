# 1.1.1 Implement the root CLI command with a `validate` subcommand

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & discoveries`, `Decision log`,
`Outcomes & retrospective`, `Conformance basis`, and `Verification plan` must
be kept up to date as work proceeds.

Status: DRAFT (revised for `ortho_config` 0.11.0 and localized help; awaiting
design review and user approval)

Branch: `1-1-1-root-cli-command-and-validate-subcommand`

Roadmap item: `1.1.1` in `docs/roadmap.md`.

## Purpose / big picture

`mdast-check` is a command-line interface (CLI) that will validate the Markdown
Abstract Syntax Tree (MDAST) of a Markdown document against a JSON Schema
document. Today the binary is a stub: `src/main.rs` prints
`Hello from MDAST Check!` and exits with status 0 whatever arguments it is
given. No configuration is read, no text is localized, and no exit code carries
meaning.

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
  (`LC_ALL`, then `LC_MESSAGES`, then `LANG`), falling back to English;
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
  `message-id = text` entries.
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
resolved (decision D15, proposed ADR 0003).

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
- Remove the `FIXME(#5)` stub and its `#[expect(clippy::print_stdout, ...)]`
  in `src/main.rs` in the same commit that replaces the stub.
- All environment reads go through one injected environment object
  (`ortho_config::EnvSource` plus `ScanEnvSource`). Only the composition root
  (`src/main.rs`) constructs `ortho_config::ProcessEnv` and reads
  `std::env::args_os()`. No code path may call the `ortho-config` loaders that
  read process state: the generated `load`, `load_from_iter`, `compose_layers`,
  and `compose_layers_from_iter`; `SubcmdConfigMerge` and
  `SelectedSubcommandMerge` loaders without injected sources; or
  `LocalizedParse::try_parse_localized` (use the `_from` form). Do not use
  `clap`'s `env = ...` argument attribute, which reads the process environment.
  Do not opt into `ortho_config` profiles. Tests must not mutate the process
  environment or working directory (`clippy.toml` `disallowed-methods`);
  end-to-end tests may use `assert_cmd` with `Command::env_clear`,
  `Command::env`, and `Command::current_dir`.
- Keep the application core (`src/app/`) free of `clap`, `ortho_config`,
  Fluent, filesystem, and terminal types. `clap`, `ortho_config`, and the
  catalogues are used only in `src/cli/`. Core value types may derive `serde`
  traits, because a data-shape derive carries no I/O.
- Use `camino::Utf8PathBuf` for paths, per `AGENTS.md`. A non-UTF-8 argument
  on the command line is an invocation error (exit `2`).
- Use `ortho_config` with a caret requirement on `0.11.0`, as the roadmap and
  design now require. Its minimum supported Rust version is 1.89.0; the
  repository's `nightly-2026-11-20` toolchain satisfies it.
- Do not change `tests/build_standard_contract.rs`,
  `tests/build_standard_support/`, `.cargo/config.toml`, `rust-toolchain.toml`,
  `.gitignore`, or the CI workflows. Changing the Makefile is limited to
  `TARGET ?= mdast-check` (decision D1).
- Do not implement Markdown parsing, schema loading, validation, output
  rendering, quiet suppression, fail-fast behaviour, error capping, implicit
  configuration-file discovery, or a `--locale` flag.
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
design and roadmap retargeted to 0.11.0). Counts exclude this ExecPlan, any
other file under `docs/execplans/`, the vendored
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
`Cargo.toml`, `Makefile`, and `typos.local.toml` are configuration; `docs/**`
(except the exclusions above) and `README.md` are documentation.

- Files: if implementation changes more than 50 non-ExecPlan files (snapshot
  and regression files included), stop and escalate.
- Code: if code changes exceed 2,900 net lines, stop and escalate.
- Configuration: if configuration changes exceed 70 net lines, stop and
  escalate.
- Documentation: if documentation changes exceed 950 net lines, stop and
  escalate.
- Interface: if a public item beyond those in `Interfaces and dependencies`
  is needed, or a listed signature must change materially, stop and escalate.
- Dependencies: if any crate beyond those listed in
  `Interfaces and dependencies` is needed, stop and escalate.
- Architecture: if `ortho_config`'s public API (`MergeComposer`,
  `merge_from_layers`, `ConfigFilePolicy` or `load_config_file_as_chain`,
  `CsvEnv::with_source`, `FluentLocalizer`, `LocalizedParse`) cannot express
  decisions D2 and D15, stop and escalate rather than writing a bespoke merge
  or localizer.
- Iterations: if a focused test still fails after 4 fix attempts, stop and
  escalate.
- Time: if a milestone takes more than 6 hours of active work, stop and
  escalate.
- Ambiguity: if multiple valid interpretations exist and the choice materially
  affects the user-facing contract (flag names, configuration keys,
  environment-variable names, message identifiers, exit codes, message wording
  fixed by snapshots), stop and present options with trade-offs.

Each category is enforced independently. Spare capacity in one category never
offsets another.

Budget rationale. Production source is estimated at 1,100 lines: library root
40, run outcome 80, application core 200, CLI adapter 735 (arguments 120, run
and reporting 160, configuration merge 110, file layers 100, environment layer
60, locale resolution 75, localizer construction and message rendering 110),
placeholder pipeline 25, main 30. The English catalogue is about 70 lines.
Tests are estimated at 1,600 lines: unit tests within modules 540 (outcome
table, argument parsing, layer presence tables, precedence property, file and
environment layer rows, run ordering and write failures, locale resolution),
argument-robustness property 110, localization tests 200 (identifier coverage,
catalogue completeness, fallback, localized help and errors with a test
catalogue), end-to-end 230, BDD steps and feature 300, tested-example loader
and runner 220. The total of 2,770 plus about 5% gives 2,900. Configuration is
about 45 lines; 70 absorbs feature tables. Documentation is about 760 lines
(users' guide 290, ADR 0002 120, ADR 0003 90, design document 90, developers'
guide 140, README 25, roadmap 5); 950 leaves a fifth in reserve. History: the
first draft of this plan budgeted 2,300 code lines for `ortho_config` 0.8.0
without localization or an environment layer; design review lowered it to
2,200; this revision raises it to 2,900 for approved scope growth
(localization, environment layer), not missed scope. No completed ExecPlan
exists on `main` to compare with.

## Risks

- Risk: `#[derive(OrthoConfig)]` emits code that trips this repository's
  strict lints. The 0.11.0 derive generates a private `clap::Parser` struct,
  public loaders that read process state, and items marked
  `#[allow(dead_code, reason = ...)]`, while `Cargo.toml` denies
  `allow_attributes`. Severity: high. Likelihood: medium. Mitigation: `EP-M0`
  compiles the derive under `make lint` (Clippy and Whitaker). Clippy normally
  skips code expanded from external macros; if a lint still fires, isolate the
  derived type in one module with one item-scoped
  `#[expect(..., reason = ...)]` per lint, recorded in `Decision log`; if
  impossible, escalate under the Architecture tolerance.
- Risk: the generated `merge_from_layers` deserializes only the merged value,
  so a type error in one source cannot be attributed to it. Severity: high.
  Likelihood: certain (unchanged from 0.8.0). Mitigation: decision D7 checks
  each file layer and the environment layer separately before pushing it.
- Risk: `CsvEnv` over an injected source yields string values; `true` or `7`
  may not deserialize into `bool` or `NonZeroUsize` after merging, and values
  containing commas are split into lists. Severity: medium. Likelihood: medium.
  Mitigation: `EP-M0` probe; the strict environment check (decision D7) types
  each value before pushing; if `CsvEnv` cannot preserve scalars, build the
  environment layer from the injected scan directly and record the deviation.
- Risk: Fluent wraps interpolated values (file paths, flag names) in Unicode
  bidirectional isolation marks (U+2068, U+2069), which would appear in
  terminal output and snapshots. Severity: medium. Likelihood: high.
  Mitigation: build the consumer bundle with `set_use_isolating(false)` and
  pass it through `FluentLocalizerBuilder::with_consumer_bundle`; this may
  require a direct `fluent-bundle` dependency, pre-approved conditionally in
  `Interfaces and dependencies`; `EP-M0` decides.
- Risk: `clap` adds its built-in `--help` and `--version` arguments after
  localization runs, so their descriptions stay in English. Severity: low.
  Likelihood: high. Mitigation: disable the built-ins and declare explicit
  `help` and `version` arguments with `ArgAction::Help` and
  `ArgAction::Version`, whose descriptions are catalogue messages; verified in
  `EP-M0`.
- Risk: a missing catalogue entry silently falls back to `clap`'s stock
  English text, producing mixed-language help. Severity: medium. Likelihood:
  medium. Mitigation: an identifier-coverage test records every identifier
  `ortho_config` requests and asserts each resolves in every shipped catalogue.
- Risk: `ortho_config`'s own bundled defaults exist only for languages `en`
  and `ja`; building a `FluentLocalizer` for any other language fails with
  `UnsupportedLocale` unless defaults are disabled. Severity: low. Likelihood:
  certain. Mitigation: locale resolution maps every locale without a shipped
  catalogue to `en` before building the localizer (decision D15); construction
  failure falls back to `NoOpLocalizer`.
- Risk: Whitaker's `no_std_fs_operations` and `no_expect_outside_tests` fire
  in tests, `rstest-bdd` steps, and shared helpers. Severity: medium.
  Likelihood: high. Mitigation: `cap-std` in tests, `Result`-returning steps
  and helpers, `rstest` fixtures; `EP-M0` probes the lints.
- Risk: `rstest` drops `#[expect]` attributes on generated tests, while
  `AGENTS.md` forbids `#[allow]`. Severity: low. Likelihood: medium.
  Mitigation: design tests so no suppression is needed.
- Risk: the vendored `docs/rstest-bdd-users-guide.md` documents `0.5.0`, but
  `0.6.0` is current. Severity: low. Likelihood: high. Mitigation: use `0.6.0`;
  its migration guide's breaking changes do not affect the planned usage.
- Risk: `Cargo.lock` is ignored, so CI resolves the newest compatible
  versions and a `clap` release could change help layout. Severity: medium.
  Likelihood: medium. Mitigation: help is checked by message content and flag
  presence, not whole-screen snapshots; decision D11.
- Risk: configuration-file paths are canonicalized (symbolic links resolved)
  and non-UTF-8 directories are converted lossily by `ortho_config`. Severity:
  low. Likelihood: medium. Mitigation: decision D6 documents rebasing against
  the canonical directory; tested with a symbolic link.
- Risk: lib/bin documentation collision and ANSI escapes in rendered help.
  Severity: low. Likelihood: medium. Mitigation: `doc = false` on `[[bin]]`;
  `ColorChoice::Never`; both verified in `EP-M0`.

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
  `ortho_config_macros`, and `ortho_l10n` 0.11.0 sources and the v0.9.0 to
  v0.11.0 migration guides; plan revised.
- [ ] Second expert-panel review applied.
- [ ] Plan approved by the user.
- [ ] `EP-M0` feasibility probe (prototype; deleted before `EP-M1`).
- [ ] `EP-M1` run-outcome contract and library/binary split.
- [ ] `EP-M2` `validate` command surface without configuration.
- [ ] `EP-M3` layered configuration: file and environment.
- [ ] `EP-M4` localized help, parse errors, and messages.
- [ ] `EP-M5` documentation, tested examples, and roadmap closure.

## Surprises & discoveries

- Observation: in both 0.8.0 and 0.11.0, the subcommand loaders
  (`SubcmdConfigMerge`, `SelectedSubcommandMerge`, and their `_with_env`
  variants) have no input for an explicit file path; they discover files from a
  base directory, the home directory, and XDG directories. The root derived
  loader honours a renamed `--config` flag but loads top-level keys only.
  Evidence: `ortho_config-0.11.0/src/subcommand/sources.rs`
  (`SubcommandFileContext` holds only a base directory and an `EnvSource`),
  `src/subcommand/paths.rs`,
  `ortho_config_macros-0.11.0/src/derive/build/ config_flag.rs`. Impact:
  decision D2 stands.
- Observation: 0.11.0 adds process-free injection: `EnvSource` (named
  lookups, current directory, home and configuration-directory fallbacks),
  `ScanEnvSource` (prefix scans), `ProcessEnv` for production, `MapEnv` for
  tests, and `CsvEnv::with_source` to build an environment provider over an
  injected scan. Evidence: `ortho_config-0.11.0/src/env_source.rs`,
  `src/csv_env/mod.rs`. Impact: decision D3 now includes the environment layer;
  the same object carries locale variables (decision D15).
- Observation: 0.11.0 adds `ConfigFilePolicy` with
  `ConfigPathSelector::cli(Option<PathBuf>)`; its default
  `ExplicitMode::RequiredExclusive` makes a selected missing file an error and
  suppresses discovery. Evidence: `src/discovery/policy.rs`,
  `src/discovery/policy_selector.rs`. Impact: preferred file-loading route,
  subject to the `EP-M0` probe confirming `extends` chains and per-file paths.
- Observation: the generated derive code still contains public loaders that
  read `std::env::args_os()` and uses `#[allow(dead_code, reason = ...)]`.
  Evidence: `ortho_config_macros-0.11.0/src/derive/load_impl_entry.rs`. Impact:
  loaders banned in `Constraints`; lint probe in `EP-M0`.
- Observation: `merge_from_layers` still deserializes only the merged value,
  `merge_value` lets a JSON `null` in a later layer replace an earlier value,
  and no deny-unknown-keys support exists. Evidence: `src/declarative/merge.rs`,
  `src/error/types.rs`. Impact: decision D7 and the `null` negative control.
- Observation: 0.11.0 booleans follow a `--flag[=BOOL]` convention (a bare
  flag means `true`; `--flag=false` switches a configured `true` off), and
  `cli_default_as_absent` now supports parser-faithful string defaults.
  Evidence: vendored guide, "Switch a boolean off from the command line", and
  the v0.10.0 migration guide. Impact: decision D8 revised.
- Observation: `ortho_config` re-exports the `ortho_l10n` localization API:
  `Localizer`, `NoOpLocalizer`, `FluentLocalizer` and its builder,
  `LocalizedParse::try_parse_localized_from`,
  `localize_clap_error_with_command`, `message_id_for`, and the
  `localization_base` derive attribute. No locale is read from the environment
  by the library; the application builds the localizer for a
  `LanguageIdentifier` it chooses. Message identifiers are hyphen-joined, for
  example `mdast-check-validate-args-schema_format-help`. `clap` error
  identifiers are `clap-error-<kind>`, shared across applications, with bundled
  English (`en-US`) and Japanese texts. Bundled defaults are matched by
  language subtag only (`en-GB` uses `en-US`). Evidence:
  `ortho_l10n-0.11.0/src/fluent.rs`, `src/identifier.rs`,
  `ortho_config-0.11.0/src/localizer/clap_error.rs`,
  `src/localizer/clap_command/`. Impact: decision D15.
- Observation: much of upstream `docs/cli-localization-design.md` (an
  environment locale resolver, embedded-catalogue localizer, coverage
  reporting) is marked proposed and absent from the 0.11.0 code. Evidence: no
  matches for those types in the 0.11.0 sources. Impact: this crate owns locale
  resolution and coverage tests.
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
  declarative API. The CLI adapter parses arguments with `clap` (through
  `ortho_config`'s localized parse), then builds a `MergeComposer` with, in
  order, a defaults layer, one file layer per element of the `--config` file's
  `extends` chain (preferably from `ConfigFilePolicy` with
  `ConfigPathSelector::cli`, otherwise `load_config_file_as_chain`; focused on
  `cmds.validate`), an environment layer from
  `CsvEnv::prefixed("MDAST_CHECK_CMDS_VALIDATE_").with_source(env)`, and a
  command-line layer of the values the user supplied.
  `ValidateConfig::merge_from_layers` performs the merge. Rationale:
  `ortho_config` 0.11.0 still offers no subcommand loader that honours an
  explicit file; the declarative API keeps merge semantics in the library and
  is process-free with injected sources. Rejected alternatives: the subcommand
  `_with_env` loaders (no explicit file); the root derived loader with
  `config_cli_long = "config"` (honours `--config` but flattens the subcommand
  and its file shape). Recorded as ADR 0002 with a revisit trigger: re-evaluate
  when `ortho_config` gains explicit-file support for subcommands. Date/Author:
  2026-11-20, planning agent; revised for 0.11.0.
- D2a. Decision: build the command-line layer with
  `CliValueExtractor::extract_user_provided` on the `clap` derive struct, with
  `cli_default_as_absent` on any defaulted argument, if `EP-M0` shows it works
  with the `--flag[=BOOL]` presence flags; otherwise use a small builder over
  `Option` fields that omits absent values. Both variants are covered by the
  same presence tests and negative controls. Rationale: moves "an absent flag
  leaves lower layers alone" into the library. Date/Author: 2026-11-20, design
  review.
- D3. Decision: configuration sources in this item are built-in defaults, a
  file named explicitly by `--config`, the environment
  (`MDAST_CHECK_CMDS_VALIDATE_<KEY>`, upper-case key, for example
  `MDAST_CHECK_CMDS_VALIDATE_MAX_ERRORS=10`), and the command line, in that
  precedence order. Implicit file discovery and a configuration-path
  environment variable are deferred; when discovery is added, an explicit
  `--config` disables it, and the discovery name `.mdast-check.toml` is
  reserved now in ADR 0002. Rationale: matches the design's conventional
  precedence (command line, environment, file, defaults) and `ortho_config`'s
  orthographic naming (one key spelling across sources); 0.11.0 makes the
  environment layer process-free through the injected source. Discovery would
  make behaviour depend on the working and home directories and is not required
  by the roadmap. Date/Author: 2026-11-20, planning agent; revised for 0.11.0.
- D4. Decision: until the validation pipeline exists, the shipped binary's
  driven adapter (`pipeline::PendingPipeline`) fails every well-formed
  `validate` run with the catalogue message
  `mdast-check-error-validation-unavailable` ("document validation is not
  implemented yet") and exit `5`. Rationale: exit `0` would tell CI an
  unchecked document is valid; `2` would blame a correct invocation; `5` is the
  only honest non-zero code, although the design defines it as an unexpected
  failure. The users' guide states that `5` is not transient. A tripwire test,
  `placeholder_pipeline_exits_5_until_1_3_2`, and an acceptance bullet on
  roadmap 1.3.2 carry the removal obligation. Date/Author: 2026-11-20, planning
  agent; confirmed by design review.
- D5. Decision: `--document` is command-line only and required by `clap`;
  every other setting (`schema`, `schema_format`, `format`, `fail_fast`,
  `max_errors`, `quiet`) may come from any source. `--schema` is required after
  merging. `--config` selects the file layers and is never itself a
  configuration key. Rationale: the design names a default schema path but
  never a default document. Date/Author: 2026-11-20, planning agent.
- D6. Decision: a relative `schema` path read from a configuration file is
  resolved against the canonical directory of the file that set it; a relative
  `schema` from the environment or command line stays relative to the current
  directory; an empty `schema` from any source is an invocation error.
  Rationale: checked-in configuration then works from any working directory;
  environment and command-line values are typed relative to where the user is.
  Date/Author: 2026-11-20, planning agent; amended after review.
- D7. Decision: every configuration source is typed strictly before merging.
  Each file in the `extends` chain is deserialized on its own into
  `#[serde(deny_unknown_fields)]` structs (root: only `cmds`; `cmds`: only
  `validate`; `validate`: only the six keys, each `Option`). The environment
  layer, after `CsvEnv` lower-cases and strips the prefix, is deserialized into
  the same `validate` struct. Failures are invocation errors naming the file or
  the variable (`MDAST_CHECK_CMDS_VALIDATE_<KEY>`) and the offending key. Only
  checked layers are re-serialized, omitting absent keys, and pushed. Rationale:
  `ortho_config` ignores unknown keys and reports type errors only after
  merging, without a source; rejecting unknown keys later would be a breaking
  change. Date/Author: 2026-11-20, design review; extended to the environment
  layer.
- D8. Decision: `--max-errors` takes a positive integer modelled as
  `NonZeroUsize`; `0` is an invocation error from any source. `--fail-fast` and
  `--quiet` follow `ortho_config`'s boolean convention `--flag[=BOOL]`: a bare
  flag sets `true`, `--quiet=false` overrides a configured `true`, and
  `--quiet false` (space-separated) is rejected so that a following
  positional-looking token is never swallowed. Repeating a flag is an
  invocation error. `--schema-format` is optional; inference from the file
  extension is deferred to roadmap item 1.3.1. Date/Author: 2026-11-20,
  planning agent; revised for 0.11.0.
- D9. Decision: help and version requests write plain, localized text to
  standard output and exit `0`; every other `clap` error, including a missing
  subcommand, is localized with `localize_clap_error_with_command`, written to
  standard error, and exits `2` (decided from `clap::Error::kind`, which
  localization preserves). Invocation errors always render as human-readable
  text on standard error, even with `--format json`, because the JSON envelope
  belongs to roadmap 2.2.1. A failed write to standard output while printing
  help or version yields exit `5`; a failed write to standard error leaves the
  decided status unchanged. Rationale: keeps the exit-code mapping in one place
  instead of `clap::Error::exit`, which bypasses injected writers. Date/Author:
  2026-11-20, planning agent; revised for localization.
- D10. Decision: no Verus or Kani artefacts in this item (see
  `Formal verification decision`). Date/Author: 2026-11-20, planning agent;
  confirmed by design review.
- D11. Decision for the approver: whether to commit `Cargo.lock` (a separate
  pull request). The plan works either way. Recommendation: commit it, as Cargo
  recommends for binaries. Date/Author: 2026-11-20, design review.
- D12. Decision: `--schema -` is an invocation error, reserving `-` for the
  document. A file named `-` remains reachable as `./-`. Date/Author:
  2026-11-20, design review.
- D13. Decision: name the application core `src/app/`, the placeholder
  adapter's module `src/pipeline/`, and the outcome type `RunOutcome`.
  Rationale: the design reserves `validate` for the validator wrapper;
  `RunOutcome` avoids confusion with `std::process::ExitStatus`. Date/Author:
  2026-11-20, design review.
- D14. Decision: the pipeline port is provisional, with input split into
  `ValidationInput` and `ReportOptions` and failures in a small
  `PipelineFailure` enum carrying a `const fn outcome(&self) -> RunOutcome`.
  Roadmap 1.1.2 replaces `PipelineFailure`; 1.3.2 adds normalized failures to
  `Verdict::Invalid`; 2.1.2 and 2.2.1 add renderers behind the CLI's single
  reporting function. Pipeline failure messages are rendered by the CLI from
  catalogue identifiers, not passed through as text. Date/Author: 2026-11-20,
  design review; amended for localization.
- D15. Decision: localize all user-facing text of this item through
  `ortho_config` 0.11.0. The locale is resolved once per run from the injected
  environment: the first non-empty of `LC_ALL`, `LC_MESSAGES`, and `LANG`;
  POSIX forms such as `ja_JP.UTF-8@euro` are normalized to a BCP 47 tag
  (`ja-JP`); `C`, `POSIX`, unparsable values, and languages without a shipped
  catalogue resolve to `en`. The CLI builds one `FluentLocalizer` for that
  locale with the matching catalogue from `locales/<language>/ messages.ftl`
  (embedded with `include_str!`), passing a consumer bundle with Unicode
  isolation disabled; if construction fails, it uses `NoOpLocalizer` and the
  built-in English text. The command is parsed with
  `LocalizedParse::try_parse_localized_from`, so `about`, argument help, value
  names, and the explicitly declared `--help` and `--version` descriptions are
  catalogue messages; `clap` errors use `ortho_config`'s shared `clap-error-*`
  texts; crate-authored messages (invocation errors, the placeholder) use
  identifiers under `mdast-check-error-*`. Rationale: the library leaves locale
  resolution to applications; resolving it at the composition boundary from the
  same injected environment keeps tests hermetic; one catalogue per language
  keeps translation work in one file. Recorded as ADR 0003. Date/Author:
  2026-11-20, planning agent.
- D16. Decision for the approver: ship only the English catalogue
  (`locales/en/messages.ftl`, written in the project's en-GB Oxford style) in
  this item, and prove the localization path with a test catalogue in a second
  language that exists only in the test suite. Rationale: a shipped translation
  is a maintenance commitment that needs a fluent reviewer and a localization
  style guide, glossary, and translator's guide (the `localization-docs` set);
  infrastructure plus complete English satisfies "localized help support"
  without an unreviewed translation. Alternative: ship a Japanese catalogue now
  (the cheapest second language, because `ortho_config` already bundles Japanese
  `clap` error texts), with the localization documentation set, adding about
  120 code and 250 documentation lines. Date/Author: 2026-11-20, planning agent.

## Outcomes & retrospective

Not yet started. To be completed at each milestone and at the end.

First design review (2026-11-20, against the 0.8.0 draft): three reviewers
returned "proceed with conditions". Adopted: typed per-source layers (D7),
banned ambient loaders, the provisional split port (D14), module renaming
(D13), Whitaker and `cap-std` handling, chain-order and `null`-override tests,
an argument-robustness property, placeholder removal obligations (D4), the
discovery reservation (D3), `--schema -` (D12), and the `Cargo.lock` question
(D11). Declined: dropping `googletest`, `pretty_assertions`, and `proptest`,
which the user's request names or a genuine many-input invariant needs.

## Context and orientation

Inspected base revision: `788b9d444c418b117b61e9eaa05a98f277228852` on `main`.
The working branch is `1-1-1-root-cli-command-and-validate-subcommand`, stacked
on `refresh-typos-config` (pull request #143), which regenerates `typos.toml`
only. Branch commits so far: the plan, the vendored `ortho-config` v0.11.0
user's guide (`5b61faf`), and the design and roadmap retargeting (`c009f8c`).

Repository layout today:

- `Cargo.toml`: package `mdast_check`, edition 2024, no runtime dependencies,
  dev-dependency `rstest = "0.28.0"`, strict `[lints.clippy]`,
  `[lints.rust] missing_docs = "deny"`, and
  `[lints.rustdoc] missing_crate_level_docs = "deny"`.
- `src/main.rs`: the only source file; a stub with `FIXME(#5)`.
- `tests/build_standard_contract.rs` and `tests/build_standard_support/`: a
  contract over `.cargo/config.toml`, `rust-toolchain.toml`, `make -n` output
  for `test`, `lint`, `build`, and `release`, and CI `setup-rust` steps. It
  does not read `Cargo.toml`, the binary name, or new test files.
- `Makefile`: `TARGET ?= mdast_check`; targets `build`, `release`, `test`,
  `lint` (rustdoc, Clippy, Whitaker), `fmt`, `check-fmt`, `markdownlint` (which
  runs `spelling` first), `spelling`, `nixie`, `workflow-contracts`.
- `.github/workflows/ci.yml`: `make check-fmt`, `make spelling`,
  `make workflow-contracts`, markdownlint, `make lint`, and coverage (which
  runs the tests). Verus and Kani are not installed in CI.
- `.gitignore` ignores `Cargo.lock`.
- `rust-toolchain.toml`: `nightly-2026-11-20` with `rustfmt` and `clippy`.
- `docs/`: the design, the roadmap, ADR 0001 (build standard), the
  developers' guide, the documentation style guide (which has no localization
  section), and vendored guides, including `ortho-config-users-guide.md` at
  v0.11.0. There is no `docs/users-guide.md`.

Tools installed on the host but not configured by the repository: `kani`,
`cargo-kani`, and `prover-tools` (which can install Verus). Whitaker lint
libraries are under `~/.local/share/whitaker/`.

Upstream libraries, versions checked on crates.io on 2026-11-20: `ortho_config`
0.11.0 (MSRV 1.89.0; depends on `ortho_l10n` 0.11.0, `clap ^4` with `derive`,
`env`, and `string`, `figment ^0.10`, `cap-std ^4` with `fs_utf8`, `camino ^1`
with `serde1`, `toml ^1`; default features `serde_json` and `toml`; re-exports
`serde_json`, `toml`, `langid!`, and `LanguageIdentifier`), `ortho_l10n` 0.11.0
(`fluent-bundle ^0.16`, `fluent-syntax ^0.12`, `unic-langid ^0.9`), `clap`
4.6.9, `serde` 1.0.229, `camino` 1.2.6, `thiserror` 2.0.21, `cap-std` 4.0.3,
`rstest` 0.28.0, `rstest-bdd` and `rstest-bdd-macros` 0.6.0, `googletest`
0.14.4, `pretty_assertions` 1.4.1, `insta` 1.49.0, `assert_cmd` 2.2.2,
`proptest` 1.12.0, `tempfile` 3.27.0.

`ortho_config` 0.11.0 facts this plan relies on, from its published source:

- `OrthoResult<T>` is `Result<T, Arc<OrthoError>>`; `OrthoError` is
  `#[non_exhaustive]` with variants including `CliParsing`,
  `File { path, source }`, `CyclicExtends`, `Gathering`, `Merge`, `Validation`,
  and `Aggregate`.
- `#[derive(OrthoConfig)]` generates `merge_from_layers`; `MergeComposer` has
  `push_defaults`, `push_file(value, Option<Utf8PathBuf>)`, `push_environment`,
  `push_cli`, and `layers()`.
- `load_config_file_as_chain(&Path) -> OrthoResult<Option<FileLayerChain>>`
  returns `(value, Utf8PathBuf)` pairs from root ancestor to the named file,
  `extends` stripped and paths canonicalized; `Ok(None)` for a missing file.
  `ConfigFilePolicy::from_builder(..)` with
  `.selectors([ConfigPathSelector::cli(path)])` and `.resolve_layers()` returns
  a `FileLayerOutcome` with `into_result()` and `push_into(&mut MergeComposer)`.
- `EnvSource`, `ScanEnvSource`, `ProcessEnv`, `MapEnv` (`new`, `with_var`,
  `with_current_dir`), and
  `CsvEnv::prefixed(..).with_source(Arc<dyn ScanEnvSource>)` provide injected
  environment access.
- `CliValueExtractor::extract_user_provided(&self, &ArgMatches)` serializes
  only user-supplied values, honouring `cli_default_as_absent`.
- `is_display_request(&clap::Error)` is true only for help and version.
- Localization: `FluentLocalizer::builder(locale)`,
  `.with_consumer_resources(..)` or `.with_consumer_bundle(..)`, `.try_build()`;
  `LocalizedParse::try_parse_localized_from(iter, &dyn Localizer)`;
  `localize_clap_error_with_command`; identifiers from `message_id_for`; the
  derive attribute `localization_base` generates identifier constants.

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
  the identifier scheme; treat sections marked proposed as not implemented.
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

Upstream artefacts at branch revision `c009f8c`:

- Terms of Reference: none exists for this repository.
- Technical design: `docs/mdast-schema-validation-cli-design.md` (status
  Proposed), which now names `ortho-config` v0.11.0 and localized help.
- Roadmap: `docs/roadmap.md` item 1.1.1, including the localization
  sub-task added at the user's direction.
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
  v0.11.0; the command line overrides the environment, which overrides
  configuration files, which override built-in defaults (roadmap 1.1.1; design
  §Schema configuration).
- `REQ-l10n`: command help and argument-parsing errors are localized through
  `ortho-config` v0.11.0 (roadmap 1.1.1 localization sub-task; design §Design
  drivers).
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
  -> cli::config::tests::precedence_matches_oracle, *_layer_presence_is_exact
REQ-l10n -> TDD §Design drivers -> ADR 0003 -> cli::l10n -> EP-M4
  -> cli::l10n::tests::every_requested_identifier_resolves
GAP-stub, GAP-no-lib -> EP-M1, EP-M2
GAP-no-users-guide, REQ-docs -> EP-M5 -> tests/users_guide_examples.rs
```

Design deviations requiring acceptance with this plan: D3 (discovery deferred),
D4 (exit `5` placeholder), D12 (`--schema -` reserved), D13 (module naming),
and D16 (English-only catalogue). All are recorded in the design document in
`EP-M5`; D4 also adds an acceptance bullet to roadmap 1.3.2.

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

If `resolve(schema)` is undefined the run is an invocation error. Each `F_i` and
`E` is admitted only if it type-checks on its own (decision D7).

Locale: given the injected values of `LC_ALL`, `LC_MESSAGES`, and `LANG`,
`locale = normalize(first non-empty)`, and `catalogue(locale)` is the shipped
catalogue for `language(locale)` if one exists, else the English catalogue.
Every message the run prints is `lookup(catalogue, id, args)`.

A run maps an argument vector, an injected environment, and the read-only file
system to exactly one `RunOutcome`. Stages: resolve the locale and build the
localizer; parse arguments; load and check each file layer; build and check the
environment layer; merge; build the validation input; call the pipeline port.
Any failure from parsing onwards before the pipeline yields `InvalidInvocation`
and the pipeline is never called. The pipeline returns `Valid`, `Invalid`, or a
failure of kind `Schema`, `MarkdownParse`, or `Internal`, mapped to `Success`,
`ValidationFailed`, `SchemaFailure`, `MarkdownParseFailure`, and
`InternalFailure`. Help and version yield `Success` without calling the
pipeline. Locale resolution never fails.

Exit codes: `Success -> 0`, `ValidationFailed -> 1`, `InvalidInvocation -> 2`,
`SchemaFailure -> 3`, `MarkdownParseFailure -> 4`, `InternalFailure -> 5`.

### Invariants and lemmas

- `INV-exit-table`: `RunOutcome::exit_code` equals the design table; codes
  are pairwise distinct; `0` if and only if `Success`. (`REQ-exit-codes`)
- `INV-single-outcome`: for every argument vector and environment, `run`
  returns exactly one `RunOutcome` without panicking, calls the pipeline at
  most once, and exactly when the outcome came from the pipeline; the binary's
  exit code equals `exit_code` of that outcome. (`REQ-exit-codes`)
- `INV-stage-order`: on any invocation or configuration error the pipeline is
  called zero times and standard output is empty. (`REQ-early-stop`)
- `INV-precedence`: every key resolves to `resolve(k)`, including across
  `extends` chains. (`REQ-config-precedence`)
- `INV-absent-transparent`: an unsupplied flag or unset variable contributes
  nothing to `C` or `E`, not even a JSON `null`.
- `INV-bool-override`: `--quiet=false` and `--fail-fast=false` override a
  `true` from the environment or a file.
- `INV-ambient-isolation`: the run reads the environment only through the
  injected object and only the variables `MDAST_CHECK_CMDS_VALIDATE_*`,
  `LC_ALL`, `LC_MESSAGES`, and `LANG`; no configuration file is read without
  `--config`, whatever the working, home, or XDG directories contain.
- `INV-explicit-config`: if `--config` names a file that is missing, a
  directory, malformed, part of an `extends` cycle, not TOML, or any of whose
  chain files contains an unknown key at any level or a wrongly typed value,
  the run exits `2`, the message names the offending file, and the pipeline is
  not called.
- `INV-env-strict`: an unknown `MDAST_CHECK_CMDS_VALIDATE_*` variable or a
  wrongly typed value exits `2` and the message names the variable.
- `INV-path-rebase`: a relative `schema` from a file becomes the canonical
  directory of that file joined with the value; absolute values and values from
  the environment or command line are unchanged.
- `INV-document-source`: `--document -` yields `DocumentSource::Stdin`; any
  other value yields `DocumentSource::Path`; `--schema -` and empty paths are
  invocation errors.
- `INV-max-errors-positive`: a resolved `max_errors` is at least 1.
- `INV-streams`: help and version go to standard output; all diagnostics go to
  standard error; write failures follow decision D9.
- `INV-enum-spellings`: the command line, environment, and file accept
  exactly the same spellings for `format` and `schema_format`.
- `INV-locale-resolution`: the locale is the normalization of the first
  non-empty of `LC_ALL`, `LC_MESSAGES`, `LANG`; resolution is total and always
  yields a language with a shipped catalogue. (`REQ-l10n`)
- `INV-catalogue-complete`: every message identifier that the localized parse
  or the CLI requests resolves in every shipped catalogue, so no English
  fallback text appears in a localized run. (`REQ-l10n`)
- `INV-localized-output`: with a catalogue for locale `L`, the help, `clap`
  error, invocation-error, and placeholder texts printed are those of `L`'s
  catalogue; exit codes are independent of the locale.

- `L-cli-layer`: the command-line layer has key `k` if and only if the flag
  for `k` was supplied, with the parsed value.
- `L-env-layer`: the environment layer has key `k` if and only if
  `MDAST_CHECK_CMDS_VALIDATE_<K>` is set in the injected environment, with the
  typed value.
- `L-file-layer`: each pushed file layer is that file's checked
  `cmds.validate` table, absent keys omitted, `schema` rebased.
- `L-layer-order`: layers are pushed as defaults, files (chain order),
  environment, command line. With `AXIOM-ortho-merge`, `L-cli-layer`,
  `L-env-layer`, `L-file-layer`, and `L-layer-order` yield `INV-precedence`.
- `L-input-total`: building `ValidationInput` and `ReportOptions` from merged
  configuration succeeds with every field typed or fails only on a missing
  schema.

### Axioms (trusted, not verified here)

- `AXIOM-ortho-merge`: `merge_from_layers` applies layers in push order; for
  scalar keys a later value replaces an earlier one; an absent key leaves it; a
  `null` replaces it. Source: `ortho_config` 0.11.0 `declarative::merge_value`.
  Exercised at the boundary by the precedence property, which uses the real
  library.
- `AXIOM-ortho-file`: the chosen file API parses TOML, follows `extends`,
  orders the chain root first, canonicalizes paths, reports cycles, parse
  failures, and non-TOML extensions as errors, and reports a missing explicitly
  selected file (directly or as `Ok(None)`). Exercised with real files.
- `AXIOM-ortho-env`: `CsvEnv::prefixed(p).with_source(s)` yields exactly the
  variables of `s` starting with `p`, prefix stripped and key lower-cased,
  without reading the process environment. Exercised by the environment
  presence test with `MapEnv`.
- `AXIOM-ortho-l10n`: `LocalizedParse::try_parse_localized_from` applies
  catalogue text for every identifier it requests and leaves `clap` text where
  a lookup returns nothing; `localize_clap_error_with_command` preserves
  `clap::Error::kind`; `FluentLocalizer` consults consumer resources before
  bundled defaults. Exercised by the localization tests.
- `AXIOM-clap`: `clap` 4 reports help and version as `DisplayHelp` and
  `DisplayVersion`, rejects repeated flags and non-UTF-8 input for UTF-8 value
  parsers; `Cli::command().debug_assert()` checks the derive.
- `AXIOM-serde`: `serde` rejects `0` for `NonZeroUsize` and names unknown
  fields under `deny_unknown_fields`.
- `AXIOM-process`: `std::process::ExitCode::from(u8)` becomes the process exit
  status. Exercised by `assert_cmd` tests.

### Methods per obligation

- Obligation: `INV-exit-table`. Method: exhaustive `rstest` table with
  literal expected codes copied from the design, plus a distinctness test over
  `RunOutcome::ALL`. Artefact: `src/outcome.rs` tests `exit_codes_match_design`
  and `codes_are_distinct_and_zero_only_for_success`. Residual gap: nothing
  forces `ALL` to grow with a new variant; review and the design table guard
  it. Negative control: swap two codes; both rows fail.
- Obligation: `INV-single-outcome`, `INV-streams`. Method: `proptest` over
  argument vectors of 0 to 8 tokens (every flag in long and `=` forms,
  `validate`, `--help`, `--version`, `-`, `--`, plausible and junk values, and
  on Unix non-UTF-8 bytes) with an empty `MapEnv`, plus end-to-end tests.
  Artefact: `src/cli/run/tests.rs`, property `run_always_yields_one_outcome`
  with a `RecordingPipeline` (call counter, returns `Valid`); unit test
  `Cli::command().debug_assert()`; `FailingWriter` and flush-failure tests;
  `tests/validate_cli.rs` with exact exit codes and stream contents, version
  compared with `env!("CARGO_PKG_VERSION")`, and the tripwire
  `placeholder_pipeline_exits_5_until_1_3_2`. Non-vacuity: the strategy
  produces a fully valid invocation shape with probability above one half, and a
  `TestRunner` with a shared counter asserts that both the pipeline-called and
  pipeline-not-called classes occurred. Regressions are committed under
  `proptest-regressions/`.
- Obligation: `INV-stage-order`. Method: `rstest` table with
  `RecordingPipeline`, one row per invocation-error class (unknown flag,
  missing document, missing schema, `--schema -`, empty schema, invalid format,
  zero cap, repeated `--quiet`, `--quiet false`, missing configuration file,
  malformed configuration, unknown file key, unknown environment variable,
  wrongly typed environment value). Artefact: `src/cli/run/tests.rs`,
  `pipeline_not_called_on_invocation_error`. Non-vacuity: a valid control row
  asserts one call; seeded fault: call the pipeline before the schema check;
  the missing-schema row fails.
- Obligation: `L-cli-layer` and `L-env-layer`. Method: exhaustive presence
  enumeration per layer. Rationale: whether a layer defines each key is a
  finite space (`2^6 = 64` subsets per layer), so enumeration is complete.
  Artefact: `src/cli/config/tests.rs`, tests `cli_layer_presence_is_exact`
  (builds the argument vector for each subset, parses with the real `clap`
  definition, builds the layer, and asserts its key set equals the subset and
  its values equal fixed witnesses) and `env_layer_presence_is_exact` (same
  through `CsvEnv` over a `MapEnv`). Negative controls: emit
  `fail_fast = false` for an absent flag; emit `null` for absent optional keys;
  both make the respective rows fail.
- Obligation: `INV-precedence`, `INV-absent-transparent`, `L-layer-order`.
  Method: property test against an independent oracle, plus targeted rows.
  Rationale: with four layers the presence space is `8^6 = 262,144`
  combinations (each key absent or present in each of `F`, `E`, `C`); each case
  runs a real `clap` parse and merge, too slow to enumerate in the default
  suite, while the per-layer presence lemmas above are enumerated exhaustively.
  The property samples the combined space with full coverage accounting. Domain:
  `proptest` presence vectors drawing each key's class (which of `F`, `E`, `C`
  define it) uniformly from the eight classes, 1,024 cases. Witness values are
  fixed and distinct per layer and from the defaults (for booleans the earlier
  layer holds `true` and the later layer `false`, using `--flag=false`).
  Artefact: `src/cli/config/tests.rs`, property `precedence_matches_oracle`,
  which writes nothing to disk (file layers are pushed through the real D7
  check from in-memory TOML parsed with `ortho_config::toml`), uses `MapEnv` for
  `E`, the real `clap` parse for `C`, and the real `merge_from_layers`; the
  oracle is a short function in the test module applying `resolve(k)` with
  literal defaults copied from the design. Non-vacuity: a `TestRunner` records
  every `(key, class)` pair visited and asserts all 48 occurred (expected about
  128 visits each). Chain order and file-versus-environment rows are explicit
  `rstest` cases: parent `format = "json"`, child `"human"` gives `human`;
  child unset gives `json`; environment `MDAST_CHECK_CMDS_VALIDATE_FORMAT=json`
  over a file `human` gives `json`. Negative control: push the environment
  layer after the command-line layer; the property and the rows fail.
- Obligation: `INV-ambient-isolation`. Method: in-process and end-to-end.
  Artefact: `src/cli/run/tests.rs` row with a `MapEnv` whose current directory
  and home hold candidate files and which sets unrelated variables (`HOME`,
  `XDG_CONFIG_HOME`, `MDAST_CHECK_CONFIG_PATH`, `MDAST_CHECK_SCHEMA`) and
  asserts the missing-schema outcome; and `tests/validate_cli.rs`,
  `ambient_configuration_is_ignored`, running the binary with `env_clear()`,
  `HOME` and `XDG_CONFIG_HOME` pointing at a temporary directory with candidate
  files, `current_dir` containing `.mdast-check.toml`, asserting exit `2` with
  the missing-schema message. Non-vacuity: a companion row passes `--config` to
  the same file and reaches the placeholder (exit `5`).
- Obligation: `INV-explicit-config`, `INV-env-strict`, `L-file-layer`.
  Method: `rstest` tables with real files and `MapEnv`. Domain: missing file,
  directory, malformed TOML, `extends` cycle, `.json` extension, extensionless
  file (accepted), unknown key, `document` or `config` under `[cmds.validate]`,
  root-level `schema`, `[cmd.validate]`, `[cmds.lint]`, wrong types
  (`max_errors = "ten"`, `0`, `format = "xml"`), `schema = ""`, errors in a
  parent file of a chain, `MDAST_CHECK_CMDS_VALIDATE_FAILFAST=1`,
  `MDAST_CHECK_CMDS_VALIDATE_MAX_ERRORS=0`. Artefact: `src/cli/config/tests.rs`,
  `invalid_configuration_is_rejected` and `invalid_environment_is_rejected`,
  plus end-to-end rows. Evidence: exit `2` and standard error names the file or
  variable. Non-vacuity: the extensionless row is accepted; parent-file rows
  prove checks cover the whole chain.
- Obligation: `INV-path-rebase`. Method: `rstest` rows on the real file
  system, including a Unix symbolic link to a configuration file elsewhere.
  Oracle independent of `join`: create the schema file at its intended place in
  a canonicalized temporary directory and compare canonical paths. Artefact:
  `relative_schema_paths_resolve_against_their_file`. Non-vacuity: absolute and
  environment-supplied relative values must not change.
- Obligation: `INV-document-source`, `INV-max-errors-positive`,
  `INV-enum-spellings`, `INV-bool-override`, `L-input-total`. Method: `rstest`
  tables through every source. Artefacts: `src/cli/args/tests.rs`,
  `src/cli/config/tests.rs` (`enum_spellings_agree`,
  `boolean_can_be_switched_off`, `input_requires_schema`).
- Obligation: `INV-locale-resolution`. Method: `rstest` table plus a
  `proptest` totality property. Domain: rows for precedence (`LC_ALL` over
  `LC_MESSAGES` over `LANG`, empty values skipped), POSIX forms (`en_GB.UTF-8`,
  `ja_JP.eucJP@mod`, `de`), `C`, `POSIX`, garbage, and unset; property over
  arbitrary strings for the three variables asserting resolution returns
  without panicking and yields a language in the shipped set. Artefact:
  `src/cli/l10n/tests.rs`, `locale_resolution_table` and
  `locale_resolution_is_total`. Non-vacuity: rows include each variable
  winning; negative control: swap the `LC_ALL` and `LANG` order; the precedence
  rows fail.
- Obligation: `INV-catalogue-complete`. Method: recording localizer. A test
  wraps the real localizer in a `RecordingLocalizer` (as in `ortho_config`'s
  own tests), parses `validate --help` and the root help, renders one of each
  crate-authored message and each `clap` error kind the run maps, and asserts
  every requested identifier resolves in every shipped catalogue and in the
  test catalogue. A second test parses each catalogue and asserts it has no
  identifiers the recorder never requested (no dead entries). Artefact:
  `src/cli/l10n/tests.rs`, `every_requested_identifier_resolves` and
  `catalogues_have_no_unused_identifiers`. Non-vacuity: the recorded set is
  asserted non-empty and to contain the identifier for every argument; negative
  control: delete one entry from the English catalogue; the test fails naming
  it.
- Obligation: `INV-localized-output`. Method: in-process tests with a test
  catalogue for a second language (decision D16) plus `insta` snapshots.
  Artefact: `src/cli/l10n/tests.rs` and `src/cli/run/tests.rs`: with a `MapEnv`
  setting `LANG` to the test language, `validate --help` contains the test
  catalogue's argument texts, an unknown flag produces the test catalogue's or
  `ortho_config`'s localized `clap` error, the missing-schema message is the
  test catalogue's, and the exit codes match the English run. Snapshots (with
  temporary paths filtered to `[TMP]`) cover the English invocation-error and
  placeholder messages. End-to-end: the binary with `LANG=C` and with
  `LANG=xx_YY.UTF-8` (no catalogue) prints English.
- Obligation: user-facing wording. Method: `insta` snapshots of every
  crate-authored message in English, as above; help is asserted by content
  rather than snapshotted (risk: `Cargo.lock` ignored).

### Formal verification decision

No Verus proof and no Kani harness are planned for this item (decision D10).

Verus is warranted when a small, pure, repository-owned kernel must satisfy a
property for unboundedly many inputs. This item has two candidate kernels, and
neither justifies a proof. Layer precedence is implemented by `ortho_config`'s
merge; this crate owns only the construction of the layers, whose presence
behaviour is a finite space enumerated exhaustively per layer and sampled with
full coverage accounting across layers against the real merge, so a Verus proof
would verify a model of a third-party library rather than the code that runs.
Locale resolution is owned and pure, but its logic is "first non-empty of three
values, then normalize"; the unbounded part is string normalization delegated to
`unic-langid` parsing, which Verus cannot reason about without trusting the
parser, so the proof would rest on an axiom equivalent to the property. A
totality property test and a precedence table give proportionate assurance. The
exit-code table is a six-element finite function, which enumeration proves.
Kani would add bounded exhaustiveness to code with no arithmetic, no `unsafe`,
and no owned state machine, and leans on `serde_json`, `clap`, and Fluent,
which are impractical to model check.

Candidates for formal verification in later items: the fallback span selection
of roadmap 2.1.1, the deterministic ordering and capping of roadmaps 2.2.1 and
2.2.2 (a prefix of a total order of length `min(n, cap)`), and the
instance-path registration of roadmap 1.2.2.

### Suggested BDD features

Feature file: `tests/features/validate_command.feature`, bound by
`tests/validate_command_bdd.rs` with `rstest-bdd` 0.6.0. Steps drive
`mdast_check::cli::run` in-process with a scripted fake pipeline and a
`MapEnv`, so every exit code and locale is reachable without touching the
process. Paths are made absolute inside a per-scenario temporary directory.
Steps return `Result` and never call `expect`. Milestones: `EP-M2` (first
block), `EP-M3` (configuration block), `EP-M4` (localization block).

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
    Given the test catalogue is installed for the language "xx"
    And the environment variable "LANG" is "xx_YY.UTF-8"
    When the user runs "validate --help"
    Then standard output contains the "xx" text for "mdast-check-validate-args-schema-help"

  Scenario: An unknown locale falls back to English
    Given the environment variable "LANG" is "zz_ZZ.UTF-8"
    When the user runs "validate --help"
    Then standard output contains the English text for "mdast-check-validate-args-schema-help"
```

The identifier in the last two scenarios is illustrative; the exact identifiers
come from `message_id_for` and are fixed in `EP-M4`. The "test catalogue" step
installs an in-memory catalogue through the same injection point the CLI uses
(see `Interfaces and dependencies`), so it is a test double for shipped
catalogues, not a production feature.

Runner: `cargo test --test validate_command_bdd` (also run by `make test`).

## Plan of work

### `EP-M0` feasibility probe (prototype, deleted)

Question: can the planned `ortho_config` 0.11.0 usage compile and behave as
assumed under this repository's Clippy and Whitaker lints? Work in the branch
worktree after approval. Add the dependencies from
`Interfaces and dependencies`, an empty `src/lib.rs` with crate documentation,
and a throwaway integration test `tests/probe_ortho_config.rs` that:

1. derives `Parser`, `OrthoConfig`, `Deserialize`, `Serialize`, and `Default`
   (or the two-struct variant) for the planned validate settings, with
   `localization_base = "mdast-check"`, and prints the generated identifier
   constants;
2. loads a two-file `extends` chain through `ConfigFilePolicy` with
   `ConfigPathSelector::cli` and through `load_config_file_as_chain`, and
   records which gives per-file paths, chain order, and a missing-file error;
3. builds the environment layer with `CsvEnv::prefixed(..).with_source(..)`
   over a `MapEnv` and records the value types for `true`, `7`, `json`, and a
   value containing a comma;
4. deserializes per-source layers into the strict structs and prints the
   messages for an unknown key, `max_errors = 0`, and `format = "xml"`;
5. merges defaults, files, environment, and command line through
   `MergeComposer` and `merge_from_layers`, including `--quiet=false` over a
   file `true` and `CliValueExtractor::extract_user_provided` (decision D2a);
6. builds a `FluentLocalizer` for `en` with a consumer bundle with isolation
   disabled, parses with `try_parse_localized_from`, renders `--help` with
   explicit help and version arguments, renders a localized `clap` error, and
   checks for ANSI escapes and U+2068 or U+2069 characters;
7. compiles one `#[rstest]` plus `#[gtest]` test, one `rstest-bdd` scenario,
   one `cap_std::fs_utf8` write, and one `Result`-returning step.

Run `make lint` (with `[[bin]] doc = false`) and record every Clippy and
Whitaker finding. Go/no-go: checks 1, 4 to 7 behave as expected and no lint
fires inside generated code that cannot be met with a recorded item-scoped
`#[expect]`. Checks 2 and 3 choose between alternatives and cannot cause a
no-go unless neither alternative works. Record the transcript in
`Artefacts and notes` and findings in `Surprises & discoveries`. If decision D2
or D15 cannot hold, set the status to `BLOCKED`. Delete the probe before
committing.

### `EP-M1` run-outcome contract and library/binary split

Assigned: `REQ-exit-codes`, `GAP-no-lib`, decisions D1 and D13.

Red: create `src/lib.rs` with crate documentation and `pub mod outcome;`, and
`src/outcome.rs` with `exit_codes_match_design` and
`codes_are_distinct_and_zero_only_for_success` first.
`cargo test --lib outcome::` fails to compile.

Green: define `RunOutcome`, `RunOutcome::ALL`, `const fn exit_code`, and
`impl From<RunOutcome> for std::process::ExitCode`. Add the `[[bin]]` table and
`TARGET ?= mdast-check`. `src/main.rs` keeps its stub in this milestone.

Refactor: add a doctest on `RunOutcome::exit_code`; run the negative control.

### `EP-M2` `validate` command surface without configuration

Assigned: `REQ-flags` (seven flags), `REQ-explicit-verb`, `REQ-stdin`,
`REQ-early-stop`, `GAP-stub`, decisions D4, D5, D8, D9, D12, D14.

Red: the first block of `tests/features/validate_command.feature` and
`tests/validate_command_bdd.rs`; `tests/validate_cli.rs` with `assert_cmd`
tests for `--help`, `--version`, `validate --help`, no subcommand (`2`), an
unknown flag (`2`), a missing `--schema` (`2`), a non-UTF-8 argument on Unix
(`2`), and `placeholder_pipeline_exits_5_until_1_3_2`; unit tests for argument
parsing, `run`, and the argument-robustness property. They fail to compile or
fail on the stub's behaviour.

Green, in this order:

1. `src/app/settings.rs`: `DocumentSource`, `SchemaFormat`, `OutputFormat`,
   `ValidationInput`, `ReportOptions` (`serde` derives on the enums, no `clap`).
2. `src/app/pipeline.rs`: `ValidationPipeline`, `Verdict`, `PipelineFailure`
   with `outcome()`. `src/pipeline/mod.rs`: `PendingPipeline`.
3. `src/cli/args.rs`: `clap` types `Cli`, `Command`, `ValidateArgs` with every
   flag except `--config` (which arrives with its behaviour in `EP-M3`);
   explicit long names; no short flags; explicit `help` and `version` arguments;
   `--fail-fast` and `--quiet` as `--flag[=BOOL]` (`num_args = 0..=1`,
   `require_equals = true`, `default_missing_value = "true"`); value parsers for
   `DocumentSource`, schema paths (rejecting `-` and empty), the enums (same
   spellings as `serde`), and `NonZeroUsize`; a `debug_assert` unit test.
4. `src/cli/config/mod.rs` (first cut): `pub(crate) ValidateConfig` with the
   `OrthoConfig` derive; the command-line layer (D2a variant chosen in `EP-M0`);
   `resolve`, merging defaults and the command-line layer and building
   `ValidationInput` and `ReportOptions`, failing with
   `InvocationError::MissingSchema`.
5. `src/cli/error.rs`: `pub(crate) InvocationError` with `Usage`,
   `MissingSchema`, `InvalidSchemaPath`, and the configuration variants added in
   `EP-M3`.
6. `src/cli/run.rs`: `run` and `RunContext` (signatures in
   `Interfaces and dependencies`), parsing with `ortho_config`'s localized
   parse and a `NoOpLocalizer` for now, handling display requests and errors
   per D9, resolving, calling the pipeline once, and passing every outcome to
   one private reporting function.
7. `src/main.rs`: the composition root: `ProcessEnv`, locked standard streams,
   `std::env::args_os()`, `PendingPipeline`; delete the stub and its
   `#[expect]` in the same commit.

Refactor: keep `run` under the complexity thresholds; `insta` snapshots of the
invocation-error and placeholder messages (English, still inline text in this
milestone); seeded fault for `INV-stage-order`.

### `EP-M3` layered configuration: file and environment

Assigned: `REQ-flags` (`--config`), `REQ-config-precedence`, `GAP-no-config`,
decisions D2, D2a, D3, D6, D7.

Red: the configuration block of the feature file; `cli_layer_presence_is_exact`,
`env_layer_presence_is_exact`, `precedence_matches_oracle`, the chain-order
and environment rows, `invalid_configuration_is_rejected`,
`invalid_environment_is_rejected`,
`relative_schema_paths_resolve_against_ their_file`, `enum_spellings_agree`, and
`boolean_can_be_switched_off`; the ambient-isolation rows and
`ambient_configuration_is_ignored`. They fail because `--config` does not exist
(`clap` rejects it) and the environment is not read, so rows expecting file- or
environment-supplied values fail.

Green: add `--config <PATH>`. Add `src/cli/config/file.rs` (chain loading
through the API chosen in `EP-M0`, missing-file error, strict per-file structs,
rebasing, re-serialization, `push_file` with path) and `src/cli/config/env.rs`
(`CsvEnv` over the injected source, strict check naming variables,
`push_environment`). Push layers in the order of `L-layer-order`. Map
`OrthoError` values to
`InvocationError::Config { source: ConfigSource, reason }`, where
`ConfigSource` names a file or a variable and the original error is kept as
`#[source]`.

Refactor: keep files under 400 lines; run the negative controls for the layer
lemmas and precedence.

### `EP-M4` localized help, parse errors, and messages

Assigned: `REQ-l10n`, `GAP-no-l10n`, decisions D9, D15, D16.

Red: `locale_resolution_table`, `locale_resolution_is_total`,
`every_requested_identifier_resolves`, `catalogues_have_no_unused_identifiers`,
the localized-output tests, the localization block of the feature file, and
end-to-end rows for `LANG=C` and an unknown locale. They fail because no
catalogue or resolver exists.

Green:

1. `src/cli/l10n/locale.rs`:
   `resolve_locale(env: &dyn EnvSource) -> LanguageIdentifier` per D15, with
   the shipped-language set as a constant.
2. `locales/en/messages.ftl`: every identifier the localized parse requests
   for the root and `validate` commands (about, usage where used, each
   argument's help and value name, the explicit help and version arguments),
   plus `mdast-check-error-*` identifiers for every crate-authored message,
   with Fluent variables for paths, keys, and variable names. English text in
   en-GB Oxford spelling.
3. `src/cli/l10n/mod.rs`: `Catalogues` (the embedded catalogues keyed by
   language), `build_localizer(locale, &Catalogues) -> Box<dyn Localizer>`
   (consumer bundle with isolation disabled; `NoOpLocalizer` on failure), and
   `render(localizer, MessageId, args) -> String`, which falls back to a
   compiled-in English string only if lookup fails.
4. Add the `catalogues` field to `RunContext` (it does not exist before this
   milestone), updating `src/main.rs` and every test caller together. Switch
   `run` to resolve the locale from the injected environment, build the
   localizer, parse with `try_parse_localized_from`, localize `clap` errors with
   `localize_clap_error_with_command`, and render every crate-authored message
   through `render`. Move the inline English texts of `EP-M2` and `EP-M3` into
   the catalogue; the `insta` snapshots must not change (this proves the move
   preserved wording).

Refactor: review the diff to confirm no user-facing string literal remains
outside the catalogue and the compiled-in fallbacks; run the negative controls
for `INV-locale-resolution` and `INV-catalogue-complete`.

### `EP-M5` documentation, tested examples, and roadmap closure

Assigned: `REQ-docs`, `GAP-no-users-guide`.

Red: `tests/support/markdown_examples.rs`, test first, with inline-Markdown
unit tests for marked, unmarked, unterminated, unidentified, duplicate,
indented, and `~~~` fences. `cargo test --test users_guide_examples` fails to
compile.

Green and documentation:

1. Implement the loader over `include_str!`-embedded documents, returning
   `(identifier, language, body)`. Allowed languages: `sh` (one `mdast-check`
   command, whitespace-split; environment assignments written as leading
   `NAME=value` tokens are allowed and passed through `Command::env`; shell
   metacharacters rejected), `toml` (a configuration file), and `text`
   (expected standard-error substring for the preceding `sh` example). Any
   other fence language in a user-facing document fails. One function, used in
   full by both consumers.
2. `docs/users-guide.md` for release `0.1.0` (unreleased, early adopter),
   with imperative headings: install and build; check an invocation; choose the
   document and schema; configure defaults with a file (TOML only,
   `[cmds.validate]`, strict keys, relative paths, `extends`); set defaults
   with environment variables (names, value spellings, strictness); override
   for one run (precedence; `--quiet=false`); choose the language of help and
   messages (locale variables, fallback, available languages); interpret exit
   codes (`5` is not transient); output streams and accessibility (plain text,
   no colour, help on standard output, diagnostics on standard error, no
   meaning carried by colour); current limitations. Every fenced example
   preceded by `<!-- tested-example: <id> -->`.
3. `tests/users_guide_examples.rs`: runs every `sh` example of
   `docs/users-guide.md` and `README.md` against the binary with `assert_cmd`
   and `env_clear()` in a temporary directory seeded with the guide's `toml`
   examples (file names mapped from identifiers), asserting the exit code
   recorded per identifier; identifier tables match one to one. The BDD step
   file reuses the loader for the configuration example.
4. `docs/adrs/adr-0002-explicit-configuration-layer-composition.md` (D2, D2a,
   D3, D6, D7; reserved discovery name; revisit trigger) and
   `docs/adrs/adr-0003-locale-resolution-and-message-catalogues.md` (D15, D16;
   identifier scheme; isolation; fallback), following ADR 0001's format.
5. `docs/mdast-schema-validation-cli-design.md`: §Schema configuration
   (sources, precedence, strictness, environment names, deferred discovery, ADR
   0002); a new §Localization (locale resolution, catalogues, identifier
   scheme, ADR 0003); §Exit codes (placeholder `5` until 1.3.2); §Module
   boundaries (`cli`, `app`, `pipeline`, `outcome`; `validate` reserved);
   §Proposed flags (`--schema -`, `--flag[=BOOL]`).
6. `docs/developers-guide.md`: module layout and hexagonal boundary; banned
   ambient loaders and the injected environment; the provisional pipeline port
   and its extension by 1.1.2, 1.3.2, 2.1.2, 2.2.1; `Streams` and the
   obligation for 1.2.1 to inject standard input through it; how to add a
   configuration key (both structs, flag, presence witnesses, catalogue
   entries, users' guide); how to add or change a message (catalogue first,
   identifier conventions, completeness tests); how to add a language (a new
   `locales/<language>/messages.ftl`, the shipped-language constant, the
   `localization-docs` documentation set); the tested-example loader; and the
   Cranelift note that behaviour tests now exist.
7. `README.md`: a short marked usage example linking the users' guide; remove
   the stale `make typecheck` mention.
8. `docs/roadmap.md`: add to 1.3.2 the bullet "Replace `PendingPipeline`; a
   valid document exits `0`." Mark 1.1.1 and its sub-bullets `[x]` only after
   all gates pass, annotating the exit-code bullet: "contract and mapping;
   codes 1, 3, and 4 become reachable with 1.2.1 to 1.3.2".

## Milestones and plateaus

- `EP-M0`: a recorded go or no-go for D2 and D15 and choices for D2a, the
  file API, and the environment-layer construction, with no committed code.
  Recovery: delete the probe; restore `Cargo.toml` if no-go. Compatibility
  decision: none.
- `EP-M1`: a library exposing `RunOutcome` and a binary named
  `mdast-check` (still the stub). Requirements: `REQ-exit-codes` (contract),
  `GAP-no-lib`. Evidence: `outcome::tests` pass; `make build` produces
  `target/debug/mdast-check`. Conformance: exit table matches the design;
  module naming per D13; public interface limited to `RunOutcome`; no persisted
  format. Recovery: revert the commit. Compatibility decision: none.
- `EP-M2`: the binary parses the `validate` flags except `--config`, rejects
  invalid invocations with `2`, prints help and version with `0`, and reports
  the placeholder with `5`; the in-process runner maps every pipeline outcome.
  Requirements: `REQ-flags` (seven), `REQ-explicit-verb`, `REQ-stdin`,
  `REQ-early-stop`, `GAP-stub`. Evidence: first-block BDD, end-to-end, unit,
  property, and snapshot tests. Conformance: flags match Table 1 except
  `--config`; `clap` confined to `src/cli/`; no environment read yet beyond the
  injected object's existence. Remaining gaps: no configuration sources;
  English text inline. Recovery: revert to `EP-M1`. Compatibility decision:
  none.
- `EP-M3`: `--config` files and `MDAST_CHECK_CMDS_VALIDATE_*` variables
  supply defaults with the command line taking precedence, all checked
  strictly. Requirements: `REQ-flags` (`--config`), `REQ-config-precedence`,
  `GAP-no-config`. Evidence: configuration BDD scenarios, presence
  enumerations, the precedence property, rejection rows, rebasing rows,
  ambient-isolation tests. Conformance: precedence matches the design;
  `ortho_config` confined to `src/cli/`; the file shape and variable names are
  new user-facing formats approved by this plan. Remaining gaps: English text
  inline. Recovery: revert to `EP-M2`. Compatibility decision: none (first
  release of both formats).
- `EP-M4`: every user-facing text comes from a catalogue selected by the
  injected locale; English is complete; a test catalogue proves a second
  language end to end in-process. Requirements: `REQ-l10n`, `GAP-no-l10n`.
  Evidence: locale tests, coverage tests, localized-output tests, unchanged
  English snapshots. Conformance: Fluent confined to `src/cli/l10n/`; message
  identifiers are a new internal contract documented in ADR 0003. Recovery:
  revert to `EP-M3`. Compatibility decision: none.
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
       Green: cargo test --lib cli::config           -> pass; coverage of 48 (key, class) pairs asserted
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

$ MDAST_CHECK_CMDS_VALIDATE_FAILFAST=1 target/debug/mdast-check validate --document README.md --schema s.json; echo "exit=$?"
error: unknown environment variable MDAST_CHECK_CMDS_VALIDATE_FAILFAST; did you mean MDAST_CHECK_CMDS_VALIDATE_FAIL_FAST?
exit=2

$ target/debug/mdast-check validate --document README.md --schema s.json --config missing.toml; echo "exit=$?"
error: cannot read configuration file missing.toml: file not found
exit=2

$ target/debug/mdast-check validate --document README.md --schema s.json; echo "exit=$?"
error: document validation is not implemented yet
exit=5
```

Wording is fixed by the English catalogue and its `insta` snapshots; the lines
above are intended wording and may be refined during implementation, in which
case this section is updated. (The "did you mean" suggestion is included only if
`serde`'s unknown-field message makes it cheap; otherwise the message lists
the accepted names.)

Quality criteria:

- Tests: `make test` passes, including the new unit, property, BDD, snapshot,
  localization, end-to-end, and tested-example tests, plus the existing
  build-standard contract.
- Verification: every obligation has its artefact passing and its negative
  controls recorded as rejected.
- Lint and format: `make check-fmt` and `make lint` (Clippy and Whitaker)
  pass with no `#[expect]` beyond any recorded in `Decision log`.
- Documentation: `make markdownlint` and `make nixie` pass.
- Performance: the full `make test` run stays within the existing CI coverage
  job's time budget; the precedence property runs 1,024 cases.
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
$ ortho_config-0.11.0/src/subcommand/sources.rs: SubcommandFileContext { base: &Path, discovery: &dyn EnvSource }
  (no explicit file path input)
$ ortho_config-0.11.0/src/csv_env/mod.rs: pub fn with_source(mut self, source: crate::SharedScanEnvSource) -> Self
$ ortho_l10n-0.11.0/src/fluent.rs: pub fn with_consumer_bundle(mut self, bundle: FluentBundle<Arc<FluentResource>>) -> Self
$ ortho_l10n-0.11.0/locales: en-US/messages.ftl, ja/messages.ftl (11 identifiers each)
```

Milestone evidence is appended here during implementation.

## Interfaces and dependencies

Runtime dependencies (caret requirements):

- `ortho_config = "0.11.0"` (default features `serde_json` and `toml`):
  declarative merge, file loading, injected environment, command-line value
  extraction, localized parsing, and the Fluent localizer. Its re-exports of
  `serde_json`, `toml`, `langid!`, and `LanguageIdentifier` are used, so no
  direct `serde_json`, `toml`, or `unic-langid` dependency is added.
- `clap = { version = "4.6.9", features = ["derive", "string"] }`.
- `serde = { version = "1.0.229", features = ["derive"] }`.
- `camino = { version = "1.2.6", features = ["serde1"] }`.
- `thiserror = "2.0.21"`.
- Conditional: `fluent-bundle = "0.16.0"`, only if `EP-M0` shows that
  `ortho_config` does not re-export `FluentBundle` and `FluentResource`, which
  are needed to disable Unicode isolation; the version must match
  `ortho_l10n`'s requirement.

Development dependencies (in addition to `rstest = "0.28.0"`):
`rstest-bdd = "0.6.0"`, `rstest-bdd-macros = "0.6.0"`, `googletest = "0.14.4"`
(with `#[gtest]` beneath `#[rstest]`), `pretty_assertions = "1.4.1"`,
`insta = "1.49.0"`, `assert_cmd = "2.2.2"`, `proptest = "1.12.0"`,
`tempfile = "3.27.0"`, and
`cap-std = { version = "4.0.3", features = ["fs_utf8"] }`.

`mockable` is not added: `ortho_config::EnvSource`, `ScanEnvSource`,
`ProcessEnv`, and `MapEnv` already provide the injected environment that the
configuration and localization code need, and the abstraction policy in
`AGENTS.md` requires reusing an equivalent from an approved dependency.
`ProcessEnv` is constructed only in `src/main.rs`; tests use `MapEnv`.

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
src/cli/run.rs (+ tests)    run, Streams, private reporting function
src/cli/error.rs            pub(crate) InvocationError, ConfigSource
src/cli/config/mod.rs       pub(crate) ValidateConfig (OrthoConfig), resolve
src/cli/config/file.rs      file chain, strict structs, rebasing
src/cli/config/env.rs       environment layer, strict check
src/cli/config/tests.rs     presence enumerations, precedence property, rows
src/cli/l10n/mod.rs         Catalogues, build_localizer, render, MessageId
src/cli/l10n/locale.rs      resolve_locale
src/cli/l10n/tests.rs       locale, coverage, and localized-output tests
locales/en/messages.ftl     English catalogue (source of truth)
tests/validate_cli.rs, tests/validate_command_bdd.rs, tests/features/validate_command.feature
tests/support/markdown_examples.rs, tests/users_guide_examples.rs
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

// src/cli/run.rs
pub struct Streams<'a> {
    pub stdout: &'a mut dyn std::io::Write,
    pub stderr: &'a mut dyn std::io::Write,
}

/// The injected environment: named lookups (locale variables) and prefix
/// scans (configuration variables). Implemented for every type that
/// implements both `ortho_config` traits.
pub trait RunEnvironment: ortho_config::EnvSource + ortho_config::ScanEnvSource {}
impl<T: ortho_config::EnvSource + ortho_config::ScanEnvSource> RunEnvironment for T {}

/// Everything a run needs besides its arguments. `main` builds it with
/// `ProcessEnv`, locked standard streams, `PendingPipeline`, and
/// `Catalogues::shipped()`; tests substitute `MapEnv`, in-memory writers, a
/// recording pipeline, and catalogues built from in-memory Fluent text.
pub struct RunContext<'a> {
    pub env: std::sync::Arc<dyn RunEnvironment>,
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
and 2.2.2; tests of `resolve` assert its values.

Test catalogue injection: `Catalogues` (in `cli::l10n`) is public, with
`Catalogues::shipped()` returning the embedded catalogues and
`Catalogues::from_sources(&[(LanguageIdentifier, &str)])` building catalogues
from Fluent text, so in-process tests and BDD scenarios can install a test
language through the same `RunContext` field that production uses. This is an
approved public interface; no `#[cfg(test)]` seam is needed.

Reuse policy:

- `RunOutcome` is the only place exit codes are defined; 1.1.2 maps its
  taxonomy onto it through `outcome()` methods.
- `ValidationPipeline` is the only boundary to the validation stages; it is
  provisional, and later items implement it outside `src/app/`.
- `Streams` is the only route to standard output and error; 1.2.1 adds
  standard input to it.
- `RunEnvironment` is the only route to environment variables; nothing calls
  `std::env` outside `src/main.rs`.
- `cli::config` owns all configuration loading; new keys are added there, in
  the strict structs, and in the catalogue.
- `cli::l10n` and `locales/` own all user-facing text; no user-facing string
  literal outside them except compiled-in English fallbacks.
- `tests/support/markdown_examples.rs` is the shared loader for tested
  examples in all user-facing documents.

## Revision note

- 2026-11-20: Initial draft for `ortho_config` 0.8.0.
- 2026-11-20: First design review applied (typed layers, banned ambient
  loaders, split provisional port, renames, stronger verification).
- 2026-11-20: Revised at the user's direction to use `ortho_config` 0.11.0
  for layered configuration with localized help. Prerequisites completed:
  vendored v0.11.0 guide; design and roadmap retargeted with a localization
  sub-task. Changed: D2 keeps explicit layer composition (0.11.0 subcommand
  loaders still lack explicit-file support) but prefers `ConfigFilePolicy`; D3
  now includes an environment layer through 0.11.0's injected sources; D7
  extends strict typing to the environment; D8 adopts `--flag[=BOOL]`; D9 and
  D14 render all text from catalogues; new D15 (locale resolution and
  catalogues) and D16 (English-only shipped catalogue, approver decision).
  Added milestone `EP-M4` for localization (documentation moves to `EP-M5`),
  locale, coverage, and localized-output obligations, environment-layer
  obligations, and ADR 0003. Precedence verification moves from exhaustive
  enumeration (infeasible with four layers in the default suite) to exhaustive
  per-layer presence enumeration plus a coverage-accounted property across
  layers. `mockable` is replaced by `ortho_config`'s injected environment.
  Budgets raised for approved scope growth: code 2,900, documentation 950,
  files 50. Awaiting second design review.
