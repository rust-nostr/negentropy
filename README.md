# Negentropy

## Description

Implementation of the [negentropy](https://github.com/hoytech/negentropy) set-reconciliation protocol.

## Project structure

The project is split up into many crates:

* [**negentropy**](./negentropy/): Rust implementation of the negentropy set-reconciliation protocol
* [**negentropy-ffi**](./negentropy-ffi/): UniFFI bindings (Swift, Kotlin and Python) of the [negentropy](./negentropy/) crate

## Flame Graph and perf

Install [flamegraph](https://github.com/flamegraph-rs/flamegraph) and then run `make graph`. 
You'll find a new file in the project root called `flamegraph.svg`: open it in a browser.

In the terminal you should see something like:

```bash
Client init took 0 ms
Relay items: 1000000
Relay reconcile took 25 ms
Client reconcile took 39 ms
[ perf record: Woken up 10 times to write data ]
[ perf record: Captured and wrote 2.406 MB perf.data (150 samples) ]
```

## Benchmarks (unstable)

To run the benchmarks use: `make bench`

## License

This project is distributed under the MIT software license - see the [LICENSE](LICENSE) file for details

## Donations

⚡ Tips: <https://getalby.com/p/yuki>

⚡ Lightning Address: yuki@getalby.com