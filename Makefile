precommit:
	cargo fmt --all -- --config format_code_in_doc_comments=true
	cargo clippy -p negentropy -- -D warnings && cargo clippy -p negentropy --no-default-features -- -D warnings
	cargo test -p negentropy && cargo test -p negentropy --no-default-features
	cargo clippy -p harness -- -D warnings && cargo clippy -p harness --no-default-features -- -D warnings
	cargo test -p harness && cargo test -p harness --no-default-features
	cargo clippy -p perf -- -D warnings && cargo clippy -p perf --no-default-features -- -D warnings
	cargo test -p perf && cargo test -p perf --no-default-features
	cd ./negentropy-ffi && make precommit

bench:
	RUSTFLAGS='--cfg=bench' cargo +nightly bench -p negentropy

graph:
	@cargo flamegraph --version || cargo install flamegraph
	CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph -p perf -o flamegraph.svg

clean:
	cargo clean

loc:
	@echo "--- Counting lines of .rs files (LOC):" && find negentropy* -type f -name "*.rs" -exec cat {} \; | wc -l
