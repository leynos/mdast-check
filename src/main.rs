//! MDAST Check application entry point.

use clap::Parser;
use mdast_check::cli::{
    Cli, Commands, ValidateArgs, merge_validate_args, validate_required_fields,
};
use mdast_check::error::{ExitCode, MdastCheckError};
use std::process::ExitCode as StdExitCode;

/// Runs the validate subcommand.
///
/// # Errors
///
/// Returns an exit code indicating the result of validation.
fn run_validate(args: &ValidateArgs) -> Result<ExitCode, MdastCheckError> {
    let merged_args = merge_validate_args(args)?;
    let paths = validate_required_fields(&merged_args)?;

    // TODO(#roadmap-1.2): Implement actual Markdown parsing and schema validation.
    // For this milestone, we only validate the command surface and configuration.
    // A successful invocation returns SUCCESS (0); actual validation occurs in 1.3.2.

    // Output is suppressed in quiet mode, but validation logic (when implemented)
    // will still run and return the appropriate exit code.
    if !merged_args.quiet {
        // Non-quiet mode: print a brief status message.
        #[expect(
            clippy::print_stdout,
            reason = "CLI status output is intentional; quiet mode available for scripting"
        )]
        {
            println!(
                "Validating {} against {}...",
                paths.document.display(),
                paths.schema.display()
            );
        }
    }

    Ok(ExitCode::SUCCESS)
}

/// Application entry point.
#[expect(
    clippy::print_stderr,
    reason = "CLI error and warning output to stderr is standard practice"
)]
fn main() -> StdExitCode {
    // Install colour-eyre error handler for enhanced error output.
    if let Err(e) = color_eyre::install() {
        // If colour_eyre is already installed or fails, continue without colours.
        eprintln!("Warning: Failed to install colour error handler: {e}");
    }

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Validate(args) => run_validate(&args),
    };

    match result {
        Ok(exit_code) => exit_code.into(),
        Err(error) => {
            let exit_code = ExitCode::from_error(&error);
            eprintln!("Error: {error}");
            exit_code.into()
        }
    }
}
