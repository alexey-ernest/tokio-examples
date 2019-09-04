extern crate tokio;
extern crate futures;

use tokio::io;
use tokio::net::TcpListener;
use tokio::timer::Interval;
use futures::{future, stream, Future, Stream, Sink};
use futures::future::lazy;
use futures::sync::mpsc;
use std::time::Duration;

fn bg_task(rx: mpsc::Receiver<usize>) -> impl Future<Item = (), Error = ()> {
    // this enum will be used for merged stream
    #[derive(PartialEq)]
    enum Item {
        Value(usize),
        Tick,
        Done,
    }

    let tick_dur = Duration::from_secs(30);
    let interval = Interval::new_interval(tick_dur)
        .map(|_| Item::Tick)
        .map_err(|_| ());

    let items = rx.map(Item::Value)
        // appending Done event at the end of the incoming stream
        .chain(stream::once(Ok(Item::Done)))
        // merge in the stream of intervals
        .select(interval)
        // terminate combined stream once `Done` is received
        // as interval stream will continue producing events forever
        // that way we know when the rx stream completed
        .take_while(|item| future::ok(*item != Item::Done));

    // using `fold` allows the state to be maintained across iterations
    // in this case the state is number of read bytes between ticks
    items.fold(0, |num, item| {
        match item {
            Item::Value(value) => future::ok(num + value),
            Item::Tick => {
                println!("bytes read = {}", num);

                // reset the byte count
                future::ok(0)
            }
            _ => unreachable!(), // this will also panic if happens
        }
    })
    .map(|_| ())
}

fn main() {
    tokio::run(lazy(|| {
        let addr = "127.0.0.1:1234".parse().unwrap();
        let listener = TcpListener::bind(&addr).unwrap();

        // create the channel that is used to communicate with the background task
        let (tx, rx) = mpsc::channel(1_024);

        // spwan the bg task
        tokio::spawn(bg_task(rx));

        // handling incoming connections
        listener.incoming().for_each(move |socket| {
            // spawn a new task to process the socket
            tokio::spawn({
                // each spawned taks will have the clone of the sender handle
                let tx = tx.clone();

                io::read_to_end(socket, vec![])
                    // drop the socket
                    .and_then(move |(_, buf)| {
                        tx.send(buf.len())
                            .map_err(|_| io::ErrorKind::Other.into())
                    })
                    .map(|_| ())
                    .map_err(|e| println!("socket error = {:?}", e))
            });

            // receive the next socket
            Ok(())
        })
        .map_err(|e| println!("listener error = {:?}", e))
    }));
}
