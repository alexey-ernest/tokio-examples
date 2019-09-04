#[macro_use]
extern crate futures;
extern crate tokio;

use futures::{Future, stream, Stream, Poll, Async};
use std::fmt;
use tokio::timer::Interval;
use std::time::Duration;

pub struct Fibonacci {
    interval: Interval,
    curr: u64,
    next: u64,
}

impl Fibonacci {
    fn new(duration: Duration) -> Fibonacci {
        Fibonacci {
            interval: Interval::new_interval(duration),
            curr: 1,
            next: 1,
        }
    }
}

impl Stream for Fibonacci {
    type Item = u64;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<u64>, ()> {
        // wait until the next interval
        try_ready!(self.interval.poll().map_err(|_| ()));

        let curr = self.curr;
        let next = curr + self.next;

        self.curr = self.next;
        self.next = next;

        Ok(Async::Ready(Some(curr)))
    }
}

pub struct Display10<T> {
    stream: T,
    curr: usize,
}

impl<T> Display10<T> {
    fn new(stream: T) -> Display10<T> {
        Display10 {
            stream,
            curr: 0,
        }
    }
}

impl<T> Future for Display10<T>
where
    T: Stream,
    T::Item: fmt::Display,
{
    type Item = ();
    type Error = T::Error;

    fn poll(&mut self) -> Poll<(), Self::Error> {
        while self.curr < 10 {
            let value = match try_ready!(self.stream.poll()) {
                Some(value) => value,
                None => break,
            };

            println!("value #{} = {}", self.curr, value);
            self.curr += 1;
        }

        Ok(Async::Ready(()))
    }
}

fn fibonacci() -> impl Stream<Item = u64, Error = ()> {
    stream::unfold((1, 1), |(curr, next)| {
        let new_next = curr + next;
        Some(Ok((curr, (next, new_next))))
    })
}

struct Fib {
    curr: u64,
    next: u64,
}

fn main() {
    // implemented by hand:
    // 
    // let fib = Fibonacci::new(Duration::from_secs(1));
    // let display = Display10::new(fib);
    // tokio::run(display);
    
    // with combinators:
    //
    // tokio::run(
    //     fibonacci().take(10)
    //         .for_each(|num| {
    //             println!("{}", num);
    //             Ok(())
    //         })
    // );

    // intervals with combinators:
    //
    let mut fib = Fib { curr: 1, next: 1 };
    let future = Interval::new_interval(Duration::from_secs(1)).map(move |_| {
        
        // updating mutable fib struct
        let curr = fib.curr;
        let next = curr + fib.next;

        fib.curr = fib.next;
        fib.next = next;

        // returning current value
        curr
    });

    tokio::run(
        future
            .take(10)
            .map_err(|_| ())
            .for_each(|num| {
                println!("{}", num);
                Ok(())
        })
    );
}
