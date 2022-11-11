use std::{
    ffi::CStr,
    mem::{self, size_of},
    ptr,
};

use windows_sys::Win32::{
    Foundation::HWND,
    Globalization::{MultiByteToWideChar, WideCharToMultiByte, CP_UTF8},
    System::{
        DataExchange::{
            CloseClipboard, CountClipboardFormats, EmptyClipboard, GetClipboardData, OpenClipboard,
            RemoveClipboardFormatListener, SetClipboardData,
        },
        Memory::{GlobalAlloc, GlobalFree, GlobalLock, GlobalSize, GlobalUnlock, GHND},
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

    pub fn clipboard_format_as_clipboard_item(
        &self,
        format: ClipboardFormat,
    ) -> Option<ClipboardItem> {
        Some(match format {
            ClipboardFormat::TEXT => ClipboardItem::Text(self.get_unicode_text_from_clipboard()?),
            _ => return None,
        })
    }

    pub fn get_clipboard_item(&self) -> Option<ClipboardItem> {
        self.clipboard_format_as_clipboard_item(ClipboardFormat::TEXT)
    }

    pub fn set_clipboard_item(&self, item: ClipboardItem) {
        match item {
            ClipboardItem::Text(text) => self.set_text_from_clipboard(&text),
            ClipboardItem::UnicodeText(text) => self.set_unicode_text_from_clipboard(&text),
            _ => todo!(),
        }
    }

    pub fn get_number_of_formats(&self) -> i32 {
        unsafe { CountClipboardFormats() }
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

    fn get_unicode_text_from_clipboard(&self) -> Option<String> {
        unsafe {
            let mut result = None;
            let hwnd = self.window_handle;

            if OpenClipboard(hwnd) != 0 {
                let hglb = GetClipboardData(ClipboardFormat::UNICODETEXT as u32);
                if hglb != 0 {
                    let pbox_copy = GlobalLock(hglb);

                    if !pbox_copy.is_null() {
                        let wstr = pbox_copy as *const u16;
                        let size_needed = WideCharToMultiByte(
                            CP_UTF8,
                            0,
                            wstr,
                            (GlobalSize(hglb) / size_of::<u16>()) as i32,
                            std::ptr::null_mut(),
                            0,
                            std::ptr::null_mut(),
                            std::ptr::null_mut(),
                        );

                        let mut s = vec![0u8; size_needed as usize];

                        WideCharToMultiByte(
                            CP_UTF8,
                            0,
                            wstr,
                            (GlobalSize(hglb) / size_of::<u16>()) as i32,
                            &mut s[0],
                            size_needed,
                            std::ptr::null_mut(),
                            std::ptr::null_mut(),
                        );

                        result = Some(
                            String::from_utf8(
                                s.iter()
                                    .take_while(|c| **c != 0)
                                    .map(|c| *c)
                                    .collect::<Vec<u8>>(),
                            )
                            .ok()?,
                        );
                    }

                    GlobalUnlock(hglb);
                }
                CloseClipboard();
            }

            result
        }
    }

    fn set_text_from_clipboard(&self, text: &str) {
        unsafe {
            if text.is_empty() {
                return;
            }

            let hwnd = self.window_handle;

            let mem = GlobalAlloc(GHND, (mem::size_of::<u16>() * (text.len()) - 1) as _);

            if OpenClipboard(hwnd) != 0 {
                let mem_ptr = GlobalLock(mem);
                let mem_ptr = mem_ptr as *mut u16;

                MultiByteToWideChar(
                    CP_UTF8,
                    0,
                    text.as_ptr() as *const _,
                    text.len() as _,
                    mem_ptr,
                    text.len() as _,
                );

                ptr::write(mem_ptr.offset(text.len() as isize), 0);

                // Empties clipboard and makes the current window the owner of the clipboard
                EmptyClipboard();

                if SetClipboardData(CF_TEXT, mem) != 0 {}

                GlobalUnlock(mem);
                CloseClipboard();
            }

            // Free the memory when finished with it
            GlobalFree(mem);
        }
    }

    fn set_unicode_text_from_clipboard(&self, text: &str) {
        unsafe {
            if text.is_empty() {
                return;
            }

            let hwnd = self.window_handle;

            let mem = GlobalAlloc(GHND, (mem::size_of::<u16>() * (text.len()) - 1) as _);

            if OpenClipboard(hwnd) != 0 {
                let mem_ptr = GlobalLock(mem);
                let mem_ptr = mem_ptr as *mut u16;

                MultiByteToWideChar(
                    CP_UTF8,
                    0,
                    text.as_ptr() as *const _,
                    text.len() as _,
                    mem_ptr,
                    text.len() as _,
                );

                ptr::write(mem_ptr.offset(text.len() as isize), 0);

                // Empties clipboard and makes the current window the owner of the clipboard
                EmptyClipboard();

                if SetClipboardData(CF_UNICODETEXT, mem) != 0 {}

                GlobalUnlock(mem);
                CloseClipboard();
            }

            // Free the memory when finished with it
            GlobalFree(mem);
        }
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
