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

relayer:
	cd scripts/turbo-evmas/ && npx ts-node src/relayer/bin.ts
	mv scripts/turbo-evmas/add11.yaml ./

evmas-build:
	cd scripts/turbo-evmas && npx gulp && chisel run --config chisel.toml

evmas-run:
	target/release/phase2-scout add11.yaml

evmas: evmas-build evmas-run
