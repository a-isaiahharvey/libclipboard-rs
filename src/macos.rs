use std::{io::Cursor, slice, sync::Once};

use image::{io::Reader, DynamicImage, ImageFormat};
use pdf::file::File;
use rust_macios::{
    appkit::{
        NSPasteboard, NSPasteboardTypeColor, NSPasteboardTypeFileURL, NSPasteboardTypeFont,
        NSPasteboardTypeHTML, NSPasteboardTypePDF, NSPasteboardTypePNG, NSPasteboardTypeRTF,
        NSPasteboardTypeRTFD, NSPasteboardTypeRuler, NSPasteboardTypeSound, NSPasteboardTypeString,
        NSPasteboardTypeTIFF, NSPasteboardTypeTabularText, NSPasteboardTypeURL,
    },
    foundation::{NSData, NSString},
};

use crate::models::ClipboardItem;

static INIT: Once = Once::new();
static mut CHANGE_COUNT: i64 = 0;

#[derive(Debug)]
enum PasteType {
    Url,
    Color,
    FileURL,
    Font,
    Html,
    MultipleTextSelection,
    Pdf,
    Png,
    Rtf,
    Rtfd,
    Ruler,
    Sound,
    String,
    TabularText,
    Tiff,
}

pub struct MacOSCC {
    pasteboard: NSPasteboard,
}

impl MacOSCC {
    pub fn new() -> Self {
        Self {
            pasteboard: NSPasteboard::general_pasteboard(),
        }
    }

    pub fn get_clipboard_item(&self) -> Option<ClipboardItem> {
        let types = self.pasteboard.types()?;
        let general_type = self.pasteboard.available_type_from_array(types)?;

        let pastetype = Self::get_paste_type(&general_type)?;

        Self::paste_type_as_clipboard_item(pastetype)
    }

    pub fn get_clipboard_items(&self) -> Option<Vec<ClipboardItem>> {
        let types = self.pasteboard.types()?;

        let mut result = Vec::new();

        for t in &types {
            let pastetype = match Self::get_paste_type(&t) {
                Some(t) => t,
                None => continue,
            };
            result.push(Self::paste_type_as_clipboard_item(pastetype)?);
        }

        Some(result)
    }

    pub fn get_number_of_formats(&self) -> i32 {
        if let Some(types) = self.pasteboard.types() {
            return types.count() as i32;
        }
        0
    }

    pub fn has_clipboard_changed(&self) -> bool {
        unsafe {
            INIT.call_once(|| {
                CHANGE_COUNT = self.pasteboard.change_count();
            });

            if CHANGE_COUNT != self.pasteboard.change_count() {
                CHANGE_COUNT = self.pasteboard.change_count();
                true
            } else {
                false
            }
        }
    }

    fn paste_type_as_clipboard_item(pastetype: PasteType) -> Option<ClipboardItem> {
        Some(match pastetype {
            PasteType::Url => ClipboardItem::Url(Self::get_url_from_clipboard()?),
            PasteType::Color => ClipboardItem::Text(Self::get_color_from_clipboard()?),
            PasteType::FileURL => ClipboardItem::FilePath(Self::get_file_url_from_clipboard()?),
            PasteType::Font => ClipboardItem::Text(Self::get_font_from_clipboard()?),
            PasteType::Html => ClipboardItem::Html(Self::get_html_from_clipboard()?),
            PasteType::MultipleTextSelection => {
                ClipboardItem::Text(Self::get_multiple_text_selection_from_clipboard()?)
            }
            PasteType::Rtf => ClipboardItem::Rtf(Self::get_rtf_from_clipboard()?),
            PasteType::Rtfd => ClipboardItem::Rtfd(Self::get_rtfd_from_clipboard()?),
            PasteType::Ruler => ClipboardItem::Text(Self::get_ruler_from_clipboard()?),
            PasteType::Sound => ClipboardItem::Text(Self::get_sound_from_clipboard()?),
            PasteType::String => ClipboardItem::Text(Self::get_string_from_clipboard()?),
            PasteType::TabularText => ClipboardItem::Text(Self::get_tabular_text_from_clipboard()?),
            PasteType::Png => ClipboardItem::Png(Self::get_png_from_clipboard()?),
            PasteType::Tiff => ClipboardItem::Tiff(Self::get_tiff_from_clipboard()?),
            PasteType::Pdf => ClipboardItem::Pdf(Self::get_pdf_from_clipboard()?),
        })
    }

    fn get_paste_type(string: &NSString) -> Option<PasteType> {
        match string.to_string().as_str() {
            "public.url" => Some(PasteType::Url),
            "com.apple.cocoa.pasteboard.color" => Some(PasteType::Color),
            "public.file-url" => Some(PasteType::FileURL),
            "com.apple.cocoa.pasteboard.character-formatting" => Some(PasteType::Font),
            "public.html" => Some(PasteType::Html),
            "com.apple.cocoa.pasteboard.multiple-text-selection" => {
                Some(PasteType::MultipleTextSelection)
            }
            "com.adobe.pdf" => Some(PasteType::Pdf),
            "public.png" => Some(PasteType::Png),
            "public.rtf" => Some(PasteType::Rtf),
            "com.apple.flat-rtfd" => Some(PasteType::Rtfd),
            "com.apple.cocoa.pasteboard.paragraph-formatting" => Some(PasteType::Ruler),
            "com.apple.cocoa.pasteboard.sound" => Some(PasteType::Sound),
            "public.utf8-plain-text" => Some(PasteType::String),
            "public.utf8-tab-separated-values-text" => Some(PasteType::TabularText),
            "public.tiff" => Some(PasteType::Tiff),
            _ => None,
        }
    }

