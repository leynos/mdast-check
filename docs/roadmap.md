# MDAST Check roadmap

This roadmap defines the implementation sequence for `mdast-check`. It follows
the design in
[MDAST schema validation CLI design](./mdast-schema-validation-cli-design.md)
and keeps phases, steps, and tasks measurable.

## 1. Deliver the baseline validation CLI

### 1.1. Establish the command and configuration shell

- [ ] 1.1.1. Implement the root CLI command with a `validate` subcommand.
  See `mdast-schema-validation-cli-design.md` §User-facing contract.
  - [ ] Parse `--document`, `--schema`, `--schema-format`, `--config`,
    `--format`, `--fail-fast`, `--max-errors`, and `--quiet`.
  - [ ] Load configuration through `ortho-config` v0.8.0 with CLI overrides
    taking precedence over config file values.
  - [ ] Return stable exit codes for success, validation failure, invalid
    invocation, schema failure, Markdown parse failure, and internal failure.
- [ ] 1.1.2. Define the crate-local error taxonomy.
  See `mdast-schema-validation-cli-design.md` §Diagnostics design.
  - [ ] Implement `thiserror` domain error types for CLI, schema, parsing, and
    validation failures.
  - [ ] Ensure each error category maps deterministically to one exit code.
  - [ ] Add unit tests for exit-code mapping and top-level error conversions.

### 1.2. Parse Markdown and build the validation input

- [ ] 1.2.1. Parse Markdown documents into MDAST using `markdown`.
  See `mdast-schema-validation-cli-design.md` §Markdown parsing and MDAST
  projection.
  - [ ] Introduce a document loader that supports file input and standard
    input.
  - [ ] Configure parser options explicitly in one module.
  - [ ] Return parse failures as source-aware diagnostics.
- [ ] 1.2.2. Project MDAST into a JSON validation model with source indexing.
  Requires 1.2.1. See `mdast-schema-validation-cli-design.md` §Data model and
  source mapping.
  - [ ] Serialize relevant MDAST nodes into `serde_json::Value`.
  - [ ] Register instance paths for nodes and relevant scalar properties.
  - [ ] Add unit tests for nested path registration and fallback span
    selection.

### 1.3. Integrate schema validation

- [ ] 1.3.1. Load and self-validate JSON Schema documents.
  See `mdast-schema-validation-cli-design.md` §Schema handling.
  - [ ] Parse schema files into `serde_json::Value`.
  - [ ] Validate the schema document itself with
    `json-schema-validator-core`.
  - [ ] Render schema-document failures against the schema file path.
- [ ] 1.3.2. Validate projected MDAST documents against compiled schemata.
  Requires 1.2.2 and 1.3.1. See `mdast-schema-validation-cli-design.md`
  §Validation pipeline.
  - [ ] Normalize validator failures into crate-local error structures.
  - [ ] Preserve both schema path and instance path in every normalized
    failure.
  - [ ] Add unit tests for validator error normalization.

## 2. Deliver friendly diagnostics and stable outputs

### 2.1. Render Markdown-author-friendly diagnostics

- [ ] 2.1.1. Map validation failures back to Markdown source spans.
  Requires 1.3.2. See `mdast-schema-validation-cli-design.md` §Data model and
  source mapping.
  - [ ] Prefer exact scalar spans where available.
  - [ ] Fall back to the nearest owning node span when a scalar span is absent.
  - [ ] Fall back to the file span for document-level failures.
- [ ] 2.1.2. Render `miette` diagnostics with consistent wording.
  Requires 2.1.1. See `mdast-schema-validation-cli-design.md` §Diagnostics
  design.
  - [ ] Provide author-facing summary messages for common validation failures.
  - [ ] Include schema path and instance path in diagnostic metadata.
  - [ ] Snapshot-test the rendered human-readable reports.

### 2.2. Stabilize machine-readable output

- [ ] 2.2.1. Implement JSON output mode for automation.
  Requires 1.3.2. See `mdast-schema-validation-cli-design.md` §JSON output mode.
  - [ ] Emit a stable envelope containing document path, schema path,
    validity, and normalized errors.
  - [ ] Ensure error ordering is deterministic for snapshot tests.
  - [ ] Snapshot-test representative success and failure payloads.
- [ ] 2.2.2. Add quiet and fail-fast behaviour without changing exit-code
  semantics. Requires 1.1.1 and 2.2.1. See
  `mdast-schema-validation-cli-design.md` §Validation pipeline.
  - [ ] Suppress success output in quiet mode while keeping failure output.
  - [ ] Stop after the first renderable validation error in fail-fast mode.
  - [ ] Verify the behaviour with command snapshots.

