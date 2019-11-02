extern crate tokio;
extern crate tokio_ping;

use std::net::{IpAddr, Ipv6Addr};
use std::time::Duration;
use tokio::prelude::*;
use tokio_ping::Pinger;

fn main() {
    let pinger = Pinger::new();

    let ping_future = pinger
        .map(|pinger| build_stream(&pinger, 5, 1074, 255, 255, 255, 255))
        .and_then(|stream| stream.for_each(|_| Ok(println!("Ping!"))));
    tokio::run(ping_future.map_err(|err| eprintln!("Error: {}", err)));
}

/// Build a stream that pings a certain pixel every 100 milliseconds. Since the
/// server does not return a resposne all pings will time out.
fn build_stream(
    pinger: &Pinger,
    x: u16,
    y: u16,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
) -> impl Stream<Item = (), Error = tokio_ping::Error> {
    pinger
        .chain(build_address(x, y, r, g, b, a))
        .timeout(Duration::from_millis(100))
        .stream()
        .map(|_| ())
}

/// Build an IPv6 address for writing a pixel. `x` and `y` should correspond to
/// some pixel on a 1920x1080 screen.
fn build_address(x: u16, y: u16, r: u8, g: u8, b: u8, a: u8) -> IpAddr {
    IpAddr::V6(Ipv6Addr::new(
        0x2001,
        0x610,
        0x1908,
        0xa000,
        x,
        y,
        ((b as u16) << 8) | (g as u16),
        ((r as u16) << 8) | (a as u16),
    ))
}
