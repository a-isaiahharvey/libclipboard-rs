use std::ffi::CStr;

use windows_sys::Win32::{
    Foundation::HWND,
    System::{
        DataExchange::{
            CloseClipboard, CountClipboardFormats, GetClipboardData, OpenClipboard,
            RemoveClipboardFormatListener,
        },
        Memory::{GlobalLock, GlobalUnlock},
        SystemServices::{
            CF_BITMAP, CF_DIB, CF_DIBV5, CF_DIF, CF_DSPBITMAP, CF_DSPENHMETAFILE,
            CF_DSPMETAFILEPICT, CF_DSPTEXT, CF_ENHMETAFILE, CF_GDIOBJFIRST, CF_GDIOBJLAST,
            CF_HDROP, CF_LOCALE, CF_METAFILEPICT, CF_OEMTEXT, CF_OWNERDISPLAY, CF_PALETTE,
            CF_PENDATA, CF_PRIVATEFIRST, CF_PRIVATELAST, CF_RIFF, CF_SYLK, CF_TEXT, CF_TIFF,
            CF_UNICODETEXT, CF_WAVE,
        },
    },
    UI::WindowsAndMessaging::{GetDesktopWindow, GetTopWindow},
};

use crate::models::ClipboardItem;

#[repr(u32)]
pub enum ClipboardFormat {
    /// A handle to a bitmap (HBITMAP).
    BITMAP = CF_BITMAP,

    /// A memory object containing a BITMAPINFO structure followed by the
    /// bitmap bits.
    DIB = CF_DIB,

    /// A memory object containing a BITMAPV5HEADER structure followed by
    /// the bitmap color space information and the bitmap bits.
    DIBV5 = CF_DIBV5,

    /// Software Arts' Data Interchange Format.
    DIF = CF_DIF,

    /// Bitmap display format associated with a private format. The hMem
    /// parameter must be a handle to data that can be displayed in bitmap
    /// format in lieu of the privately formatted data.
    DSPBITMAP = CF_DSPBITMAP,

    /// Enhanced metafile display format associated with a private format.
    /// The hMem parameter must be a handle to data that can be displayed
    /// in enhanced metafile format in lieu of the privately formatted data.
    DSPENHMETAFILE = CF_DSPENHMETAFILE,

    /// Metafile-picture display format associated with a private format.
    /// The hMem parameter must be a handle to data that can be displayed
    /// in metafile-picture format in lieu of the privately formatted data.
    DSPMETAFILEPICT = CF_DSPMETAFILEPICT,

    /// Text display format associated with a private format. The hMem
    /// parameter must be a handle to data that can be displayed in text
    /// format in lieu of the privately formatted data.
    DSPTEXT = CF_DSPTEXT,

    /// A handle to an enhanced metafile (HENHMETAFILE).
    ENHMETAFILE = CF_ENHMETAFILE,

    GDIOBJFIRST = CF_GDIOBJFIRST,

    GDIOBJLAST = CF_GDIOBJLAST,

    HDROP = CF_HDROP,

    LOCALE = CF_LOCALE,

    METAFILEPICT = CF_METAFILEPICT,

    OEMTEXT = CF_OEMTEXT,

    OWNERDISPLAY = CF_OWNERDISPLAY,

    PALETTE = CF_PALETTE,

    PENDATA = CF_PENDATA,

    PRIVATEFIRST = CF_PRIVATEFIRST,

    PRIVATELAST = CF_PRIVATELAST,

    RIFF = CF_RIFF,

    SYLK = CF_SYLK,

    TEXT = CF_TEXT,

    TIFF = CF_TIFF,

    UNICODETEXT = CF_UNICODETEXT,

    WAVE = CF_WAVE,
}

pub fn count_clipboard_formats() -> i32 {
    unsafe { CountClipboardFormats() }
}

pub struct WindowsCC {
    window_handle: HWND,
}

impl WindowsCC {
    pub fn new() -> Self {
        unsafe {
            let window_handle = GetTopWindow(GetDesktopWindow());

            Self { window_handle }
        }
    }

    fn get_text_from_clipboard(&self) -> Option<String> {
        unsafe {
            let mut result = None;
            let hwnd = self.window_handle;

            if OpenClipboard(hwnd) != 0 {
                let hglb = GetClipboardData(ClipboardFormat::TEXT as u32);
                if hglb != 0 {
                    let pbox_copy = GlobalLock(hglb);

                    if !pbox_copy.is_null() {
                        result = Some(
                            CStr::from_ptr(pbox_copy as *const i8)
                                .to_str()
                                .ok()?
                                .to_string(),
                        );
                    }

                    GlobalUnlock(hglb);
                }
                CloseClipboard();
            }

            result
        }
    }

    pub fn clipboard_format_as_clipboard_item(
        &self,
        format: ClipboardFormat,
    ) -> Option<ClipboardItem> {
        Some(match format {
            ClipboardFormat::TEXT => ClipboardItem::Text(self.get_text_from_clipboard()?),
            _ => return None,
        })
    }

    pub fn read_clipboard_item(&self) -> Option<ClipboardItem> {
        self.clipboard_format_as_clipboard_item(ClipboardFormat::TEXT)
    }

    pub fn get_number_of_formats(&self) -> i32 {
        unsafe { CountClipboardFormats() }
    }
}

impl Drop for WindowsCC {
    fn drop(&mut self) {
        unsafe {
            RemoveClipboardFormatListener(self.window_handle);
        }
    }
}

impl Default for WindowsCC {
    fn default() -> Self {
        Self::new()
    }
}
