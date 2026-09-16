# Developers' guide

This guide records development practices specific to maintaining mdast-check.
Follow the project-wide guidance in `AGENTS.md` first.

## Spelling policy

Run `make spelling` to enforce en-GB-oxendict prose spelling. The gate
regenerates `typos.toml` from the live shared dictionary and the
`typos.local.toml` overlay on every run, so `typos.toml` is never drift checked
in CI. Put narrow repository-specific exceptions in `typos.local.toml`; never
edit generated entries by hand.
