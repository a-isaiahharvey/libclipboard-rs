use std::{
    ffi::CStr,
    mem::{self, size_of},
    ptr,
};

use uuid::Uuid;
use windows_sys::Win32::{
    Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM},
    Globalization::{MultiByteToWideChar, WideCharToMultiByte, CP_UTF8},
    System::{
        DataExchange::{
            AddClipboardFormatListener, CloseClipboard, CountClipboardFormats, EmptyClipboard,
            EnumClipboardFormats, GetClipboardData, OpenClipboard, RemoveClipboardFormatListener,
            SetClipboardData,
        },
        LibraryLoader::GetModuleHandleA,
        Memory::{GlobalAlloc, GlobalFree, GlobalLock, GlobalSize, GlobalUnlock, GHND},
        Ole::{
            CF_BITMAP, CF_DIB, CF_DIBV5, CF_DIF, CF_DSPBITMAP, CF_DSPENHMETAFILE,
            CF_DSPMETAFILEPICT, CF_DSPTEXT, CF_ENHMETAFILE, CF_GDIOBJFIRST, CF_GDIOBJLAST,
            CF_HDROP, CF_LOCALE, CF_METAFILEPICT, CF_OEMTEXT, CF_OWNERDISPLAY, CF_PALETTE,
            CF_PENDATA, CF_PRIVATEFIRST, CF_PRIVATELAST, CF_RIFF, CF_SYLK, CF_TEXT, CF_TIFF,
            CF_UNICODETEXT, CF_WAVE,
        },
    },
    UI::WindowsAndMessaging::{
        CreateWindowExA, DefWindowProcA, DispatchMessageA, GetMessageA, RegisterClassExA,
        CS_DBLCLKS, CS_HREDRAW, CS_VREDRAW, HWND_MESSAGE, MSG, WM_CLIPBOARDUPDATE, WNDCLASSEXA,
        WS_OVERLAPPEDWINDOW,
    },
};

use crate::models::ClipboardItem;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u16)]
pub enum ClipboardFormat {
    BITMAP = CF_BITMAP,

    DIB = CF_DIB,

    DIBV5 = CF_DIBV5,

    DIF = CF_DIF,

    DSPBITMAP = CF_DSPBITMAP,

    DSPENHMETAFILE = CF_DSPENHMETAFILE,

    DSPMETAFILEPICT = CF_DSPMETAFILEPICT,

