use libclipboard::{Clipboard, ClipboardItem};

fn main() -> Result<(), String> {
    let clipboard = Clipboard::new()?;

    clipboard.set_item(ClipboardItem::Text("Hello World!".to_owned()));
    println!("{:?}", clipboard.get_item());

    clipboard.set_item(ClipboardItem::Text("Goodbye World!".to_owned()));
    println!("{:?}", clipboard.get_item());

    Ok(())
}
