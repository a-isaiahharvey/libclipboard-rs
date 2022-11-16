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
            GetClipboardData, OpenClipboard, RemoveClipboardFormatListener, SetClipboardData,
        },
        LibraryLoader::GetModuleHandleA,
        Memory::{GlobalAlloc, GlobalFree, GlobalLock, GlobalSize, GlobalUnlock, GHND},
        SystemServices::{
            CF_BITMAP, CF_DIB, CF_DIBV5, CF_DIF, CF_DSPBITMAP, CF_DSPENHMETAFILE,
            CF_DSPMETAFILEPICT, CF_DSPTEXT, CF_ENHMETAFILE, CF_GDIOBJFIRST, CF_GDIOBJLAST,
            CF_HDROP, CF_LOCALE, CF_METAFILEPICT, CF_OEMTEXT, CF_OWNERDISPLAY, CF_PALETTE,
            CF_PENDATA, CF_PRIVATEFIRST, CF_PRIVATELAST, CF_RIFF, CF_SYLK, CF_TEXT, CF_TIFF,
            CF_UNICODETEXT, CF_WAVE,
        },
    },
    UI::WindowsAndMessaging::{
        CreateWindowExA, DefWindowProcA, DispatchMessageA, GetDesktopWindow, GetMessageA,
        GetTopWindow, RegisterClassExA, CS_DBLCLKS, CS_HREDRAW, CS_VREDRAW, HWND_MESSAGE, MSG,
        WM_CLIPBOARDUPDATE, WNDCLASSEXA, WS_OVERLAPPEDWINDOW,
    },
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

pub static mut CLIPBOARD_CHANGED: bool = false;

pub struct WindowsCC {
    msg_only_hwnd: HWND,
    window_handle: HWND,
}

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

            // Creates a new window
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

            let window_handle = GetTopWindow(GetDesktopWindow());

            Ok(Self {
                window_handle,
                msg_only_hwnd,
            })
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

    fn set_text_from_clipboard(&self, text: &str) {
        unsafe {
            if text.is_empty() {
                return;
            }

            let hwnd = self.window_handle;

            let mem = GlobalAlloc(GHND, (mem::size_of::<u8>() * (text.len())) as _);

            if OpenClipboard(hwnd) != 0 {
                let mem_ptr = GlobalLock(mem);
                let mem_ptr = mem_ptr as *mut u8;

                for (i, byte) in text.as_bytes().iter().enumerate() {
                    ptr::write(mem_ptr.offset(i as _), *byte);
                }

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
                            ptr::null_mut(),
                            0,
                            ptr::null_mut(),
                            ptr::null_mut(),
                        );

                        let mut s = vec![0u8; size_needed as usize];

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

    fn set_unicode_text_from_clipboard(&self, text: &str) {
        unsafe {
            if text.is_empty() {
                return;
            }

            let hwnd = self.window_handle;

            let mem = GlobalAlloc(GHND, (mem::size_of::<u16>() * (text.len())) as _);

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
}

pub unsafe extern "system" fn window_proc(
    param0: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if msg == WM_CLIPBOARDUPDATE {
        if !CLIPBOARD_CHANGED {
            unsafe { CLIPBOARD_CHANGED = true };
        }
    }

    return DefWindowProcA(param0, msg, w_param, l_param);
}

impl Drop for WindowsCC {
    fn drop(&mut self) {
        unsafe {
            RemoveClipboardFormatListener(self.window_handle);
            GlobalFree(self.msg_only_hwnd);
        }
    }
}
