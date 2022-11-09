use std::{thread::sleep, time::Duration};

use libclipboard::Clipboard;

fn main() {
    let clipboard = Clipboard::new().unwrap();

    println!("Starting loop");
    loop {
        if clipboard.has_changed() {
            println!("Clipboard contents has changed to:");
            println!("{:#?}", clipboard.get_item());
        }

        sleep(Duration::from_millis(500));
    }
}
