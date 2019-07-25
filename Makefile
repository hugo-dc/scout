all: build test

build:
	cd scripts/jaredtrie && cargo build --release && chisel run --config chisel.toml
	cd scripts/asjtrie && npm install && npm run asbuild && chisel run --config chisel.toml
	cargo build --release

test:
	target/release/phase2-scout jaredtrie.yaml
	target/release/phase2-scout asjtrie.yaml

asjtrie:
	target/release/phase2-scout asjtrie.yaml

jaredtrie:
	target/release/phase2-scout jaredtrie.yaml
