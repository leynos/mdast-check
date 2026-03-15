//! MDAST Check: A CLI tool for validating Markdown documents against schemas.
//!
//! This crate provides the core library functionality for the `mdast-check` CLI
//! tool. It supports validating Markdown Abstract Syntax Tree (MDAST) documents
//! against JSON Schema or equivalent schemas.
//!
//! # Exit codes
//!
//! The tool uses stable exit codes to indicate different outcomes:
//!
//! | Code | Constant           | Meaning                          |
//! |------|--------------------|----------------------------------|
//! | 0    | `SUCCESS`          | Validation passed                |
//! | 1    | `VALIDATION_FAILED`| Document failed schema validation|
//! | 2    | `INVALID_INVOCATION`| Invalid command-line arguments  |
//! | 3    | `SCHEMA_FAILURE`   | Schema could not be loaded/parsed|
//! | 4    | `PARSE_FAILURE`    | Markdown document parsing failed |
//! | 5    | `INTERNAL_FAILURE` | Unexpected internal error        |

pub mod cli;
pub mod error;
