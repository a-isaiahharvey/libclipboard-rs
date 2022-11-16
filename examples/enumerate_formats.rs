use libclipboard::Clipboard;

fn main() {
    let clipboard = Clipboard::new().unwrap();

    println!("{:#?}", clipboard.get_items())
}
