# 1.1.1 Implement the root CLI command with a `validate` subcommand

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & discoveries`, `Decision log`,
`Outcomes & retrospective`, `Conformance basis`, and `Verification plan` must
be kept up to date as work proceeds.

Status: DRAFT (design review applied; awaiting user approval)

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
  `--quiet`) is parsed, type-checked, and listed in `--help`;
- a TOML configuration file named by `--config` supplies defaults for the
  configurable settings, loaded through `ortho-config` v0.8.0, and any flag on
  the command line overrides the file;
- mistakes in the configuration file (unknown keys, wrong types, keys in the
  wrong table) are reported against the file that contains them;
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
- *Layer*: one source of configuration values (built-in defaults, each
  configuration file, the command line). Layers are merged in precedence order;
  a later layer wins for every key it defines.

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

This plan closes the design's open question of how configuration is layered for
a subcommand (decision D2 and the proposed ADR 0002).

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
  `allow_attributes`, `missing_docs`, and `result_large_err`. `make lint` also
  runs the Whitaker Dylint suite, whose installed lints include
  `no_std_fs_operations`, `no_expect_outside_tests`,
  `test_must_not_have_example`, `module_max_lines`,
  `module_must_have_inner_docs`, `rstest_helper_should_be_fixture`,
  `bumpy_road_function`, `conditional_max_n_branches`, and
  `function_attrs_follow_docs`. Output goes through injected `std::io::Write`
  sinks, never `print!`. Tests that touch files use `cap_std::fs_utf8`, never
  `std::fs`. Suppression is a last resort and uses item-scoped
  `#[expect(lint, reason = "...")]` only.
- Remove the `FIXME(#5)` stub and its `#[expect(clippy::print_stdout, ...)]`
  in `src/main.rs` in the same commit that replaces the stub; leaving the
  expectation without the `println!` makes it unfulfilled and fails
  `-D warnings`.
- Production code must not read ambient process state except at the
  composition root (`src/main.rs`), which passes `std::env::args_os()` into the
  library. In particular, no code path may call the `ortho-config` generated
  loaders `load`, `load_from_iter`, `compose_layers`, or
  `compose_layers_from_iter`, nor the subcommand helpers, because they read the
  process arguments, environment, and home directory. Tests must not mutate the
  process environment or working directory (`clippy.toml`
  `disallowed-methods`); end-to-end tests may use `assert_cmd` with
  `Command::env_clear`, `Command::env`, and `Command::current_dir`, which
  affect only the child process.
- Keep the application core free of `clap`, `ortho_config`, filesystem, and
  terminal types. `clap` and `ortho_config` appear only in `src/cli/`. Core
  value types may derive `serde` traits, because a data-shape derive carries no
  I/O; this keeps one spelling for each enum value across the command line and
  the configuration file.
- Use `camino::Utf8PathBuf` for paths, per `AGENTS.md`. A non-UTF-8 argument
  on the command line is an invocation error (exit `2`).
- Use `ortho-config` with a caret requirement on `0.8.0`, as the roadmap and
  design mandate, even though `0.11.0` exists. Upgrading is out of scope.
- Do not change `tests/build_standard_contract.rs`,
  `tests/build_standard_support/`, `.cargo/config.toml`, `rust-toolchain.toml`,
  `.gitignore`, or the CI workflows. Changing the Makefile is limited to
  `TARGET ?= mdast-check` (decision D1).
- Do not implement Markdown parsing, schema loading, validation, output
  rendering, quiet suppression, fail-fast behaviour, or error capping. Those
  are roadmap items 1.2.1 to 2.2.2.
- Do not introduce the full `thiserror` error taxonomy of roadmap item 1.1.2.
  This plan introduces only the invocation error type it needs and a
  provisional pipeline-failure enum whose variants the exit-code contract
  requires; 1.1.2 replaces it.
- Dependency changes are limited to those listed in
  `Interfaces and dependencies`. Any other dependency requires escalation.
- No source-API compatibility machinery. The crate is pre-1.0 and has no
  release tag, so interfaces change together with their callers.
- Do not edit `typos.toml` by hand; use `typos.local.toml` for exceptions.
  If `make spelling` regenerates `typos.toml` for reasons unrelated to this
  work, commit it on a separate base branch and stack this branch on it.

If satisfying the objective requires violating a constraint, do not proceed.
Document the conflict in `Decision log` and escalate.

## Tolerances (exception triggers)

