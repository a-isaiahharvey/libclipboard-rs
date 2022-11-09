use libclipboard::Clipboard;

fn main() {
    let clipboard = Clipboard::new().unwrap();
    println!("Number of formats: {:?}", clipboard.number_of_formats())
}
