use std::{
    ffi::{c_void, CStr},
    ptr,
};

use uuid::Uuid;
use windows::{
    core::Error,
    Win32::{
        Foundation::{GlobalFree, HGLOBAL, HWND, LPARAM, LRESULT, POINT, WPARAM},
        Globalization::{MultiByteToWideChar, WideCharToMultiByte, CP_UTF8},
        System::{
            DataExchange::{
                AddClipboardFormatListener, CloseClipboard, CountClipboardFormats, EmptyClipboard,
                EnumClipboardFormats, GetClipboardData, OpenClipboard,
                RemoveClipboardFormatListener, SetClipboardData,
            },
            LibraryLoader::GetModuleHandleA,
            Memory::{GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GHND},
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
            CS_DBLCLKS, CS_HREDRAW, CS_VREDRAW, HWND_MESSAGE, MSG, WINDOW_EX_STYLE,
            WM_CLIPBOARDUPDATE, WNDCLASSEXA, WS_OVERLAPPEDWINDOW,
        },
    },
};

use crate::models::ClipboardItem;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u16)]
pub enum ClipboardFormat {
    BITMAP = CF_BITMAP.0,

    DIB = CF_DIB.0,

    DIBV5 = CF_DIBV5.0,

    DIF = CF_DIF.0,

    DSPBITMAP = CF_DSPBITMAP.0,

    DSPENHMETAFILE = CF_DSPENHMETAFILE.0,

    DSPMETAFILEPICT = CF_DSPMETAFILEPICT.0,

    DSPTEXT = CF_DSPTEXT.0,

    ENHMETAFILE = CF_ENHMETAFILE.0,

    GDIOBJFIRST = CF_GDIOBJFIRST.0,

    GDIOBJLAST = CF_GDIOBJLAST.0,

    HDROP = CF_HDROP.0,

    LOCALE = CF_LOCALE.0,

    METAFILEPICT = CF_METAFILEPICT.0,

    OEMTEXT = CF_OEMTEXT.0,

    OWNERDISPLAY = CF_OWNERDISPLAY.0,

    PALETTE = CF_PALETTE.0,

    PENDATA = CF_PENDATA.0,

    PRIVATEFIRST = CF_PRIVATEFIRST.0,

    PRIVATELAST = CF_PRIVATELAST.0,

    RIFF = CF_RIFF.0,

    SYLK = CF_SYLK.0,

    TEXT = CF_TEXT.0,

    TIFF = CF_TIFF.0,

    UNICODETEXT = CF_UNICODETEXT.0,

    WAVE = CF_WAVE.0,
}

