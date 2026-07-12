//! macOS backend: drives `NSFontPanel`/`NSFontManager` via `objc2`.
//!
//! `NSFontPanel` is natively a persistent, non-modal floating panel — there
//! is no built-in "OK" button, and Apple's own HIG expects the app to react
//! to a stream of `changeFont:` messages as the user browses, not to get a
//! single final answer back. This backend adds a small "Done" button as the
//! panel's accessory view and runs the panel modally (scoped to just that
//! window, not the whole app) so [`crate::FontDialog::show`] can still
//! return one confirmed-or-cancelled result, matching every other platform
//! this crate supports.
//!
//! See `native-font-dialog-crate`'s design.md for the full rationale.

use std::cell::Cell;

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{
    AnyThread, DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel,
};
use objc2_app_kit::{
    NSApplication, NSButton, NSFontManager, NSFontTraitMask, NSWindowWillCloseNotification,
};
use objc2_foundation::{NSNotification, NSNotificationCenter, NSObject, NSString};

use crate::{FontDialogBackend, FontSelection};

define_class!(
    /// Target object for the accessory "Done" button and the panel's
    /// will-close notification. Exists purely to receive Objective-C action
    /// messages and flip `confirmed`/`should_stop` so the Rust-side modal
    /// loop knows why it ended.
    ///
    /// # Safety
    ///
    /// `NSObject` has no subclassing requirements, and `DoneSignal` does not
    /// implement `Drop`.
    #[unsafe(super(NSObject))]
    #[name = "FontDialogDoneSignal"]
    #[ivars = DoneSignalIvars]
    struct DoneSignal;

    impl DoneSignal {
        #[unsafe(method(buttonClicked:))]
        fn button_clicked(&self, _sender: &AnyObject) {
            self.ivars().confirmed.set(true);
            stop_modal();
        }

        #[unsafe(method(panelWillClose:))]
        fn panel_will_close(&self, _notification: &NSNotification) {
            stop_modal();
        }
    }
);

/// Interior-mutable state backing [`DoneSignal`] — `Cell` is required for
/// any ivar an Objective-C class method needs to mutate through `&self`.
struct DoneSignalIvars {
    /// Set to `true` only when the user clicks "Done"; left `false` if the
    /// panel is closed any other way, so the caller can tell confirm from
    /// cancel.
    confirmed: Cell<bool>,
}

impl DoneSignal {
    fn new() -> Retained<Self> {
        let this = Self::alloc().set_ivars(DoneSignalIvars { confirmed: Cell::new(false), });
        // SAFETY: `NSObject::init` has no preconditions beyond a freshly
        // allocated, not-yet-initialized instance, which `this` is.
        unsafe { msg_send![super(this), init] }
    }
}

/// Stops whichever modal session is currently running for the shared
/// application, if any. Called from both the "Done" button and the panel's
/// will-close notification, so either dismissal path ends the blocking
/// `show()` call.
fn stop_modal() {
    if let Some(mtm) = MainThreadMarker::new() {
        NSApplication::sharedApplication(mtm).stopModal();
    }
}

pub struct MacOsBackend;

impl FontDialogBackend for MacOsBackend {
    fn show(&self, initial: Option<&FontSelection>) -> Option<FontSelection> {
        let mtm = MainThreadMarker::new()
            .expect("FontDialog::show must be called from the main thread on macOS");
        let font_manager = NSFontManager::sharedFontManager(mtm);
        let panel = font_manager.fontPanel(true)?;

        if let Some(initial) = initial {
            if let Some(font) = build_ns_font(&font_manager, initial) {
                font_manager.setSelectedFont_isMultiple(&font, false);
            }
        }

        let signal = DoneSignal::new();
        let done_button = build_done_button(&signal, mtm);
        panel.setAccessoryView(Some(&done_button));

        // SAFETY: `signal` is retained for the lifetime of this call (kept
        // alive by the local `signal` binding), and `panelWillClose:` is a
        // method this class actually defines above.
        unsafe {
            NSNotificationCenter::defaultCenter().addObserver_selector_name_object(
                &signal,
                sel!(panelWillClose:),
                Some(NSWindowWillCloseNotification),
                Some(&panel),
            );
        }

        panel.makeKeyAndOrderFront(None);
        NSApplication::sharedApplication(mtm).runModalForWindow(&panel);

        // SAFETY: `signal` was only ever added as an observer for `panel`'s
        // will-close notification above; removing it here is the matching
        // teardown before `signal` (and `panel`'s accessory view reference
        // to `done_button`) go out of scope.
        unsafe {
            NSNotificationCenter::defaultCenter().removeObserver(&signal);
        }
        panel.setAccessoryView(None);

        if signal.ivars().confirmed.get() {
            font_manager.selectedFont()
                        .map(|font| font_to_selection(&font_manager, &font))
        }
        else {
            None
        }
    }
}

/// Builds the "Done" accessory button, wired to call `signal`'s
/// `buttonClicked:` method.
fn build_done_button(signal: &Retained<DoneSignal>, mtm: MainThreadMarker) -> Retained<NSButton> {
    let frame = objc2_foundation::NSRect::new(objc2_foundation::NSPoint::new(0.0, 0.0),
                                              objc2_foundation::NSSize::new(72.0, 32.0));
    let button = NSButton::initWithFrame(NSButton::alloc(mtm), frame);
    button.setTitle(&NSString::from_str("Done"));
    // SAFETY: `signal` outlives `button` for the duration of `show()` (both
    // are dropped together at the end of this call), and `buttonClicked:` is
    // a method `DoneSignal` actually implements.
    unsafe {
        button.setTarget(Some(signal));
        button.setAction(Some(sel!(buttonClicked:)));
    }
    button
}

/// Resolves an `NSFont` matching `selection`'s family/size/style, via
/// `NSFontManager::fontWithFamily_traits_weight_size` so bold/italic can be
/// expressed independently of the family name string.
fn build_ns_font(font_manager: &NSFontManager, selection: &FontSelection)
                 -> Option<Retained<objc2_app_kit::NSFont>> {
    let mut traits = NSFontTraitMask::empty();
    if selection.bold {
        traits |= NSFontTraitMask::BoldFontMask;
    }
    if selection.italic {
        traits |= NSFontTraitMask::ItalicFontMask;
    }
    // 5 is NSFontManager's "regular" weight; bold is expressed via `traits`
    // rather than a heavier weight value here, matching how `traitsOfFont`/
    // `weightOfFont` are read back symmetrically in `font_to_selection`.
    font_manager.fontWithFamily_traits_weight_size(&NSString::from_str(&selection.family),
                                                   traits,
                                                   5,
                                                   f64::from(selection.size))
}

/// Translates an `NSFont` back into a [`FontSelection`], reading style via
/// `NSFontManager::traitsOfFont` rather than parsing the font's display name.
fn font_to_selection(font_manager: &NSFontManager, font: &objc2_app_kit::NSFont) -> FontSelection {
    let traits = font_manager.traitsOfFont(font);
    FontSelection { family: font.familyName().map(|s| s.to_string()).unwrap_or_default(),
                    // NSFont point sizes are small (well under f32's integer-precision
                    // limit), so this narrowing cast never loses meaningful precision.
                    #[allow(clippy::cast_possible_truncation)]
                    size: font.pointSize() as f32,
                    bold: traits.contains(NSFontTraitMask::BoldFontMask),
                    italic: traits.contains(NSFontTraitMask::ItalicFontMask), }
}
