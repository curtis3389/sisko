build:
	cargo build

clean:
	cargo clean

run:
	RUST_BACKTRACE=1 cargo run 2> err.txt

test:
	cargo test
	cd sisko_lib && cargo test
