all: pkg pkg-parallel

pkg: src/*
	@echo "-------------------------------------------"
	@echo " Building single-threaded version to ./pkg "
	@echo "-------------------------------------------"
	wasm-pack build --target web --release

pkg-parallel: src/*
	@echo "---------------------------------------------------"
	@echo " Building multi-threaded version to ./pkg-parallel "
	@echo "---------------------------------------------------"
	rustup component add rust-src
	RUSTC_BOOTSTRAP=1 RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+mutable-globals" \
		wasm-pack build --target web --release --out-dir pkg-parallel \
		-- . --features parallel -Z build-std=panic_abort,std

clean:
	rm -rf pkg pkg-parallel
	cargo clean
