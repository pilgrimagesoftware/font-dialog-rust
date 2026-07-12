# Contributing code

## Conventional Commits

To enhance our development workflow, enable automated changelog generation, and pave the way for Continuous Delivery,
the `font-dialog-rust` project has adopted the [Conventional Commits
standard](https://www.conventionalcommits.org/en/v1.0.0/) for all commit messages.

All commits to this repository **MUST** adhere to the Conventional Commits standard. Commits not adhering to this
standard will cause the CI build to fail. PRs will not be merged if they include non-conventional commits.

Common types used in this project:

- `feat` — a new feature, surfaced in the changelog under "Added"
- `fix` — a bug fix, surfaced under "Fixed"
- `perf` — a performance improvement, surfaced under "Performance"
- `refactor` — a code change that neither fixes a bug nor adds a feature, surfaced under "Changed"
- `doc` — documentation only changes, surfaced under "Documentation"
- `chore`, `ci`, `test`, `style`, `build` — internal maintenance, excluded from the changelog

## Development

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

Run these before opening a PR — CI enforces all three. This crate targets macOS, Windows, and Linux; if you
can only build on one platform, say so in the PR and a maintainer will validate the others via the PR
Validation workflow.

## Pull requests

- Keep PRs focused on a single change
- Reference any related issue in the PR description
- CI must pass before merge
