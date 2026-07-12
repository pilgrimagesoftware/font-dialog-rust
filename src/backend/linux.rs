//! Linux backend: drives `gtk4::FontDialog` as a blocking call.
//!
//! `gtk4::FontDialog::choose_font` is natively callback-based (GTK4 moved
//! away from blocking modal dialogs generally). This backend adapts it into
//! a blocking call by running the GTK main loop's iterations directly until
//! the callback fires, rather than exposing an async API to this crate's
//! callers — see `native-font-dialog-crate`'s design.md.
//!
//! # Precondition
//!
//! The embedding application must have already called `gtk4::init()` before
//! [`crate::FontDialog::show`] is invoked. This backend does not call it on
//! the caller's behalf.
//!
//! Not compiled or tested on this machine (implemented on macOS, without a
//! GTK4 development environment available) — verified by this crate's Linux
//! CI job (see `.github/workflows/ci.yaml`) and by manual testing on Linux
//! before release, per `tasks.md`'s verification section. The exact
//! `FontDialog::choose_font` signature used below follows gtk4-rs's
//! established `*Dialog::choose_*` convention (mirrored by `FileDialog`,
//! `ColorDialog`, `AlertDialog`), but has not been confirmed against a live
//! GTK4 environment — double-check against the installed `gtk4` version's
//! docs before relying on this in CI.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::glib::MainContext;
use gtk4::pango::{FontDescription, Style, Weight};
use gtk4::prelude::*;
use gtk4::{FontDialog as GtkFontDialog, gio};

use crate::{FontDialogBackend, FontSelection};

pub struct LinuxBackend;

impl FontDialogBackend for LinuxBackend {
    fn show(&self, initial: Option<&FontSelection>) -> Option<FontSelection> {
        let dialog = GtkFontDialog::new();
        let initial_desc = initial.map(selection_to_font_description);

        let result: Rc<RefCell<Option<Option<FontDescription>>>> = Rc::new(RefCell::new(None));
        let result_for_callback = Rc::clone(&result);

        dialog.choose_font(gtk4::Window::NONE,
                           initial_desc.as_ref(),
                           gio::Cancellable::NONE,
                           move |chosen| {
                               *result_for_callback.borrow_mut() = Some(chosen.ok());
                           });

        let main_context = MainContext::default();
        while result.borrow().is_none() {
            main_context.iteration(true);
        }

        result.borrow_mut()
              .take()
              .flatten()
              .map(|desc| font_description_to_selection(&desc))
    }
}

/// Builds the `pango::FontDescription` `choose_font` should open with
/// pre-selected, from `selection`'s family/size/style.
fn selection_to_font_description(selection: &FontSelection) -> FontDescription {
    let mut desc = FontDescription::new();
    desc.set_family(&selection.family);
    desc.set_size(font_size_to_pango_units(selection.size));
    desc.set_weight(if selection.bold {
                        Weight::Bold
                    }
                    else {
                        Weight::Normal
                    });
    desc.set_style(if selection.italic {
                       Style::Italic
                   }
                   else {
                       Style::Normal
                   });
    desc
}

/// Translates a confirmed `pango::FontDescription` back into a
/// [`FontSelection`].
fn font_description_to_selection(desc: &FontDescription) -> FontSelection {
    FontSelection { family: desc.family().map(|f| f.to_string()).unwrap_or_default(),
                    size:   pango_units_to_font_size(desc.size()),
                    bold:   desc.weight() >= Weight::Bold,
                    italic: matches!(desc.style(), Style::Italic | Style::Oblique), }
}

/// Pango sizes are in "Pango units" — 1024ths of a point.
fn font_size_to_pango_units(size: f32) -> i32 {
    (size * 1024.0) as i32
}

/// Inverse of [`font_size_to_pango_units`].
fn pango_units_to_font_size(pango_units: i32) -> f32 {
    pango_units as f32 / 1024.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_size_round_trips_through_pango_units() {
        let original = 14.0_f32;
        let round_tripped = pango_units_to_font_size(font_size_to_pango_units(original));
        assert!((round_tripped - original).abs() < 0.01);
    }

    #[test]
    fn selection_to_font_description_sets_bold_and_italic() {
        let selection = FontSelection { family: "Noto Sans".to_string(),
                                        size:   11.0,
                                        bold:   true,
                                        italic: true, };
        let desc = selection_to_font_description(&selection);
        assert_eq!(desc.weight(), Weight::Bold);
        assert_eq!(desc.style(), Style::Italic);
    }

    #[test]
    fn font_description_to_selection_reads_back_family_size_and_style() {
        let selection = FontSelection { family: "DejaVu Sans".to_string(),
                                        size:   10.0,
                                        bold:   false,
                                        italic: false, };
        let desc = selection_to_font_description(&selection);
        let result = font_description_to_selection(&desc);

        assert_eq!(result.family, "DejaVu Sans");
        assert!((result.size - 10.0).abs() < 0.01);
        assert!(!result.bold);
        assert!(!result.italic);
    }
}
