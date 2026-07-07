//! Error types and exit codes for MDAST Check.
//!
//! This module defines the error taxonomy and exit-code mapping used throughout
//! the application. Errors are categorised into semantic types that map to
//! stable exit codes for scripting reliability.

use std::process::ExitCode as StdExitCode;

/// Exit codes used by the MDAST Check CLI.
///
/// These codes are stable and suitable for scripting. Each code represents a
/// distinct category of outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub struct ExitCode(u8);

impl ExitCode {
    /// Successful execution (exit code 0).
    pub const SUCCESS: Self = Self(0);
    /// Document failed schema validation (exit code 1).
    pub const VALIDATION_FAILED: Self = Self(1);
    /// Invalid command-line invocation (exit code 2).
    pub const INVALID_INVOCATION: Self = Self(2);
    /// Schema could not be loaded or parsed (exit code 3).
    pub const SCHEMA_FAILURE: Self = Self(3);
    /// Markdown document parsing failed (exit code 4).
    pub const PARSE_FAILURE: Self = Self(4);
    /// Unexpected internal error (exit code 5).
    pub const INTERNAL_FAILURE: Self = Self(5);

    /// Returns the numeric exit code.
    #[inline]
    #[must_use]
    pub const fn code(self) -> u8 {
        self.0
    }
}

impl From<ExitCode> for StdExitCode {
    fn from(value: ExitCode) -> Self {
        Self::from(value.0)
    }
}

/// Errors that can occur during MDAST Check execution.
///
/// Each variant maps to a specific exit code via [`ExitCode::from_error`].
#[derive(Debug, Clone, thiserror::Error)]
pub enum MdastCheckError {
    /// Invalid command-line arguments or invocation.
    #[error("Invalid invocation: {message}")]
    InvalidInvocation {
        /// Human-readable error message.
        message: String,
    },

    /// Schema could not be loaded or parsed.
    #[error("Schema failure: {message}")]
    SchemaFailure {
        /// Human-readable error message.
        message: String,
    },

    /// Markdown document could not be parsed.
    #[error("Parse failure: {message}")]
    ParseFailure {
        /// Human-readable error message.
        message: String,
    },

    /// Document failed schema validation.
    #[error("Validation failed: {message}")]
    ValidationFailed {
        /// Human-readable error message.
        message: String,
    },

    /// Unexpected internal error.
    #[error("Internal failure: {message}")]
    InternalFailure {
        /// Human-readable error message.
        message: String,
    },
}

impl ExitCode {
    /// Maps an error to its corresponding exit code.
    ///
    /// # Examples
    ///
    /// ```
    /// use mdast_check::error::{ExitCode, MdastCheckError};
    ///
    /// let error = MdastCheckError::InvalidInvocation {
    ///     message: "missing required argument".into(),
    /// };
    /// assert_eq!(ExitCode::from_error(&error), ExitCode::INVALID_INVOCATION);
    /// ```
    pub const fn from_error(error: &MdastCheckError) -> Self {
        match error {
            MdastCheckError::InvalidInvocation { .. } => Self::INVALID_INVOCATION,
            MdastCheckError::SchemaFailure { .. } => Self::SCHEMA_FAILURE,
            MdastCheckError::ParseFailure { .. } => Self::PARSE_FAILURE,
            MdastCheckError::ValidationFailed { .. } => Self::VALIDATION_FAILED,
            MdastCheckError::InternalFailure { .. } => Self::INTERNAL_FAILURE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn exit_code_success_is_zero() {
        assert_eq!(ExitCode::SUCCESS.code(), 0);
    }

    /// Verifies that each exit-code constant holds its documented numeric value.
    #[rstest]
    #[case::success(ExitCode::SUCCESS, 0)]
    #[case::validation_failed(ExitCode::VALIDATION_FAILED, 1)]
    #[case::invalid_invocation(ExitCode::INVALID_INVOCATION, 2)]
    #[case::schema_failure(ExitCode::SCHEMA_FAILURE, 3)]
    #[case::parse_failure(ExitCode::PARSE_FAILURE, 4)]
    #[case::internal_failure(ExitCode::INTERNAL_FAILURE, 5)]
    fn exit_code_has_expected_value(#[case] exit_code: ExitCode, #[case] expected: u8) {
        assert_eq!(exit_code.code(), expected);
    }

    /// Verifies that each error variant maps to its corresponding exit code.
    #[rstest]
    #[case::invalid_invocation(
        MdastCheckError::InvalidInvocation { message: "test".into() },
        ExitCode::INVALID_INVOCATION,
    )]
    #[case::schema_failure(
        MdastCheckError::SchemaFailure { message: "test".into() },
        ExitCode::SCHEMA_FAILURE,
    )]
    #[case::parse_failure(
        MdastCheckError::ParseFailure { message: "test".into() },
        ExitCode::PARSE_FAILURE,
    )]
    #[case::validation_failed(
        MdastCheckError::ValidationFailed { message: "test".into() },
        ExitCode::VALIDATION_FAILED,
    )]
    #[case::internal_failure(
        MdastCheckError::InternalFailure { message: "test".into() },
        ExitCode::INTERNAL_FAILURE,
    )]
    fn from_error_maps_variant_to_exit_code(
        #[case] error: MdastCheckError,
        #[case] expected: ExitCode,
    ) {
        assert_eq!(ExitCode::from_error(&error), expected);
    }

    /// Verifies that exit codes are ordered by increasing severity.
    #[rstest]
    #[case::success_lt_validation(ExitCode::SUCCESS, ExitCode::VALIDATION_FAILED)]
    #[case::validation_lt_invocation(ExitCode::VALIDATION_FAILED, ExitCode::INVALID_INVOCATION)]
    #[case::invocation_lt_schema(ExitCode::INVALID_INVOCATION, ExitCode::SCHEMA_FAILURE)]
    #[case::schema_lt_parse(ExitCode::SCHEMA_FAILURE, ExitCode::PARSE_FAILURE)]
    #[case::parse_lt_internal(ExitCode::PARSE_FAILURE, ExitCode::INTERNAL_FAILURE)]
    fn exit_codes_are_ordered_by_severity(#[case] lower: ExitCode, #[case] higher: ExitCode) {
        assert!(lower.code() < higher.code());
    }
}
