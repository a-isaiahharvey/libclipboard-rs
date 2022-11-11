use image::DynamicImage;
use pdf::file::File;

#[cfg(target_os = "macos")]
use crate::macos::MacOSCC;
#[cfg(target_os = "windows")]
use crate::windows::WindowsCC;

pub enum Clipboard {
    #[cfg(target_os = "windows")]
    Windows(WindowsCC),
    #[cfg(target_os = "macos")]
    MacOS(MacOSCC),
}

impl Clipboard {
    pub fn new() -> Result<Self, &'static str> {
        cfg_if::cfg_if! {
            if #[cfg(target_os = "windows")] {
                Ok(Clipboard::Windows(WindowsCC::new()))
            } else if #[cfg(target_os = "macos")] {
                Ok(Clipboard::MacOS(MacOSCC::new()))
            } else {
                Err("Do not support this OS")
            }
        }
    }

    pub fn get_item(&self) -> Option<ClipboardItem> {
        match self {
            #[cfg(target_os = "windows")]
            Clipboard::Windows(cc) => cc.get_clipboard_item(),
            #[cfg(target_os = "macos")]
            Clipboard::MacOS(cc) => cc.get_clipboard_item(),
        }
    }

    pub fn set_item(&self, item: ClipboardItem) {
        match self {
            #[cfg(target_os = "windows")]
            Clipboard::Windows(cc) => cc.set_clipboard_item(item),
            #[cfg(target_os = "macos")]
            Clipboard::MacOS(_) => todo!(),
        }
    }

    pub fn number_of_formats(&self) -> i32 {
        match self {
            #[cfg(target_os = "windows")]
            Clipboard::Windows(cc) => cc.get_number_of_formats(),
            #[cfg(target_os = "macos")]
            Clipboard::MacOS(cc) => cc.get_number_of_formats(),
        }
    }

    pub fn has_changed(&self) -> bool {
        match self {
            #[cfg(target_os = "windows")]
            Clipboard::Windows(_) => todo!(),
            #[cfg(target_os = "macos")]
            Clipboard::MacOS(cc) => cc.has_clipboard_changed(),
        }
    }
}

pub enum ClipboardItem {
    Html(String),
    Text(String),
    UnicodeText(String),
    Rtf(String),
    Rtfd(String),
    Url(String),
    FilePath(String),
    Png(DynamicImage),
    Tiff(DynamicImage),
    Pdf(File<Vec<u8>>),
}

impl std::fmt::Debug for ClipboardItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Html(arg0) => f.debug_tuple("Html").field(arg0).finish(),
            Self::Text(arg0) => f.debug_tuple("Text").field(arg0).finish(),
            Self::Rtf(arg0) => f.debug_tuple("Rtf").field(arg0).finish(),
            Self::Rtfd(arg0) => f.debug_tuple("Rtfd").field(arg0).finish(),
            Self::Url(arg0) => f.debug_tuple("Url").field(arg0).finish(),
            Self::FilePath(arg0) => f.debug_tuple("FilePath").field(arg0).finish(),
            Self::Png(arg0) => f.debug_tuple("Png").field(arg0).finish(),
            Self::Tiff(arg0) => f.debug_tuple("Tiff").field(arg0).finish(),
            Self::Pdf(arg) => f
                .debug_tuple("Pdf")
                .field(&format!("num_of_pages:{}", arg.num_pages()))
                .finish(),
            Self::UnicodeText(arg0) => f.debug_tuple("Unicode Text").field(arg0).finish(),
        }
    }
}
