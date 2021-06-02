all : prepare serve

prepare : build copy

build : 
	cargo build --example clipboardinator --target wasm32-unknown-unknown --features serde_wrapper

copy :
	cp ../js/wasmnastics.js .
	cp ../target/wasm32-unknown-unknown/debug/examples/clipboardinator.wasm ./game.wasm

# Handy thing to serve the folder
serve :
	python -m http.server