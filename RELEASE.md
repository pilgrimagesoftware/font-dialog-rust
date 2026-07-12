# Release process

## Summary

1. Trigger the [**Prepare Release**](https://github.com/pilgrimagesoftware/font-dialog-rust/actions/workflows/prepare-release.yaml) workflow.
1. Review and merge the generated PR into `master`.
1. Merging tags the release automatically, which triggers publishing.

## Breakdown

Instead of manually bumping `Cargo.toml` and writing the changelog by hand on a release branch, the
`prepare-release` workflow does it for you:

Trigger the [**Prepare Release**](https://github.com/pilgrimagesoftware/font-dialog-rust/actions/workflows/prepare-release.yaml) workflow (`workflow_dispatch` in the Actions tab). It:

1. Runs `git-cliff --bump` against `master` to determine the next version (e.g. `0.3.0`) from commits
   since the last tag.
2. Updates `Cargo.toml` to that version.
3. Prepends the generated changelog section to `CHANGELOG.md`.
4. Opens a PR from an auto-created `release/0.3.0` branch into `master`.

You review the PR (catch anything that shouldn't ship, fix as needed) — the
[**CI**](https://github.com/pilgrimagesoftware/font-dialog-rust/actions/workflows/ci.yaml) workflow runs
against it — and merge into `master`. Merging a `release/*` branch into `master` triggers the
[**Tag Release**](https://github.com/pilgrimagesoftware/font-dialog-rust/actions/workflows/tag-release.yaml)
workflow, which tags the merge commit `vX.Y.Z`.

The tag push triggers the [**Release**](https://github.com/pilgrimagesoftware/font-dialog-rust/actions/workflows/release.yaml)
workflow, which tests, builds, publishes to crates.io, generates the changelog scoped to that tag, and
attaches it to the GitHub Release.

## Triggering the run

```sh
gh workflow run prepare-release.yaml
```

Or trigger it directly from the [Prepare Release workflow page](https://github.com/pilgrimagesoftware/font-dialog-rust/actions/workflows/prepare-release.yaml).
