# CryptoPanic Portfolio Fetcher
Same as [cryptopanic-portfolio-tracker](https://github.com/RonquilloAeon/cryptopanic-portfolio-tracker)
but implemented in Rust.

## Getting started
- Get an API token from CryptoPanic
- Run `cargo run -- configure -t [YOUR TOKEN]` to set the API token
- Run `cargo run -- fetch` to fetch and save your portfolio

## Specifying data dir
By default, data is saved to `CryptoPanicData` directory in home (as specified by [home_dir()](https://docs.rs/dirs/3.0.2/dirs/fn.home_dir.html)).

To set a custom directory, run `cargo run -- configure -d [path/to/dir]`. If the directory does not
exist, it'll be created.