### 2.3. Harden the end-to-end test harness

- [ ] 2.3.1. Add `rstest` fixture coverage for projection and diagnostics.
  See `mdast-schema-validation-cli-design.md` §Testing strategy.
  - [ ] Move large Markdown and schema samples into `.fixture` files.
  - [ ] Load fixture contents with `include_str!`.
  - [ ] Parameterize repeated projection and mapping tests with `rstest`.
- [ ] 2.3.2. Add BDD scenarios for user-visible validation behaviour.
  Requires 2.1.2 and 2.2.1. See `mdast-schema-validation-cli-design.md` §BDD
  tests.
  - [ ] Cover valid documents, invalid documents, invalid schemata, and parse
    failures.
  - [ ] Keep feature files under `tests/features/`.
  - [ ] Use `assert-cmd` and `insta` where the CLI surface is the behaviour
    under test.

## 3. Add experimental Markdown-native schemata

### 3.1. Define and compile the Markdoc schema subset

- [ ] 3.1.1. Specify the supported Markdoc tag subset for schema authoring.
  See `mdast-schema-validation-cli-design.md` §Stretch goal: Markdoc-native
  schema mode.
  - [ ] Document the supported `{% schema %}`, `{% property %}`, `{% rule %}`,
    and `{% ref %}` tags in user-facing documentation.
  - [ ] Define how headings and fenced JSON blocks contribute to the compiled
    schema.
  - [ ] Mark unsupported Markdoc constructs as explicit errors.
- [ ] 3.1.2. Compile Markdoc-authored schemata into the internal JSON Schema
  representation. Requires 3.1.1. See `mdast-schema-validation-cli-design.md`
  §Schema handling.
  - [ ] Reuse the existing schema validation pipeline after compilation.
  - [ ] Snapshot-test the compiled JSON Schema form.
  - [ ] Report schema authoring errors against the Markdoc source file.

### 3.2. Validate the experimental mode end to end

- [ ] 3.2.1. Add BDD scenarios for Markdoc-authored schemata.
  Requires 3.1.2 and 2.3.2. See `mdast-schema-validation-cli-design.md`
  §Testing strategy.
  - [ ] Cover one successful validation scenario and one failing validation
    scenario.
  - [ ] Cover one invalid Markdoc schema scenario.
  - [ ] Keep compiled-schema snapshots alongside the fixtures that produce
    them.
- [ ] 3.2.2. Gate the feature as experimental in CLI help and diagnostics.
  Requires 3.1.2. See `mdast-schema-validation-cli-design.md` §Stretch goal:
  Markdoc-native schema mode.
  - [ ] Mark `--schema-format markdoc` as experimental in help output.
  - [ ] Emit a clear message when unsupported Markdoc constructs are used.
  - [ ] Ensure the baseline JSON Schema path remains the default.

## 4. Prepare the project for sustained development

### 4.1. Document the internal contracts

- [ ] 4.1.1. Publish the projected MDAST JSON shape used for validation.
  Requires 1.2.2. See `mdast-schema-validation-cli-design.md` §Markdown parsing
  and MDAST projection.
  - [ ] Document which fields are intentionally present in the validation
    model.
  - [ ] Document which parser metadata is intentionally excluded.
  - [ ] Link the published shape to snapshot tests that guard it.
- [ ] 4.1.2. Document fixture, snapshot, and BDD conventions for contributors.
  Requires 2.3.2. See `mdast-schema-validation-cli-design.md` §Testing strategy.
  - [ ] Describe fixture naming and placement conventions.
  - [ ] Describe when to use unit tests, BDD tests, and snapshots.
  - [ ] Document how to refresh snapshots intentionally.

### 4.2. Keep repository gates aligned with the implementation

- [ ] 4.2.1. Ensure all validation-related tests and docs pass repository gates.
  Requires 1.3.2, 2.3.2, and 4.1.2.
  - [ ] Pass `make check-fmt`.
  - [ ] Pass `make lint`.
  - [ ] Pass `make test`.
  - [ ] Pass `make markdownlint` and `make nixie`.
- [ ] 4.2.2. Add CI coverage for fixtures and snapshot regressions.
  Requires 4.2.1.
  - [ ] Ensure snapshot tests run in CI with deterministic output.
  - [ ] Ensure fixture files are packaged and available to test targets.
  - [ ] Keep failure output concise enough for CI log review.
