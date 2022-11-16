use std::{thread::sleep, time::Duration};

use libclipboard::Clipboard;

fn main() -> Result<(), String> {
    let clipboard = Clipboard::new()?;

    println!("Starting loop");
    loop {
        if clipboard.has_changed() {
            println!("Clipboard contents has changed to:");
            println!("{:#?}", clipboard.get_item());
        }

        sleep(Duration::from_millis(500));
    }
}
