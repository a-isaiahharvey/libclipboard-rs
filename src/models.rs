use image::DynamicImage;
use pdf::file::File;

use crate::windows::WindowsCC;

pub enum Clipboard {
    Windows(WindowsCC),
}

impl Clipboard {
    pub fn new() -> Result<Self, &'static str> {
        cfg_if::cfg_if! {
            if #[cfg(target_os = "windows")] {
                Ok(Clipboard::Windows(WindowsCC::new()))
            } else {
                Err("Do not support this OS")
            }
        }
    }

    pub fn get_item(&self) -> Option<ClipboardItem> {
        match self {
            Clipboard::Windows(cc) => cc.read_clipboard_item(),
        }
    }

    pub fn number_of_formats(&self) -> i32 {
        match self {
            Clipboard::Windows(cc) => cc.get_number_of_formats(),
        }
    }

    pub fn has_changed(&self) -> bool {
        match self {
            Clipboard::Windows(_) => todo!(),
        }
    }
}

pub enum ClipboardItem {
    Html(String),
    Text(String),
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
        }
    }
}
