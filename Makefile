generator = target/debug/ttydr-60hz-generator
object-file = target/patches.o

.PHONY: clean both

all: target/v1.0.0.pchtxt target/v1.0.1.pchtxt

target/%.pchtxt: $(generator) $(object-file)
	$^ $* > $@

$(generator): Cargo.toml *.rs
	cargo build

target/%.o: %.S
	aarch64-linux-gnu-as $^ -o $@

clean:
	-rm -r target
