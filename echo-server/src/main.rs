extern crate tokio;

use tokio::io;
use tokio::net::TcpListener;
use tokio::prelude::*;

const SERVER_ADDR: &'static str = "127.0.0.1:6142";

fn main() {
    let addr = SERVER_ADDR.parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();

    let server = listener
        .incoming()
        .for_each(|socket| {
            // split the socket stream into readable and writable parts
            let (reader, writer) = socket.split();

            // copy
            let amount_bytes = io::copy(reader, writer);

            let msg = amount_bytes.then(|result| {
                match result {
                    Ok((amount, ..)) => println!("wrote {} bytes", amount),
                    Err(e) => eprintln!("error: {}", e),
                }

                Ok(())
            });

            // spawn the task that handles the client connection socket
            // it means each client connection will be handled concurrently
            tokio::spawn(msg);
            Ok(())
        })
        .map_err(|err| {
            eprintln!("accept error = {:?}", err);
        });

    println!("server running on {}", SERVER_ADDR);

    // Start the server
    //
    // This does a few things:
    //
    // * Start the Tokio runtime
    // * Spawns the `server` task onto the runtime.
    // * Blocks the current thread until the runtime becomes idle, i.e. all
    //   spawned tasks have completed.
    tokio::run(server);
}
