precommit:
	cargo fmt --all -- --config format_code_in_doc_comments=true
	cargo build -p negentropy-ffi
	cargo clippy -p negentropy-ffi
	cargo test -p negentropy-ffi

python:
	rm -rf bindings-python/dist
	pip install -r bindings-python/requirements.txt --break-system-packages
	cargo build --release
	cargo run --features=uniffi/cli --bin uniffi-bindgen generate src/negentropy.udl --language python --no-format -o bindings-python/src/negentropy/
	cp ../target/release/libnegentropy_ffi.so bindings-python/src/negentropy/ | true
	cp ../target/release/libnegentropy_ffi.dylib bindings-python/src/negentropy/ | true
	cd bindings-python && python setup.py --verbose bdist_wheel
	pip install ./bindings-python/dist/negentropy*.whl --force-reinstall --break-system-packages
