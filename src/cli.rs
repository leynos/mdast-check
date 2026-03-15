//! CLI argument definitions for MDAST Check.
//!
//! This module defines the command-line interface using clap's derive macros
//! and ortho-config's declarative merging for layered configuration support.
//!
//! # Configuration layers (highest to lowest precedence)
//!
//! 1. Command-line arguments
//! 2. Environment variables (`MDAST_CHECK_*`)
//! 3. Configuration file (`.mdast-check.toml`)

use clap::{Parser, Subcommand, ValueEnum};
use ortho_config::declarative::MergeComposer;
use ortho_config::{OrthoConfig, serde_json};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::MdastCheckError;

/// Schema format options for validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default, Serialize, Deserialize)]
pub enum SchemaFormat {
    /// JSON Schema format.
    #[default]
    Json,
    /// Custom schema format (future use).
    Custom,
}

/// Output format options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default, Serialize, Deserialize)]
pub enum OutputFormat {
    /// Human-readable text output.
    #[default]
    Text,
    /// JSON output for machine consumption.
    Json,
}

/// Arguments for the `validate` subcommand.
///
/// These fields support layered configuration via ortho-config, with CLI
/// arguments taking precedence over environment variables and config files.
#[derive(Debug, Clone, Deserialize, Serialize, OrthoConfig, clap::Args)]
#[ortho_config(namespace = "validate")]
pub struct ValidateArgs {
    /// Path to the Markdown document to validate.
    ///
    /// Required unless provided via configuration file.
    #[arg(long, value_name = "FILE")]
    pub document: Option<PathBuf>,

    /// Path to the schema file for validation.
    ///
    /// Required unless provided via configuration file.
    #[arg(long, value_name = "FILE")]
    pub schema: Option<PathBuf>,

    /// Format of the schema file.
    ///
    /// If not specified, defaults to `json`.
    #[arg(long, value_enum)]
    pub schema_format: Option<SchemaFormat>,

    /// Path to the configuration file.
    ///
    /// If not specified, searches for `.mdast-check.toml` in the current
    /// directory and parent directories.
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Output format for results.
    ///
    /// If not specified, defaults to `text`.
    #[arg(long, value_enum)]
    pub format: Option<OutputFormat>,

    /// Stop validation after the first error.
    #[arg(long, default_value_t = false)]
    pub fail_fast: bool,

    /// Maximum number of errors to report.
    ///
    /// Use `0` for unlimited errors.
    #[arg(long, value_name = "COUNT")]
    pub max_errors: Option<usize>,

    /// Suppress non-error output.
    #[arg(long, short, default_value_t = false)]
    pub quiet: bool,
}

impl Default for ValidateArgs {
    fn default() -> Self {
        Self {
            document: None,
            schema: None,
            schema_format: Some(SchemaFormat::Json),
            config: None,
            format: Some(OutputFormat::Text),
            fail_fast: false,
            max_errors: Some(0),
            quiet: false,
        }
    }
}

/// MDAST Check: Validate Markdown documents against schemas.
#[derive(Debug, Clone, Parser)]
#[command(name = "mdast-check", version, about, long_about = None)]
pub struct Cli {
    /// Path to the configuration file.
    ///
    /// Overrides automatic discovery of `.mdast-check.toml`.
    #[arg(long, global = true, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Subcommand to execute.
    #[command(subcommand)]
    pub command: Commands,
}

/// Available subcommands.
#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    /// Validate a Markdown document against a schema.
    Validate(ValidateArgs),
}

/// Merges CLI arguments with configuration layers using declarative merging.
///
/// # Errors
///
/// Returns [`MdastCheckError::InternalFailure`] if merging fails.
pub fn merge_validate_args(cli_args: &ValidateArgs) -> Result<ValidateArgs, MdastCheckError> {
    let defaults = serde_json::to_value(ValidateArgs::default()).map_err(|e| {
        MdastCheckError::InternalFailure {
            message: format!("Failed to serialize defaults: {e}"),
        }
    })?;

    let cli_layer =
        serde_json::to_value(cli_args).map_err(|e| MdastCheckError::InternalFailure {
            message: format!("Failed to serialize CLI args: {e}"),
        })?;

    let mut composer = MergeComposer::new();
    composer.push_defaults(defaults);
    // TODO: Add file and environment layers when discovery is implemented.
    composer.push_cli(cli_layer);

    let merged = ValidateArgs::merge_from_layers(composer.layers()).map_err(|e| {
        MdastCheckError::InternalFailure {
            message: format!("Failed to merge configuration layers: {e}"),
        }
    })?;

    Ok(merged)
}