impl ClipboardFormat {
    pub fn from_u16(value: u16) -> Option<Self> {
        Some(match value {
            value if CF_BITMAP.0 == value => Self::BITMAP,

            value if CF_DIB.0 == value => Self::DIB,

            value if CF_DIBV5.0 == value => Self::DIBV5,

            value if CF_DIF.0 == value => Self::DIF,

            value if CF_DSPBITMAP.0 == value => Self::DSPBITMAP,

            value if CF_DSPENHMETAFILE.0 == value => Self::DSPENHMETAFILE,

            value if CF_DSPMETAFILEPICT.0 == value => Self::DSPMETAFILEPICT,

            value if CF_DSPTEXT.0 == value => Self::DSPTEXT,

            value if CF_ENHMETAFILE.0 == value => Self::ENHMETAFILE,

            value if CF_GDIOBJFIRST.0 == value => Self::GDIOBJFIRST,

            value if CF_GDIOBJLAST.0 == value => Self::GDIOBJLAST,

            value if CF_HDROP.0 == value => Self::HDROP,

            value if CF_LOCALE.0 == value => Self::LOCALE,

            value if CF_METAFILEPICT.0 == value => Self::METAFILEPICT,

            value if CF_OEMTEXT.0 == value => Self::OEMTEXT,

            value if CF_OWNERDISPLAY.0 == value => Self::OWNERDISPLAY,

            value if CF_PALETTE.0 == value => Self::PALETTE,

            value if CF_PENDATA.0 == value => Self::PENDATA,

            value if CF_PRIVATEFIRST.0 == value => Self::PRIVATEFIRST,

            value if CF_PRIVATELAST.0 == value => Self::PRIVATELAST,

            value if CF_RIFF.0 == value => Self::RIFF,

            value if CF_SYLK.0 == value => Self::SYLK,

            value if CF_TEXT.0 == value => Self::TEXT,

            value if CF_TIFF.0 == value => Self::TIFF,

            value if CF_UNICODETEXT.0 == value => Self::UNICODETEXT,

            value if CF_WAVE.0 == value => Self::WAVE,

            _ => return None,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
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
                hInstance: GetModuleHandleA(None).unwrap().into(),
                hIcon: windows::Win32::UI::WindowsAndMessaging::HICON(0),
                hCursor: windows::Win32::UI::WindowsAndMessaging::HCURSOR(0),
                hbrBackground: windows::Win32::Graphics::Gdi::HBRUSH(0),
                lpszMenuName: windows::core::PCSTR(ptr::null_mut()),
                lpszClassName: windows::core::PCSTR(sz_class_name.as_ptr()),
                hIconSm: windows::Win32::UI::WindowsAndMessaging::HICON(0),
            };

            if RegisterClassExA(&wc) == 0 {
                return Err("Failed to create Window class");
            }

            let window_name = Uuid::new_v4().to_string();

            // Creates a new window handle
            let msg_only_hwnd = CreateWindowExA(
                WINDOW_EX_STYLE::default(),
                wc.lpszClassName,
                windows::core::PCSTR(window_name.as_ptr()),
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                0,
                0,
                HWND_MESSAGE,
                None,
                wc.hInstance,
                None,
            );

            if msg_only_hwnd.0 == 0 {
                return Err("Window handle could not be created");
            }

            // Registers the window to receive clipboard updates
            let _ = AddClipboardFormatListener(msg_only_hwnd);

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
            ClipboardItem::Text(text) => self.set_text_from_clipboard(&text).unwrap(),
            ClipboardItem::UnicodeText(text) => {
                self.set_unicode_text_from_clipboard(&text).unwrap()
            }
            _ => todo!(),
        }
    }

    pub fn get_number_of_formats(&self) -> i32 {
        unsafe { CountClipboardFormats() }
    }