The comparison base is the merge base with `main` after the base branch
`refresh-typos-config` (pull request #143) lands; until then it is
`d6c36547382993de91ce9166147a693506bb945f`, the head of that base branch.
Counts exclude this ExecPlan file and any other file under `docs/execplans/`.
Counts also exclude generated `insta` snapshot files (`**/snapshots/*.snap`)
and committed `proptest-regressions/` files, which are reviewed as output
rather than authored; their number still counts towards the file limit. All
line limits measure net lines (additions minus deletions):

```sh
git diff --numstat d6c3654 -- . ':(exclude)docs/execplans' \
  ':(exclude)**/*.snap' ':(exclude)**/proptest-regressions/**'
```

Classification is by whole file: `src/**`, `tests/**/*.rs`, and
`tests/features/*.feature` are code; `Cargo.toml`, `Makefile`, and
`typos.local.toml` are configuration; `docs/**` (except `docs/execplans/`) and
`README.md` are documentation.

- Files: if implementation changes more than 40 non-ExecPlan files (snapshot
  and regression files included), stop and escalate.
- Code: if code changes exceed 2,200 net lines, stop and escalate.
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
  affects the user-facing contract (flag names, configuration keys, exit codes,
  message wording fixed by snapshots), stop and present options with trade-offs.

Each category is enforced independently. Spare capacity in one category never
offsets another.

Budget rationale. Production source is estimated at 850 lines: library root 40,
run outcome 80, application core 200, CLI adapter 480 (arguments 110, run and
reporting 150, configuration merge 110, file layers 110), placeholder pipeline
25, main 25. Tests are estimated at 1,250 lines: unit tests within modules 420
(outcome table, argument parsing, exhaustive precedence enumeration, file-layer
rows, run ordering and write failures), argument robustness property 110,
end-to-end 200, BDD steps and feature 300, tested example loader and runner
220. The total of 2,100 plus about 5% gives 2,200. The thin contingency is
deliberate: a breach signals scope growth. The design review cut a bespoke key
list, a value-level precedence property test, and a path-rebasing property
whose oracle repeated the implementation, and added end-to-end and robustness
obligations; the net estimate fell by about 200 lines. Configuration is about
40 lines (`[[bin]]` table, 5 runtime and 9 development dependencies, one
Makefile line, a few spelling exceptions); 70 absorbs feature tables.
Documentation is about 580 lines (users' guide 250, ADR 0002 120, design
document 80, developers' guide 90, README 25, roadmap 10); 750 leaves a quarter
in reserve because the users' guide style rules are demanding. No comparable
completed ExecPlan exists on `main` (`docs/execplans/` is absent), so there is
no history of budget revisions to learn from. A stale earlier attempt (see
`Context and orientation`) used about 600 source and 470 test lines without
configuration layering, tested examples, or property tests, which is consistent
with these estimates.

## Risks

- Risk: `#[derive(OrthoConfig)]` emits code that trips this repository's
  strict lints. The 0.8.0 derive generates a hidden `clap::Parser` struct, an
  inherent `compose_layers_from_iter` marked `#[expect(dead_code)]` (which may
  be unfulfilled), and public `compose_layers` and `load` functions. Severity:
  high. Likelihood: medium. Mitigation: `EP-M0` compiles the derive under
  `make lint` (Clippy and Whitaker) before any design is committed to. Mark
  every field `#[ortho_config(skip_cli)]` so the hidden parser has no fields
  and imposes no `clap` bounds; keep the derived type `pub(crate)` in one
  module. If lints still fire inside generated code, use one item-scoped
  `#[expect(..., reason = ...)]` per lint, recorded in `Decision log`; if that
  is impossible, escalate under the Architecture tolerance.
- Risk: the generated `merge_from_layers` deserializes only the merged value,
  so a type error in one file cannot be attributed to that file. Severity:
  high. Likelihood: certain (confirmed in
  `ortho_config_macros-0.8.0/src/derive/generate/declarative/merge_tokens.rs`).
  Mitigation: decision D7 types and checks each file layer separately before it
  is pushed.
- Risk: Whitaker's `no_std_fs_operations` rejects `std::fs` in tests and the
  tested-example loader. Severity: medium. Likelihood: high. Mitigation:
  `cap-std` (`fs_utf8`) is pre-approved as a development dependency; the loader
  embeds documents with `include_str!`; `EP-M0` probes the lint.
- Risk: `no_expect_outside_tests` fires inside `rstest-bdd` step functions
  and shared helpers, which are not `#[test]` items (observed in the prior
  attempt). Severity: low. Likelihood: high. Mitigation: steps and helpers
  return `Result` and use `?`; fixtures are `rstest` fixtures
  (`rstest_helper_should_be_fixture`).
- Risk: `rstest` drops `#[expect]` attributes on generated tests
  (`docs/rust-testing-with-rstest-fixtures.md` lines 555 to 557), while
  `AGENTS.md` forbids `#[allow]`. Severity: low. Likelihood: medium.
  Mitigation: design tests so no suppression is needed.
- Risk: the vendored `docs/rstest-bdd-users-guide.md` documents `0.5.0`, but
  `0.6.0` is current. Severity: low. Likelihood: high. Mitigation: use `0.6.0`,
  whose migration guide was read during planning; its breaking changes (one
  leading underscore normalized in implicit fixture names; MSRV 1.88; `Result`
  aliases now propagate `Err`) do not affect the planned usage.
- Risk: `Cargo.lock` is ignored by `.gitignore`, so CI resolves the newest
  compatible versions and a `clap` release could change help layout. Severity:
  medium. Likelihood: medium. Mitigation: do not snapshot whole help screens;
  assert that help lists every flag, and snapshot only text this crate authors.
  Committing `Cargo.lock` is decision D11, left to the approver.
- Risk: `load_config_file_as_chain` canonicalizes paths (resolving symbolic
  links) and converts non-UTF-8 paths lossily. Severity: low. Likelihood:
  medium. Mitigation: decision D6 defines rebasing against the canonical
  directory, documented and tested with a symbolic link; non-UTF-8 directories
  in a configuration path are a documented limitation.
- Risk: the library crate `mdast_check` and binary `mdast-check` produce an
  output-filename collision warning under `cargo doc`. Severity: low.
  Likelihood: medium. Mitigation: `doc = false` on the `[[bin]]` target;
  verified in `EP-M0`.
- Risk: `clap` rendering emits ANSI escapes into injected writers. Severity:
  low. Likelihood: low. Mitigation: render with `to_string()` and set
  `ColorChoice::Never`; verified in `EP-M0`.

## Progress

- [x] (2026-11-20 12:20Z) Reconnaissance completed: repository survey, ortho
  config 0.8.0 source review, prior-attempt review, dependency survey.
- [x] (2026-11-20 13:30Z) Initial draft written and committed.
- [x] (2026-11-20 14:10Z) Expert-panel design review (structure and
  contracts; verification and testing; failure modes, alternatives, and scope)
  applied. Verdict: proceed with conditions; conditions adopted below.
- [ ] Plan approved by the user.
- [ ] `EP-M0` feasibility probe (prototype; deleted before `EP-M1`).
- [ ] `EP-M1` run-outcome contract and library/binary split.
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
  `src/subcommand/paths.rs`, which call `std::env::var_os("HOME")`. Impact:
  drives decision D2.
- Observation: `#[derive(OrthoConfig)]` generates a hidden `clap::Parser`
  (`__<Name>Cli`) whose fields are `Option<T>` with derived long and short
  flags, plus a `--config-path` flag, and public loaders that read the process
  arguments and environment. Evidence:
  `ortho_config_macros-0.8.0/src/derive/generate/structs.rs`,
  `src/derive/build/cli/cli_flags.rs`, `src/derive/load_impl.rs`. Impact:
  `skip_cli` on every field; derived type kept `pub(crate)`; loaders banned (see
  `Constraints`).
- Observation: `merge_from_layers` deserializes only the merged JSON value
  (`declarative::from_value_merge`), and its `Merge` error carries no file
  path. Evidence: `merge_tokens.rs` and `error/types.rs`. Impact: decision D7.
- Observation: `merge_value` replaces a scalar with whatever the later layer
  holds, including JSON `null`. Evidence:
  `ortho_config-0.8.0/src/declarative/merge.rs`. Impact: the command-line layer
  must omit absent values rather than send `null`; a negative control checks
  this.
- Observation: `FileLayerChain` holds `(value, Utf8PathBuf)` pairs ordered
  from the root ancestor to the named file, with the `extends` key stripped and
  paths canonicalized. Evidence: `ortho_config-0.8.0/src/file/loader.rs`.
  Impact: decision D6; chain-order tests.
- Observation: `ortho_config::load_config_file` and
  `load_config_file_as_chain` return `Ok(None)` for a missing file, and pick
  the parser by extension: with default features `.json`, `.json5`, `.yaml`, and
  `.yml` are errors and any other extension (or none) is parsed as TOML.
  Evidence: `src/file/loader.rs`, `src/file/parser.rs`. Impact: a missing
  explicit `--config` is turned into an invocation error by this crate; the
  users' guide states that configuration is TOML only.
- Observation: `ortho_config::is_display_request` matches only
  `DisplayHelp` and `DisplayVersion`, not
  `DisplayHelpOnMissingArgumentOrSubcommand`. Evidence:
  `ortho_config-0.8.0/src/error/helpers.rs`. Impact: a bare `mdast-check` with
  no subcommand exits `2` with help on standard error.
- Observation: the earlier attempt on branch
  `1-1-1-implement-the-root-cli-command-with-a-validate-subcommand` (based on
  `e725436`, far behind `main`) declared `ortho_config` but never called it,
  copied boolean flags unconditionally over lower layers, and returned exit `0`
  from a stub. Evidence: review of an exported copy of that branch. Impact:
  this plan forbids a bespoke overlay, tests that an absent boolean flag leaves
  the file value intact, and refuses to report success without validation.

## Decision log

- D1. Decision: build a library crate `mdast_check` (`src/lib.rs`) and a
  binary named `mdast-check`
  (`[[bin]] name = "mdast-check", path = "src/main.rs", doc = false`), set
  `TARGET ?= mdast-check` in the Makefile, and fix
  `#[command(name = "mdast-check", bin_name = "mdast-check")]` so usage text
  does not depend on `argv[0]`. Rationale: the design's command is
  `mdast-check`; Cargo would otherwise name the binary `mdast_check`. A library
  target makes the run boundary testable in-process and lets doctests compile
  (`docs/rust-doctest-dry-guide.md`). `doc = false` avoids the lib/bin
  documentation collision. The build-standard contract does not read
  `Cargo.toml` or the binary name. Date/Author: 2026-11-20, planning agent.
- D2. Decision: compose configuration layers explicitly with `ortho-config`'s
  declarative API rather than its subcommand helpers. The CLI adapter parses
  arguments with `clap`, then builds a `MergeComposer` with, in order, a
  defaults layer, one file layer per element of the `--config` file's `extends`
  chain (from `ortho_config::load_config_file_as_chain`, checked and focused
  per decision D7), and a command-line layer containing only the values the
  user supplied. `ValidateConfig::merge_from_layers` (generated by
  `#[derive(OrthoConfig)]`) performs the merge. Rationale: the helpers ignore
  `--config` and read ambient state; the declarative API is part of
  `ortho-config` 0.8.0, keeps merge semantics in the library, and is pure apart
  from the explicit file read. Rejected alternative: top-level
  `#[derive(OrthoConfig)]` with `load_from_iter` and `discovery(...)`, which
  honours `--config` but flattens the subcommand structure and reads the
  process environment and home directory. Recorded as ADR 0002, with a revisit
  trigger: re-evaluate when upgrading beyond `ortho-config` 0.8. Date/Author:
  2026-11-20, planning agent; amended after design review.
- D2a. Decision: build the command-line layer with `ortho-config`'s own
  `CliValueExtractor::extract_user_provided` if `EP-M0` shows it works with a
  `clap` derive struct whose presence flags use `ArgAction::SetTrue` and carry
  `#[ortho_config(cli_default_as_absent)]`; otherwise use a small handwritten
  builder over `Option` fields that omits absent values. Both variants are
  covered by the same exhaustive precedence test and negative controls.
  Rationale: the design reviewers' strongest alternative; using the library's
  extractor moves "an absent flag leaves the file value alone" from this
  crate's code into the library, removing the exact bug class of the prior
  attempt. The probe decides because the 0.8.0 documentation is silent on
  `SetTrue` flags. Date/Author: 2026-11-20, design review.
- D3. Decision: in this item a configuration file is read only when named
  explicitly with `--config`. Implicit discovery (current directory, home, XDG)
  and environment-variable layers are deferred. When discovery is added later,
  an explicit `--config` disables it, and the discovery file name is reserved
  now as `.mdast-check.toml` in ADR 0002, so a repository that already passes
  `--config` sees no change. Rationale: the roadmap requires only command-line
  precedence over configuration-file values; the design lists environment
  variables as "if exposed". Both deferred layers read ambient process state
  and would need an injected environment (`mockable::Env`) and a discovery
  port. Note that adding discovery is not purely additive for users who rely on
  running without `--config`; ADR 0002 records this. Date/Author: 2026-11-20,
  planning agent; amended after design review.
- D4. Decision: until the validation pipeline exists, the shipped binary's
  driven adapter (`pipeline::PendingPipeline`) fails every well-formed
  `validate` run with `error: document validation is not implemented yet` and
  exit `5`. Rationale: exit `0` would tell CI a document is valid when nothing
  was checked; `2` would blame the user for a correct invocation; `5` is the
  only honest non-zero code, although the design defines it as an *unexpected*
  internal failure and this case is expected. The users' guide states that `5`
  is not transient and must not be retried. A tripwire test,
  `placeholder_pipeline_exits_5_until_1_3_2`, and a new acceptance bullet on
  roadmap item 1.3.2 ("replace `PendingPipeline`; a valid document exits `0`")
  carry the removal obligation beyond this plan. Alternative for the approver:
  exit `0` with a warning, which is unsafe in CI. Date/Author: 2026-11-20,
  planning agent; confirmed by design review.
- D5. Decision: `--document` is command-line only and required by `clap`;
  every other setting (`schema`, `schema_format`, `format`, `fail_fast`,
  `max_errors`, `quiet`) may come from the configuration file. `--schema` is
  required after merging: if neither source supplies it, the run exits `2`.
  `--config` selects the file layers and is never itself a configuration key.
  Rationale: the design's configuration list names a default schema path but
  never a default document. Date/Author: 2026-11-20, planning agent.
- D6. Decision: a relative `schema` path read from a configuration file is
  resolved against the canonical directory of the file that set it (each file
  in an `extends` chain uses its own directory; a symbolic link to a
  configuration file therefore resolves against the target's directory). A
  relative `--schema` on the command line stays relative to the current
  directory. An empty `schema` value from either source is an invocation error.
  Rationale: resolving against the file makes a checked-in configuration work
  from any working directory; canonical directories are what `ortho-config`
  provides, and documenting them is simpler than reconstructing typed paths.
  Date/Author: 2026-11-20, planning agent; amended after design review.
- D7. Decision: the whole configuration file is typed strictly. Each element
  of the `extends` chain is deserialized on its own into
  `#[serde(deny_unknown_fields)]` structs: the root may contain only `cmds`
  (the loader has already removed `extends`); `cmds` may contain only
  `validate`; `validate` may contain only the six configurable keys, each
  `Option`. Any failure is an invocation error naming the file (as reported by
  the chain) and, via `serde`'s message, the offending key and the expected
  keys. Only after this check is the layer re-serialized, omitting absent keys,
  and pushed. Rationale: `ortho-config` silently ignores unknown keys and
  reports type errors only after merging, without a path; rejecting unknown
  keys later would be a breaking change; one typed struct replaces a manually
  maintained key list. Date/Author: 2026-11-20, design review.
- D8. Decision: `--max-errors` takes a positive integer modelled as
  `NonZeroUsize`; `0` is an invocation error from either source. A file can
  express values up to `i64::MAX` (the TOML integer limit). `--fail-fast` and
  `--quiet` are presence flags; the command line can turn them on but not off.
  Repeating a presence flag is an invocation error (`clap` 4 default).
  `--schema-format` is optional; inference from the file extension is deferred
  to roadmap item 1.3.1, which owns schema loading. Rationale: a zero cap has
  no meaning; negation flags (`--no-quiet`) can be added compatibly later.
  Date/Author: 2026-11-20, planning agent.
- D9. Decision: help and version requests write plain text to standard output
  and exit `0`; every other `clap` error, including a missing subcommand,
  writes plain text to standard error and exits `2`. Invocation errors always
  render as human-readable text on standard error, even with `--format json`,
  because the JSON envelope belongs to roadmap item 2.2.1. A failed write to
  standard output while printing help or version yields exit `5`; a failed
  write to standard error leaves the already-decided status unchanged, because
  no channel remains to report it. Rationale: keeps the exit-code mapping in
  one place instead of calling `clap::Error::exit`, which would bypass injected
  writers. Date/Author: 2026-11-20, planning agent.
- D10. Decision: no Verus or Kani artefacts in this item (see
  `Formal verification decision`). Date/Author: 2026-11-20, planning agent;
  confirmed by design review.
- D11. Decision for the approver: whether to commit `Cargo.lock` (a separate
  pull request, since `.gitignore` is out of scope here). This plan works
  either way, because help output is asserted by flag presence rather than
  snapshotted. Recommendation: commit it, as Cargo recommends for binaries.
  Date/Author: 2026-11-20, design review.
- D12. Decision: `--schema -` is an invocation error ("the schema cannot be
  read from standard input"), reserving `-` so that standard input is never
  ambiguous between document and schema. Rationale: a file literally named `-`
  remains reachable as `./-`. Date/Author: 2026-11-20, design review.
- D13. Decision: name the application core `src/app/` and the placeholder
  adapter's module `src/pipeline/`, not `src/validate/`. Rationale: the
  design's Table 2 reserves `validate` for the module that wraps
  `json-schema-validator-core` (roadmap 1.3.2); using it for the core would
  collide. Name the outcome type `RunOutcome` rather than `ExitStatus`, to
  avoid confusion with `std::process::ExitStatus` in `assert_cmd` tests.
  Date/Author: 2026-11-20, design review.
- D14. Decision: the pipeline port is provisional. Its input is split into
  `ValidationInput` (what to validate) and `ReportOptions` (how to report), so
  later validation stages never see presentation settings. Failures are a small
  enum, `PipelineFailure`, with one variant per pipeline category, each
  carrying a crate-authored message, and a
  `const fn outcome(&self) -> RunOutcome`. `Verdict::Invalid` carries no
  failure list yet. Roadmap 1.1.2 replaces `PipelineFailure` with its
  `thiserror` taxonomy, keeping the `outcome` method shape; 1.3.2 adds
  normalized failures to `Invalid`; 2.1.2 and 2.2.1 add renderers behind the
  CLI's single reporting function. Rationale: avoids speculative payloads while
  making the extension points explicit. Date/Author: 2026-11-20, design review.

## Outcomes & retrospective

Not yet started. To be completed at each milestone and at the end.

Design review outcome (2026-11-20): three reviewers (structure and contracts;
verification and testing; failure modes, alternatives, and scope) returned
"proceed with conditions". Adopted: typed per-file layers (D7), banned ambient
loaders and `skip_cli` fields, the provisional split port (D14), module
renaming (D13), Whitaker and `cap-std` handling, extends-order and
`null`-override tests, a no-ambient-configuration end-to-end test, an
argument-robustness property, exhaustive precedence enumeration in place of a
statistical property, removal obligations for the placeholder (D4), the
discovery reservation (D3), `--schema -` reservation (D12), and the
`Cargo.lock` question (D11). Declined: dropping `googletest`,
`pretty_assertions`, and `proptest` (the user's request names the first two,
and the argument-robustness property is a genuine many-input invariant).

## Context and orientation

Inspected base revision: `788b9d444c418b117b61e9eaa05a98f277228852` on `main`.
The working branch is `1-1-1-root-cli-command-and-validate-subcommand`, stacked
on `refresh-typos-config` (pull request #143), which regenerates `typos.toml`
from the shared dictionary and nothing else.

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
  `lint` (rustdoc, Clippy, Whitaker), `fmt`, `check-fmt`, `markdownlint` (which
  runs `spelling` first), `spelling`, `nixie`, `workflow-contracts`.
- `.github/workflows/ci.yml`: runs `make check-fmt`, `make spelling`,
  `make workflow-contracts`, markdownlint, `make lint`, and coverage (which
  runs the tests). Verus and Kani are not installed in CI.
- `.gitignore` ignores `Cargo.lock`.
- `rust-toolchain.toml`: `nightly-2026-11-20` with `rustfmt` and `clippy`.
- `docs/`: the design (`mdast-schema-validation-cli-design.md`), the roadmap,
  ADR 0001 (build standard), the developers' guide, the documentation style
  guide, and vendored guides for `ortho-config`, `rstest-bdd`, `rstest`
  fixtures, dependency injection, doctests, and complexity. There is no
  `docs/users-guide.md` and no `docs/execplans/` directory on `main`.

Tools installed on the host but not configured by the repository: `kani` and
`cargo-kani` in `~/.cargo/bin`, and `prover-tools` (which can install Verus) in
`~/.local/bin`. Neither is wired into the Makefile or CI. The Whitaker lint
libraries are installed under `~/.local/share/whitaker/`.

Upstream libraries, versions checked on crates.io on 2026-11-20: `ortho_config`
0.8.0 (current 0.11.0; 0.8.0 depends on `clap ^4` with `derive`,
`figment ^0.10`, `cap-std ^3.4.1`, and enables `serde_json` and `toml` by
default; it re-exports `serde_json` and `toml`), `clap` 4.6.9, `serde` 1.0.229,
`camino` 1.2.6, `thiserror` 2.0.21, `cap-std` 4.0.3, `rstest` 0.28.0,
`rstest-bdd` and `rstest-bdd-macros` 0.6.0, `googletest` 0.14.4,
`pretty_assertions` 1.4.1, `insta` 1.49.0, `assert_cmd` 2.2.2, `proptest`
1.12.0, `tempfile` 3.27.0.

`ortho-config` 0.8.0 facts this plan relies on, from its published source:

- `OrthoResult<T>` is `Result<T, Arc<OrthoError>>`. `OrthoError` variants
  include `CliParsing`, `File { path, source }`, `CyclicExtends`, `Gathering`,
  `Merge`, `Validation { key, message }`, and `Aggregate`; the enum is
  `#[non_exhaustive]`.
- `#[derive(OrthoConfig)]` generates
  `merge_from_layers<'a, I: IntoIterator<Item = MergeLayer<'a>>>(layers: I)
  -> OrthoResult<Self>`,
  which folds layers with `merge_value` and then deserializes the merged value
  once. `MergeComposer` has `push_defaults`,
  `push_file(value, Option<Utf8PathBuf>)`, `push_environment`, `push_cli`, and
  `layers()`.
- `load_config_file_as_chain(&Path) -> OrthoResult<Option<FileLayerChain>>`
  returns the chain from root ancestor to the named file, detects cycles, and
  returns `Ok(None)` when the file does not exist.
- `CliValueExtractor::extract_user_provided(&self, &ArgMatches)` serializes
  only values the user supplied, honouring `cli_default_as_absent`.
- `is_display_request(&clap::Error) -> bool` is true only for help and
  version display.

Prior art: a stale earlier attempt exists on the remote branch
`1-1-1-implement-the-root-cli-command-with-a-validate-subcommand`. Its lessons
are recorded under `Surprises & discoveries`; none of its code is reused.

Documentation and skills to consult while implementing:

- `docs/mdast-schema-validation-cli-design.md` §User-facing contract, §Schema
  configuration, §Validation pipeline, §Diagnostics design.
- `docs/ortho-config-users-guide.md` (layering, `MergeComposer`,
  `cmds.<name>` convention, `cli_default_as_absent`, error variants). Where it
  differs from the 0.8.0 source, the source wins.
- `docs/rstest-bdd-users-guide.md` plus the upstream `v0-6-0-migration-guide`
  in `leynos/rstest-bdd`.
- `docs/rust-testing-with-rstest-fixtures.md`,
  `docs/reliable-testing-in-rust-via-dependency-injection.md`,
  `docs/rust-doctest-dry-guide.md`,
  `docs/complexity-antipatterns-and-refactoring-strategies.md`,
  `docs/documentation-style-guide.md` (users' guide tested-example markers, ADR
  format), and `docs/developers-guide.md`.
- Skills: `rust-router` (then `domain-cli-and-daemons`, `rust-errors`,
  `rust-types-and-apis`), `hexagonal-architecture`, `rust-unit-testing`,
  `proptest`, `rust-verification`, `addressing-whitaker-findings`, `execplans`,
  `commit-message`, `en-gb-oxendict-style`, and `codegraph-mcp` for navigation.

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
REQ-exit-codes -> TDD §Exit codes -> outcome::RunOutcome -> EP-M1
  -> outcome::tests::exit_codes_match_design
REQ-flags, REQ-explicit-verb, REQ-stdin -> TDD Table 1 -> cli::args
  -> EP-M2 (seven flags), EP-M3 (--config)
  -> tests/features/validate_command.feature, tests/validate_cli.rs
REQ-early-stop -> TDD §Validation stages -> cli::run -> EP-M2, EP-M3
  -> cli::run::tests::pipeline_not_called_on_invocation_error
REQ-config-precedence -> TDD §Schema configuration -> ADR 0002 -> cli::config -> EP-M3
  -> cli::config::tests::precedence_is_exhaustively_correct
GAP-stub, GAP-no-lib -> EP-M1, EP-M2
GAP-no-users-guide, REQ-docs -> EP-M4 -> tests/users_guide_examples.rs
```

Design deviations requiring acceptance with this plan: decisions D3 (defer
environment and discovery layers), D4 (exit `5` placeholder), D12 (`--schema -`
reserved), and D13 (module naming differs from design Table 2). All are
recorded in the design document in `EP-M4`. D4 also adds an acceptance bullet
to roadmap item 1.3.2.

## Verification plan

### Abstract model

Let `K = {schema, schema_format, format, fail_fast, max_errors, quiet}` be the
configurable keys. A layer is a partial function from `K` to values. There are
three kinds of layer in precedence order: `D` (defaults: `format = human`,
`fail_fast = false`, `quiet = false`; undefined elsewhere), `F_1 .. F_n` (one
per file in the `extends` chain, root ancestor first, `n = 0` without
`--config`), and `C` (the command line, defined exactly on the keys whose flags
were supplied). Write `F` for the chain folded so that later files win. The
resolved setting is:

```plaintext
resolve(k) = C(k) if C defines k, else F(k) if F defines k, else D(k)
```

If `resolve(schema)` is undefined the run is an invocation error. Each `F_i` is
admitted only if it type-checks on its own against the file schema (decision
D7); otherwise the run is an invocation error naming that file.

A run maps an argument vector and the file system (read-only) to exactly one
`RunOutcome`. Stages are ordered: parse arguments, load and check each file
layer, merge, build the validation input, call the pipeline port. Any failure
in the first four stages yields `InvalidInvocation` and the pipeline port is
never called. Otherwise the pipeline returns `Valid`, `Invalid`, or a failure
of kind `Schema`, `MarkdownParse`, or `Internal`, mapped to `Success`,
`ValidationFailed`, `SchemaFailure`, `MarkdownParseFailure`, and
`InternalFailure`. Help and version displays yield `Success` without calling
the pipeline.

The exit-code function is the fixed table `Success -> 0`,
`ValidationFailed -> 1`, `InvalidInvocation -> 2`, `SchemaFailure -> 3`,
`MarkdownParseFailure -> 4`, `InternalFailure -> 5`.

### Invariants and lemmas

- `INV-exit-table`: `RunOutcome::exit_code` equals the design table for all
  six variants; codes are pairwise distinct; `exit_code(o) = 0` if and only if
  `o = Success`. Establishes `REQ-exit-codes`.
- `INV-single-outcome`: for every argument vector, `run` returns exactly one
  `RunOutcome` without panicking, calls the pipeline at most once, and calls it
  exactly when the outcome came from the pipeline; the binary's process exit
  code equals `exit_code` of that outcome. Establishes `REQ-exit-codes`.
- `INV-stage-order`: on any invocation or configuration error, the pipeline
  port is called zero times and standard output is empty. Establishes
  `REQ-early-stop`.
- `INV-precedence`: for every key in `K`, the resolved value equals
  `resolve(k)`, including across `extends` chains. Establishes
  `REQ-config-precedence`.
- `INV-absent-cli-transparent`: a flag the user did not supply contributes
  nothing to `C`, not even a JSON `null`; an absent `--fail-fast` or `--quiet`
  never overrides a file value of `true`, and an absent `--format` never
  overrides a file value of `json`.
- `INV-no-ambient-config`: with no `--config`, the resolved settings depend
  only on the argument vector; candidate files in the working directory, home
  directory, or XDG directories and `MDAST_CHECK_*` variables have no effect.
- `INV-explicit-config`: if `--config` names a file that is missing, a
  directory, malformed, part of an `extends` cycle, not TOML, or any of whose
  chain files contains an unknown key at any level or a value of the wrong
  type, the run exits `2`, the message names the offending file, and the
  pipeline is not called.
- `INV-path-rebase`: a relative `schema` from a configuration file becomes the
  canonical directory of that file joined with the value; an absolute path is
  unchanged; a command-line `--schema` is never rebased.
- `INV-document-source`: `--document -` yields `DocumentSource::Stdin`; any
  other value yields `DocumentSource::Path` with that exact path; `--schema -`
  and empty paths are invocation errors.
- `INV-max-errors-positive`: `max_errors`, when resolved, is at least 1.
- `INV-streams`: help and version go to standard output; all diagnostics go to
  standard error; write failures follow decision D9.
- `INV-enum-spellings`: the command line and the configuration file accept
  exactly the same spellings for `format` (`human`, `json`) and `schema_format`
  (`json`, `markdoc`).

- `L-cli-layer`: the command-line layer has key `k` if and only if the flag
  for `k` was supplied, and its value is the parsed value. With
  `AXIOM-ortho-merge` this yields `INV-precedence` and
  `INV-absent-cli-transparent`.
- `L-file-layer`: each pushed file layer equals that file's `cmds.validate`
  table with absent keys omitted and `schema` rebased per `INV-path-rebase`,
  and is pushed only after passing the strict check of decision D7.
- `L-chain-order`: file layers are pushed in chain order, root ancestor first,
  between the defaults and the command-line layer.
- `L-input-total`: building `ValidationInput` and `ReportOptions` from merged
  configuration either succeeds with every field typed or fails only because
  the schema is missing.

### Axioms (trusted, not verified here)

- `AXIOM-ortho-merge`: `merge_from_layers` applies layers in the order pushed
  and, for scalar keys, a later layer's value replaces an earlier one; a key
  absent from a later layer leaves the earlier value; a `null` value replaces
  it. Source: `ortho_config` 0.8.0 `declarative::merge_value` and
  `docs/ortho-config-users-guide.md`. Depended on by `cli::config`. Exercised
  at the boundary by the exhaustive precedence test, which uses the real
  library.
- `AXIOM-ortho-file`: `load_config_file_as_chain` parses TOML, follows
  `extends`, orders the chain root first, canonicalizes paths, reports cycles as
  `CyclicExtends`, reports parse failures and non-TOML extensions as errors,
  and returns `Ok(None)` for a missing file. Depended on by
  `cli::config::file`. Exercised with real files in temporary directories.
- `AXIOM-clap`: `clap` 4 reports help and version requests as `DisplayHelp`
  and `DisplayVersion`, all other parse failures as other kinds, rejects a
  repeated `SetTrue` flag, and rejects non-UTF-8 input for `Utf8PathBuf` and
  string value parsers. Depended on by `cli::run`.
  `Cli::command() .debug_assert()` checks the derive's own consistency.
- `AXIOM-serde`: `serde` rejects `0` for `NonZeroUsize` and reports unknown
  fields under `deny_unknown_fields` with the field name and expected names.
- `AXIOM-process`: `std::process::ExitCode::from(u8)` becomes the process exit
  status. Depended on by `src/main.rs`. Exercised by `assert_cmd` tests.

### Methods per obligation

- Obligation: `INV-exit-table`.
  Method: exhaustive parameterized test. Rationale: six values; enumerating
  them proves the table. Artefact: `src/outcome.rs`, tests
  `exit_codes_match_design` (an `rstest` table whose expected codes are
  literals copied from the design, not computed) and
  `codes_are_distinct_and_zero_only_for_success`, iterating `RunOutcome::ALL`.
  Residual gap: nothing forces `ALL` to grow when a variant is added; the
  design table and review are the guard. Evidence: `cargo test --lib outcome::`
  fails to compile before `RunOutcome` exists and passes after. Non-vacuity:
  every variant appears as a case; negative control: swap the codes of
  `SchemaFailure` and `MarkdownParseFailure`; both rows fail; revert.
- Obligation: `INV-single-outcome` and `INV-streams`.
  Method: property test plus end-to-end tests. Rationale: robustness over
  arbitrary argument vectors is a genuine many-input invariant, and third-party
  parsing code is outside this crate's lints. Domain: `proptest` vectors of 0
  to 8 tokens drawn from every flag (long and `=`-joined forms), `validate`,
  `--help`, `--version`, `-`, `--`, plausible and junk values, and on Unix
  non-UTF-8 byte strings (`OsStr::from_bytes`). Artefact:
  `src/cli/run/tests.rs`, property `run_always_yields_one_outcome`, using a
  `RecordingPipeline` (a test double with a call counter) that returns `Valid`.
  Properties: `run` returns; calls are at most one; calls are one exactly when
  the outcome is `Success` and no help or version token is present; standard
  output is non-empty only when a help or version token is present. Also
  `Cli::command().debug_assert()` as a unit test. End-to-end:
  `tests/validate_cli.rs` asserts the exact exit code and stream content for
  `--help` and `--version` (version compared with `env!("CARGO_PKG_VERSION")`,
  not snapshotted), invocation errors, and the placeholder
  (`placeholder_pipeline_exits_5_until_1_3_2`). Write failures: unit tests with
  a `FailingWriter` (fails on write) and a writer that fails on flush.
  Non-vacuity: the strategy guarantees at least one fully valid invocation shape
  (`validate --document <p> --schema <p>`) with probability above one half by
  construction, and the test asserts at the end, via a `TestRunner` with a
  shared counter, that both the pipeline-called and pipeline-not-called classes
  occurred. Regression files are committed under `proptest-regressions/`.
- Obligation: `INV-stage-order`.
  Method: parameterized in-process tests with `RecordingPipeline`. Domain: one
  case per invocation-error class (unknown flag, missing document, missing
  schema, `--schema -`, empty schema, invalid format, zero cap, repeated
  `--quiet`, missing configuration file, malformed configuration, unknown key).
  Artefact: `src/cli/run/tests.rs`, test
  `pipeline_not_called_on_invocation_error`. Evidence: each case asserts
  `InvalidInvocation`, zero calls, empty standard output, non-empty standard
  error. Non-vacuity: a control row with a valid invocation asserts exactly one
  call; seeded fault: call the pipeline before checking the schema is present;
  the missing-schema row must fail; revert.
- Obligation: `INV-precedence` with `L-cli-layer`,
  `INV-absent-cli-transparent`, and `L-chain-order`. Method: exhaustive
  enumeration. Rationale: the precedence contract depends only on which layers
  define each key, which is finite (each key absent or present in each of `F`
  and `C`: `4^6 = 4,096` combinations); enumerating it is a complete check,
  unlike sampling. Witness values are fixed and differ between layers and from
  the defaults (`F`: `schema = "f.json"`, `schema_format = markdoc`,
  `format = json`, `fail_fast = true` when only `F` defines it and `false` when
  both do, `max_errors = 7`, `quiet` likewise; `C`: `--schema c.json`,
  `--schema-format json`, `--format human`, presence flags set,
  `--max-errors 3`). Artefact: `src/cli/config/tests.rs`, test
  `precedence_is_exhaustively_correct`, which for each combination builds the
  argument vector, parses it with the real `clap` definition, pushes the file
  layer as JSON through the real D7 check, merges with the real
  `merge_from_layers`, and compares with an oracle: a ten-line function in the
  test module applying `resolve(k)` per key with the default literals copied
  from the design, not from `ValidateConfig::default()`. Chain order: rows in
  the file-layer table cover a parent setting `format = "json"` with the child
  setting `"human"` (expect `human`), the child not setting it (expect `json`),
  and three-file chains. Evidence: fails to compile before `cli::config`
  exists; passes after. Non-vacuity: the enumeration visits every combination
  by construction, and the oracle distinguishes all four presence classes for
  each key because the witnesses differ. Negative controls, each applied and
  reverted: (1) make the command-line layer emit `fail_fast = false` when the
  flag is absent; the `F`-only `fail_fast` rows fail; (2) emit `null` for absent
  `schema`, `schema_format`, and `max_errors`; the `F`-only rows for those
  keys fail; (3) push the chain in reverse; the chain-order rows fail.
- Obligation: `INV-no-ambient-config`.
  Method: end-to-end test with child-process isolation. Artefact:
  `tests/validate_cli.rs`, test `ambient_configuration_is_ignored`, which runs
  the binary with `env_clear()`, `HOME` and `XDG_CONFIG_HOME` pointing at a
  temporary directory holding candidate configuration files that set `schema`,
  `MDAST_CHECK_CMDS_VALIDATE_SCHEMA=x`, and `current_dir` set to a directory
  containing `.mdast-check.toml` that sets `schema`, and asserts exit `2` with
  the missing-schema message. Non-vacuity: a companion row passes `--config`
  pointing at the same `.mdast-check.toml` and reaches the placeholder (exit
  `5`), proving the candidate file is valid and would have changed the outcome
  had it been read.
- Obligation: `INV-explicit-config` and `L-file-layer`.
  Method: parameterized tests with real files. Domain: missing file, directory
  instead of file, malformed TOML, `extends` cycle, `.json` extension,
  extensionless file (accepted as TOML), unknown key in `[cmds.validate]`,
  `document` or `config` under `[cmds.validate]`, root-level `schema`,
  `[cmd.validate]` typo, `[cmds.lint]`, wrong-type values (`max_errors = "ten"`,
  `max_errors = 0`, `format = "xml"`), `schema = ""`, and an unknown key or
  type error in a parent file of a chain. Artefact: `src/cli/config/tests.rs`,
  test `invalid_configuration_is_rejected`, plus end-to-end rows in
  `tests/validate_cli.rs`. Evidence: each row asserts exit `2` and that
  standard error names the offending file (the parent file for chain rows).
  Non-vacuity: the extensionless row is accepted, proving the check is not a
  blanket rejection; the parent-file rows prove checks are not applied to the
  top file only.
- Obligation: `INV-path-rebase`.
  Method: parameterized tests on the real file system. Domain: relative,
  absolute, and `..`-containing schema values; base and extending files in
  different directories; on Unix a symbolic link to a configuration file in
  another directory. Artefact: `src/cli/config/tests.rs`, test
  `relative_schema_paths_resolve_against_their_file`. Oracle independent of
  `join`: the test creates the schema file at its intended location inside a
  canonicalized temporary directory and asserts that the resolved schema path,
  canonicalized, equals the canonical path of that created file. Non-vacuity:
  includes an absolute path that must not change and a command-line relative
  path that must not be rebased.
- Obligation: `INV-document-source`, `INV-max-errors-positive`,
  `INV-enum-spellings`. Method: parameterized tests. Domain: `-`, `./-`, a
  normal path, an empty string; `--max-errors` values `0`, `1`,
  `18446744073709551615`, `-1`, `ten`; every enum spelling and a wrong one,
  through both the command line and the file. Artefact: `src/cli/args/tests.rs`
  and `src/cli/config/tests.rs`, test `enum_spellings_agree`.
- Obligation: `L-input-total`.
  Method: parameterized tests on input construction. Artefact:
  `src/cli/config/tests.rs`, test `input_requires_schema`.
- Obligation: user-facing error wording.
  Method: `insta` snapshots of every `InvocationError` rendering and of the
  placeholder message, with filters that replace temporary-directory paths with
  `[TMP]`. Artefact: `src/cli/snapshots/`. Help is not snapshotted (risk:
  `Cargo.lock` is ignored); a test asserts that root and `validate` help list
  every flag name.

### Formal verification decision

No Verus proof and no Kani harness are planned for this item (decision D10).

Verus is warranted when a small, pure, repository-owned kernel must satisfy a
property for unboundedly many inputs. This item has no such kernel. The only
non-trivial business rule, layer precedence, is implemented by `ortho-config`'s
merge; this crate owns only the construction of the layers. A Verus proof would
have to be about a model of the merge, not the code that runs, so it would
prove the oracle rather than the implementation. Precedence depends only on
which layers define each key, a finite space of 4,096 combinations that the
exhaustive test enumerates against the real merge, so it is fully checked
without a prover. The exit-code table is a six-element finite function, which
enumeration proves. Path rebasing is a single join. Kani would add bounded
exhaustiveness to code with no arithmetic, no `unsafe`, and no owned state
machine; the code that matters leans on `serde_json` and `clap`, which are
impractical to model check. The argument-robustness property is the right tool
for "never panics" because the risk lives in third-party parsing reached
through arbitrary inputs.

Candidates for formal verification in later items, recorded so they are not
forgotten: the fallback span selection of roadmap 2.1.1 (the narrowest
available span is chosen and always contains the reported instance), the
deterministic error ordering and capping of roadmaps 2.2.1 and 2.2.2 (the
reported list is a prefix of a total order and its length is `min(n, cap)`),
and the instance-path registration of roadmap 1.2.2 (every registered path
addresses a node in the projected JSON). Each is a pure, owned kernel over
unbounded input.

### Suggested BDD features

Feature file: `tests/features/validate_command.feature`, bound by
`tests/validate_command_bdd.rs` with `rstest-bdd` 0.6.0. Steps drive the public
`mdast_check::cli::run` entry point in-process with a scripted fake pipeline,
so every exit code is reachable. Every path in a step is made absolute inside a
per-scenario temporary directory, so no scenario depends on the test process's
working directory. Steps return `Result` and never call `expect`. Linked
milestones: `EP-M2` (first block) and `EP-M3` (configuration block).

```gherkin
Feature: Validate command invocation
  The validate command parses its flags, merges configuration, and reports a
  stable exit code for every outcome.

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

  Scenario: The command line overrides the configuration file
    Given a configuration file "mdast-check.toml" containing:
      """
      [cmds.validate]
      format = "json"
      """
    And a validation pipeline that reports the document valid
    When the user runs "validate --document a.md --schema s.json --format human" with that configuration
    Then the pipeline received output format "human"

  Scenario: An absent flag keeps the configured value
    Given a configuration file "mdast-check.toml" containing:
      """
      [cmds.validate]
      fail_fast = true
      """
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
```

Step semantics: relative paths in `When` steps and in the configuration step
are resolved against the scenario's temporary directory before the argument
vector is built; "received schema" compares the canonical path of the expected
file (created by the step) with the canonical resolved path, so the scenario
proves rebasing happened. The `Given a configuration file … containing:` step
takes a docstring and is also used to load the users' guide configuration
example through the shared loader.

Runner: `cargo test --test validate_command_bdd` (also run by `make test`).

## Plan of work

### `EP-M0` feasibility probe (prototype, deleted)

Question: can the planned `ortho-config` 0.8.0 usage compile and behave as
assumed under this repository's Clippy and Whitaker lints? Work in the branch
worktree after approval. Add the runtime and development dependencies from
`Interfaces and dependencies` to `Cargo.toml`, an empty `src/lib.rs` with crate
documentation, and a throwaway integration test `tests/probe_ortho_config.rs`
that:

1. derives `OrthoConfig`, `Deserialize`, `Serialize`, and `Default` on a
   probe `ValidateConfig` with the planned field types, every field marked
   `#[ortho_config(skip_cli)]`;
2. merges defaults, file layers from `load_config_file_as_chain` (a two-file
   `extends` chain), and a sparse command-line JSON layer through
   `MergeComposer` and `merge_from_layers`;
3. deserializes each chain element into the planned
   `#[serde(deny_unknown_fields)]` file structs and prints the messages for an
   unknown key, `max_errors = 0`, and `format = "xml"`;
4. tries decision D2a: a `clap` derive struct with `SetTrue` presence flags
   and `#[ortho_config(cli_default_as_absent)]`, calling
   `extract_user_provided` and printing the layer for "no flags" and "all
   flags";
5. renders `clap` help and an error through `to_string()` and checks for ANSI
   escapes;
6. compiles one `#[rstest]` plus `#[gtest]` test, one `rstest-bdd` scenario,
   one `cap_std::fs_utf8` file write in a test, and one `rstest-bdd` step
   returning `Result`.

Run `make lint` (with `[[bin]] doc = false`) and record every Clippy and
Whitaker finding. Go/no-go: checks 1 to 3, 5, and 6 behave as expected and no
lint fires inside generated code that cannot be met with a recorded item-scoped
`#[expect]`. Check 4 decides between the two D2a variants and cannot cause a
no-go. Record the transcript in `Artefacts and notes` and the findings in
`Surprises & discoveries`. If decision D2 cannot hold, set the status to
`BLOCKED` under the Architecture tolerance. Delete
`tests/probe_ortho_config.rs` before committing; keep the dependency additions
only if go.

### `EP-M1` run-outcome contract and library/binary split

Assigned: `REQ-exit-codes`, `GAP-no-lib`, decisions D1 and D13.

Red: create `src/lib.rs` with only the crate-level `//!` documentation and
`pub mod outcome;`, and `src/outcome.rs` with the test module first:
`exit_codes_match_design` and `codes_are_distinct_and_zero_only_for_success`.
Run `cargo test --lib outcome::`; it fails to compile because `RunOutcome` does
not exist.

Green: define `RunOutcome` with the six variants, `RunOutcome::ALL`,
`const fn exit_code(self) -> u8`, and
`impl From<RunOutcome> for std::process::ExitCode`. Add the `[[bin]]` table and
set `TARGET ?= mdast-check` in the Makefile. `src/main.rs` keeps its stub in
this milestone so the plateau is coherent; the stub is replaced in `EP-M2`.

Refactor: add a doctest example on `RunOutcome::exit_code`. Run the negative
control from `Verification plan` and revert it.

### `EP-M2` `validate` command surface without a configuration file

Assigned: `REQ-flags` (seven flags), `REQ-explicit-verb`, `REQ-stdin`,
`REQ-early-stop`, `GAP-stub`, decisions D4, D5, D8, D9, D12, D14.

Red: write the first block of `tests/features/validate_command.feature` and its
step file `tests/validate_command_bdd.rs`; write `tests/validate_cli.rs` with
`assert_cmd` tests for `--help`, `--version`, `validate --help`, no subcommand
(exit `2`), an unknown flag (exit `2`), a missing `--schema` (exit `2`), a
non-UTF-8 argument on Unix (exit `2`), and
`placeholder_pipeline_exits_5_until_1_3_2`; write unit tests for argument
parsing, `run`, and the argument-robustness property. Run
`cargo test --test validate_command_bdd` and `cargo test --test validate_cli`;
they fail to compile (no `mdast_check::cli::run`) or fail on the stub's
behaviour (exit `0` everywhere).

Green, in this order:

1. `src/app/settings.rs`: core value types `DocumentSource`, `SchemaFormat`,
   `OutputFormat`, `ValidationInput`, and `ReportOptions` (see
   `Interfaces and dependencies`). No `clap` here; the two enums derive `serde`
   with lower-case spellings.
2. `src/app/pipeline.rs`: the provisional driven port `ValidationPipeline`,
   `Verdict`, and `PipelineFailure` with
   `const fn outcome(&self) -> RunOutcome`. `src/pipeline/mod.rs`:
   `PendingPipeline`, returning `PipelineFailure::Internal` with the
   placeholder message.
3. `src/cli/args.rs`: `clap` derive types `Cli`, `Command`, and
   `ValidateArgs`, carrying every flag except `--config`, which arrives with
   its behaviour in `EP-M3` so that no plateau accepts a file it ignores.
   Explicit long names; no short flags except `-h` and `-V`; value parsers for
   `DocumentSource` (rejecting empty), schema paths (rejecting `-` and empty),
   the two enums (through `PossibleValuesParser` with the same spellings as
   `serde`), and `NonZeroUsize`. A unit test calls
   `Cli::command().debug_assert()`.
4. `src/cli/config/mod.rs` (first cut): `pub(crate) ValidateConfig` with
   `#[derive(OrthoConfig, Deserialize, Serialize, Default)]` and `skip_cli` on
   every field; the command-line layer (decision D2a, using whichever variant
   `EP-M0` selected); and `resolve`, which merges defaults and the command-line
   layer through `MergeComposer` and builds `ValidationInput` and
   `ReportOptions`, failing with `InvocationError::MissingSchema` when no
   schema is resolved.
5. `src/cli/error.rs`: `pub(crate) InvocationError` (`thiserror`), with
   `Usage(Box<clap::Error>)`, `MissingSchema`, and the configuration variants
   added in `EP-M3`.
6. `src/cli/run.rs`: the `run` function (signature in
   `Interfaces and dependencies`), which parses with `Cli::try_parse_from`,
   handles display requests and errors per decision D9, resolves the request,
   calls the pipeline once, and passes every outcome to one private reporting
   function that writes crate-authored text to `Streams`.
7. `src/main.rs`: the composition root. Lock `std::io::stdout()` and
   `std::io::stderr()`, build `Streams`, call `mdast_check::cli::run` with
   `std::env::args_os()` and `PendingPipeline`, and return the
   `std::process::ExitCode`. Delete the stub and its `#[expect]` in the same
   commit.

Refactor: extract helpers to keep `run` under the complexity thresholds, add
`insta` snapshots of the invocation-error and placeholder messages, and run the
seeded fault for `INV-stage-order`.

### `EP-M3` configuration-file layering through `ortho-config`

Assigned: `REQ-flags` (`--config`), `REQ-config-precedence`, `GAP-no-config`,
decisions D2, D3, D6, D7.

Red: add the configuration scenarios to the feature file; add
`precedence_is_exhaustively_correct`, `invalid_configuration_is_rejected`,
`relative_schema_paths_resolve_against_their_file`, and `enum_spellings_agree`
to `src/cli/config/tests.rs`; add `ambient_configuration_is_ignored` and
configuration rows to `tests/validate_cli.rs`. Run the focused commands; they
fail because `--config` does not exist yet (`clap` rejects it with exit `2`),
so every row expecting a file-supplied value fails.

Green: add `--config <PATH>` to `ValidateArgs`. Add `src/cli/config/file.rs`:
load the chain with `load_config_file_as_chain`, turn `Ok(None)` into
`InvocationError::Config` for the typed path, check each chain element against
the `deny_unknown_fields` file structs (decision D7), rebase `schema` (decision
D6), re-serialize omitting absent keys, and push each element with its path. Map
`OrthoError` values to `InvocationError::Config { path, reason }` with a
crate-authored `reason` and the source error kept as `#[source]`.

Refactor: confirm each file is under 400 lines and each module has its own
`tests.rs` where tests would push it over; run the three negative controls for
`INV-precedence` and revert them.

### `EP-M4` documentation, tested examples, and roadmap closure

Assigned: `REQ-docs`, `GAP-no-users-guide`.

Red: write `tests/support/markdown_examples.rs`, the shared loader, test first:
unit tests with inline Markdown for a marked fence, an unmarked fence, an
unterminated fence, a missing identifier, a duplicate identifier, an indented
fence, and a `~~~` fence. Run `cargo test --test users_guide_examples`; it
fails to compile.

Green and documentation, in this order:

1. Implement the loader. It parses Markdown passed as a string (documents are
   embedded with `include_str!`, so the tests read the shipped text without
   touching the file system) and returns `(identifier, language, body)`
   triples. Allowed languages: `sh` (one `mdast-check` command, split on
   whitespace; shell metacharacters such as `<`, `|`, `$`, and quotes are
   rejected), `toml` (a configuration file), and `text` (expected standard
   error for the preceding `sh` example, asserted as a substring). Any other
   fence language in a user-facing document fails. To avoid unused-function
   warnings, the loader exposes one function used in full by both consumers.
2. Create `docs/users-guide.md` for release `0.1.0` (unreleased, early
   adopter), with imperative headings: install and build; check an invocation
   (the first-validation walkthrough is deferred to roadmap 1.3.2, because
   today it would end in exit `5`); choose the document and schema (`-` for
   standard input; `--schema -` is rejected); configure defaults with a file
   (TOML only, `[cmds.validate]` keys, unknown keys rejected, relative paths
   resolved against the canonical directory of the file, `extends`); understand
   precedence; interpret exit codes (table; `5` is not transient, do not
   retry); output streams and accessibility (plain text, no colour, help on
   standard output, diagnostics on standard error); current limitations
   (validation not implemented, flags accepted but not yet acting, no
   environment variables or implicit discovery, non-UTF-8 directories in
   configuration paths). Precede every fenced example with
   `<!-- tested-example: <id> -->`.
3. Add `tests/users_guide_examples.rs`, which loads `docs/users-guide.md`
   and `README.md`, writes each `toml` example into a temporary directory under
   a file name mapped from its identifier (so an `extends` example can
   reference its parent), runs each `sh` example against the binary with
   `assert_cmd` in that directory, and asserts the exit code recorded for that
   identifier. The test's identifier table must match the documents'
   identifiers one to one. The BDD step file reuses the loader for the
   configuration example.
4. Write `docs/adrs/adr-0002-explicit-configuration-layer-composition.md`
   (decisions D2, D2a, D3, D6, and D7, the reserved discovery name, and the
   revisit trigger), following the ADR 0001 format.
5. Update `docs/mdast-schema-validation-cli-design.md`: in §Schema
   configuration, the 1.1.1 precedence, the strict `[cmds.validate]` file
   shape, the relative-path rule, and the deferral of environment variables and
   discovery, linking ADR 0002; in §Exit codes, the placeholder exit `5` until
   roadmap 1.3.2; in §Module boundaries, the realized `cli`, `app`, `pipeline`,
   and `outcome` modules, the provisional `ValidationPipeline` port, and the
   note that `validate` remains reserved for the validator wrapper; in
   §Proposed flags, the `--schema -` reservation.
6. Update `docs/developers-guide.md` with: the module layout and hexagonal
   boundary (only `src/cli/` may use `clap` and `ortho_config`; the generated
   `ortho-config` loaders are banned); the provisional `ValidationPipeline`
   port and how 1.1.2, 1.3.2, 2.1.2, and 2.2.1 extend it; the `Streams`
   injection rule and the obligation for roadmap 1.2.1 to inject standard input
   through the same structure rather than calling `std::io::stdin()` inside the
   pipeline; how to add a configuration key (field on `ValidateConfig` and on
   the file struct, flag on `ValidateArgs`, witness values in the precedence
   enumeration, users' guide entry); the tested-example loader and its reuse
   policy; and the Cranelift paragraph, noting that behaviour tests now exist
   so the measurement is due (the measurement itself is out of scope).
7. Update `README.md`: a short usage section with a marked example linking the
   users' guide, and remove the stale `make typecheck` mention.
8. In `docs/roadmap.md`, add to item 1.3.2 the bullet "Replace
   `PendingPipeline`; a valid document exits `0`." Then mark item 1.1.1 and its
   sub-bullets `[x]` only after all gates pass, annotating the exit-code
   bullet: "contract and mapping; codes 1, 3, and 4 become reachable with 1.2.1
   to 1.3.2".

## Milestones and plateaus

- Identifier and outcome: `EP-M0`, a recorded go or no-go for decision D2 and
  a choice for D2a, with no committed code.
  - Requirements and gaps: none discharged; de-risks `REQ-config-precedence`.
  - Acceptance evidence: probe transcript in `Artefacts and notes`.
  - Conformance check: design unchanged unless the probe falsifies D2, in
    which case the plan becomes `BLOCKED`.
  - Recovery: delete the probe file; restore `Cargo.toml` if no-go.
  - Remaining gaps: everything.
  - Compatibility decision: none.
- Identifier and outcome: `EP-M1`, a library crate exposing the run-outcome
  contract and a binary named `mdast-check` (still the stub).
  - Requirements and gaps: `REQ-exit-codes` (contract only), `GAP-no-lib`.
  - Acceptance evidence: `outcome::tests` pass; `make build` produces
    `target/debug/mdast-check`.
  - Conformance check, one line per item:
    - exit table matches design §Exit codes;
    - module naming per D13;
    - no assumption falsified;
    - public interface limited to `outcome::RunOutcome`;
    - no trust boundary or persisted format;
    - trace link `REQ-exit-codes -> EP-M1` current.
  - Recovery: revert the milestone commit.
  - Remaining gaps: the stub, the CLI, configuration, docs.
  - Compatibility decision: none.
- Identifier and outcome: `EP-M2`, the shipped binary parses the `validate`
  flag set except `--config`, rejects invalid invocations with exit `2`, prints
  help and version with exit `0`, and reports the placeholder with exit `5`;
  the in-process runner maps every pipeline outcome to its code.
  - Requirements and gaps: `REQ-flags` (seven flags), `REQ-explicit-verb`,
    `REQ-stdin`, `REQ-early-stop`, `GAP-stub`.
  - Acceptance evidence: first-block BDD scenarios, `tests/validate_cli.rs`,
    unit and property tests, and message snapshots pass.
  - Conformance check, one line per item:
    - flags match design Table 1 (except `--config`, deferred to `EP-M3`);
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
  command line taking precedence, through `ortho-config`, with strict per-file
  checking.
  - Requirements and gaps: `REQ-flags` (`--config`),
    `REQ-config-precedence`, `GAP-no-config`.
  - Acceptance evidence: configuration BDD scenarios, the exhaustive
    precedence test, file-layer rows, rebasing rows, the no-ambient test, and
    end-to-end rows pass; negative controls rejected.
  - Conformance check, one line per item:
    - precedence matches design §Schema configuration as amended by D3;
    - `ortho_config` confined to `src/cli/config/`;
    - `AXIOM-ortho-merge` held under the exhaustive test;
    - no unapproved interface or dependency;
    - the configuration-file shape is a new user-facing format, approved by
      this plan;
    - trace links current.
  - Recovery: revert the milestone commits; `EP-M2` remains valid.
  - Remaining gaps: documentation.
  - Compatibility decision: none (first release of the format).
- Identifier and outcome: `EP-M4`, documentation matches behaviour and every
  users' guide and README example is executed by the suite; roadmap 1.1.1 is
  done and roadmap 1.3.2 carries the placeholder-removal bullet.
  - Requirements and gaps: `REQ-docs`, `GAP-no-users-guide`.
  - Acceptance evidence: `tests/users_guide_examples.rs` passes; Markdown
    gates pass; roadmap updated.
  - Conformance check: design document and ADR 0002 updated for D2, D2a, D3,
    D4, D6, D7, D12, and D13; no remaining unrecorded deviation.
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
`typos.toml`, commit the regenerated file in its own commit (on a base branch
if the change is unrelated to this work).

Focused commands per milestone:

```plaintext
EP-M0  cargo test --test probe_ortho_config -> probe cases print observations; make lint findings recorded
EP-M1  Red:   cargo test --lib outcome::          -> error[E0412]: cannot find type `RunOutcome`
       Green: cargo test --lib outcome::          -> pass
EP-M2  Red:   cargo test --test validate_cli      -> stub exits 0; assertions on codes 2 and 5 fail
       Red:   cargo test --test validate_command_bdd -> cannot find `mdast_check::cli::run`
       Green: both pass; cargo test --lib cli:: passes, including the robustness property
EP-M3  Red:   cargo test --lib cli::config        -> rows supplying values by file fail (--config unknown)
       Green: cargo test --lib cli::config        -> pass, 4,096 precedence combinations checked
       Green: cargo test --test validate_command_bdd --test validate_cli -> pass
EP-M4  Red:   cargo test --test users_guide_examples -> loader unit tests fail to compile
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
  end-to-end, and tested-example tests, plus the existing build-standard
  contract.
- Verification: every obligation in `Verification plan` has its artefact
  passing and its negative controls recorded as rejected.
- Lint and format: `make check-fmt` and `make lint` (Clippy and Whitaker)
  pass with no new `#[expect]` other than any recorded in `Decision log`.
- Documentation: `make markdownlint` and `make nixie` pass.
- Performance: none required for this item.
- Security: the only untrusted inputs are command-line arguments and the
  configuration files named by the user; both are parsed by typed parsers and
  never executed. No new network or privileged effect.

## Idempotence and recovery

All steps are re-runnable. Tests write only to `tempfile` temporary
directories, accessed through `cap_std::fs_utf8`, which are removed on drop. If
a milestone fails part-way, reset to the previous milestone commit
(`git reset --hard <commit>` on the branch, after confirming with `git status`
that nothing unrelated is lost) and repeat. Negative controls are applied as
uncommitted edits and reverted with `git checkout -- <file>`; confirm with
`git diff --stat` that the tree is clean before committing. Delete the `EP-M0`
probe file before the first `EP-M1` commit. Remove stray `*.snap.new` files with
`cargo insta reject` or by deleting them. Commit any new
`proptest-regressions/` file together with the fix it guards.

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
$ ls ~/.local/share/whitaker/crates
... module_max_lines  no_expect_outside_tests  no_std_fs_operations
... rstest_helper_should_be_fixture  test_must_not_have_example ...
```

Milestone evidence is appended here during implementation.

## Interfaces and dependencies

Runtime dependencies (caret requirements, `[dependencies]`):

- `ortho_config = "0.8.0"` (default features `serde_json` and `toml`): the
  declarative merge, file loading, command-line value extraction, and
  display-request helper. The re-exported `ortho_config::serde_json` and
  `ortho_config::toml` are used, so no direct `serde_json` or `toml` dependency
  is added.
- `clap = { version = "4.6.9", features = ["derive"] }`: argument parsing.
- `serde = { version = "1.0.229", features = ["derive"] }`: configuration
  (de)serialization.
- `camino = { version = "1.2.6", features = ["serde1"] }`: UTF-8 paths.
- `thiserror = "2.0.21"`: `InvocationError`.

Development dependencies (`[dev-dependencies]`, in addition to
`rstest = "0.28.0"`):

- `rstest-bdd = "0.6.0"`, `rstest-bdd-macros = "0.6.0"`: behavioural tests.
- `googletest = "0.14.4"`: matchers (`assert_that!`, `expect_that!`), used
  with `#[gtest]` beneath `#[rstest]`.
- `pretty_assertions = "1.4.1"`: readable equality diffs.
- `insta = "1.49.0"`: message snapshots with path filters.
- `assert_cmd = "2.2.2"`: end-to-end process tests.
- `proptest = "1.12.0"`: the argument-robustness property.
- `tempfile = "3.27.0"`: temporary directories.
- `cap-std = { version = "4.0.3", features = ["fs_utf8"] }`: file access in
  tests without `std::fs` (Whitaker `no_std_fs_operations`).

No `mockable` dependency is added, because no environment-dependent behaviour
is introduced (decision D3). No `eyre` dependency is added: `main` returns an
`ExitCode` and all error text is crate-authored.

Module layout and public items at the end of `EP-M3`:

```plaintext
src/main.rs                 composition root (binary `mdast-check`)
src/lib.rs                  crate docs; pub mod app, cli, outcome, pipeline
src/outcome.rs              RunOutcome
src/app/mod.rs              application core: re-exports
src/app/settings.rs         DocumentSource, SchemaFormat, OutputFormat, ValidationInput, ReportOptions
src/app/pipeline.rs         ValidationPipeline, Verdict, PipelineFailure (provisional)
src/pipeline/mod.rs         PendingPipeline (driven adapter; replaced by roadmap 1.3.2)
src/cli/mod.rs              driving adapter: pub use run, Streams
src/cli/args.rs             pub(crate) Cli, Command, ValidateArgs (clap)
src/cli/args/tests.rs       argument parsing tests
src/cli/run.rs              run, Streams, private reporting function
src/cli/run/tests.rs        ordering, write-failure, and robustness tests
src/cli/error.rs            pub(crate) InvocationError
src/cli/config/mod.rs       pub(crate) ValidateConfig (OrthoConfig), resolve
src/cli/config/file.rs      chain loading, strict file structs, rebasing
src/cli/config/tests.rs     precedence enumeration and file-layer tests
tests/validate_cli.rs       end-to-end tests
tests/validate_command_bdd.rs and tests/features/validate_command.feature
tests/support/markdown_examples.rs and tests/users_guide_examples.rs
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
    Schema { message: String },
    MarkdownParse { message: String },
    Internal { message: String },
}

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

pub fn run<I, T>(args: I, streams: &mut Streams<'_>, pipeline: &dyn ValidationPipeline) -> RunOutcome
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone;
```

`ReportOptions` is resolved and validated by `run` but not yet consumed by a
renderer. It is a public type, so its fields raise no dead-code warnings; `run`
passes it to the private reporting function, whose documentation names it as
the extension point for roadmap items 2.1.2, 2.2.1, and 2.2.2. Tests of
`resolve` assert its values, so the resolution is verified even though no
behaviour depends on it yet.

Reuse policy:

- `RunOutcome` is the only place exit codes are defined. Roadmap 1.1.2 maps
  its error taxonomy onto `RunOutcome` through `outcome()` methods and must not
  introduce a second table.
- `ValidationPipeline` is the only boundary between the CLI adapter and the
  validation stages. It is provisional: later items may reshape it, changing
  every implementation and caller together. Roadmap items 1.2.1 to 1.3.2
  implement it outside `src/app/` and replace `PendingPipeline` in
  `src/main.rs`; they must not use `clap` or `ortho_config`.
- `Streams` is the only route to standard output and standard error. Later
  renderers (2.1.2, 2.2.1) write through it; standard input is added to it by
  roadmap 1.2.1.
- `cli::config` owns all configuration loading. New configuration keys are
  added there and in the strict file structs, never parsed ad hoc elsewhere.
- `tests/support/markdown_examples.rs` is the shared loader for tested
  examples in all user-facing documents; no copies.

## Revision note

- 2026-11-20: Initial draft from reconnaissance of the repository, the
  `ortho-config` 0.8.0 source, the stale prior attempt, and the vendored guides.
- 2026-11-20: Design review applied. Changed: typed per-file configuration
  layers (D7, now covering the whole file); banned the derive's ambient loaders
  and marked fields `skip_cli`; split the pipeline port into `ValidationInput`
  and `ReportOptions` and made it provisional (D14); renamed modules and the
  outcome type (D13); added D2a (library command-line extractor,
  probe-decided), D11 (`Cargo.lock`), and D12 (`--schema -`); amended D3
  (discovery reservation), D4 (removal obligations), and D6 (canonical
  directories). Verification: replaced the sampled precedence property with
  exhaustive enumeration of all 4,096 presence combinations against the real
  merge, added `null`-override and chain-order negative controls, a
  no-ambient-configuration end-to-end test, an argument-robustness property,
  write-failure tests, and an independent rebasing oracle; removed the rebasing
  property whose oracle repeated the implementation; replaced help snapshots
  with flag-presence assertions. Added Whitaker lints to constraints, `cap-std`
  to development dependencies, and a base branch for the regenerated
  `typos.toml`. Code budget lowered from 2,300 to 2,200 net lines. Milestone
  order is unchanged.
