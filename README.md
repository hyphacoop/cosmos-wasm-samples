# cosmos-wasm-samples
Sample CosmWasm contracts

## Setup the Build Environment 

* Install rust and cargo
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
* Install pkg-config and libssl-dev
```
sudo apt-get install pkg-config libssl-dev
```
* Install cargo-generate and cargo-run-script
```
cargo install cargo-generate
cargo install cargo-run-script
```

### Build the Contracts

* Set Rust 1.81.0 as the build toolchain
```
rustup override set 1.81.0
```
* Download the wasm target
```
rustup target add wasm32-unknown-unknown
```

* Compile: this will create a wasm file in `target/wasm-32-unknown-unknown-release/`
```
cargo wasm
```

* Compile a smaller version: this will create a wasm file in `target/wasm-32-unknown-unknown-release/`
```
RUSTFLAGS='-C link-arg=-s' cargo wasm
```

* Compile with cosmwasm/optimizer: this will create a wasm file in `<sample project>/artifacts/`
```
docker run --rm -v "$(pwd)/<sample project>":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.16.0
```