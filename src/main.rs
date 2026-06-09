//! Thin binary entry point. The engine itself lives in the `underrated` library
//! crate (`src/lib.rs`); this binary will grow into the browser shell.

fn main() {
    let mut stream = underrated::encoding::InputStream::from_utf8(b"a");
    let _ = stream.next();
    stream.reconsume();
    let _ = stream.peek();

    println!("underrated: nothing rendered yet.");
}
