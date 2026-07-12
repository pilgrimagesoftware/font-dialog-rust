# font-dialog

OS-native font picker dialogs behind one cross-platform Rust API — the same
role `rfd` plays for file/folder dialogs, for fonts instead.

```rust
use font_dialog::FontDialog;

if let Some(selection) = FontDialog::new().show() {
    println!("{} {}pt", selection.family, selection.size);
}
```

## Platform backends

| Platform | Native dialog | Notes |
| --- | --- | --- |
| macOS | `NSFontPanel` / `NSFontManager` | `NSFontPanel` has no built-in "OK" button; this crate adds one and runs the panel modally so `show()` still returns a single confirmed-or-cancelled result. |
| Windows | `ChooseFont` common dialog | Already blocking and modal — a close to direct passthrough. |
| Linux | `gtk4::FontDialog` | Requires the embedding application to have already called `gtk4::init()`. This crate does not call it for you. |

## Verification status

The macOS backend is built and unit-tested on macOS. The Windows and Linux
backends were written against each platform's documented API but have not
been compiled or run on their target platforms yet — that verification
happens in CI (`.github/workflows/ci.yaml`, one job per platform) and via
manual testing before each release. Showing a real native dialog and
interacting with it isn't automatable in a headless test runner, so
dialog-showing behavior itself is always manually verified per platform
before release, not covered by automated tests — automated tests cover the
pure data-translation logic only (`LOGFONTW`/`FontDescription` ↔
`FontSelection`, and `FontDialog`'s builder methods).

## License

MIT
