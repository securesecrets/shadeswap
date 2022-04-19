all:
	RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown
	cp ../../target/wasm32-unknown-unknown/release/*.wasm ./contract.wasm

deploy:
	RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown
	cp ../../target/wasm32-unknown-unknown/release/*.wasm ./contract.wasm
	docker run -it --rm -p 26657:26657 -p 26656:26656 -p 1337:1337 -v %cd%:/root/code --name secretdev enigmampc/secret-network-sw-dev

clean:
	cargo clean
	-rm -f ./contract.wasm ./contract.wasm.gz