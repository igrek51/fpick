run:
	RUST_BACKTRACE=1 cargo run

build:
	cargo build --release

install:
	cargo install --path .

doc:
	cargo doc

test:
	cargo test

publish:
	cargo publish
