
SOURCE_FILES = $(shell find src -type f) $(shell find ../protochess-engine-rs/src -type f)

all: pkg pkg-parallel

pkg: $(SOURCE_FILES)
	@echo "-------------------------------------------"
	@echo " Building single-threaded version to ./pkg "
	@echo "-------------------------------------------"
	wasm-pack build --target web --release

pkg-parallel: $(SOURCE_FILES)
	@echo "---------------------------------------------------"
	@echo " Building multi-threaded version to ./pkg-parallel "
	@echo "---------------------------------------------------"
	rustup component add rust-src
	RUSTC_BOOTSTRAP=1 RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+mutable-globals" \
		wasm-pack build --target web --release --out-dir pkg-parallel \
		-- . --features parallel -Z build-std=panic_abort,std
		
# This is a hack to make wasm-bindgen-rayon work with Vite
# Replace the string "new URL('./workerHelpers.js', import.meta.url)" with "location.pathname" in all files under pkg-parallel/snippets
use-pathname:
	find pkg-parallel/snippets -type f -exec sed -i 's/new URL(.*workerHelpers.js.*import.meta.url.*),/location.pathname,/g' {} +
	
clean:
	rm -rf pkg pkg-parallel
	cargo clean