    fn get_url_from_clipboard() -> Option<String> {
        let pasteboard = NSPasteboard::general_pasteboard();
        unsafe {
            Some(
                pasteboard
                    .string_for_type(NSPasteboardTypeURL.clone())?
                    .to_string(),
            )
        }
    }

    fn get_file_url_from_clipboard() -> Option<String> {
        let pasteboard = NSPasteboard::general_pasteboard();
        unsafe {
            Some(
                pasteboard
                    .string_for_type(NSPasteboardTypeFileURL.clone())?
                    .to_string(),
            )
        }
    }

    fn get_string_from_clipboard() -> Option<String> {
        let pasteboard = NSPasteboard::general_pasteboard();
        unsafe {
            Some(
                pasteboard
                    .string_for_type(NSPasteboardTypeString.clone())?
                    .to_string(),
            )
        }
    }

    fn get_ruler_from_clipboard() -> Option<String> {
        let pasteboard = NSPasteboard::general_pasteboard();
        unsafe {
            Some(
                pasteboard
                    .string_for_type(NSPasteboardTypeRuler.clone())?
                    .to_string(),
            )
        }
    }

    fn get_sound_from_clipboard() -> Option<String> {
        let pasteboard = NSPasteboard::general_pasteboard();
        unsafe {
            Some(
                pasteboard
                    .string_for_type(NSPasteboardTypeSound.clone())?
                    .to_string(),
            )
        }
    }

    fn get_font_from_clipboard() -> Option<String> {
        let pasteboard = NSPasteboard::general_pasteboard();
        unsafe {
            Some(
                pasteboard
                    .string_for_type(NSPasteboardTypeFont.clone())?
                    .to_string(),
            )
        }
    }

    fn get_color_from_clipboard() -> Option<String> {
        let pasteboard = NSPasteboard::general_pasteboard();
        unsafe {
            Some(
                pasteboard
                    .string_for_type(NSPasteboardTypeColor.clone())?
                    .to_string(),
            )
        }
    }

    fn get_rtf_from_clipboard() -> Option<String> {
        let pasteboard = NSPasteboard::general_pasteboard();
        unsafe {
            Some(
                pasteboard
                    .string_for_type(NSPasteboardTypeRTF.clone())?
                    .to_string(),
            )
        }
    }

    fn get_rtfd_from_clipboard() -> Option<String> {
        let pasteboard = NSPasteboard::general_pasteboard();
        unsafe {
            Some(
                pasteboard
                    .string_for_type(NSPasteboardTypeRTFD.clone())?
                    .to_string(),
            )
        }
    }

    fn get_tabular_text_from_clipboard() -> Option<String> {
        let pasteboard = NSPasteboard::general_pasteboard();
        unsafe {
            Some(
                pasteboard
                    .string_for_type(NSPasteboardTypeTabularText.clone())?
                    .to_string(),
            )
        }
    }

    fn get_multiple_text_selection_from_clipboard() -> Option<String> {
        let pasteboard = NSPasteboard::general_pasteboard();
        unsafe {
            Some(
                pasteboard
                    .string_for_type(NSPasteboardTypeTabularText.clone())?
                    .to_string(),
            )
        }
    }

    fn get_html_from_clipboard() -> Option<String> {
        let pasteboard = NSPasteboard::general_pasteboard();
        unsafe {
            Some(
                pasteboard
                    .string_for_type(NSPasteboardTypeHTML.clone())?
                    .to_string(),
            )
        }
    }

    fn get_png_from_clipboard() -> Option<DynamicImage> {
        let pasteboard = NSPasteboard::general_pasteboard();

        unsafe {
            let mut reader = Reader::new(Cursor::new(
                Self::nsdata_as_bytes(pasteboard.data_for_type(NSPasteboardTypePNG.clone())?)
                    .to_vec(),
            ));

            reader.set_format(ImageFormat::Png);

            reader.decode().ok()
        }
    }

    fn get_tiff_from_clipboard() -> Option<DynamicImage> {
        let pasteboard = NSPasteboard::general_pasteboard();

        unsafe {
            let mut reader = Reader::new(Cursor::new(
                Self::nsdata_as_bytes(pasteboard.data_for_type(NSPasteboardTypeTIFF.clone())?)
                    .to_vec(),
            ));

            reader.set_format(ImageFormat::Png);

            reader.decode().ok()
        }
    }

    fn get_pdf_from_clipboard() -> Option<File<Vec<u8>>> {
        let pasteboard = NSPasteboard::general_pasteboard();

        unsafe {
            File::from_data(
                Self::nsdata_as_bytes(pasteboard.data_for_type(NSPasteboardTypePDF.clone())?)
                    .to_vec(),
            )
            .ok()
        }
    }

    fn nsdata_as_bytes<'bytes>(nsdata: NSData) -> &'bytes [u8] {
        let ptr = nsdata.bytes();

        // The bytes pointer may be null for length zero
        let (ptr, len) = if ptr.is_null() {
            (0x1 as *const u8, 0)
        } else {
            (ptr as *const u8, nsdata.length())
        };

        unsafe { slice::from_raw_parts(ptr, len as usize) }
    }
}

impl Default for MacOSCC {
    fn default() -> Self {
        Self::new()
    }
}
