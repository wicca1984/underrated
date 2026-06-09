mod encoding;
mod geom;
mod infra;

fn main() {
    let mut stream = encoding::InputStream::from_utf8(b"a");
    let _ = stream.next();
    stream.reconsume();
    let _ = stream.peek();

    println!("underrated: nothing rendered yet.");
}
