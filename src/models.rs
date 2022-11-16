use std::io::Cursor;

use cfg_if::cfg_if;

#[cfg(target_os = "macos")]
use crate::macos::MacOSCC;
#[cfg(target_os = "windows")]
use crate::windows::WindowsCC;

#[derive(Debug, PartialEq, Clone)]
pub enum Clipboard {
    #[cfg(target_os = "windows")]
    Windows(WindowsCC),
    #[cfg(target_os = "macos")]
    MacOS(MacOSCC),
}

impl Clipboard {
    pub fn new() -> Result<Self, &'static str> {
        cfg_if! {
            if #[cfg(target_os = "windows")] {
                Ok(Clipboard::Windows(WindowsCC::new()?))
            } else if #[cfg(target_os = "macos")] {
                Ok(Clipboard::MacOS(MacOSCC::new()))
            } else {
                Err("Does not support this OS")
            }
        }
    }

    pub fn get_item(&self) -> Option<ClipboardItem> {
        cfg_if! {
            if #[cfg(target_os = "windows")] {
                match self {
                    Clipboard::Windows(cc) => cc.get_clipboard_item(),
                }
            } else if #[cfg(target_os = "macos")] {
                match self {
                    Clipboard::MacOS(cc) => cc.get_clipboard_item(),
                }
            } else {
                None
            }
        }
    }

    pub fn get_items(&self) -> Option<Vec<ClipboardItem>> {
        cfg_if! {
            if #[cfg(target_os = "windows")] {
                match self {
                    Clipboard::Windows(cc) => cc.get_clipboard_items(),
                }
            } else if #[cfg(target_os = "macos")] {
                match self {
                    Clipboard::MacOS(cc) => cc.get_clipboard_items(),
                }
            } else {
                None
            }
        }
    }

    pub fn set_item(&mut self, item: ClipboardItem) {
        cfg_if! {
            if #[cfg(target_os = "windows")] {
                match self {
                    Clipboard::Windows(cc) => cc.set_clipboard_item(item),
                }
            } else if #[cfg(target_os = "macos")] {
                match self {
                    Clipboard::MacOS(cc) => cc.set_clipboard_item(item),
                }
            }
        }
    }

    pub fn number_of_formats(&self) -> i32 {
        cfg_if! {
            if #[cfg(target_os = "windows")] {
                match self {
                    Clipboard::Windows(cc) => cc.get_number_of_formats(),
                }
            } else if #[cfg(target_os = "macos")] {
                match self {
                    Clipboard::MacOS(cc) => cc.get_number_of_formats(),
                }
            } else {
                0
            }
        }
    }

    pub fn has_changed(&self) -> bool {
        cfg_if! {
            if #[cfg(target_os = "macos")] {
                match self {
                    Clipboard::MacOS(cc) => cc.has_clipboard_changed(),
                }
            } else if #[cfg(target_os = "windows")] {
                match self {
                    Clipboard::Windows(cc) => cc.has_clipboard_changed(),
                }
            } else {
                false
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardItem {
    Html(String),
    Text(String),
    UnicodeText(String),
    Rtf(String),
    Rtfd(String),
    Url(String),
    FilePath(String),
    Png(Cursor<Vec<u8>>),
    Tiff(Cursor<Vec<u8>>),
    Pdf(Cursor<Vec<u8>>),
    RawBytes(Vec<i8>),
}
