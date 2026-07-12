//! Windows backend: drives the native `ChooseFont` common dialog via
//! `windows-rs`'s `CHOOSEFONTW`/`LOGFONTW` bindings.
//!
//! Unlike the macOS backend, `ChooseFont` is already a blocking, modal,
//! single-answer dialog — this backend is close to a direct passthrough,
//! with no interaction-model gap to bridge.
//!
//! Not compiled or tested on this machine (implemented on macOS, without a
//! Windows toolchain available) — verified by this crate's Windows CI job
//! (see `.github/workflows/ci.yaml`) and by manual testing on Windows before
//! release, per `tasks.md`'s verification section.

use windows::Win32::Graphics::Gdi::LOGFONTW;
use windows::Win32::UI::Controls::Dialogs::{
    ChooseFontW, CF_EFFECTS, CF_INITTOLOGFONT, CF_SCREENFONTS, CHOOSEFONTW,
};

use crate::{FontDialogBackend, FontSelection};

/// `LOGFONTW.lfWeight` value `ChooseFont` reports for a bold selection;
/// anything at or above this is treated as bold on the way back out.
const FW_BOLD: i32 = 700;
/// `LOGFONTW.lfWeight` value used when populating an initial bold selection.
const FW_NORMAL: i32 = 400;

pub struct WindowsBackend;

impl FontDialogBackend for WindowsBackend {
    fn show(&self, initial: Option<&FontSelection>) -> Option<FontSelection> {
        let mut log_font = LOGFONTW::default();
        if let Some(initial) = initial {
            populate_log_font(&mut log_font, initial);
        }

        let mut choose_font = CHOOSEFONTW {
            lStructSize: std::mem::size_of::<CHOOSEFONTW>() as u32,
            lpLogFont: &mut log_font,
            Flags: CF_SCREENFONTS | CF_EFFECTS | CF_INITTOLOGFONT,
            ..Default::default()
        };

        // SAFETY: `choose_font.lpLogFont` points at `log_font`, a valid,
        // stack-allocated `LOGFONTW` that outlives this call; `choose_font`
        // itself is fully initialized (all non-explicitly-set fields come
        // from `Default::default()`, matching `ChooseFontW`'s documented
        // zero-initialization expectations for unused members).
        let confirmed = unsafe { ChooseFontW(&mut choose_font) }.as_bool();

        confirmed.then(|| log_font_to_selection(&log_font, choose_font.iPointSize))
    }
}

/// Populates `log_font`'s family/weight/italic fields from `selection`. Point
/// size is carried separately, via `CHOOSEFONTW.iPointSize` (tenths of a
/// point), rather than `lfHeight` (device-dependent logical units) — set by
/// the caller alongside `CF_INITTOLOGFONT`.
fn populate_log_font(log_font: &mut LOGFONTW, selection: &FontSelection) {
    log_font.lfWeight = if selection.bold { FW_BOLD } else { FW_NORMAL };
    log_font.lfItalic = u8::from(selection.italic);

    let name_utf16: Vec<u16> = selection.family.encode_utf16().collect();
    let len = name_utf16.len().min(log_font.lfFaceName.len() - 1);
    log_font.lfFaceName[..len].copy_from_slice(&name_utf16[..len]);
    log_font.lfFaceName[len] = 0;
}

/// Translates the `ChooseFont`-populated `log_font`/`point_size` back into a
/// [`FontSelection`]. `point_size` is `CHOOSEFONTW::iPointSize`, in tenths of
/// a point (e.g. `120` means 12pt).
fn log_font_to_selection(log_font: &LOGFONTW, point_size: i32) -> FontSelection {
    let nul_pos =
        log_font.lfFaceName.iter().position(|&c| c == 0).unwrap_or(log_font.lfFaceName.len());
    let family = String::from_utf16_lossy(&log_font.lfFaceName[..nul_pos]);

    FontSelection {
        family,
        size: f32::from(point_size as i16) / 10.0,
        bold: log_font.lfWeight >= FW_BOLD,
        italic: log_font.lfItalic != 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn populate_log_font_sets_bold_weight_and_italic_flag() {
        let mut log_font = LOGFONTW::default();
        let selection =
            FontSelection { family: "Segoe UI".to_string(), size: 12.0, bold: true, italic: true };

        populate_log_font(&mut log_font, &selection);

        assert_eq!(log_font.lfWeight, FW_BOLD);
        assert_eq!(log_font.lfItalic, 1);
    }

    #[test]
    fn populate_log_font_writes_face_name_as_nul_terminated_utf16() {
        let mut log_font = LOGFONTW::default();
        let selection =
            FontSelection { family: "Arial".to_string(), size: 10.0, bold: false, italic: false };

        populate_log_font(&mut log_font, &selection);

        let nul_pos = log_font.lfFaceName.iter().position(|&c| c == 0).unwrap();
        let name = String::from_utf16_lossy(&log_font.lfFaceName[..nul_pos]);
        assert_eq!(name, "Arial");
    }

    #[test]
    fn log_font_to_selection_reads_back_family_size_and_style() {
        let mut log_font = LOGFONTW::default();
        let selection = FontSelection {
            family: "Times New Roman".to_string(),
            size: 14.0,
            bold: true,
            italic: false,
        };
        populate_log_font(&mut log_font, &selection);

        let result = log_font_to_selection(&log_font, 140);

        assert_eq!(result.family, "Times New Roman");
        assert!((result.size - 14.0).abs() < f32::EPSILON);
        assert!(result.bold);
        assert!(!result.italic);
    }

    #[test]
    fn log_font_to_selection_handles_negative_point_size_gracefully() {
        // `iPointSize` should never actually be negative in practice, but the
        // conversion must not panic if it somehow is.
        let log_font = LOGFONTW::default();
        let result = log_font_to_selection(&log_font, -10);
        assert!(result.size < 0.0);
    }
}
