app: out/emu.wasm

out:
	mkdir out
	cp -r static/* out/

out/emu.wasm: out
	cargo build --release --target=wasm32-unknown-unknown
	cp ../target/wasm32-unknown-unknown/release/wasm.wasm out/emu.wasm

clean:
	rm -rf out