    DSPTEXT = CF_DSPTEXT,

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

impl ClipboardFormat {
    pub fn from_u16(value: u16) -> Option<Self> {
        Some(match value {
            CF_BITMAP => Self::BITMAP,

            CF_DIB => Self::DIB,

            CF_DIBV5 => Self::DIBV5,

            CF_DIF => Self::DIF,

            CF_DSPBITMAP => Self::DSPBITMAP,

            CF_DSPENHMETAFILE => Self::DSPENHMETAFILE,

            CF_DSPMETAFILEPICT => Self::DSPMETAFILEPICT,

            CF_DSPTEXT => Self::DSPTEXT,

            CF_ENHMETAFILE => Self::ENHMETAFILE,

            CF_GDIOBJFIRST => Self::GDIOBJFIRST,

            CF_GDIOBJLAST => Self::GDIOBJLAST,

            CF_HDROP => Self::HDROP,

            CF_LOCALE => Self::LOCALE,

            CF_METAFILEPICT => Self::METAFILEPICT,

            CF_OEMTEXT => Self::OEMTEXT,

            CF_OWNERDISPLAY => Self::OWNERDISPLAY,

            CF_PALETTE => Self::PALETTE,

            CF_PENDATA => Self::PENDATA,

            CF_PRIVATEFIRST => Self::PRIVATEFIRST,

            CF_PRIVATELAST => Self::PRIVATELAST,

            CF_RIFF => Self::RIFF,

            CF_SYLK => Self::SYLK,

            CF_TEXT => Self::TEXT,

            CF_TIFF => Self::TIFF,

            CF_UNICODETEXT => Self::UNICODETEXT,

            CF_WAVE => Self::WAVE,

            _ => return None,
        })
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct WindowsCC {
    msg_only_hwnd: HWND,
}

pub static mut CLIPBOARD_CHANGED: bool = false;

impl WindowsCC {
    pub fn new() -> Result<Self, &'static str> {
        unsafe {
            let sz_class_name = "#32769";

            let wc = WNDCLASSEXA {
                cbSize: std::mem::size_of::<WNDCLASSEXA>() as u32,
                style: CS_HREDRAW | CS_VREDRAW | CS_DBLCLKS,
                lpfnWndProc: Some(window_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: GetModuleHandleA(ptr::null_mut()) as HINSTANCE,
                hIcon: 0,
                hCursor: 0,
                hbrBackground: 0,
                lpszMenuName: ptr::null_mut(),
                lpszClassName: sz_class_name.as_ptr() as *const u8,
                hIconSm: 0,
            };

            if RegisterClassExA(&wc) == 0 {
                return Err("Failed to create Window class");
            }

            let window_name = Uuid::new_v4().to_string();

            // Creates a new window handle
            let msg_only_hwnd = CreateWindowExA(
                0,
                wc.lpszClassName,
                window_name.as_ptr() as *const u8,
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                0,
                0,
                HWND_MESSAGE,
                0,
                wc.hInstance,
                ptr::null_mut(),
            );

            if msg_only_hwnd == 0 {
                return Err("Window handle could not be created");
            }

            // Registers the window to receive clipboard updates
            AddClipboardFormatListener(msg_only_hwnd);

            Ok(Self { msg_only_hwnd })
        }
    }

    pub fn clipboard_format_as_clipboard_item(
        &self,
        format: ClipboardFormat,
    ) -> Option<ClipboardItem> {
        Some(match format {
            ClipboardFormat::TEXT => ClipboardItem::Text(self.get_text_from_clipboard()?),
            ClipboardFormat::UNICODETEXT => {
                ClipboardItem::UnicodeText(self.get_unicode_text_from_clipboard()?)
            }
            _ => return None,
        })
    }

    pub fn get_clipboard_items(&self) -> Option<Vec<ClipboardItem>> {
        let mut result = vec![];
        let no_formats = self.get_number_of_formats();
        let mut next_format = 0;

        for _ in 0..no_formats {
            next_format = self.get_next_format(next_format);

            result.push(match self.get_clipboard_item_with_format(next_format) {
                Some(item) => item,
                None => continue,
            });
        }

        Some(result)
    }

    pub fn get_clipboard_item(&self) -> Option<ClipboardItem> {
        let next_available_format = self.get_next_format(0);
        let format = ClipboardFormat::from_u16(next_available_format);

        self.clipboard_format_as_clipboard_item(format?)
    }

    pub fn set_clipboard_item(&mut self, item: ClipboardItem) {
        match item {
            ClipboardItem::Text(text) => self.set_text_from_clipboard(&text),
            ClipboardItem::UnicodeText(text) => self.set_unicode_text_from_clipboard(&text),
            _ => todo!(),
        }
    }

    pub fn get_number_of_formats(&self) -> i32 {
        unsafe { CountClipboardFormats() }
    }

    pub fn has_clipboard_changed(&self) -> bool {
        let mut msg = MSG {
            hwnd: 0,
            message: 0,
            wParam: 0,
            lParam: 0,
            time: 0,
            pt: POINT { x: 0, y: 0 },
        };

        unsafe {
            let res = GetMessageA(&mut msg, 0, 0, 0);

            if res == 0 || res == -1 {
                return false;
            }
            DispatchMessageA(&msg);

            match CLIPBOARD_CHANGED {
                true => {
                    CLIPBOARD_CHANGED = false;
                    true
                }
                false => false,
            }
        }
    }

    fn get_next_format(&self, mut next_format: u16) -> u16 {
        unsafe {
            if OpenClipboard(0) != 0 {
                next_format = EnumClipboardFormats(next_format as u32) as u16;
                CloseClipboard();
            }

            next_format
        }
    }

    fn get_clipboard_item_with_format(&self, format: u16) -> Option<ClipboardItem> {
        unsafe {
            OpenClipboard(0);

            let format = ClipboardFormat::from_u16(format);

            CloseClipboard();

            self.clipboard_format_as_clipboard_item(format?)
        }
    }

    fn get_text_from_clipboard(&self) -> Option<String> {
        unsafe {
            let mut result = None;

            if OpenClipboard(0) != 0 {
                let hglb = GetClipboardData(ClipboardFormat::TEXT as u32);
                if hglb != 0 {
                    let pbox_copy = GlobalLock(hglb);

                    if !pbox_copy.is_null() {
                        result = Some(
                            match CStr::from_ptr(pbox_copy as *const i8).to_str() {
                                Ok(value) => value,
                                Err(_) => {
                                    GlobalUnlock(hglb);
                                    CloseClipboard();
                                    return None;
                                }
                            }
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

    fn set_text_from_clipboard(&mut self, text: &str) {
        unsafe {
            let mem = GlobalAlloc(GHND, (mem::size_of::<u8>() * (text.len() + 1)) as _);

            if OpenClipboard(0) != 0 {
                let mem_ptr = GlobalLock(mem);
                let mem_ptr = mem_ptr as *mut u8;

                for (i, byte) in text.as_bytes().iter().enumerate() {
                    ptr::write(mem_ptr.add(i), *byte);
                }

                ptr::write(mem_ptr.add(text.len()), 0);

                // Empties clipboard and makes the current window the owner of the clipboard
                EmptyClipboard();

                if SetClipboardData(CF_TEXT as u32, mem) != 0 {}

                GlobalUnlock(mem);
                CloseClipboard();
            }

            // Free the memory when finished with it
            GlobalFree(mem);
        }
    }

    fn get_unicode_text_from_clipboard(&self) -> Option<String> {
        unsafe {
            let mut result = None;

            if OpenClipboard(0) != 0 {
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
                            ptr::null_mut(),
                            0,
                            ptr::null_mut(),
                            ptr::null_mut(),
                        );

                        let mut s = vec![0u8; GlobalSize(hglb) / size_of::<u16>()];

                        WideCharToMultiByte(
                            CP_UTF8,
                            0,
                            wstr,
                            (GlobalSize(hglb) / size_of::<u16>()) as i32,
                            &mut s[0],
                            size_needed,
                            ptr::null_mut(),
                            ptr::null_mut(),
                        );

                        result = Some(
                            match CStr::from_ptr(s.as_ptr() as *const i8).to_str() {
                                Ok(value) => value,
                                Err(_) => {
                                    GlobalUnlock(hglb);
                                    CloseClipboard();
                                    return None;
                                }
                            }
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

    fn set_unicode_text_from_clipboard(&mut self, text: &str) {
        unsafe {
            let mem = GlobalAlloc(GHND, (mem::size_of::<u16>() * (text.len() + 1)) as _);

            if OpenClipboard(0) != 0 {
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

                ptr::write(mem_ptr.add(text.len()), 0);

                // Empties clipboard and makes the current window the owner of the clipboard
                EmptyClipboard();

                if SetClipboardData(CF_UNICODETEXT as u32, mem) != 0 {}

                GlobalUnlock(mem);
                CloseClipboard();
            }

            // Free the memory when finished with it
            GlobalFree(mem);
        }
    }
}

pub fn count_clipboard_formats() -> i32 {
    unsafe { CountClipboardFormats() }
}

/// # Safety
///
/// This function changes the static `CLIPBOARD_CHANGED` when called
pub unsafe extern "system" fn window_proc(
    param0: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if msg == WM_CLIPBOARDUPDATE && !CLIPBOARD_CHANGED {
        unsafe { CLIPBOARD_CHANGED = true };
    }

    DefWindowProcA(param0, msg, w_param, l_param)
}

impl Drop for WindowsCC {
    fn drop(&mut self) {
        unsafe {
            RemoveClipboardFormatListener(self.msg_only_hwnd);
        }
    }
}
