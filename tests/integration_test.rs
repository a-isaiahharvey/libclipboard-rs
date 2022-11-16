use libclipboard::{Clipboard, ClipboardItem};

#[test]
fn test_set_text() {
    let mut clipboard = Clipboard::new().unwrap();

    clipboard.set_item(ClipboardItem::UnicodeText("".to_owned()));
    assert_eq!(
        ClipboardItem::UnicodeText("".to_string()),
        clipboard.get_item().unwrap()
    );

    clipboard.set_item(ClipboardItem::UnicodeText("Hello World!".to_owned()));
    assert_eq!(
        ClipboardItem::UnicodeText("Hello World!".to_string()),
        clipboard.get_item().unwrap()
    );

    clipboard.set_item(ClipboardItem::UnicodeText("Goodbye World!".to_owned()));
    assert_eq!(
        ClipboardItem::UnicodeText("Goodbye World!".to_string()),
        clipboard.get_item().unwrap()
    );
}
