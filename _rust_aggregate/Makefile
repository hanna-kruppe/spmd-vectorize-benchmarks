all:
	xargo build --release --target nyuzi-elf-none
	cp target/nyuzi-elf-none/release/lib_rust_aggregate.a bench.a

all-host:
	cargo +nightly build --release
	cp target/release/lib_rust_aggregate.a bench-host.a

clean:
	xargo clean
	rm -f bench.a
