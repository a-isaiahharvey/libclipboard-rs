use std::{thread::sleep, time::Duration};

use libclipboard::macos::{get_clipboard_item, has_clipboard_changed};

fn main() {
    loop {
        if has_clipboard_changed() {
            println!("Clipboard contents has changed to:");
            println!("{:#?}", get_clipboard_item());
        }

        sleep(Duration::from_millis(500));
    }
}
