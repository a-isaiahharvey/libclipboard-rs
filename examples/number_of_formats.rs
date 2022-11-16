use libclipboard::Clipboard;

fn main() -> Result<(), String> {
    let clipboard = Clipboard::new()?;
    println!("Number of formats: {:?}", clipboard.number_of_formats());
    Ok(())
}
