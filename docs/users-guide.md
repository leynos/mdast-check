# User's guide

`mdast-check` validates Markdown documents against schemas. It provides a
stable command-line interface suitable for local development and continuous
integration (CI) pipelines.

## Installation

Build from source using Cargo:

```sh
cargo build --release
```

The compiled binary is placed at `target/release/mdast_check`.

## Quick start

Validate a Markdown document against a JSON Schema:

```sh
mdast-check validate --document docs/example.md --schema schema.json
```

A successful invocation returns exit code `0`. When validation fails the tool
returns a non-zero exit code — see [Exit codes](#exit-codes) for the full table.

## The `validate` subcommand

`validate` is the primary subcommand. It accepts a Markdown document and a
schema, then checks the document's structure against the schema.

### Usage

```plaintext
mdast-check validate [OPTIONS]
```

### Required arguments

Both `--document` and `--schema` must be supplied either on the command line or
through a configuration file. When a required argument is missing the tool
exits with code `2` (invalid invocation).

### Flags

_Table 1: `validate` subcommand flags._

| Flag                       | Purpose                                                             |
| -------------------------- | ------------------------------------------------------------------- |
| `--document <FILE>`        | Path to the Markdown document to validate.                          |
| `--schema <FILE>`          | Path to the schema file for validation.                             |
| `--schema-format <FORMAT>` | Force schema parsing mode. Values: `json` (default), `custom`.      |
| `--config <FILE>`          | Path to a configuration file. Overrides automatic discovery.        |
| `--format <FORMAT>`        | Output format. Values: `text` (default), `json`.                    |
| `--fail-fast`              | Stop validation after the first error.                              |
| `--max-errors <COUNT>`     | Cap the number of reported errors. Use `0` for unlimited (default). |
| `-q`, `--quiet`            | Suppress non-error output.                                          |

### Examples

Validate with explicit paths:

```sh
mdast-check validate --document README.md --schema article.json
```

Validate in quiet mode (only the exit code signals the result):

```sh
mdast-check validate --document README.md --schema article.json --quiet
```

Stop after the first error:

```sh
mdast-check validate \
  --document README.md \
  --schema article.json \
  --fail-fast
```

Report at most five errors:

```sh
mdast-check validate \
  --document README.md \
  --schema article.json \
  --max-errors 5
```

## Configuration

`mdast-check` supports layered configuration via
[ortho-config](https://crates.io/crates/ortho_config). Values are merged from
multiple sources in the following precedence order (highest first):

1. Command-line arguments
2. Environment variables (`MDAST_CHECK_*`)
3. Configuration file (`.mdast-check.toml`)
4. Application defaults

### Configuration file

When no `--config` flag is provided, `mdast-check` searches for a file named
`.mdast-check.toml` in the current directory and its parent directories. A
minimal configuration file:

```toml
[validate]
document = "docs/example.md"
schema = "schema/article.json"
schema_format = "json"
format = "text"
fail_fast = false
max_errors = 0
quiet = false
```

### Environment variables

Every configuration key can be set through an environment variable with the
`MDAST_CHECK_` prefix and an upper-case snake-case name. For example:

```sh
export MDAST_CHECK_DOCUMENT=docs/example.md
export MDAST_CHECK_SCHEMA=schema/article.json
export MDAST_CHECK_QUIET=true
mdast-check validate
```

## Exit codes

`mdast-check` uses stable process exit codes so that scripts and CI pipelines
can distinguish between different failure modes.

_Table 2: Process exit codes._

| Code | Meaning                                                                         |
| ---- | ------------------------------------------------------------------------------- |
| `0`  | Validation succeeded.                                                           |
| `1`  | Validation failed — the document does not satisfy the schema.                   |
| `2`  | Invalid invocation or configuration (for example, a missing required argument). |
| `3`  | Schema document could not be loaded or parsed.                                  |
| `4`  | Markdown document parsing failed.                                               |
| `5`  | Unexpected internal failure.                                                    |

## Version information

Display the installed version:

```sh
mdast-check --version
```

## Help

Display usage information:

```sh
mdast-check validate --help
```
