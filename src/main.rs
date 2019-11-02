extern crate clap;
extern crate tokio;
extern crate tokio_ping;

use clap::{App, Arg};
use std::net::{IpAddr, Ipv6Addr};
use std::time::Duration;
use tokio::prelude::*;
use tokio_ping::Pinger;

/// The rate of the pings in miliseconds.
const PING_RATE: u64 = 100;

fn main() {
    let matches = App::new("pingas")
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .set_term_width(80)
        .about("A pinger for jinglepings.")
        .arg(
            Arg::with_name("filename")
                .help("A path to an image. Most bitmap format are supported.")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("x")
                .help("The x coordinate to draw at. (range [1..1920])")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("y")
                .help("The y coordinate to draw at. (range [1..1080])")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("width")
                .help("The width of the scaled bitmap.")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("height")
                .help(
                    "The height of the scaled bitmap. If set, the bitmap will be resized to fit \
                     within the specified width and height.",
                )
                .takes_value(true),
        )
        .get_matches();

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
        .timeout(Duration::from_millis(PING_RATE))
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
