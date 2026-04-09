generator = target/debug/ttydr-60hz-generator
object-file = target/patches.o

.PHONY: clean both

both: target/v1.0.0.pchtxt target/v1.0.1.pchtxt

target/v1.0.0.pchtxt: $(generator) $(object-file)
	$^ 100 > $@

target/v1.0.1.pchtxt: $(generator) $(object-file)
	$^ 101 > $@

$(generator): Cargo.toml generator.rs
	cargo build

$(object-file): patches.S
	aarch64-linux-gnu-as $< -o $@

clean:
	rm -r target
