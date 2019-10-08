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

evmas-mul256: evmas-build evmas-run-mul256

evmas-build:
	cd scripts/evmas && npm run asbuild && chisel run --config chisel.toml

evmas-run-mul256:
	target/release/phase2-scout evmas.yaml

evmas-add: evmas-build evmas-run-add

evmas-run-add:
	target/release/phase2-scout evmas-add.yaml
