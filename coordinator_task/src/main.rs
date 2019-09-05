extern crate tokio;
extern crate futures;

use tokio::io;
use tokio::timer::Delay;
use futures::{future, Future, Stream, Sink};
use futures::future::lazy;
use futures::sync::{mpsc, oneshot};
use std::time::{Duration, Instant};

type Message = oneshot::Sender<Duration>;

struct Transport;

impl Transport {
	fn send_ping(&self) {
		println!("sending ping");
	}

	fn recv_pong(&self) -> impl Future<Item = (), Error = ()> {
		let when = Instant::now() + Duration::from_millis(1000);

		Delay::new(when)
			.and_then(|_| {
				println!("pong received");
				Ok(())
			})
			.map_err(|_| ())
	}
}

fn coordinator_task(rx: mpsc::Receiver<Message>) -> impl Future<Item = (), Error = ()> {
	let transport = Transport;

	rx.for_each(move |pong_tx| {
		let start = Instant::now();

		transport.send_ping();

		transport.recv_pong()
			.and_then(move |_| {
				let rtt = start.elapsed();
				pong_tx.send(rtt).unwrap();
				Ok(())
			})
	})
}

// request an rtt
fn request_rtt(tx: mpsc::Sender<Message>) -> impl Future<Item = (Duration, mpsc::Sender<Message>), Error = ()> {
	let (resp_tx, resp_rx) = oneshot::channel();

	// sending request through channel
	tx.send(resp_tx)
		.map_err(|_| ())
		.and_then(|tx| {
			// waiting for response of the coordinator task here
			resp_rx.map(|dur| (dur, tx))
				.map_err(|_| ())
		})
}

fn main() {
    tokio::run(lazy(|| {
    	let (tx, rx) = mpsc::channel(1_024);

    	// spawn the background coordinator task
    	tokio::spawn(coordinator_task(rx));

    	// spawn a few tasks that use the coordinator to request RTTs.
    	for _ in 0..4 {
    		let tx = tx.clone();

    		tokio::spawn(lazy(|| {
    			request_rtt(tx)
    				.and_then(|(dur, _)| {
    					println!("duration = {:?}", dur);
    					Ok(())
    				})
    		}));
    	}

    	Ok(())
    }));
}