/// Validates that required fields are present and returns them.
///
/// Returns borrowed references to the document and schema paths on success,
/// eliminating the need for callers to use `unwrap()` after validation.
///
/// # Errors
///
/// Returns [`MdastCheckError::InvalidInvocation`] if required fields are missing.
///
/// # Examples
///
/// ```
/// use mdast_check::cli::{ValidateArgs, validate_required_fields};
/// use std::path::PathBuf;
///
/// let args = ValidateArgs {
///     document: Some(PathBuf::from("doc.md")),
///     schema: Some(PathBuf::from("schema.json")),
///     ..Default::default()
/// };
/// let (doc, schema) = validate_required_fields(&args).unwrap();
/// assert_eq!(doc, &PathBuf::from("doc.md"));
/// assert_eq!(schema, &PathBuf::from("schema.json"));
/// ```
pub fn validate_required_fields(
    args: &ValidateArgs,
) -> Result<(&PathBuf, &PathBuf), MdastCheckError> {
    let document = args
        .document
        .as_ref()
        .ok_or_else(|| MdastCheckError::InvalidInvocation {
            message: "missing required argument: --document".into(),
        })?;

    let schema = args
        .schema
        .as_ref()
        .ok_or_else(|| MdastCheckError::InvalidInvocation {
            message: "missing required argument: --schema".into(),
        })?;

    Ok((document, schema))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn default_validate_args_has_expected_defaults() {
        let args = ValidateArgs::default();
        assert!(args.document.is_none());
        assert!(args.schema.is_none());
        assert_eq!(args.schema_format, Some(SchemaFormat::Json));
        assert_eq!(args.format, Some(OutputFormat::Text));
        assert!(!args.fail_fast);
        assert_eq!(args.max_errors, Some(0));
        assert!(!args.quiet);
    }

    #[test]
    fn validate_required_fields_missing_document() {
        let args = ValidateArgs {
            schema: Some(PathBuf::from("schema.json")),
            ..Default::default()
        };
        let error =
            validate_required_fields(&args).expect_err("should fail when document is missing");
        assert!(matches!(error, MdastCheckError::InvalidInvocation { .. }));
    }

    #[test]
    fn validate_required_fields_missing_schema() {
        let args = ValidateArgs {
            document: Some(PathBuf::from("doc.md")),
            ..Default::default()
        };
        let error =
            validate_required_fields(&args).expect_err("should fail when schema is missing");
        assert!(matches!(error, MdastCheckError::InvalidInvocation { .. }));
    }

    #[test]
    fn validate_required_fields_both_present() {
        let args = ValidateArgs {
            document: Some(PathBuf::from("doc.md")),
            schema: Some(PathBuf::from("schema.json")),
            ..Default::default()
        };
        let (doc, schema) =
            validate_required_fields(&args).expect("should succeed when both fields are present");
        assert_eq!(doc, &PathBuf::from("doc.md"));
        assert_eq!(schema, &PathBuf::from("schema.json"));
    }

    /// Verifies that CLI values override defaults via `MergeComposer`.
    #[rstest]
    #[case::cli_overrides_document(
        serde_json::json!({"document": "cli.md"}),
        "cli.md",
    )]
    #[case::cli_overrides_schema(
        serde_json::json!({"schema": "cli-schema.json"}),
        "cli-schema.json",
    )]
    fn cli_layer_overrides_defaults(
        #[case] cli_value: serde_json::Value,
        #[case] expected_path: &str,
    ) {
        let defaults =
            serde_json::to_value(ValidateArgs::default()).expect("should serialize defaults");
        let mut composer = MergeComposer::new();
        composer.push_defaults(defaults);
        composer.push_cli(cli_value);

        let merged = ValidateArgs::merge_from_layers(composer.layers())
            .expect("should merge layers successfully");

        // Check that the CLI-supplied value won.
        let has_doc = merged
            .document
            .as_ref()
            .is_some_and(|d| d.to_str() == Some(expected_path));
        let has_schema = merged
            .schema
            .as_ref()
            .is_some_and(|s| s.to_str() == Some(expected_path));

        assert!(has_doc || has_schema, "CLI value should override defaults");
    }

    /// Verifies that defaults survive when no CLI override is provided.
    #[test]
    fn defaults_survive_without_cli_override() {
        let defaults =
            serde_json::to_value(ValidateArgs::default()).expect("should serialize defaults");
        let mut composer = MergeComposer::new();
        composer.push_defaults(defaults);
        // Push an empty CLI layer — no overrides.
        composer.push_cli(serde_json::json!({}));

        let merged = ValidateArgs::merge_from_layers(composer.layers())
            .expect("should merge layers successfully");

        assert_eq!(merged.schema_format, Some(SchemaFormat::Json));
        assert_eq!(merged.format, Some(OutputFormat::Text));
        assert_eq!(merged.max_errors, Some(0));
        assert!(!merged.fail_fast);
        assert!(!merged.quiet);
    }

    /// Verifies that `merge_validate_args` produces defaults when called with
    /// bare `ValidateArgs`.
    #[test]
    fn merge_validate_args_applies_defaults() {
        let args = ValidateArgs::default();
        let merged = merge_validate_args(&args).expect("should merge successfully");

        assert_eq!(merged.schema_format, Some(SchemaFormat::Json));
        assert_eq!(merged.format, Some(OutputFormat::Text));
        assert_eq!(merged.max_errors, Some(0));
    }
}
