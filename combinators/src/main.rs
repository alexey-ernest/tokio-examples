extern crate tokio;
extern crate bytes;
extern crate futures;

use tokio::io;
use tokio::net::TcpStream;
use futures::Future;

fn main() {
    let addr = "127.0.0.1:1234".parse().unwrap();

    let future = TcpStream::connect(&addr)
        .and_then(|socket| {
            io::write_all(socket, b"hello world\n")
        })
        .map(|_| println!("write complete"))
        .map_err(|_| println!("failed"));

    tokio::run(future);
}
