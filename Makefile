precommit:
	cargo fmt --all -- --config format_code_in_doc_comments=true
	cargo clippy && cargo clippy --no-default-features
	cargo test && cargo test --no-default-features

clean:
	cargo clean

loc:
	@echo "--- Counting lines of .rs files (LOC):" && find src/ -type f -name "*.rs" -exec cat {} \; | wc -l