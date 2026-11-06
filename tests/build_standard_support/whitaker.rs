//! The Whitaker half of the Makefile reader. Whitaker runs on its installer-managed toolchain, which
//! must not inherit the repository's development flags, so a direct Whitaker command is held to the
//! opposite of the standard: it assigns the warning policy alone.

use super::{
    config::{Flags, Pin, Problems},
    make::{Assignment, Host, MakeRunner, development_problem, make_commands},
};

/// Returns whether a word is a leading `NAME=value` environment assignment.
fn is_assignment(word: &str) -> bool {
    word.split_once('=').is_some_and(|(name, _)| {
        !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    })
}

/// Returns whether a command runs Whitaker directly: the executable is `whitaker`, past any
/// leading environment assignments, so a Cargo test filter that merely names it does not count.
///
/// ```text
/// runs_whitaker("RUSTFLAGS=\"-D warnings\" whitaker --all") == true
/// runs_whitaker("RUSTFLAGS=\"-D warnings\" cargo test whitaker") == false
/// ```
pub(super) fn runs_whitaker(command: &str) -> bool {
    command_words(command)
        .first()
        .is_some_and(|word| is_whitaker(word))
}

/// Returns whether a command runs the Whitaker lint suite: it runs Whitaker directly and is not
/// a probe, so `whitaker --version` and `whitaker --help` do not count.
///
/// ```text
/// runs_whitaker_suite("RUSTFLAGS=\"-D warnings\" whitaker --all") == true
/// runs_whitaker_suite("RUSTFLAGS=\"-D warnings\" whitaker --version") == false
/// ```
pub(super) fn runs_whitaker_suite(command: &str) -> bool {
    let words = command_words(command);
    words.first().is_some_and(|word| is_whitaker(word))
        && !words
            .iter()
            .skip(1)
            .any(|word| matches!(*word, "--version" | "-V" | "--help" | "-h"))
}

/// Returns whether an executable path names Whitaker.
fn is_whitaker(word: &str) -> bool {
    word.rsplit('/').next() == Some("whitaker")
}

/// Returns the words of a command past its leading environment assignments.
fn command_words(command: &str) -> Vec<&str> {
    // A quoted value may hold spaces (`RUSTFLAGS="-D warnings"`), so keep only the unquoted spans,
    // which are every other piece when the command is split at its quotation marks.
    command
        .split('"')
        .step_by(2)
        .flat_map(str::split_whitespace)
        .skip_while(|word| is_assignment(word))
        .collect::<Vec<_>>()
        .into_iter()
        .skip_while(skip_env())
        .collect()
}

/// Skips an `env` wrapper: the word itself, its `-u NAME` removals and its own assignments.
fn skip_env() -> impl FnMut(&&str) -> bool {
    let mut after_removal_flag = false;
    let mut seen_env = false;
    move |word| {
        if !seen_env {
            seen_env = *word == "env";
            return seen_env;
        }
        if after_removal_flag {
            after_removal_flag = false;
            return true;
        }
        if *word == "-u" {
            after_removal_flag = true;
            return true;
        }
        is_assignment(word)
    }
}

/// Returns the complaint about a direct Whitaker command, if any: it must assign the warning
/// policy and nothing of the caller's `RUSTFLAGS`, and must carry neither the frontend flag nor the
/// linker flag.
pub(super) fn isolated_problem(
    target: &str,
    host: Host,
    flags: &Flags,
    inherits: bool,
) -> Option<String> {
    let on = host.make_value();
    if inherits {
        return Some(format!(
            "`make {target}` on {on} lets Whitaker inherit the caller's RUSTFLAGS"
        ));
    }
    if flags.names_threads() || flags.names_linker() {
        return Some(format!(
            "`make {target}` on {on} gives Whitaker the development flags: {flags:?}"
        ));
    }
    (!flags.is_warning_policy_alone()).then(|| {
        format!("`make {target}` on {on} runs Whitaker without -D warnings alone: {flags:?}")
    })
}

/// Returns the complaint about one development command, if any. A direct Whitaker command is held to
/// [`isolated_problem`]; every other command to the standard's flags, as [`development_problem`] reads
/// them.
pub fn command_problem(
    target: &str,
    host: Host,
    pin: Pin,
    command: &(String, Assignment),
) -> Option<String> {
    let (text, assignment) = command;
    match assignment {
        Assignment::Flags(flags, inherits) if runs_whitaker(text) => {
            // Cargo reads the encoded flags in preference to `RUSTFLAGS`, so an inherited value
            // would displace the warning policy the route assigns.
            let clears_encoded = text
                .split_whitespace()
                .collect::<Vec<_>>()
                .windows(2)
                .any(|pair| pair == ["-u", "CARGO_ENCODED_RUSTFLAGS"]);
            if !clears_encoded {
                return Some(format!(
                    "`make {target}` on {} lets Whitaker inherit the caller's CARGO_ENCODED_RUSTFLAGS: {text}",
                    host.make_value()
                ));
            }
            isolated_problem(target, host, flags, *inherits)
        }
        other => development_problem(target, host, pin, other),
    }
}

/// Returns the complaint when the `lint` target runs no isolated Whitaker command: the suite is the
/// lint gate's second half, and a bare, inheriting or missing Whitaker route must not pass for it.
///
/// # Errors
///
/// Returns the reason when the `lint` target is not defined or unreadable.
pub fn whitaker_route_problems(runner: MakeRunner, host: Host) -> Result<Problems, String> {
    let commands = make_commands(runner, "lint", host)?;
    let isolated = commands.iter().any(|(text, assignment)| {
        runs_whitaker_suite(text) && matches!(assignment, Assignment::Flags(..))
    });
    let mut problems = Problems::new();
    if !isolated {
        problems.push(format!(
            "`make lint` on {} runs no Whitaker suite command that assigns RUSTFLAGS",
            host.make_value()
        ));
    }
    Ok(problems)
}
