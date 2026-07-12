# Architecture

`font-dialog` wraps three unrelated native font-picker APIs behind one Rust surface: NSFontPanel/
NSFontManager on macOS (`objc2`, `objc2-app-kit`), `ChooseFont` on Windows (`windows` crate), and GTK4's
`FontDialog` on Linux (`gtk4`). Platform code lives behind `cfg(target_os = ...)` and only the target
platform's dependency compiles — expect to only be able to build and test the platform you're on.

# Conventions

- Commits follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) (`feat`, `fix`,
  `perf`, `refactor`, `doc`, `chore`, etc.) — CI rejects non-conforming messages and `cliff.toml` uses the
  prefix to route entries into `CHANGELOG.md`. See [CONTRIBUTING.md](CONTRIBUTING.md).
- Branching is Git Flow: feature work branches off `develop` as `feature/<name>` and merges back into
  `develop`. Only automated `release/*` branches (created by the Prepare Release workflow) merge into
  `master`. See [RELEASE.md](RELEASE.md) for the full release flow and how `master`/`develop` are kept in
  sync.
- Releases are version-bumped and changelogged automatically via `git-cliff` — don't hand-edit the version
  in `Cargo.toml` or prepend `CHANGELOG.md` entries manually; trigger the Prepare Release workflow instead.

# Testing

- `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and
  `cargo test --all-features` must all pass before a PR merges; run them locally first.
- Linux CI installs `libgtk-4-dev` before building — GTK4 is a system dependency, not vendored.
