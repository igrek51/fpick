run:
	RUST_BACKTRACE=1 cargo run

# generate a new release at ./target/release/fpick
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
