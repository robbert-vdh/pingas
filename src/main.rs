use clap::{value_t, value_t_or_exit, App, Arg};
use fastping_rs::Pinger;
use image::{FilterType, GenericImageView, Rgba};
use std::net::{IpAddr, Ipv6Addr};
use std::process::exit;
use std::thread;
use std::time::Duration;

/// The default rate of the pings in milliseconds.
const DEFAULT_PING_RATE: &str = "50";

fn main() {
    let matches = App::new("pingas")
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .set_term_width(80)
        .about("A Jinglepings pinger")
        .arg(
            Arg::with_name("rate")
                .short("r")
                .help("The delay in milliseconds between pings for every pixel in the image.")
                .takes_value(true)
                .default_value(DEFAULT_PING_RATE),
        )
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

    let delay = value_t_or_exit!(matches, "rate", u64);
    let filename = matches.value_of("filename").unwrap();
    let origin_x = value_t_or_exit!(matches, "x", u16);
    let origin_y = value_t_or_exit!(matches, "y", u16);
    let width = value_t_or_exit!(matches, "width", u32);
    let height = value_t!(matches, "height", u32);

    let image = {
        let image = image::open(filename).unwrap_or_else(|err| {
            eprintln!("Can't open file:\n{}", err);
            exit(1);
        });

        // Either fit the image in the specified area if height is set or
        // calculate the new height based on the given width
        let height = height.unwrap_or_else(|_| {
            ((width as f32) / (image.width() as f32) * (image.height() as f32)) as u32
        });
        image.resize(width, height, FilterType::Lanczos3).to_rgba()
    };

    // These are the dimensions of the resized image, they can be slightly
    // different from the ones specified.
    let image_height = image.height();
    let image_width = image.width();

    println!(
        "Printing '{}' to ({}, {}) @ {}x{} pixels every {} ms",
        filename, origin_x, origin_y, image_width, image_height, delay
    );
    if delay == 0 {
        eprintln!("\nSetting a delay of 0 will likely cause adverse effects.");
    }
    eprintln!(
        "\nErrors will be printed below, this can happen when the queues are congested. \
         Try decreasing the rate if this keeps happening."
    );

    // We will ping per row to avoid hammering the server
    let handles: Vec<_> = image
        .enumerate_rows()
        .map(|(row_id, row)| {
            let addresses: Vec<_> = row
                // Skip any completely transparent pixels
                .filter(|(_, _, &Rgba([_, _, _, alpha]))| alpha > 0)
                .map(|(x, y, pixel)| {
                    build_address(origin_x + x as u16, origin_y + y as u16, pixel).to_string()
                })
                .collect();

            (row_id, addresses)
        })
        // Skip any completely transparent rows
        .filter(|(_, addresses)| addresses.len() > 0)
        .map(|(row_id, addresses)| {
            // TODO: Print the errors so we know when the network is congested
            let (pinger, _) = Pinger::new(Some(delay), Some(0)).unwrap();
            for address in &addresses {
                pinger.add_ipaddr(address);
            }

            thread::spawn(move || {
                // By delaying every row a little we can hopefully avoid a few clashes
                thread::sleep(Duration::from_millis(row_id as u64 * 13));

                loop {
                    pinger.ping_once();

                    thread::sleep(Duration::from_millis(delay));
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

/// Build an IPv6 address for writing a pixel. `x` and `y` should correspond to
/// some pixel on a 1920x1080 screen.
#[allow(clippy::many_single_char_names)]
fn build_address(x: u16, y: u16, pixel: &Rgba<u8>) -> IpAddr {
    let &Rgba([r, g, b, a]) = pixel;

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
