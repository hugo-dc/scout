all: build test

build:
	cd scripts/helloworld && cargo build --release && chisel run --config chisel.toml
	cd scripts/bazaar && cargo build --release && chisel run --config chisel.toml
	cd scripts/evmas && npm run asbuild && chisel run --config chisel.toml
	cargo build --release

test:
	target/release/phase2-scout
	target/release/phase2-scout bazaar.yaml
	target/release/phase2-scout evmas.yaml

evmas: evmas-build evmas-run

evmas-build:
	cd scripts/evmas && npm run asbuild && chisel run --config chisel.toml
evmas-run:
	target/release/phase2-scout evmas.yaml
