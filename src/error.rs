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

    #[test]
    fn exit_code_success_is_zero() {
        assert_eq!(ExitCode::SUCCESS.code(), 0);
    }

    #[test]
    fn exit_code_ordering() {
        assert!(ExitCode::SUCCESS.code() < ExitCode::VALIDATION_FAILED.code());
        assert!(ExitCode::VALIDATION_FAILED.code() < ExitCode::INVALID_INVOCATION.code());
        assert!(ExitCode::INVALID_INVOCATION.code() < ExitCode::SCHEMA_FAILURE.code());
        assert!(ExitCode::SCHEMA_FAILURE.code() < ExitCode::PARSE_FAILURE.code());
        assert!(ExitCode::PARSE_FAILURE.code() < ExitCode::INTERNAL_FAILURE.code());
    }
}
