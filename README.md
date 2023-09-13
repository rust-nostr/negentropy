# Negentropy

## Description

Implementation of the [negentropy](https://github.com/hoytech/negentropy) set-reconcilliation protocol.

## Project structure

The project is split up into many crates:

* [**negentropy**](./negentropy/): Rust implementation of the negentropy set-reconcilliation protocol
* [**negentropy-ffi**](./negentropy-ffi/): UniFFI bindings (Swift, Kotlin and Python) of the [negentropy](./negentropy/) crate (TODO)

## Benchmarks (unstable)

To run the benchmarks use: `make bench`

## Flame Graph

Install [flamegraph](https://github.com/flamegraph-rs/flamegraph) and then run `make graph`. 
You'll find a new file in the project root called `flamegraph.svg`: open it in a browser.

## License

This project is distributed under the MIT software license - see the [LICENSE](LICENSE) file for details

## Donations

⚡ Tips: <https://getalby.com/p/yuki>

⚡ Lightning Address: yuki@getalby.com