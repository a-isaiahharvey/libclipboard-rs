use rust_macios::{
    appkit::{
        NSPasteboard, NSPasteboardTypeColor, NSPasteboardTypeFileURL, NSPasteboardTypeFont,
        NSPasteboardTypeHTML, NSPasteboardTypePDF, NSPasteboardTypePNG, NSPasteboardTypeRTF,
        NSPasteboardTypeRTFD, NSPasteboardTypeRuler, NSPasteboardTypeSound, NSPasteboardTypeString,
        NSPasteboardTypeTIFF, NSPasteboardTypeTabularText, NSPasteboardTypeURL,
    },
    foundation::{NSData, NSString},
    nsarray,
    objective_c_runtime::{nil, traits::PNSObject},
};
use std::{io::Cursor, slice, sync::Once};

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

#[derive(Debug, Clone)]
pub struct MacOSCC {
    pasteboard: NSPasteboard,
}

impl PartialEq for MacOSCC {
    fn eq(&self, other: &Self) -> bool {
        self.pasteboard.m_is_equal(&other.pasteboard)
    }
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

        self.paste_type_as_clipboard_item(pastetype)
    }

    pub fn set_clipboard_item(&mut self, item: ClipboardItem) {
        match item {
            ClipboardItem::Text(string) => self.set_string_from_clipboard(string),
            ClipboardItem::UnicodeText(string) => self.set_string_from_clipboard(string),
            _ => todo!(),
        }
    }

    pub fn get_clipboard_items(&self) -> Option<Vec<ClipboardItem>> {
        let types = self.pasteboard.types()?;

        let mut result = Vec::new();

        for t in &types {
            let pastetype = match Self::get_paste_type(&t) {
                Some(t) => t,
                None => continue,
            };
            result.push(self.paste_type_as_clipboard_item(pastetype)?);
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

    fn paste_type_as_clipboard_item(&self, pastetype: PasteType) -> Option<ClipboardItem> {
        Some(match pastetype {
            PasteType::Url => ClipboardItem::Url(self.get_url_from_clipboard()?),
            PasteType::Color => ClipboardItem::UnicodeText(self.get_color_from_clipboard()?),
            PasteType::FileURL => ClipboardItem::FilePath(self.get_file_url_from_clipboard()?),
            PasteType::Font => ClipboardItem::UnicodeText(self.get_font_from_clipboard()?),
            PasteType::Html => ClipboardItem::Html(self.get_html_from_clipboard()?),
            PasteType::MultipleTextSelection => {
                ClipboardItem::Text(self.get_multiple_text_selection_from_clipboard()?)
            }
            PasteType::Rtf => ClipboardItem::Rtf(self.get_rtf_from_clipboard()?),
            PasteType::Rtfd => ClipboardItem::Rtfd(self.get_rtfd_from_clipboard()?),
            PasteType::Ruler => ClipboardItem::UnicodeText(self.get_ruler_from_clipboard()?),
            PasteType::Sound => ClipboardItem::UnicodeText(self.get_sound_from_clipboard()?),
            PasteType::String => ClipboardItem::UnicodeText(self.get_string_from_clipboard()?),
            PasteType::TabularText => {
                ClipboardItem::UnicodeText(self.get_tabular_text_from_clipboard()?)
            }
            PasteType::Png => ClipboardItem::Png(self.get_png_from_clipboard()?),
            PasteType::Tiff => ClipboardItem::Tiff(self.get_tiff_from_clipboard()?),
            PasteType::Pdf => ClipboardItem::Pdf(self.get_pdf_from_clipboard()?),
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

    fn get_url_from_clipboard(&self) -> Option<String> {
        unsafe {
            Some(
                self.pasteboard
                    .string_for_type(NSPasteboardTypeURL.clone())?
                    .to_string(),
            )
        }
    }

    fn get_file_url_from_clipboard(&self) -> Option<String> {
        unsafe {
            Some(
                self.pasteboard
                    .string_for_type(NSPasteboardTypeFileURL.clone())?
                    .to_string(),
            )
        }
    }

    fn get_string_from_clipboard(&self) -> Option<String> {
        unsafe {
            Some(
                self.pasteboard
                    .string_for_type(NSPasteboardTypeString.clone())?
                    .to_string(),
            )
        }
    }

    fn set_string_from_clipboard(&mut self, string: String) {
        unsafe {
            self.pasteboard
                .declare_types_owner(nsarray![NSPasteboardTypeString.clone()], nil);

            self.pasteboard
                .set_string_for_type(string.into(), NSPasteboardTypeString.clone());
        }
    }

    fn get_ruler_from_clipboard(&self) -> Option<String> {
        unsafe {
            Some(
                self.pasteboard
                    .string_for_type(NSPasteboardTypeRuler.clone())?
                    .to_string(),
            )
        }
    }

    fn get_sound_from_clipboard(&self) -> Option<String> {
        unsafe {
            Some(
                self.pasteboard
                    .string_for_type(NSPasteboardTypeSound.clone())?
                    .to_string(),
            )
        }
    }

    fn get_font_from_clipboard(&self) -> Option<String> {
        unsafe {
            Some(
                self.pasteboard
                    .string_for_type(NSPasteboardTypeFont.clone())?
                    .to_string(),
            )
        }
    }

    fn get_color_from_clipboard(&self) -> Option<String> {
        unsafe {
            Some(
                self.pasteboard
                    .string_for_type(NSPasteboardTypeColor.clone())?
                    .to_string(),
            )
        }
    }

    fn get_rtf_from_clipboard(&self) -> Option<String> {
        unsafe {
            Some(
                self.pasteboard
                    .string_for_type(NSPasteboardTypeRTF.clone())?
                    .to_string(),
            )
        }
    }

    fn get_rtfd_from_clipboard(&self) -> Option<String> {
        unsafe {
            Some(
                self.pasteboard
                    .string_for_type(NSPasteboardTypeRTFD.clone())?
                    .to_string(),
            )
        }
    }

    fn get_tabular_text_from_clipboard(&self) -> Option<String> {
        unsafe {
            Some(
                self.pasteboard
                    .string_for_type(NSPasteboardTypeTabularText.clone())?
                    .to_string(),
            )
        }
    }

    fn get_multiple_text_selection_from_clipboard(&self) -> Option<String> {
        unsafe {
            Some(
                self.pasteboard
                    .string_for_type(NSPasteboardTypeTabularText.clone())?
                    .to_string(),
            )
        }
    }

    fn get_html_from_clipboard(&self) -> Option<String> {
        unsafe {
            Some(
                self.pasteboard
                    .string_for_type(NSPasteboardTypeHTML.clone())?
                    .to_string(),
            )
        }
    }

    fn get_png_from_clipboard(&self) -> Option<Cursor<Vec<u8>>> {
        unsafe {
            Some(Cursor::new(
                Self::nsdata_as_bytes(self.pasteboard.data_for_type(NSPasteboardTypePNG.clone())?)
                    .to_vec(),
            ))
        }
    }

    fn get_tiff_from_clipboard(&self) -> Option<Cursor<Vec<u8>>> {
        unsafe {
            Some(Cursor::new(
                Self::nsdata_as_bytes(
                    self.pasteboard
                        .data_for_type(NSPasteboardTypeTIFF.clone())?,
                )
                .to_vec(),
            ))
        }
    }

    fn get_pdf_from_clipboard(&self) -> Option<Cursor<Vec<u8>>> {
        unsafe {
            Some(Cursor::new(
                Self::nsdata_as_bytes(self.pasteboard.data_for_type(NSPasteboardTypePDF.clone())?)
                    .to_vec(),
            ))
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
