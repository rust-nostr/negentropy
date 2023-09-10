precommit:
	cargo fmt --all -- --config format_code_in_doc_comments=true
	cargo clippy
	cargo test

clean:
	cargo clean

loc:
	@echo "--- Counting lines of .rs files (LOC):" && find src/ -type f -name "*.rs" -exec cat {} \; | wc -l