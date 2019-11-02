# pingas

A Jinglepings pinger.

## Running

Simply build and run through [cargo](https://rustup.rs/). To list all options,
run:

```shell
cargo run --release -- --help
```

Keep in mind that sending ICMP pings requires elevated privileges on most
systems. It might be useful to run `./target/release/pingas` directly after
compiling to prevent the `./target` directory to be owned by root.
