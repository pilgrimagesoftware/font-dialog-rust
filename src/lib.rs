//! OS-native font picker dialogs behind one cross-platform API.
//!
//! No existing Rust crate wraps the platform-native font dialog the way
//! `rfd` wraps file/folder dialogs. `font-dialog` fills that gap: one
//! blocking call, one result type, backed by the real native dialog on each
//! platform — `NSFontPanel` on macOS, the `ChooseFont` common dialog on
//! Windows, and `gtk4::FontDialog` on Linux.
//!
//! # Examples
//!
//! ```no_run
//! use font_dialog::FontDialog;
//!
//! if let Some(selection) = FontDialog::new().show() {
//!     println!("{} {}pt", selection.family, selection.size);
//! }
//! ```
//!
//! # Platform notes
//!
//! - **Linux**: the GTK4 backend requires the embedding application to have
//!   already called `gtk4::init()` before [`FontDialog::show`] is invoked.
//!   This crate does not call it on the caller's behalf, since doing so
//!   twice (once here, once in an app that also uses GTK4 directly) is
//!   itself an error in some GTK4 versions.
//! - **macOS**: `NSFontPanel` is natively a persistent, non-modal floating
//!   panel with no built-in "OK" button. This crate's backend adds a small
//!   "Done" affordance and runs the panel modally so `show()` can still
//!   return a single confirmed-or-cancelled result, matching this crate's
//!   API on every platform.

#![warn(clippy::pedantic, clippy::nursery, missing_docs, rust_2018_idioms)]
#![deny(unsafe_op_in_unsafe_fn)]

mod backend;

/// A font selection returned by [`FontDialog::show`]: the family name, point
/// size, and bold/italic style the user chose.
///
/// This is the intersection of what all three native dialogs expose, not the
/// union of every platform-specific styling option (e.g. Windows' strikeout
/// and underline flags aren't represented here).
#[derive(Debug, Clone, PartialEq)]
pub struct FontSelection {
    /// The chosen font family name (e.g. `"Helvetica Neue"`).
    pub family: String,
    /// The chosen point size.
    pub size: f32,
    /// Whether the chosen style includes bold.
    pub bold: bool,
    /// Whether the chosen style includes italic.
    pub italic: bool,
}

/// Presents the platform's native font-picker dialog.
///
/// Construct with [`FontDialog::new`], optionally set an initial selection
/// with [`FontDialog::initial`], then call [`FontDialog::show`] to present
/// the dialog and block until the user confirms or cancels.
#[derive(Debug, Clone, Default)]
pub struct FontDialog {
    initial: Option<FontSelection>,
}

impl FontDialog {
    /// Creates a new font dialog with no initial selection; the native
    /// dialog will open with the platform's own default selection.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the font the dialog should open with already selected.
    #[must_use]
    pub fn initial(mut self, selection: FontSelection) -> Self {
        self.initial = Some(selection);
        self
    }

    /// Presents the native font-picker dialog and blocks the calling thread
    /// until the user confirms a selection or cancels.
    ///
    /// Returns `Some(FontSelection)` if the user confirmed a font, or `None`
    /// if the dialog was dismissed without confirming.
    #[must_use]
    pub fn show(&self) -> Option<FontSelection> {
        backend::current().show(self.initial.as_ref())
    }
}

/// Internal trait implemented once per platform; not part of the public API.
///
/// Kept as a trait (rather than inline `cfg`-gated branches in
/// [`FontDialog::show`]) because the macOS backend in particular carries
/// enough of its own state (an accessory "Done" button, a completion flag)
/// that a small struct-per-backend reads more clearly than one function with
/// three large `cfg`-gated bodies.
pub(crate) trait FontDialogBackend {
    fn show(&self, initial: Option<&FontSelection>) -> Option<FontSelection>;
}

#[cfg(test)]
mod tests {
    use super::{FontDialog, FontSelection};

    #[test]
    fn new_dialog_has_no_initial_selection() {
        let dialog = FontDialog::new();
        assert_eq!(dialog.initial, None);
    }

    #[test]
    fn initial_stores_the_given_selection() {
        let selection =
            FontSelection { family: "Georgia".to_string(), size: 12.0, bold: false, italic: true };

        let dialog = FontDialog::new().initial(selection.clone());

        assert_eq!(dialog.initial, Some(selection));
    }

    #[test]
    fn default_dialog_matches_new() {
        assert_eq!(FontDialog::default().initial, FontDialog::new().initial);
    }
}
