serve: build
	miniserve --index index.html ./target/wasm32/
build:
	wasm-pack build -d target/wasm32/pkg --target web
	cp index.html target/wasm32