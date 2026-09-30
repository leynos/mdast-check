# Developers' guide

This guide records development practices specific to maintaining mdast-check.
Follow the project-wide guidance in `AGENTS.md` first.

## Spelling policy

Run `make spelling` to enforce en-GB-oxendict prose spelling. The gate
regenerates `typos.toml` from the live shared dictionary and the
`typos.local.toml` overlay on every run, so `typos.toml` is never drift checked
in continuous integration (CI). Put narrow repository-specific exceptions in
`typos.local.toml`; never edit generated entries by hand.

`TYPOS_CONFIG_BUILDER_VERSION` in the `Makefile` pins the
`typos-config-builder` release the gate runs (currently `v0.1.3`). Raise it
together with the regenerated `typos.toml`, never on its own.

## Coverage ownership

The trunk owns both persistent coverage outputs. On a push to `main`,
`.github/workflows/coverage-main.yml` measures coverage, writes the ratchet
baseline, and uploads the report to CodeScene. Pull-request CI measures the
same selection only to compare it with that baseline: it archives no report,
never calls CodeScene, and never receives `CS_ACCESS_TOKEN`. The call is what
moves to the trunk, not the archive: the uploader pins the `cs-coverage`
archive by digest, but the client refuses to run whenever CodeScene's API
changes shape, and on the trunk such a change no longer fails every pull
request.

The publisher never binds the token in an `env` block, because the uploader is
a composite action that would pass a step's environment on to its nested steps.
A check step writes whether the secret is set, the upload runs only when it is
and only for `refs/heads/main`, and the token reaches the uploader solely as its
`access-token` input. Runs share one concurrency group per ref and are never
cancelled, so triggered runs (a push or a dispatch) upload in commit order. A
manual re-run of an older run is an operator action: it republishes that
commit's coverage and baseline until the next push supersedes it.

Two gaps are known and accepted. Merges made by the Dependabot automerge
workflow with `GITHUB_TOKEN` fire no push, so they reach the publisher only
through a dispatch or the next ordinary push. A dispatch that replaces a
pending push uploads the same or a newer commit, but the shared action writes
the baseline only on a push, so the baseline stays one push behind until the
next one.

`make workflow-contracts`, which pull-request CI runs, holds this shape by
running `cv005-contracts check`, the shared contract library in
`leynos/shared-actions` (`packages/cv005-contracts`), from a full commit named
by `CV005_CONTRACTS_REF` in the Makefile. A fix to the rules is therefore a pin
bump. The target needs `uv`, which fetches the Python 3.13 the library runs
under. The repository's parameters are in `.github/cv005.toml`: its
`repository` name and the `[selection]` inputs the baseline measures, which the
publisher's generator must carry and every pull-request lane must match. The
library's own suite proves each rule refuses the shape it exists to refuse, so
this repository keeps no copy of the readers or the refusal cases. It reads
every workflow strictly (a repeated key is an error) and follows local
reusable-workflow calls transitively, so a called workflow cannot reach
CodeScene on a pull request's behalf.
