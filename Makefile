precommit:
	cargo fmt --all -- --config format_code_in_doc_comments=true
	cargo clippy --all && cargo clippy --all --no-default-features
	cargo test --all && cargo test --all --no-default-features

bench:
	RUSTFLAGS='--cfg=bench' cargo +nightly bench -p negentropy

graph:
	CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --release -p perf -o bench-baseline.svg

clean:
	cargo clean

loc:
	@echo "--- Counting lines of .rs files (LOC):" && find src/ -type f -name "*.rs" -exec cat {} \; | wc -l