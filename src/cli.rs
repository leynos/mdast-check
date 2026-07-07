//! CLI argument definitions for MDAST Check.
//!
//! This module defines the command-line interface using clap's derive macros
//! and supports layered configuration via ortho-config's declarative merging.
//!
//! # Configuration layers (highest to lowest precedence)
//!
//! 1. Command-line arguments
//! 2. Environment variables (`MDAST_CHECK_*`)
//! 3. Configuration file (`.mdast-check.toml`)

use clap::{Parser, Subcommand, ValueEnum};
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
#[derive(Debug, Clone, Deserialize, Serialize, clap::Args)]
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
    #[arg(long, value_enum, default_value_t = SchemaFormat::Json)]
    pub schema_format: SchemaFormat,

    /// Output format for results.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,

    /// Stop validation after the first error.
    #[arg(long, default_value_t = false)]
    pub fail_fast: bool,

    /// Maximum number of errors to report.
    ///
    /// Use `0` for unlimited errors.
    #[arg(long, value_name = "COUNT", default_value_t = 0usize)]
    pub max_errors: usize,

    /// Suppress non-error output.
    #[arg(long, short, default_value_t = false)]
    pub quiet: bool,
}

impl Default for ValidateArgs {
    fn default() -> Self {
        Self {
            document: None,
            schema: None,
            schema_format: SchemaFormat::Json,
            format: OutputFormat::Text,
            fail_fast: false,
            max_errors: 0,
            quiet: false,
        }
    }
}

impl ValidateArgs {
    /// Overlays non-`None` fields from `overrides` onto `self`.
    ///
    /// Boolean fields (`fail_fast`, `quiet`) are always taken from `overrides`
    /// since they have unambiguous defaults and should not persist from lower
    /// layers when explicitly set at a higher layer.
    #[must_use]
    pub fn overlay(mut self, overrides: &Self) -> Self {
        if overrides.document.is_some() {
            self.document.clone_from(&overrides.document);
        }
        if overrides.schema.is_some() {
            self.schema.clone_from(&overrides.schema);
        }
        self.schema_format = overrides.schema_format;
        self.format = overrides.format;
        self.fail_fast = overrides.fail_fast;
        self.max_errors = overrides.max_errors;
        self.quiet = overrides.quiet;

        self
    }
}

/// MDAST Check: Validate Markdown documents against schemas.
#[derive(Debug, Clone, Parser)]
#[command(name = "mdast-check", version, about, long_about = None)]
pub struct Cli {
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

/// Merges CLI arguments with defaults using typed overlay.
///
/// # Errors
///
/// Returns [`MdastCheckError::InternalFailure`] if merging fails.
pub fn merge_validate_args(cli_args: &ValidateArgs) -> Result<ValidateArgs, MdastCheckError> {
    // Defaults are the lowest layer; CLI is the highest.
    let merged = ValidateArgs::default().overlay(cli_args);
    Ok(merged)
}

/// Validated paths extracted from `ValidateArgs`.
///
/// This struct provides named access to the required document and schema paths
/// after validation, improving readability at call sites.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedPaths {
    /// Path to the Markdown document to validate.
    pub document: PathBuf,
    /// Path to the schema file for validation.
    pub schema: PathBuf,
}

/// Validates that required fields are present and returns them.
///
/// # Errors
///
/// Returns [`MdastCheckError::InvalidInvocation`] if required fields are missing.
///
/// # Examples
///
/// ```
/// use mdast_check::cli::{ValidateArgs, ValidatedPaths, validate_required_fields};
/// use std::path::PathBuf;
///
/// let args = ValidateArgs {
///     document: Some(PathBuf::from("doc.md")),
///     schema: Some(PathBuf::from("schema.json")),
///     ..Default::default()
/// };
/// let paths = validate_required_fields(&args).unwrap();
/// assert_eq!(paths.document, PathBuf::from("doc.md"));
/// assert_eq!(paths.schema, PathBuf::from("schema.json"));
/// ```
pub fn validate_required_fields(args: &ValidateArgs) -> Result<ValidatedPaths, MdastCheckError> {
    let document = args
        .document
        .clone()
        .ok_or_else(|| MdastCheckError::InvalidInvocation {
            message: "missing required argument: --document".into(),
        })?;

    let schema = args
        .schema
        .clone()
        .ok_or_else(|| MdastCheckError::InvalidInvocation {
            message: "missing required argument: --schema".into(),
        })?;

    Ok(ValidatedPaths { document, schema })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_validate_args_has_expected_defaults() {
        let args = ValidateArgs::default();
        assert!(args.document.is_none());
        assert!(args.schema.is_none());
        assert_eq!(args.schema_format, SchemaFormat::Json);
        assert_eq!(args.format, OutputFormat::Text);
        assert!(!args.fail_fast);
        assert_eq!(args.max_errors, 0);
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
        let paths =
            validate_required_fields(&args).expect("should succeed when both fields are present");
        assert_eq!(paths.document, PathBuf::from("doc.md"));
        assert_eq!(paths.schema, PathBuf::from("schema.json"));
    }

    #[test]
    fn overlay_applies_document_when_present() {
        let defaults = ValidateArgs::default();
        let cli_args = ValidateArgs {
            document: Some(PathBuf::from("cli.md")),
            ..Default::default()
        };

        let merged = defaults.overlay(&cli_args);

        assert_eq!(
            merged.document.as_deref(),
            Some(std::path::Path::new("cli.md"))
        );
        // Other fields retain defaults.
        assert!(merged.schema.is_none());
        assert_eq!(merged.schema_format, SchemaFormat::Json);
    }

    #[test]
    fn overlay_applies_schema_when_present() {
        let defaults = ValidateArgs::default();
        let cli_args = ValidateArgs {
            schema: Some(PathBuf::from("cli-schema.json")),
            ..Default::default()
        };

        let merged = defaults.overlay(&cli_args);

        assert_eq!(
            merged.schema.as_deref(),
            Some(std::path::Path::new("cli-schema.json"))
        );
    }

    #[test]
    fn overlay_applies_all_boolean_flags() {
        let defaults = ValidateArgs::default();
        let cli_args = ValidateArgs {
            fail_fast: true,
            quiet: true,
            ..Default::default()
        };

        let merged = defaults.overlay(&cli_args);

        assert!(merged.fail_fast);
        assert!(merged.quiet);
    }

    #[test]
    fn merge_validate_args_combines_defaults_and_cli() {
        let cli_args = ValidateArgs {
            document: Some(PathBuf::from("doc.md")),
            max_errors: 10,
            ..Default::default()
        };

        let merged = merge_validate_args(&cli_args).expect("merge should succeed");

        assert_eq!(
            merged.document.as_deref(),
            Some(std::path::Path::new("doc.md"))
        );
        assert_eq!(merged.max_errors, 10);
        // Defaults applied.
        assert_eq!(merged.schema_format, SchemaFormat::Json);
        assert_eq!(merged.format, OutputFormat::Text);
    }
}
