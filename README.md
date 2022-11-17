# libclipboard

[![CI](https://github.com/a-isaiahharvey/libclipboard-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/a-isaiahharvey/libclipboard-rs/actions/workflows/ci.yml)
[![rust-clippy analyze](https://github.com/a-isaiahharvey/libclipboard-rs/actions/workflows/rust-clippy.yml/badge.svg)](https://github.com/a-isaiahharvey/libclipboard-rs/actions/workflows/rust-clippy.yml)

> Cross-platform Rust system clipboard library

libclipboard is a cross-platform library for getting and setting the contents of the OS-level clipboard.

## Platform Support

* macOS
* Windows 10 and newer

## Building

```console
cargo build
```

## Example

```rust
use libclipboard::{Clipboard, ClipboardItem};

fn main() -> Result<(), String> {
    let mut clipboard = Clipboard::new()?;

    clipboard.set_item(ClipboardItem::UnicodeText("Hello World!".to_owned()));
    println!("{:?}", clipboard.get_item());

    clipboard.set_item(ClipboardItem::UnicodeText("Goodbye World!".to_owned()));
    println!("{:?}", clipboard.get_item());

    clipboard.set_item(ClipboardItem::UnicodeText("Hi! 👋".to_string()));
    println!("{:?}", clipboard.get_item());

    Ok(())
}
```
