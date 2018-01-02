app: out/emu.wasm

out/emu.wasm: static
	cargo build --release --target=wasm32-unknown-unknown
	cp ../target/wasm32-unknown-unknown/release/wasm.wasm out/emu.wasm

static: static/**.js out
	cp -r static/* out/

out:
	mkdir out

clean:
	rm -rf out