    pub fn has_clipboard_changed(&self) -> bool {
        let mut msg = MSG {
            hwnd: windows::Win32::Foundation::HWND::default(),
            message: 0,
            wParam: windows::Win32::Foundation::WPARAM::default(),
            lParam: windows::Win32::Foundation::LPARAM::default(),
            time: 0,
            pt: POINT { x: 0, y: 0 },
        };

        unsafe {
            let res = GetMessageA(&mut msg, HWND::default(), 0, 0);

            if res.as_bool() {
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
            if OpenClipboard(None).is_ok() {
                next_format = EnumClipboardFormats(next_format as u32) as u16;
                let _ = CloseClipboard();
            }

            next_format
        }
    }

    fn get_clipboard_item_with_format(&self, format: u16) -> Option<ClipboardItem> {
        unsafe {
            let _ = OpenClipboard(None);

            let format = ClipboardFormat::from_u16(format);

            let _ = CloseClipboard();

            self.clipboard_format_as_clipboard_item(format?)
        }
    }

    fn get_text_from_clipboard(&self) -> Option<String> {
        unsafe {
            let mut result = None;

            if OpenClipboard(None).is_ok() {
                let hglb = GetClipboardData(ClipboardFormat::TEXT as u32);
                if let Ok(hglb) = hglb {
                    let pbox_copy = GlobalLock(HGLOBAL(hglb.0 as *mut c_void));

                    if !pbox_copy.is_null() {
                        result = Some(
                            match CStr::from_ptr(pbox_copy as *const i8).to_str() {
                                Ok(value) => value,
                                Err(_) => {
                                    let _ = GlobalUnlock(HGLOBAL(pbox_copy));
                                    let _ = CloseClipboard();
                                    return None;
                                }
                            }
                            .to_string(),
                        );
                    }

                    let _ = GlobalUnlock(HGLOBAL(hglb.0 as *mut c_void));
                }
                let _ = CloseClipboard();
            }

            result
        }
    }

    fn set_text_from_clipboard(&mut self, text: &str) -> Result<(), Error> {
        unsafe {
            let mem = GlobalAlloc(GHND, (std::mem::size_of::<u8>() * (text.len() + 1)) as _)?;

            if OpenClipboard(None).is_ok() {
                let mem_ptr = GlobalLock(mem);
                let mem_ptr = mem_ptr as *mut u8;

                for (i, byte) in text.as_bytes().iter().enumerate() {
                    ptr::write(mem_ptr.add(i), *byte);
                }

                ptr::write(mem_ptr.add(text.len()), 0);

                // Empties clipboard and makes the current window the owner of the clipboard
                let _ = EmptyClipboard();

                let _ = SetClipboardData(
                    CF_TEXT.0 as u32,
                    windows::Win32::Foundation::HANDLE(mem.0 as isize),
                );

                let _ = GlobalUnlock(mem);
                let _ = CloseClipboard();
            }

            // Free the memory when finished with it
            let _ = GlobalFree(mem);
            Ok(())
        }
    }

    fn get_unicode_text_from_clipboard(&self) -> Option<String> {
        unsafe {
            let mut result = None;

            if OpenClipboard(None).is_ok() {
                let hglb = GetClipboardData(ClipboardFormat::UNICODETEXT as u32);
                if let Ok(hglb) = hglb {
                    let pbox_copy = GlobalLock(HGLOBAL(hglb.0 as *mut c_void));
                    let wstr = core::slice::from_raw_parts(
                        pbox_copy as *const u16,
                        GlobalSize(HGLOBAL(pbox_copy)) / std::mem::size_of::<u16>(),
                    );

                    if !pbox_copy.is_null() {
                        let size_needed = WideCharToMultiByte(CP_UTF8, 0, wstr, None, None, None);

                        let mut s = vec![0u8; size_needed as usize];

                        WideCharToMultiByte(CP_UTF8, 0, wstr, Some(&mut s), None, None);

                        result = Some(
                            match CStr::from_ptr(s.as_ptr() as *const i8).to_str() {
                                Ok(value) => value,
                                Err(_) => {
                                    let _ = GlobalUnlock(HGLOBAL(hglb.0 as *mut c_void));
                                    let _ = CloseClipboard();
                                    return None;
                                }
                            }
                            .to_string(),
                        );
                    }

                    let _ = GlobalUnlock(HGLOBAL(hglb.0 as *mut c_void));
                }
                let _ = CloseClipboard();
            }

            result
        }
    }

    fn set_unicode_text_from_clipboard(&mut self, text: &str) -> Result<(), Error> {
        unsafe {
            if OpenClipboard(None).is_ok() {
                let size_needed = MultiByteToWideChar(
                    CP_UTF8,
                    windows::Win32::Globalization::MULTI_BYTE_TO_WIDE_CHAR_FLAGS::default(),
                    text.as_bytes(),
                    None,
                );

                let mem = GlobalAlloc(
                    GHND,
                    (std::mem::size_of::<u16>() * (size_needed as usize + 1)) as _,
                )?;

                let mem_ptr = GlobalLock(mem);
                let mem_ptr = mem_ptr as *mut u16;

                for (i, c) in text.encode_utf16().enumerate() {
                    ptr::write(mem_ptr.add(i), c);
                }

                ptr::write(mem_ptr.add(text.encode_utf16().count()), 0);

                // Empties clipboard and makes the current window the owner of the clipboard
                let _ = EmptyClipboard();

                let _ = SetClipboardData(
                    CF_UNICODETEXT.0 as u32,
                    windows::Win32::Foundation::HANDLE(mem.0 as isize),
                );

                let _ = GlobalUnlock(mem);

                // Free the memory when finished with it
                let _ = GlobalFree(mem);

                let _ = CloseClipboard();
            }
            Ok(())
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
            let _ = RemoveClipboardFormatListener(self.msg_only_hwnd);
        }
    }
}
